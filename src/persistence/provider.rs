use crate::persistence::{PersistError, safety::PersistentActor};

#[async_trait::async_trait]
pub trait SnapshotProvider: 'static + Sync + Send {
    async fn persist<P: PersistentActor>(&self, payload: &P) -> Result<(), PersistError>;
}

pub mod safety {
    use crate::persistence::PersistError;
    use crate::persistence::safety::PersistentActor;
    
    use super::SnapshotProvider as NotSafeSnapshotProvider;
    
    pub trait Sealed {}
    
    #[async_trait::async_trait]
    pub trait SnapshotProvider: Sealed {
        async fn persist(&self, payload: &dyn PersistentActor) -> Result<(), PersistError>;
    }
    
    #[async_trait::async_trait]
    impl<T> SnapshotProvider for T 
        where T: ?Sized + NotSafeSnapshotProvider
    {
        async fn persist(&self, payload: &dyn PersistentActor) -> Result<(), PersistError> {
            Ok(())
        }
    }
    
    impl<T> Sealed for T where T: ?Sized + NotSafeSnapshotProvider {}
}

// pub trait ExtractSnapshotProvider: 'static + Sync + Send {
//     fn snapshot_provider(ctx: &Context) -> &SnapshotModule;
// }
// 
// impl<A: PersistentActor> ExtractSnapshotProvider for A {
//     fn snapshot_provider(ctx: &Context) -> &SnapshotModule {
//         &ctx.snapshot_module
//     }
// }