use crate::actor::{Actor, ActorRef};
use crate::errors::ActorError;
use crate::id::AnyId;
use crate::system::ActorSystem;

pub async fn spawn<A: Actor>(
    id: impl Into<AnyId>,
    actor: A,
    system: &ActorSystem,
) -> Result<ActorRef<A>, ActorError> {
    system.spawn(id, actor).await
}
