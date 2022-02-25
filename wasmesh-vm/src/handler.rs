use std::collections::HashMap;
use std::sync::RwLock;

use lazy_static::lazy_static;
pub use protobuf::{self, CodedOutputStream, Message};

pub use wasmesh_abi::*;

type Handler = fn(&Any) -> Result<Any>;

lazy_static! {
    static ref MUX: RwLock<HashMap<Method, Handler>> = RwLock::new(HashMap::<Method, Handler>::new());
}

pub fn set_handler(method: Method, hdl: Handler) {
    MUX.write().unwrap().insert(method, hdl);
}

#[allow(dead_code)]
pub(crate) fn host_call(args_pb: &Vec<u8>) -> OutResult {
    match InArgs::parse_from_bytes(&args_pb) {
        Ok(host_args) => {
            handle(host_args)
        }
        Err(err) => {
            ERR_CODE_PROTO.to_code_msg(err).into()
        }
    }
}


fn handle(args: InArgs) -> OutResult {
    let res: Result<Any> = MUX.read().unwrap().get(&args.get_method())?(args.get_data());
    match res {
        Ok(a) => a.into(),
        Err(e) => e.into(),
    }
}


#[cfg(test)]
mod test {
    use wasmesh_abi::test::{TestArgs, TestResult};

    use super::*;

    #[test]
    fn add() {
        #[crate::vm_handler(1)]
        fn add1(args: TestArgs) -> Result<TestResult> {
            let mut res = TestResult::new();
            res.set_sum(args.a + args.b);
            Ok(res)
        }
        fn add2(args: &Any) -> Result<Any> {
            let args: TestArgs = unpack_any(args)?;
            add1(args).and_then(|res| pack_any(&res))
        }
    }
}
