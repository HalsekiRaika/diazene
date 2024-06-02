use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::actor::{Actor, Context, Message};
use crate::errors::ActorError;

pub trait PersistentActor: 'static + Sync + Send
    where Self: Serialize + DeserializeOwned
{
    fn persist<M: Message>(&self, _msg: M, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::debug!(name: "unimplemented", "received");
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A: PersistentActor> Actor for A {
}
