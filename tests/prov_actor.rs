#![allow(unused_variables, dead_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub struct Book {
    id: Uuid,
    title: String,
    stock: u32,
}

#[derive(Debug)]
pub struct User {
    id: Uuid,
    name: String,
    rental: HashSet<Uuid>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            rental: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub enum KernelError {
    InvalidValue,
}

impl Display for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KernelError")
    }
}

impl Error for KernelError {}

pub enum UserCommand {
    Rental { book: Uuid },
}

#[derive(Debug)]
pub enum UserEvent {
    Rental { book: Uuid },
}

impl Actor for Book {}

impl Actor for User {}

impl Message for UserCommand {}

impl Handler<UserCommand> for User {
    type Accept = UserEvent;
    type Rejection = KernelError;

    async fn handle(&mut self, msg: UserCommand) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            UserCommand::Rental { book } => {
                self.rental.insert(book);
                println!("{:?}", self);
                Ok(UserEvent::Rental { book })
            }
        }
    }
}

#[derive(Debug)]
pub enum ActorError {
    CallbackSend,
}

impl Display for ActorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error from actor")
    }
}

impl Error for ActorError {}

pub trait Actor: 'static + Sync + Send {}

pub trait Handler<M: Message>: 'static + Sync + Send
where
    Self: Actor,
{
    type Accept: 'static + Sync + Send;
    type Rejection: 'static + Sync + Send;
    fn handle(
        &mut self,
        msg: M,
    ) -> impl Future<Output = Result<Self::Accept, Self::Rejection>> + Send;
}

pub trait Message: 'static + Sync + Send {}

#[async_trait::async_trait]
pub trait Applier<A: Actor>: 'static + Sync + Send {
    async fn apply(self: Box<Self>, actor: &mut A) -> Result<(), ActorError>;
}

pub struct Callback<A: Actor, M: Message>
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
            .map_err(|_| ActorError::CallbackSend)?)
    }
}

pub struct Void<A: Actor, M: Message>
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
                .map_err(|_| ActorError::CallbackSend),
            Err(e) => self
                .oneshot
                .send(Err(e))
                .map_err(|_| ActorError::CallbackSend),
        }
    }
}

pub struct ActorRef<A: Actor> {
    sender: UnboundedSender<Box<dyn Applier<A>>>,
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
        let Ok(_) = self.sender.send(Box::new(Callback {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallbackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallbackSend);
        };

        Ok(res)
    }

    pub async fn tell<M: Message>(&self, msg: M) -> Result<Result<(), A::Rejection>, ActorError>
    where
        A: Handler<M>,
    {
        let (tx, rx) = oneshot::channel();
        let Ok(_) = self.sender.send(Box::new(Void {
            message: msg,
            oneshot: tx,
        })) else {
            return Err(ActorError::CallbackSend);
        };
        let Ok(res) = rx.await else {
            return Err(ActorError::CallbackSend);
        };

        Ok(res)
    }
}

pub async fn spawn_actor<A: Actor>(mut actor: A) -> ActorRef<A> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = msg.apply(&mut actor).await {
                tracing::error!("{}", e);
            }
        }
    });

    ActorRef { sender: tx }
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let user = User::default();
    let actor_ref = spawn_actor(user).await;

    for _ in 0..5 {
        let ev = actor_ref
            .ask(UserCommand::Rental {
                book: Uuid::new_v4(),
            })
            .await??;
        println!("{:?}", ev);
    }

    for _ in 0..5 {
        actor_ref
            .tell(UserCommand::Rental {
                book: Uuid::new_v4(),
            })
            .await??;
    }

    tokio::time::sleep(Duration::new(3, 0)).await;
    Ok(())
}
