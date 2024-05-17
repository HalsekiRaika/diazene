use std::collections::HashMap;

use anyid::AnyId;

use crate::actor::{Actor, ActorRef, AnyRef, Applier, Handler, Message};
use crate::errors::ActorError;

pub struct Supervisor {
    pub(crate) actors: HashMap<AnyId, AnyRef>
}

pub struct SupervisorRef(ActorRef<Supervisor>);

impl Supervisor {
    pub fn new() -> Supervisor {
        Self { actors: HashMap::new() }
    }
    
    pub fn self_activate(mut self) -> SupervisorRef {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<Supervisor>>>();

        let refs = ActorRef::new(tx);
        
        tokio::spawn(async move {
            tracing::info!("supervisor started.");
            while let Some(payload) = rx.recv().await {
                if let Err(e) = payload.apply(&mut self).await {
                    tracing::error!("{}", e);
                }
            }
        });
        
        SupervisorRef(refs)
    }
}

impl SupervisorRef {
    pub async fn spawn<A: Actor>(&self, id: impl Into<AnyId>, actor: A) -> Result<ActorRef<A>, ActorError> {
        self.0.ask(RunnableActor { id: id.into(), actor }).await?
    }
    
    pub async fn shutdown(&self, id: impl Into<AnyId>) -> Result<(), ActorError> {
        self.0.tell(ShutdownActor { id: id.into() }).await?
    }
}

impl Actor for Supervisor {}

impl<A: Actor> Handler<RunnableActor<A>> for Supervisor {
    type Accept = ActorRef<A>;
    type Rejection = ActorError;

    async fn handle(&mut self, mut msg: RunnableActor<A>) -> Result<Self::Accept, Self::Rejection> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();

        let refs = ActorRef::new(tx);

        if self.actors.insert(msg.id.clone(), refs.clone().into()).is_some() {
            return Err(ActorError::AlreadySpawned { id: msg.id })
        }

        tokio::spawn(async move {
            tracing::info!("actor: [id={}] spawned.", msg.id);
            while let Some(payload) = rx.recv().await {
                if let Err(e) = payload.apply(&mut msg.actor).await {
                    tracing::error!("{}", e);
                }
            }
            tracing::warn!("actor: [id={}] was shutdown.", msg.id);
        });

        Ok(refs)
    }
}

impl Handler<ShutdownActor> for Supervisor {
    type Accept = ();
    type Rejection = ActorError;

    async fn handle(&mut self, msg: ShutdownActor) -> Result<Self::Accept, Self::Rejection> {
        if self.actors.remove(&msg.id).is_none() {
            tracing::error!("target actor: [id={}] could not be found.", msg.id);
            return Err(ActorError::NotFoundActor { id: msg.id })
        }
        
        tracing::warn!("actor: [id={}] is now subject to shutdown, and if the ActorRef of the Actor is not used, this Actor will be shutdown immediately.", msg.id);
        Ok(())
    }
}

pub struct RunnableActor<A: Actor> {
    id: AnyId,
    actor: A
}

impl<A: Actor> Message for RunnableActor<A> {}

impl<A: Actor> From<(&'static str, A)> for RunnableActor<A> {
    fn from(value: (&'static str, A)) -> Self {
        Self { id: value.0.into(), actor: value.1 }
    }
}

pub struct ShutdownActor {
    id: AnyId
}

impl Message for ShutdownActor {}