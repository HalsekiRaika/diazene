use std::future::Future;

use crate::actor::{Actor, Context, Message};
use crate::errors::ActorError;

pub trait Handler<M: Message>: 'static + Sync + Send
where
    Self: Actor,
{
    type Accept: 'static + Sync + Send;
    type Rejection: 'static + Sync + Send;
    fn handle(
        &mut self,
        msg: M,
        ctx: &mut Context
    ) -> impl Future<Output = Result<Self::Accept, Self::Rejection>> + Send;
}

#[derive(Eq, PartialEq)]
pub struct Terminate;

impl Message for Terminate {}

impl<A: Actor> Handler<Terminate> for A {
    type Accept = ();
    type Rejection = ActorError;

    async fn handle(&mut self, _: Terminate, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        tracing::warn!("received terminate signal.");
        ctx.shutdown();
        Ok(())
    }
}
