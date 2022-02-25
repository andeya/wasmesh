use proc_macro::TokenStream;

use quote::quote;

/// Entry pointer of function, take function handler as argument.
///
/// `target fn type: fn<R: wasmesh_abi::Message>(wasmesh_abi::Ctx, wasmesh_abi::InArgs) -> wasmesh_abi::Result<R>`
/// command to check expanded code: `cargo +nightly rustc -- -Zunstable-options --pretty=expanded`
#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn entry(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut handler_block = item.clone();
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let handler_ident = input.sig.ident;
    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn _wasm_main(ctx_id: i32, size: i32) {
            wasmesh_abi::wasm_main(ctx_id, size, #handler_ident)
        }
    };
    handler_block.extend(TokenStream::from(expanded));

    #[cfg(debug_assertions)]
    println!("{}", handler_block);

    handler_block
}
