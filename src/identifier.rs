use std::sync::Arc;

#[derive(Debug)]
pub struct ActorId {
    id: Arc<str>,
}

pub trait IntoActorId {
    fn into_actor_id(self) -> ActorId;
}

pub trait ToActorId {
    fn to_actor_id(&self) -> ActorId;
}

impl<T: ToString + Sync + Send> IntoActorId for T {
    fn into_actor_id(self) -> ActorId {
        self.to_string().into()
    }
}

impl<T: ToString + Sync + Send> ToActorId for T {
    fn to_actor_id(&self) -> ActorId {
        self.to_string().into()
    }
}