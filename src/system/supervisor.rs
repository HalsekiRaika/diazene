use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;

use anyid::AnyId;
use tracing::Instrument;

use crate::actor::{Actor, ActorRef, AnyRef, Applier, Context, Handler, Message, behavior::RegularBehavior};
use crate::errors::ActorError;

pub struct Supervisor {
    pub(crate) actors: HashMap<AnyId, AnyRef>
}

pub struct SupervisorRef(ActorRef<Supervisor>);

impl Supervisor {
    pub(crate) fn new() -> Supervisor {
        Self { actors: HashMap::new() }
    }
    
    pub fn activate(mut self) -> SupervisorRef {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<Supervisor>>>();

        let refs = ActorRef::new(tx);

        let supervisor_ref = SupervisorRef(refs);

        let ctx = Context::new(supervisor_ref.clone());
        
        tokio::spawn(async move {
            let mut ctx = ctx;
            
            match Actor::activate(&mut self, &mut ctx).await {
                Ok(_) => {
                    while let Some(payload) = rx.recv().await {
                        if let Err(e) = payload.apply(&mut self, &mut ctx).await {
                            tracing::error!("{}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("{}", e);
                }
            }
        });
        
        supervisor_ref
    }
}

impl SupervisorRef {
    pub async fn spawn<A: Actor>(&self, id: impl Into<AnyId>, actor: A) -> Result<ActorRef<A>, ActorError> {
        self.0.ask(RunnableActor { id: id.into(), actor }).await?
    }
    
    pub async fn shutdown(&self, id: impl Into<AnyId>) -> Result<(), ActorError> {
        self.0.tell(ShutdownActor { id: id.into() }).await?
    }

    pub async fn find<A: Actor>(&self, id: impl Into<AnyId>) -> Result<Option<ActorRef<A>>, ActorError> {
        self.0.ask(FindActor { id: id.into(), _mark: PhantomData }).await?
    }

    pub async fn find_or<A: Actor, I: Into<AnyId> + Copy, Fut>(&self, id: I, or_nothing: impl FnOnce(I) -> Fut) -> Result<ActorRef<A>, ActorError> 
        where Fut: Future<Output=A> + 'static + Send,
    {
        match self.0.ask(FindActor { id: id.into(), _mark: PhantomData }).await?? {
            Some(actor) => Ok(actor),
            None => {
                let data = or_nothing(id).await;
                
                self.0.ask(RunnableActor { id: id.into(), actor: data }).await?
            }
        }
    }
}

impl Clone for SupervisorRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl Actor for Supervisor {
    async fn activate(&mut self, _ctx: &mut Context) -> Result<(), ActorError> {
        tracing::info!("supervisor activate.");
        Ok(())
    }
}

impl<A: Actor> Handler<RunnableActor<A>> for Supervisor {
    type Accept = ActorRef<A>;
    type Rejection = ActorError;

    async fn handle(&mut self, mut msg: RunnableActor<A>, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();

        let refs = ActorRef::new(tx);

        if self.actors.insert(msg.id.clone(), refs.clone().into()).is_some() {
            return Err(ActorError::AlreadySpawned { id: msg.id })
        }

        let ctx = Context::new(ctx.supervisor());

        tokio::spawn(async move {
            let mut ctx = ctx;
            
            match msg.actor.activate(&mut ctx).await {
                Ok(_) => {
                    tracing::info!("spawned.");
                    while let Some(payload) = rx.recv().await {
                        if let Err(e) = payload.apply(&mut msg.actor, &mut ctx).await {
                            tracing::error!("{}", e);
                        }

                        if ctx.running_state().available_shutdown() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(name: "activation", "{}", e);
                }
            }
            
            tracing::warn!("shutdown.");
        }.instrument(tracing::info_span!("actor", id = %msg.id)));

        Ok(refs)
    }
}

impl Handler<ShutdownActor> for Supervisor {
    type Accept = ();
    type Rejection = ActorError;

    async fn handle(&mut self, msg: ShutdownActor, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        if self.actors.remove(&msg.id).is_none() {
            tracing::error!("target actor: [id={}] could not be found.", msg.id);
            return Err(ActorError::NotFoundActor { id: msg.id })
        }
        
        tracing::warn!("actor: [id={}] is now subject to shutdown, and if the ActorRef of the Actor is not used, this Actor will be shutdown immediately.", msg.id);
        Ok(())
    }
}

impl<A: Actor> Handler<FindActor<A>> for Supervisor {
    type Accept = Option<ActorRef<A>>;
    type Rejection = ActorError;

    async fn handle(&mut self, msg: FindActor<A>, _ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        self.actors.iter()
            .find(|(id, _)| PartialEq::eq(*id, &msg.id))
            .map(|(_, refs)| refs.clone())
            .map(|refs| refs.downcast::<A>())
            .transpose()
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

pub struct FindActor<A: Actor> {
    id: AnyId,
    _mark: PhantomData<A>
}

impl<A: Actor> Message for FindActor<A> {}