pub use wasp_macros::handler;

// pub mod proto;
pub mod errors;
pub mod guest;

// flatc --rust  message.fbs
#[allow(unused_imports)]
mod message_generated;
pub mod message;
