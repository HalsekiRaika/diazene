mod refs;
mod handler;
mod message;

pub use self::{
    refs::*,
    handler::*,
    message::*
};

pub trait Actor: 'static + Sync + Send {
}
