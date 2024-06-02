use crate::actor::{RunningState, State};
use crate::persistence::SnapshotModule;
use crate::system::SupervisorRef;

pub struct Context {
    running: RunningState,
    supervisor: SupervisorRef,
    
    #[cfg(feature = "persistence")]
    persistence: crate::persistence::Journal,
    
    #[cfg(feature = "persistence")]
    pub(crate) snapshot_module: SnapshotModule
}

impl Context {
    pub(crate) fn new(supervisor: SupervisorRef) -> Context {
        Self { 
            running: RunningState::default(), 
            supervisor,
            
            #[cfg(feature = "persistence")]
            persistence: crate::persistence::Journal::new(),
            
            // #[cfg(feature = "persistence")]
            // snapshot_module: 
        }
    }
}

impl Context {
    pub fn shutdown(&mut self) {
        self.running.switch(|prev| { *prev = State::Shutdown });
    }
}

impl Context {
    pub fn supervisor(&self) -> SupervisorRef {
        self.supervisor.clone()
    }
    
    pub(crate) fn running_state(&self) -> &RunningState {
        &self.running
    }
    
    #[cfg(feature = "persistence")]
    pub fn persistence(&self) -> &crate::persistence::Journal {
        &self.persistence
    }

    #[cfg(feature = "persistence")]
    pub fn persistence_mut(&mut self) -> &mut crate::persistence::Journal {
        &mut self.persistence
    }
}
