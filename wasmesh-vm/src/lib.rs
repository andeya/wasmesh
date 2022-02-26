#![feature(unboxed_closures, fn_traits, thread_id_value, const_fn_fn_ptr_basics)]

pub use sandbox::*;

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
