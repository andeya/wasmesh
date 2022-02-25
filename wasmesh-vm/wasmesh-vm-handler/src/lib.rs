use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::quote;

/// Entry pointer of function, take function handler as argument.
///
/// `target fn type: fn<A: wasmesh_abi::Message, R: wasmesh_abi::Message>(A) -> wasmesh_abi::Result<R>`
/// command to check expanded code: `cargo +nightly rustc -- -Zunstable-options --pretty=expanded`
#[proc_macro_attribute]
#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub fn vm_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut handler_block = item.clone();
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let handler_ident = input.sig.ident;
    let method = args.to_string();
    let fn_ident = Ident::new(&format!("hdl_{}_{}", method.parse::<u16>().unwrap(), handler_ident), handler_ident.span());
    let expanded = quote! {
        fn #fn_ident(args: &wasmesh_abi::Any) -> wasmesh_abi::Result<wasmesh_abi::Any> {
            let args: TestArgs = wasmesh_abi::unpack_any(args)?;
            #handler_ident(args).and_then(|res|wasmesh_abi::pack_any(&res))
        }
    };
    handler_block.extend(TokenStream::from(expanded));

    #[cfg(debug_assertions)]
    println!("{}", handler_block);

    handler_block
}

