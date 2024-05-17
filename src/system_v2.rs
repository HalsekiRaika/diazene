use std::sync::Arc;

use anyid::AnyId;

use crate::actor::{Actor, ActorRef};
use crate::errors::ActorError;

mod supervisor;

pub use self::{
    supervisor::*
};

pub struct ActorSystem(pub(crate) Arc<System>);

impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl ActorSystem {
    pub fn new() -> ActorSystem {
        Self(Arc::new(System::new()))
    }
    
    pub async fn spawn<A: Actor>(&self, id: impl Into<AnyId>, actor: A) -> Result<ActorRef<A>, ActorError> {
        self.0.supervisor.spawn(id, actor).await
    }
    
    pub async fn shutdown(&self, id: impl Into<AnyId>) -> Result<(), ActorError> {
        self.0.supervisor.shutdown(id).await
    }
}

pub(crate) struct System {
    pub(crate) supervisor: SupervisorRef,
}

impl System {
    pub fn new() -> System {
        Self::default()
    }
}

impl Default for System {
    fn default() -> Self {
        let supervisor = Supervisor::new();
        
        Self {
            supervisor: supervisor.self_activate(),
        }
    }
}
