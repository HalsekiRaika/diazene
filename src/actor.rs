mod handler;
mod message;
mod refs;
mod context;
mod state;
pub mod behavior;

pub use self::{
    handler::*,
    message::*,
    refs::*,
    state::*,
    context::*,
};

use crate::errors::ActorError;

#[async_trait::async_trait]
pub trait Actor: 'static + Sync + Send + Sized {
    async fn activate(&mut self, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::debug!(name: "actor", "activate");
        Ok(())
    }
}
