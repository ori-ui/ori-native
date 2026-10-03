#![warn(unused_crate_dependencies, missing_docs)]

//! Macros for `ori-native`.

use quote::quote;

mod builder;

fn find_ori_native() -> syn::Path {
    syn::parse_quote!(ori_native)
}

/// Mark function as the entry point for mobile applications.
#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);
    let ident = &item.sig.ident;

    let expanded = quote! {
        #item

        const _: () = {
            let _ = #ident;

            #[unsafe(no_mangle)]
            #[cfg(target_os = "android")]
            extern "C" fn Java_ori_OriActivity_main(
                env: *mut ::std::ffi::c_void,
                this: *mut ::std::ffi::c_void,
            ) {
                unsafe { ori_native::platform::entry(env, this, #ident) }
            }
        };
    };

    expanded.into()
}

/// Derive the builder pattern for a function.
#[proc_macro_attribute]
pub fn builder(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match builder::builder(attr, item) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error().into(),
    }
}
