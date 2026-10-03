#![warn(unused_crate_dependencies)]

use quote::quote;

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
