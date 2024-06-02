use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::Context;
use crate::persistence::PersistentActor;

pub trait Event: 'static + Send + Sync
    where Self: Serialize + DeserializeOwned
{
    const VERSION: &'static str;
    type Actor: EventSourced + Replay<Self>;
    fn apply(self, actor: &mut Self::Actor);
}

#[async_trait::async_trait(?Send)]
pub trait Replay<E>: 'static + Sync + Send + Sized
    where Self: EventSourced,
          E: Event<Actor=Self>,
{
    async fn replay(&mut self, events: impl IntoIterator<Item=E>) {
        events.into_iter().for_each(|event| {
            event.apply(self);
        })
    }
}

#[async_trait::async_trait]
pub trait EventSourced: 'static + Sync + Send 
    where Self: PersistentActor
{
    async fn activate(&mut self, ctx: &mut Context);
}

impl<E: Event<Actor=A>, A: EventSourced> Replay<E> for A { /* auto-impl */ }
