mod handler;
mod message;
mod refs;

pub use self::{handler::*, message::*, refs::*};

pub trait Actor: 'static + Sync + Send {}
