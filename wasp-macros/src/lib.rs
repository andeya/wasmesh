use proc_macro::TokenStream;

use quote::quote;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

/// Entry pointer of function, take function handler as argument.
///
/// `target fn type: Fn(Message) -> Message`
/// command to check expanded code: `cargo +nightly rustc -- -Zunstable-options --pretty=expanded`
#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn handler(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut handler_block = item.clone();
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let handler_ident = input.sig.ident;
    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn _wasp_handler(size: i32) {
            wasp_sdk::guest::run_handler(size, #handler_ident);
        }
    };
    handler_block.extend(TokenStream::from(expanded));
    handler_block
}
