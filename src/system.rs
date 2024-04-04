use std::collections::HashMap;
use std::future::IntoFuture;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::actor::{Actor, ActorRef, AnyRef, Applier};
use crate::errors::ActorError;
use crate::id::AnyId;
use crate::Identifier;

pub struct ActorSystem {
    inner: Arc<Mutex<InnerSystem>>,
}

pub(crate) struct InnerSystem {
    actors: HashMap<AnyId, AnyRef>,
}

impl ActorSystem {
    pub async fn spawn<A: Actor>(
        &self,
        id: impl Into<AnyId>,
        actor: A,
    ) -> Result<ActorRef<A>, ActorError> {
        let refs = self.inner.lock().await.spawn(id.into(), actor).await;
        Ok(refs)
    }

    pub async fn find<A: Actor>(&self, id: &impl Identifier) -> Option<ActorRef<A>> {
        self.inner.lock().await.find::<A>(id).await
    }

    pub async fn find_or<A: Actor>(&self, id: impl Identifier) -> FindOr<A> {
        FindOr {
            exact: self.find::<A>(&id).await,
            id: AnyId::new(id),
            system: self.inner.clone(),
        }
    }
}

impl InnerSystem {
    pub async fn spawn<A: Actor>(&mut self, id: impl Into<AnyId>, mut actor: A) -> ActorRef<A> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Box<dyn Applier<A>>>();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = msg.apply(&mut actor).await {
                    tracing::error!("{}", e);
                }
            }
        });

        let refs = ActorRef::new(tx);

        self.actors.insert(id.into(), refs.clone().into());

        refs
    }

    // fixme: Do not use `Any` directly, but will replace this process with a Trait that derive from `Any`.
    //        this premise is that this issue <https://github.com/rust-lang/rust/issues/65991> needs to be resolved.
    pub async fn find<A: Actor>(&self, id: &impl Identifier) -> Option<ActorRef<A>> {
        self.actors
            .iter()
            .find(|(running, _)| PartialEq::eq(running, &id))
            .map(|(_, refs)| refs.clone())
            .map(|refs| refs.downcast::<A>())
            .transpose()
            .ok()
            .flatten()
    }
}

pub struct FindOr<A: Actor> {
    id: AnyId,
    exact: Option<ActorRef<A>>,
    system: Arc<Mutex<InnerSystem>>,
}

impl<A: Actor> FindOr<A> {
    pub async fn spawn(self, f: impl FnOnce() -> A) -> Result<ActorRef<A>, ActorError> {
        match self.exact {
            None => Ok(self.system.lock().await.spawn(self.id, f()).await),
            Some(a) => Ok(a),
        }
    }

    pub async fn spawn_async<Fut: IntoFuture<Output = A>>(
        self,
        fut: impl FnOnce() -> Fut,
    ) -> Result<ActorRef<A>, ActorError> {
        match self.exact {
            None => Ok(self.system.lock().await.spawn(self.id, fut().await).await),
            Some(a) => Ok(a),
        }
    }
}

impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerSystem::default())),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for InnerSystem {
    fn default() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }
}
