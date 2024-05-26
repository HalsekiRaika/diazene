#[cfg(not(all(feature = "unstable", feature = "persistence")))]
compile_error!("This feature requires the unstable feature to be enabled.");
use crate::actor::{Actor, Context, Message};
use crate::errors::ActorError;
// Todo: impl Persistence Module
pub trait PersistentActor: 'static + Sync + Send {
    fn persist<M: Message>(&self, _msg: M, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::debug!(name: "unimplemented", "received");
        Ok(())
    }
}
impl<A: PersistentActor> Actor for A {
    
}