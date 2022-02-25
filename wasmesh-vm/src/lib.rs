#![feature(unboxed_closures, fn_traits, thread_id_value)]

pub use sandbox::*;
pub use wasmesh_vm_handler::*;

pub mod handler;
mod sandbox;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
