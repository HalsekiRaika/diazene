use std::ops::Deref;
use std::sync::Arc;

pub use self::supervisor::*;

mod supervisor;

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
}

impl Deref for ActorSystem {
    type Target = SupervisorRef;
    fn deref(&self) -> &Self::Target {
        &self.0.supervisor
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
            supervisor: supervisor.activate(),
        }
    }
}
