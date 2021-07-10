use proc_macro::TokenStream;

use quote::quote;

/// Entry pointer of function, take function handler as argument.
///
/// `target fn type: Fn(wasp::Request) -> Option<wasp::Response>`
/// command to check expanded code: `cargo +nightly rustc -- -Zunstable-options --pretty=expanded`
#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn handler(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut handler_block = item.clone();
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let handler_ident = input.sig.ident;
    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn _wasp_guest_handler(thread_id: i32, ctx_id: i32, size: i32) {
            wasp::guest::handle_request(thread_id, ctx_id, size, #handler_ident)
        }
    };
    handler_block.extend(TokenStream::from(expanded));
    handler_block
}
