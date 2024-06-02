//! This module provides an implementation suited to a design 
//! called `EventSourcing`, which is considered a good match for `ActorModel`.

#[cfg(not(any(feature = "unstable", feature = "persistence")))]
compile_error!("To use this feature, `persistence` and `unstable` features must be enabled");

mod refs;
pub mod behavior;
pub mod provider;
mod actor;

pub use self::{
    actor::*,
};