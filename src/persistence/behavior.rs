use std::future::Future;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::actor::{Actor, Handler, Message};
use crate::errors::ActorError;

pub trait PersistenceBehavior<A: Actor>: 'static + Sync + Send {
    fn ask<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<A::Accept, A::Rejection>, ActorError>> + Send
        where A: Handler<M>,
              A::Accept: Serialize + DeserializeOwned;

    fn tell<M: Message>(&self, msg: M) -> impl Future<Output=Result<Result<(), A::Rejection>, ActorError>> + Send
        where A: Handler<M>,
              A::Accept: Serialize + DeserializeOwned;
}
