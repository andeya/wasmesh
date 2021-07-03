pub use wasp_macros::handler;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod proto;
pub mod errors;
pub mod guest;
