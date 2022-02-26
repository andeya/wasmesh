use std::env;

use rand::random;

use wasmesh_abi::*;
use wasmesh_abi::test;

#[wasm_entry]
fn main(ctx: Ctx, args: InArgs) -> Result<Any> {
    println!("[Simple] env={:?}", env::args().collect::<Vec<String>>());
    println!("[Simple] ctx={:?}, args={{{:?}}}", ctx, args);

    match args.get_method() {
        0 => {
            let args: test::TestArgs = args.get_args()?;
            let sum: test::TestResult = ctx.call_host(0, &args)?;
            println!("[Simple] call host add: args={{{:?}}}, sum={}", args, sum.get_sum());
            pack_any(sum)
        },
        _ => { pack_empty() }
    }
}
