#[cfg(not(all(feature = "unstable", feature = "persistence")))]
compile_error!("This feature requires the unstable feature to be enabled.");

mod journal;
mod error;

mod provider;

#[cfg(not(feature = "event"))]
mod actor;

#[cfg(feature = "event")]
pub mod event;

pub use self::{
    error::*,
    journal::*,
};

#[cfg(not(feature = "event"))]
pub use self::actor::*;
