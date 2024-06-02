use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::oneshot;

use crate::actor::{Actor, ActorRef, Applier, Context, Handler, Message};
use crate::errors::ActorError;
use crate::persistence::event::behavior::PersistenceBehavior;
use crate::persistence::event::EventSourced;

impl<A: EventSourced> PersistenceBehavior<A> for ActorRef<A> {
    async fn ask<M: Message>(&self, msg: M) -> Result<Result<A::Accept, A::Rejection>, ActorError>
        where A: Handler<M>,
              A::Accept: Serialize + DeserializeOwned
    {
        let (tx, rx) = oneshot::channel();
        let Ok(_) = self.ctx.sender.send(Box::new(Callback {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallBackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallBackSend);
        };

        Ok(res)
    }

    async fn tell<M: Message>(&self, msg: M) -> Result<Result<(), A::Rejection>, ActorError>
        where A: Handler<M>,
              A::Accept: Serialize + DeserializeOwned
    {
        let (tx, rx) = oneshot::channel();
        let Ok(_) = self.ctx.sender.send(Box::new(Void {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallBackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallBackSend);
        };

        Ok(res)
    }
}


pub(crate) struct Callback<A: Actor, M: Message>
    where
        A: Handler<M>,
        A::Accept: Serialize + DeserializeOwned
{
    message: M,
    oneshot: oneshot::Sender<Result<A::Accept, A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: EventSourced, M: Message> Applier<A> for Callback<A, M>
    where
        A: Handler<M>,
        A::Accept: Serialize + DeserializeOwned
{
    async fn apply(self: Box<Self>, actor: &mut A, ctx: &mut Context) -> Result<(), ActorError> {
        let msg = actor.handle(self.message, ctx).await;

        if let Ok(msg) = &msg {
            ctx.persistence_mut().persist(msg).await?;
        }

        Ok(self
            .oneshot
            .send(msg)
            .map_err(|_| ActorError::CallBackSend)?)
    }
}


pub(crate) struct Void<A: Actor, M: Message>
    where
        A: Handler<M>,
        A::Accept: Serialize + DeserializeOwned
{
    pub(crate) message: M,
    pub(crate) oneshot: oneshot::Sender<Result<(), A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: EventSourced, M: Message> Applier<A> for Void<A, M>
    where
        A: Handler<M>,
        A::Accept: Serialize + DeserializeOwned

{
    async fn apply(self: Box<Self>, actor: &mut A, ctx: &mut Context) -> Result<(), ActorError> {
        match actor.handle(self.message, ctx).await {
            Ok(ev) => {

                ctx.persistence_mut()
                    .persist(&ev)
                    .await?;

                self.oneshot
                    .send(Ok(()))
                    .map_err(|_| ActorError::CallBackSend)
            },
            Err(e) => self
                .oneshot
                .send(Err(e))
                .map_err(|_| ActorError::CallBackSend),
        }
    }
}
