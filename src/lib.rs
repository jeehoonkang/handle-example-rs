#![feature(thread_local_state)]

extern crate crossbeam_epoch as epoch;
extern crate crossbeam_utils;
#[macro_use]
extern crate lazy_static;

mod queue;
mod internal;
mod logger;
mod default;

pub use internal::{Global, Local};
pub use logger::{Logger, Handle};
pub use default::{log, flush};
