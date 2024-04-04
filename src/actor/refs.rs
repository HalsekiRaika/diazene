use std::any::Any;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::actor::{Actor, Handler, Message};
use crate::errors::ActorError;

pub struct ActorRef<A: Actor> {
    ctx: Arc<RefContext<A>>,
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self {
            ctx: Arc::clone(&self.ctx),
        }
    }
}

pub(crate) struct RefContext<A> {
    sender: UnboundedSender<Box<dyn Applier<A>>>,
}

impl<A: Actor> ActorRef<A> {
    pub(crate) fn new(sender: UnboundedSender<Box<dyn Applier<A>>>) -> ActorRef<A> {
        Self {
            ctx: Arc::new(RefContext { sender }),
        }
    }
}

impl<A: Actor> ActorRef<A> {
    pub async fn ask<M: Message>(
        &self,
        msg: M,
    ) -> Result<Result<A::Accept, A::Rejection>, ActorError>
    where
        A: Handler<M>,
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

    pub async fn tell<M: Message>(&self, msg: M) -> Result<Result<(), A::Rejection>, ActorError>
    where
        A: Handler<M>,
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

impl<A: Actor> ActorRef<A> {
    pub async fn ask_flat<M: Message>(&self, msg: M) -> Result<A::Accept, A::Rejection>
    where
        A: Handler<M>,
        A::Rejection: From<ActorError>,
    {
        self.ask(msg).await.unwrap_or_else(|e| Err(e.into()))
    }

    pub async fn tell_flat<M: Message>(&self, msg: M) -> Result<(), A::Rejection>
    where
        A: Handler<M>,
        A::Rejection: From<ActorError>,
    {
        self.tell(msg).await.unwrap_or_else(|e| Err(e.into()))
    }
}

#[async_trait::async_trait]
pub(crate) trait Applier<A: Actor>: 'static + Sync + Send {
    async fn apply(self: Box<Self>, actor: &mut A) -> Result<(), ActorError>;
}

pub(crate) struct Callback<A: Actor, M: Message>
where
    A: Handler<M>,
{
    message: M,
    oneshot: oneshot::Sender<Result<A::Accept, A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: Actor, M: Message> Applier<A> for Callback<A, M>
where
    A: Handler<M>,
{
    async fn apply(self: Box<Self>, actor: &mut A) -> Result<(), ActorError> {
        Ok(self
            .oneshot
            .send(actor.handle(self.message).await)
            .map_err(|_| ActorError::CallBackSend)?)
    }
}

pub(crate) struct Void<A: Actor, M: Message>
where
    A: Handler<M>,
{
    message: M,
    oneshot: oneshot::Sender<Result<(), A::Rejection>>,
}

#[async_trait::async_trait]
impl<A: Actor, M: Message> Applier<A> for Void<A, M>
where
    A: Handler<M>,
{
    async fn apply(self: Box<Self>, actor: &mut A) -> Result<(), ActorError> {
        match actor.handle(self.message).await {
            Ok(_) => self
                .oneshot
                .send(Ok(()))
                .map_err(|_| ActorError::CallBackSend),
            Err(e) => self
                .oneshot
                .send(Err(e))
                .map_err(|_| ActorError::CallBackSend),
        }
    }
}

pub(crate) struct AnyRef(Arc<dyn Any + Sync + Send>);

impl AnyRef {
    pub fn downcast<A: Actor>(self) -> Result<ActorRef<A>, ActorError> {
        Ok(ActorRef {
            ctx: self
                .0
                .downcast::<RefContext<A>>()
                .map_err(|_| ActorError::DownCastFromAny)?,
        })
    }
}

impl Clone for AnyRef {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<A: Actor> From<ActorRef<A>> for AnyRef {
    fn from(value: ActorRef<A>) -> Self {
        Self(value.ctx)
    }
}
