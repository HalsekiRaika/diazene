use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::{Actor, Context};
use crate::errors::ActorError;

pub trait Event: 'static + Send + Sync
    where Self: Serialize + DeserializeOwned {
    const VERSION: &'static str;
    type Actor: EventSourcedActor + Replay<Self>;
    fn apply(self, actor: &mut Self::Actor);
}

#[async_trait::async_trait(?Send)]
pub trait Replay<E>: 'static + Sync + Send + Sized
    where Self: EventSourcedActor,
          E: Event<Actor=Self>,
{
    async fn replay(&mut self, events: impl IntoIterator<Item=E>) {
        events.into_iter().for_each(|event| {
            event.apply(self);
        })
    }
}

#[async_trait::async_trait]
pub trait EventSourcedActor: 'static + Sync + Send {
    async fn activate(&mut self, ctx: &mut Context);
}

impl<E: Event<Actor=A>, A: EventSourcedActor> Replay<E> for A { /* auto-impl */ }

#[async_trait::async_trait]
impl<A: EventSourcedActor> Actor for A {
    async fn activate(&mut self, ctx: &mut Context) -> Result<(), ActorError> {
        self.activate(ctx).await;
        Ok(())
    }
}
