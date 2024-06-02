#[cfg(not(all(feature = "unstable", feature = "persistence")))]
compile_error!("This feature requires the unstable feature to be enabled.");

mod actor;
mod journal;
mod error;

mod provider;

mod snapshot;

#[cfg(feature = "event")]
pub mod event;

pub use self::{
    actor::*,
    error::*,
    journal::*,
    snapshot::*,
};

pub mod providers {
    pub use self::{
        super::provider::*,
        super::event::provider::*
    };
}
