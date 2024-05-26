use crate::actor::{RunningState, State};
use crate::system::SupervisorRef;

pub struct Context {
    supervisor: SupervisorRef
    running: RunningState,
    supervisor: SupervisorRef,
}

impl Context {
    pub(crate) fn new(supervisor: SupervisorRef) -> Context {
        Self { 
            running: RunningState::default(), 
            supervisor,
            
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
    
}
