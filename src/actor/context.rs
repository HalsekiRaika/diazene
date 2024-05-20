use crate::errors::ActorError;
use crate::system_v2::SupervisorRef;

pub struct Context {
    supervisor: SupervisorRef
}

// impl Context {
//     pub async fn shutdown(&self) -> Result<(), ActorError> {
//         self.supervisor.shutdown()
//     }
// }