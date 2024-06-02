use std::sync::Arc;
use crate::persistence::provider::SnapshotProvider;
use crate::persistence::provider::safety::SnapshotProvider as SafetySnapshotProvider;

pub struct SnapshotModule {
    pool: Arc<dyn SafetySnapshotProvider>
}

impl Clone for SnapshotModule {
    fn clone(&self) -> Self {
        Self { pool: Arc::clone(&self.pool) }
    }
}

impl SnapshotModule {
    pub fn new<P: SnapshotProvider>(provider: P) -> SnapshotModule {
        Self { pool: Arc::new(provider) }
    }
    
    pub fn a(&self) {
        self.pool.persist().unwrap()
    }
}