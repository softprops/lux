//! # lux
//!
//! lux is a kubernetes log multiplexor
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate term;
extern crate rand;
extern crate url;

mod errors;
pub use errors::*;
mod logs;
pub use logs::*;
mod color;
mod pod;
