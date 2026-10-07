use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Attribute, Expr, FnArg, GenericArgument, GenericParam, Generics, ItemFn, Pat, Path,
    PathArguments, PredicateType, ReturnType, TraitBound, Type, TypeParam, TypeParamBound,
    TypePath, WherePredicate, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Mut,
    visit_mut::VisitMut,
};

use crate::find_ori_native;

pub fn builder(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> syn::Result<proc_macro::TokenStream> {
    let item: ItemFn = syn::parse(item)?;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.sig.ident;
    let output = &item.sig.output;
    let body = &item.block;

    let ori_native = find_ori_native();
    let builder_ident = Ident::new(
        &snake_to_pascal_case(&ident.to_string()),
        ident.span(),
    );

    let builder_impl_ident = Ident::new(
        &format!(
            "{}Builder",
            snake_to_pascal_case(&ident.to_string())
        ),
        ident.span(),
    );

    let mut impl_generics = item.sig.generics.clone();
    let mut visitor = ImplVisitor::new("T", &mut impl_generics);

    let arguments = item
        .sig
        .inputs
        .iter()
        .map(|arg| Argument::from_fn_arg(&mut visitor, arg))
        .collect::<Result<Vec<_>, _>>()?;

    let styles = Style::get_all(&arguments);
    let style_traits = styles.clone().map(Style::trait_ident);
    let style_impls = styles.clone().map(|style| {
        style.trait_impl(
            &ori_native,
            &builder_impl_ident,
            &impl_generics,
            &arguments,
        )
    });

    let (trait_impl_generics, trait_type_generics, trait_where_clause) =
        item.sig.generics.split_for_impl();

    let (impl_impl_generics, impl_type_generics, impl_where_clause) =
        impl_generics.split_for_impl();

    let marker = marker(&item.sig.generics.params);
    let data = get_data(&item.sig.output).ok_or_else(|| {
        syn::Error::new(
            item.sig.output.span(),
            "invalid return type, must be either `impl View<...>` or `impl Effect<...>`",
        )
    })?;

    let impl_fields = arguments.iter().map(Argument::field);
    let impl_inits = arguments.iter().map(Argument::initializer);

    let mut build_generics = item.sig.generics.clone();
    let mut visitor = ImplVisitor::new("T", &mut build_generics);

    let build_arguments = arguments.iter().map(|arg| {
        let ident = &arg.ident;
        quote!(self.#ident)
    });

    let build_inputs = arguments
        .iter()
        .map(|arg| {
            let mutability = &arg.mutability;
            let input = arg.input(&mut visitor);
            quote!(#mutability #input)
        })
        .collect::<Vec<_>>();

    let (build_impl_generics, _, build_where_clause) = build_generics.split_for_impl();

    let mut fn_generics = item.sig.generics.clone();
    let mut visitor = ImplVisitor::new("T", &mut fn_generics);

    let fn_inputs = arguments
        .iter()
        .filter(|arg| arg.is_input())
        .map(|arg| arg.input(&mut visitor))
        .collect::<Vec<_>>();

    let (fn_impl_generics, _, fn_where_clause) = fn_generics.split_for_impl();

    let builder = quote!(impl #builder_ident #trait_type_generics);

    let setters_trait = arguments
        .iter()
        .filter(|arg| arg.has_setter())
        .map(|arg| arg.setter_trait(&builder));

    let setters_impl = arguments.iter().filter(|arg| arg.has_setter()).map(|arg| {
        arg.setter_impl(
            &arguments,
            &builder,
            &builder_impl_ident,
        )
    });

    let expanded = quote! {
        #[automatically_derived]
        #(#attrs)* #vis
        trait #builder_ident #trait_impl_generics: #ori_native::View<#data> #(+#style_traits)*
            #trait_where_clause
        {
            #(#setters_trait)*
        }

        #(#attrs)* #vis
        fn #ident #fn_impl_generics (
            #(#fn_inputs,)*
        ) -> #builder
            #fn_where_clause
        {
            #[automatically_derived]
            struct #builder_impl_ident #impl_impl_generics
                #impl_where_clause
            {
                #(#impl_fields,)*
                marker: #marker,
            }

            #(#style_impls)*

            #[automatically_derived]
            impl #impl_impl_generics #builder_ident #trait_type_generics
                for #builder_impl_ident #impl_type_generics
                #impl_where_clause
            {
                #(#setters_impl)*
            }

            #[automatically_derived]
            impl #impl_impl_generics #ori_native::BuilderMarker
                for #builder_impl_ident #impl_type_generics
                #impl_where_clause
            {}

            #[automatically_derived]
            impl #impl_impl_generics #ori_native::Builder<#ori_native::Context, #data>
                for #builder_impl_ident #impl_type_generics
                #impl_where_clause
            {
                type Element = #ori_native::BoxedWidget;

                fn build(self) -> impl #ori_native::View<#data> {
                    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
                    fn #ident #build_impl_generics (
                        #(#build_inputs),*
                    ) #output
                        #build_where_clause
                    #body

                    #ident(#(#build_arguments),*)
                }
            }

            #builder_impl_ident {
                #(#impl_inits,)*
                marker: ::std::marker::PhantomData,
            }
        }
    };

    Ok(expanded.into())
}

fn snake_to_pascal_case(name: &str) -> String {
    let mut output = String::new();
    let mut is_upper = true;

    for c in name.chars() {
        if c == '_' {
            is_upper = true;
        } else if is_upper {
            is_upper = false;
            output.push_str(&c.to_uppercase().to_string());
        } else {
            output.push(c);
        }
    }

    output
}

fn marker<'a>(params: impl IntoIterator<Item = &'a GenericParam>) -> Type {
    let params = params.into_iter().map(|param| match param {
        GenericParam::Lifetime(param) => param.to_token_stream(),
        GenericParam::Type(param) => param.ident.to_token_stream(),
        GenericParam::Const(param) => param.ident.to_token_stream(),
    });

    parse_quote!(::std::marker::PhantomData<(#(#params),*)>)
}

struct Argument {
    attrs:         Attributes,
    mutability:    Option<Mut>,
    ident:         Ident,
    ty:            Type,
    generic:       Type,
    is_impl_trait: bool,
}

struct ImplVisitor<'a> {
    count:    usize,
    prefix:   &'a str,
    generics: &'a mut Generics,
}

impl<'a> ImplVisitor<'a> {
    pub fn new(prefix: &'a str, generics: &'a mut Generics) -> Self {
        Self {
            count: 0,
            prefix,
            generics,
        }
    }
}

impl<'a> VisitMut for ImplVisitor<'a> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        let Type::ImplTrait(impl_ty) = ty else {
            return syn::visit_mut::visit_type_mut(self, ty);
        };

        self.visit_type_impl_trait_mut(impl_ty);

        self.count += 1;
        let ident = Ident::new(
            &format!("{}{}", self.prefix, self.count),
            impl_ty.span(),
        );

        let generic = Type::Path(TypePath {
            attrs: Vec::new(),
            qself: None,
            path:  Path::from(ident.clone()),
        });

        self.generics.params.push(GenericParam::Type(TypeParam {
            attrs: Vec::new(),
            ident,
            colon_token: None,
            bounds: Punctuated::new(),
            default: None,
        }));

        self.generics
            .make_where_clause()
            .predicates
            .push(WherePredicate::Type(PredicateType {
                attrs:       Vec::new(),
                lifetimes:   None,
                bounded_ty:  generic.clone(),
                colon_token: Default::default(),
                bounds:      impl_ty.bounds.clone(),
            }));

        *ty = generic;
    }
}

impl Argument {
    fn from_fn_arg(visitor: &mut ImplVisitor<'_>, arg: &FnArg) -> syn::Result<Self> {
        match arg {
            FnArg::Receiver(_) => Err(syn::Error::new(
                arg.span(),
                "`self` arguments are not allowed in `builder` functions",
            )),
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                Pat::Ident(pat) => {
                    let attrs = Attributes::from_attrs(&arg.attrs)?;

                    let mut generic = arg.ty.as_ref().clone();

                    let count = visitor.count;
                    visitor.visit_type_mut(&mut generic);

                    let is_impl_trait = visitor.count != count;
                    if is_impl_trait && matches!(attrs.default, Some(None)) {
                        return Err(syn::Error::new(
                            arg.span(),
                            "arguments with `impl trait` types must provide an explicit default, try `#[default = ...]`",
                        ));
                    }

                    Ok(Self {
                        attrs,
                        mutability: pat.mutability,
                        ident: pat.ident.clone(),
                        ty: arg.ty.as_ref().clone(),
                        generic,
                        is_impl_trait,
                    })
                }

                _ => Err(syn::Error::new(
                    arg.span(),
                    "only identifier patterns allowed in `builder` functions",
                )),
            },
        }
    }

    fn is_input(&self) -> bool {
        self.attrs.default.is_none()
    }

    fn has_setter(&self) -> bool {
        self.attrs.default.is_some() && self.attrs.style.is_none()
    }

    fn is_into(&self) -> bool {
        !self.is_impl_trait
            && self.ty != parse_quote!(f32)
            && self.ty != parse_quote!(f64)
            && self.ty != parse_quote!(i8)
            && self.ty != parse_quote!(i16)
            && self.ty != parse_quote!(i32)
            && self.ty != parse_quote!(i64)
            && self.ty != parse_quote!(isize)
            && self.ty != parse_quote!(u8)
            && self.ty != parse_quote!(u16)
            && self.ty != parse_quote!(u32)
            && self.ty != parse_quote!(u64)
            && self.ty != parse_quote!(usize)
            && self.ty != parse_quote!(bool)
            && self.ty != parse_quote!(&'static str)
    }

    fn field(&self) -> TokenStream {
        let ident = &self.ident;
        let ty = &self.generic;

        quote!(#ident: #ty)
    }

    fn input(&self, visitor: &mut ImplVisitor<'_>) -> TokenStream {
        let ident = &self.ident;

        let mut ty = self.ty.clone();

        match ty {
            Type::ImplTrait(ref mut ty) => visitor.visit_type_impl_trait_mut(ty),
            _ => visitor.visit_type_mut(&mut ty),
        }

        quote!(#ident: #ty)
    }

    fn initializer(&self) -> TokenStream {
        match self.attrs.default {
            Some(Some(ref default)) => {
                let ident = &self.ident;
                quote!(#ident: #default)
            }

            Some(None) => {
                let ident = &self.ident;
                quote!(#ident: ::std::default::Default::default())
            }

            None => self.ident.to_token_stream(),
        }
    }

    fn setter_trait(&self, builder: &TokenStream) -> TokenStream {
        let docs = &self.attrs.docs;
        let ident = &self.ident;

        if self.is_into() {
            let ty = &self.ty;

            return quote! {
                #[automatically_derived]
                #(#docs)*
                fn #ident(self, #ident: impl Into<#ty>) -> #builder;
            };
        }

        let mut generics = Generics::default();
        let mut visitor = ImplVisitor::new("T", &mut generics);
        let input = self.input(&mut visitor);

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            #(#docs)*
            fn #ident #impl_generics (self, #input) -> #builder
                #where_clause;
        }
    }

    fn setter_impl(
        &self,
        arguments: &[Argument],
        builder: &TokenStream,
        builder_impl: &Ident,
    ) -> TokenStream {
        let ident = &self.ident;

        let idents = arguments
            .iter()
            .filter(|argument| argument.ident != self.ident)
            .map(|argument| {
                let ident = &argument.ident;
                quote!(#ident: self.#ident)
            });

        if self.is_into() {
            let ty = &self.ty;

            return quote! {
                #[automatically_derived]
                fn #ident(self, #ident: impl ::std::convert::Into<#ty>) -> #builder {
                    #builder_impl {
                        #ident: ::std::convert::Into::into(#ident),
                        #(#idents,)*
                        marker: ::std::marker::PhantomData,
                    }
                }
            };
        }

        let mut generics = Generics::default();
        let mut visitor = ImplVisitor::new("U", &mut generics);
        let input = self.input(&mut visitor);

        let (impl_generics, _, where_clause) = generics.split_for_impl();

        quote! {
            #[automatically_derived]
            fn #ident #impl_generics (self, #input) -> #builder
                #where_clause
            {
                #builder_impl {
                    #ident,
                    #(#idents,)*
                    marker: ::std::marker::PhantomData,
                }
            }
        }
    }
}

struct Attributes {
    docs:    Vec<Attribute>,
    default: Option<Option<Expr>>,
    style:   Option<Style>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Style {
    Layout,
    Padding,
    Corners,
    Shadow,
    Border,
    Flex,
}

impl Style {
    const NAMES: &[(&str, Self)] = &[
        ("layout", Self::Layout),
        ("padding", Self::Padding),
        ("corners", Self::Corners),
        ("shadow", Self::Shadow),
        ("border", Self::Border),
        ("flex", Self::Flex),
    ];

    fn trait_name(self) -> &'static str {
        match self {
            Style::Layout => "StyleLayout",
            Style::Padding => "StylePadding",
            Style::Corners => "StyleCorners",
            Style::Shadow => "StyleShadow",
            Style::Border => "StyleBorder",
            Style::Flex => "StyleFlexContainer",
        }
    }

    fn trait_ident(self) -> Ident {
        Ident::new(self.trait_name(), Span::call_site())
    }

    fn get_all(arguments: &[Argument]) -> impl Iterator<Item = Self> + Clone {
        arguments.iter().filter_map(|argument| argument.attrs.style)
    }

    fn trait_impl(
        self,
        ori_native: &Path,
        builder_impl_ident: &Ident,
        impl_generics: &Generics,
        arguments: &[Argument],
    ) -> TokenStream {
        let trait_ident = self.trait_ident();
        let fn_ident = self.trait_fn_ident();
        let trait_type = self.trait_type(ori_native);

        let argument = arguments
            .iter()
            .find(|argument| argument.attrs.style == Some(self))
            .unwrap();

        let ident = &argument.ident;

        let (impl_generics, type_generics, where_clause) = impl_generics.split_for_impl();

        quote_spanned! {
            argument.ty.span() =>
            impl #impl_generics #ori_native::#trait_ident
                for #builder_impl_ident #type_generics
                #where_clause
            {
                fn #fn_ident(&mut self) -> &mut #trait_type {
                    &mut self.#ident
                }
            }
        }
    }

    fn trait_fn_name(self) -> &'static str {
        match self {
            Style::Layout => "get_layout_style_mut",
            Style::Padding => "get_padding_mut",
            Style::Corners => "get_corners_mut",
            Style::Shadow => "get_shadow_mut",
            Style::Border => "get_border_style_mut",
            Style::Flex => "get_flex_style_mut",
        }
    }

    fn trait_fn_ident(self) -> Ident {
        Ident::new(self.trait_fn_name(), Span::call_site())
    }

    fn trait_type(self, ori_native: &Path) -> Type {
        match self {
            Style::Layout => parse_quote!(#ori_native::LayoutStyle),
            Style::Padding => parse_quote!(#ori_native::Sides<#ori_native::Length>),
            Style::Corners => parse_quote!(#ori_native::Corners<::std::primitive::f32>),
            Style::Shadow => parse_quote!(#ori_native::Shadow),
            Style::Border => parse_quote!(#ori_native::BorderStyle),
            Style::Flex => parse_quote!(#ori_native::FlexStyle),
        }
    }
}

impl Attributes {
    fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self {
            docs:    Vec::new(),
            default: None,
            style:   None,
        };

        for attr in attrs {
            if attr.path().is_ident("default") {
                match attr.meta {
                    syn::Meta::Path(..) => {
                        this.default = Some(None);
                    }

                    syn::Meta::NameValue(ref meta) => {
                        this.default = Some(Some(meta.value.clone()));
                    }

                    syn::Meta::List(..) => {
                        attr.meta.require_name_value()?;
                    }
                }
            }

            for (name, style) in Style::NAMES {
                if attr.path().is_ident(name) {
                    if this.style.is_some() {
                        return Err(syn::Error::new(
                            attr.span(),
                            "only one style attribute is allowed per argument",
                        ));
                    }

                    this.style = Some(*style);

                    if this.default.is_none() {
                        this.default = Some(None);
                    }
                }
            }

            if attr.path().is_ident("doc") {
                this.docs.push(attr.clone());
            }
        }

        Ok(this)
    }
}

fn get_data(ty: &ReturnType) -> Option<&Type> {
    if let ReturnType::Type(_, ty) = ty
        && let Type::ImplTrait(ty) = ty.as_ref()
    {
        let bound = ty.bounds.iter().find_map(|bound| {
            if let TypeParamBound::Trait(bound) = bound
                && is_view_trait(bound)
            {
                Some(bound)
            } else {
                None
            }
        })?;

        if let PathArguments::AngleBracketed(ref args) = bound.path.segments.last()?.arguments
            && args.args.len() == 1
            && let Some(arg) = args.args.first()
            && let GenericArgument::Type(data) = arg
        {
            Some(data)
        } else {
            None
        }
    } else {
        None
    }
}

fn is_view_trait(bound: &TraitBound) -> bool {
    is_path(&bound.path, &["View"])
}

fn is_path(path: &Path, segments: &[&str]) -> bool {
    path.segments.len() == segments.len()
        && path
            .segments
            .iter()
            .zip(segments)
            .all(|(a, b)| a.ident == b)
}
