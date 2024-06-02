use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::actor::{Actor, Context, Message};
use crate::errors::ActorError;

#[async_trait::async_trait]
pub trait PersistentActor: 'static + Sync + Send
    where Self: Serialize + DeserializeOwned
{
    async fn persist<M: Message>(&self, msg: M, ctx: &mut Context) -> Result<(), ActorError>;
}

pub mod safety {
    use super::PersistentActor as NotSafetyPersistentActor;
    
    pub trait Sealed {}
    
    pub(crate) trait PersistentActor: Sealed {
        
    }
    
    impl<T> PersistentActor for T 
        where T: ?Sized + NotSafetyPersistentActor
    {
        
    }
    
    impl<T> Sealed for T where T: ?Sized + NotSafetyPersistentActor {}
}

#[async_trait::async_trait]
impl<A: PersistentActor> Actor for A {
}
