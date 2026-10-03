use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Attribute, Expr, FnArg, GenericArgument, GenericParam, Generics, ItemFn, Pat, Path,
    PathArguments, PredicateType, ReturnType, Signature, TraitBound, Type, TypeGenerics,
    TypeImplTrait, TypeParam, TypeParamBound, TypePath, WherePredicate, fold::Fold,
    punctuated::Punctuated, spanned::Spanned, token::Mut,
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
    let body = &item.block;

    let ori_native = find_ori_native();
    let builder: Ident = Ident::new(
        &snake_to_pascal_case(&ident.to_string()),
        Span::call_site(),
    );

    let (fn_impl_generics, _, fn_where_clause) = item.sig.generics.split_for_impl();

    let mut impl_types = Vec::new();
    let mut generics = item.sig.generics.clone();

    let arguments = get_arguments(
        &item.sig,
        &mut impl_types,
        &mut generics,
    )?;

    let fn_type_generics = fn_type_generics(&item.sig.generics, &impl_types);

    let pats = argument_pats(&arguments);
    let fields = argument_fields(&arguments);
    let inits = argument_inits(&arguments);
    let inputs = argument_inputs(&arguments);
    let setters = argument_setters(&builder, &fn_type_generics, &arguments);

    let layout_impl = layout_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let padding_impl = padding_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let corners_impl = corners_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let shadow_impl = shadow_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let border_impl = border_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let flex_impl = flex_impl(
        &ori_native,
        &builder,
        &generics,
        &arguments,
    )?;

    let marker_generics = marker_generics(&generics);

    let data = get_data(&item.sig.output).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "invalid return type, must be either `impl View<...>` or `impl Effect<...>`",
        )
    })?;

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let build_impl = quote_spanned! {
        item.block.span() =>
        fn build(self) -> impl #ori_native::View<#data> {
            let #builder { #(#pats,)* .. } = self;

            #body
        }
    };

    let expanded = quote! {
        #[automatically_derived]
        #(#attrs)* #vis
        struct #builder #type_generics
            #where_clause
        {
            #(#fields,)*
            marker: ::std::marker::PhantomData #marker_generics,
        }

        #[automatically_derived]
        impl #impl_generics #ori_native::BuilderMarker
            for #builder #type_generics #where_clause {}

        #[automatically_derived]
        impl #impl_generics #ori_native::Builder<#ori_native::Context, #data>
            for #builder #type_generics #where_clause
        {
            type Element = #ori_native::BoxedWidget;

            #build_impl
        }

        #[automatically_derived]
        impl #impl_generics #builder #type_generics
            #where_clause
        {
            #(#setters)*
        }

        #layout_impl
        #padding_impl
        #corners_impl
        #shadow_impl
        #border_impl
        #flex_impl

        #(#attrs)* #vis
        fn #ident #fn_impl_generics (#(#inputs),*) -> #builder #fn_type_generics
            #fn_where_clause
        {
            #builder {
                #(#inits,)*
                marker: ::std::marker::PhantomData,
            }
        }
    };

    Ok(expanded.into())
}

struct Argument {
    ident:      Ident,
    path:       Path,
    mutability: Option<Mut>,

    is_impl: bool,

    arg_ty:   Type,
    field_ty: Type,

    span: Span,

    attrs: ArgumentAttrs,
}

struct ImplFolder<'a> {
    ident:      Option<&'a Ident>,
    impl_types: &'a mut Vec<TypeImplTrait>,
    generics:   &'a mut Generics,
}

impl ImplFolder<'_> {}

impl<'a> Fold for ImplFolder<'a> {
    fn fold_type(&mut self, ty: Type) -> Type {
        match ty {
            Type::ImplTrait(ty) => {
                self.impl_types.push(ty.clone());

                let name = match self.ident {
                    Some(ident) => snake_to_pascal_case(&ident.to_string()),
                    None => format!("T{}", self.impl_types.len()),
                };

                let ident = Ident::new(&name, ty.span());

                self.ident = None;
                self.generics.params.push(GenericParam::Type(TypeParam {
                    attrs:       Vec::new(),
                    ident:       ident.clone(),
                    colon_token: None,
                    bounds:      Punctuated::new(),
                    default:     None,
                }));

                let generic = Type::Path(TypePath {
                    attrs: Vec::new(),
                    qself: None,
                    path:  Path::from(ident),
                });

                let bounds = ty
                    .bounds
                    .into_iter()
                    .filter(|bound| {
                        !matches!(
                            bound,
                            TypeParamBound::PreciseCapture(..)
                        )
                    })
                    .collect();

                self.generics
                    .make_where_clause()
                    .predicates
                    .push(WherePredicate::Type(PredicateType {
                        attrs: Vec::new(),
                        lifetimes: None,
                        bounded_ty: generic.clone(),
                        colon_token: Default::default(),
                        bounds,
                    }));

                generic
            }

            _ => syn::fold::fold_type(self, ty),
        }
    }
}

fn get_arguments(
    sig: &Signature,
    impl_types: &mut Vec<TypeImplTrait>,
    generics: &mut Generics,
) -> syn::Result<Vec<Argument>> {
    sig.inputs
        .iter()
        .map(|argument| match argument {
            FnArg::Typed(pat_ty) => match pat_ty.pat.as_ref() {
                Pat::Ident(pat) => {
                    let mut impl_folder = ImplFolder {
                        ident: Some(&pat.ident),
                        impl_types,
                        generics,
                    };

                    let field_ty = impl_folder.fold_type(pat_ty.ty.as_ref().clone());

                    Ok(Argument {
                        ident: pat.ident.clone(),
                        path: Path::from(pat.ident.clone()),
                        mutability: pat.mutability,

                        is_impl: impl_folder.ident.is_none(),

                        span: argument.span(),

                        arg_ty: pat_ty.ty.as_ref().clone(),
                        field_ty,

                        attrs: ArgumentAttrs::new(&pat_ty.attrs)?,
                    })
                }

                _ => Err(syn::Error::new(
                    pat_ty.pat.span(),
                    "only identifier arguments are allowed in `builder` functions",
                )),
            },

            FnArg::Receiver(..) => Err(syn::Error::new(
                argument.span(),
                "`self` arguments are not allowed in `builder` functions",
            )),
        })
        .collect()
}

fn argument_pats(arguments: &[Argument]) -> Vec<TokenStream> {
    arguments
        .iter()
        .map(|argument| {
            let ident = &argument.ident;
            let mutability = &argument.mutability;

            quote_spanned! {
                argument.span =>
                #mutability #ident
            }
        })
        .collect()
}

fn argument_fields(arguments: &[Argument]) -> Vec<TokenStream> {
    arguments
        .iter()
        .map(|argument| {
            let ident = &argument.ident;
            let ty = &argument.field_ty;

            quote!(#ident: #ty)
        })
        .collect()
}

fn argument_inits(arguments: &[Argument]) -> Vec<TokenStream> {
    arguments
        .iter()
        .map(|argument| {
            let ident = &argument.ident;

            let expr = match argument.attrs.default {
                Some(ref expr) => expr.to_token_stream(),
                None => argument.path.to_token_stream(),
            };

            quote!(#ident: #expr)
        })
        .collect()
}

fn argument_setters(
    builder: &Ident,
    type_generics: &TokenStream,
    arguments: &[Argument],
) -> Vec<TokenStream> {
    arguments
        .iter()
        .filter(|argument| {
            argument.attrs.default.is_some()
                && !argument.attrs.layout
                && !argument.attrs.padding
                && !argument.attrs.corners
                && !argument.attrs.shadow
                && !argument.attrs.border
                && !argument.attrs.flex
        })
        .map(|argument| {
            let docs = &argument.attrs.docs;
            let ident = &argument.ident;
            let ty = &argument.arg_ty;

            if argument.is_impl {
                let fields =
                    arguments
                        .iter()
                        .filter(|a| a.ident != argument.ident)
                        .map(|argument| {
                            let ident = &argument.ident;

                            quote!(#ident: self.#ident)
                        });

                quote! {
                    #(#docs)*
                    #[automatically_derived]
                    pub fn #ident(
                        mut self,
                        #ident: #ty,
                    ) -> #builder #type_generics {
                        #builder {
                            #(#fields,)*
                            #ident: #ident,
                            marker: ::std::marker::PhantomData,
                        }
                    }
                }
            } else {
                quote! {
                    #(#docs)*
                    #[automatically_derived]
                    pub fn #ident(
                        mut self,
                        #ident: impl ::std::convert::Into<#ty>,
                    ) -> #builder #type_generics {
                        self.#ident = ::std::convert::Into::into(#ident);
                        self
                    }
                }
            }
        })
        .collect()
}

fn argument_inputs(arguments: &[Argument]) -> Vec<TokenStream> {
    arguments
        .iter()
        .filter(|argument| argument.attrs.default.is_none())
        .map(|argument| {
            let attrs = &argument.attrs.attrs;
            let ident = &argument.ident;
            let ty = &argument.arg_ty;

            quote!(
                #(#attrs)*
                #ident: #ty
            )
        })
        .collect()
}

macro_rules! style {
    ($impl:ident, $name:ident, $trait:ident, $fn:ident, $on:ident => $($ty:tt)*) => {
        fn $impl(
            $on: &Path,
            builder: &Ident,
            generics: &Generics,
            arguments: &[Argument],
        ) -> syn::Result<Option<TokenStream>> {
            if arguments.iter().filter(|a| a.attrs.$name).count() > 1
                && let Some(argument) = arguments.iter().rfind(|arg| arg.attrs.$name)
            {
                return Err(syn::Error::new(
                    argument.span,
                    concat!(
                        "only one `",
                        stringify!($name),
                        "` argument is allowed"
                    ),
                ));
            }

            Ok(
                arguments.iter().find(|a| a.attrs.$name).map(|argument| {
                    let path = &argument.path;
                    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

                    quote_spanned! {
                        argument.span =>
                        #[automatically_derived]
                        impl #impl_generics #$on::$trait
                            for #builder #type_generics #where_clause
                        {
                            fn $fn(
                                &mut self
                            ) -> &mut $($ty)* {
                                &mut self.#path
                            }
                        }
                    }
                }),
            )
        }
    };
}

style!(
    layout_impl,
    layout,
    StyleLayout,
    get_layout_style_mut,
    on => #on::LayoutStyle
);

style!(
    padding_impl,
    padding,
    StylePadding,
    get_padding_mut,
    on => #on::Sides<#on::Length>
);

style!(
    corners_impl,
    corners,
    StyleCorners,
    get_corners_mut,
    on => #on::Corners<::std::primitive::f32>
);

style!(
    shadow_impl,
    shadow,
    StyleShadow,
    get_shadow_mut,
    on => #on::Shadow
);

style!(
    border_impl,
    border,
    StyleBorder,
    get_border_style_mut,
    on => #on::BorderStyle
);

style!(
    flex_impl,
    flex,
    StyleFlexContainer,
    get_flex_style_mut,
    on => #on::FlexStyle
);

fn marker_generics(generics: &Generics) -> TokenStream {
    let arguments = generics.params.iter().filter_map(|param| match param {
        GenericParam::Type(param) => Some(param.ident.to_token_stream()),
        _ => None,
    });

    quote!(<(#(#arguments,)*)>)
}

fn fn_type_generics(generics: &Generics, types: &[TypeImplTrait]) -> TokenStream {
    let arguments = generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(param) => param.to_token_stream(),
            GenericParam::Type(param) => param.ident.to_token_stream(),
            GenericParam::Const(param) => param.ident.to_token_stream(),
        })
        .chain(types.iter().map(|ty| ty.to_token_stream()));

    quote!(<#(#arguments,)*>)
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

struct ArgumentAttrs {
    attrs:   Vec<Attribute>,
    docs:    Vec<Attribute>,
    default: Option<Expr>,
    layout:  bool,
    padding: bool,
    corners: bool,
    shadow:  bool,
    border:  bool,
    flex:    bool,
}

impl ArgumentAttrs {
    fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut docs = Vec::new();
        let mut default = None;
        let mut layout = false;
        let mut padding = false;
        let mut corners = false;
        let mut shadow = false;
        let mut border = false;
        let mut flex = false;

        let ori_native = find_ori_native();

        for attr in attrs {
            if attr.path().is_ident("default") {
                match attr.meta {
                    syn::Meta::Path(..) => {
                        default = Some(syn::parse_quote!(
                            ::std::default::Default::default()
                        ));
                    }

                    syn::Meta::NameValue(ref meta) => {
                        default = Some(meta.value.clone());
                    }

                    syn::Meta::List(..) => {
                        attr.meta.require_name_value()?;
                    }
                }
            }

            if attr.path().is_ident("layout") {
                layout = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        ::std::default::Default::default()
                    ));
                }
            }

            if attr.path().is_ident("padding") {
                padding = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        #ori_native::Sides::all(#ori_native::Length::Length(0.0))
                    ));
                }
            }

            if attr.path().is_ident("corners") {
                corners = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        ::std::default::Default::default()
                    ));
                }
            }

            if attr.path().is_ident("shadow") {
                shadow = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        ::std::default::Default::default()
                    ));
                }
            }

            if attr.path().is_ident("border") {
                border = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        ::std::default::Default::default()
                    ));
                }
            }

            if attr.path().is_ident("flex") {
                flex = true;

                if default.is_none() {
                    default = Some(syn::parse_quote!(
                        ::std::default::Default::default()
                    ));
                }
            }

            if attr.path().is_ident("doc") {
                docs.push(attr.clone());
            }
        }

        Ok(Self {
            attrs: attrs.to_vec(),
            docs,
            default,
            layout,
            padding,
            corners,
            shadow,
            border,
            flex,
        })
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
