use wasmy_vm::*;

mod http;

pub(crate) fn init() {
    HandlerAPI::collect_and_register_all()
}
