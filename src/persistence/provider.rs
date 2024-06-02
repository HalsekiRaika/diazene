#[async_trait::async_trait]
pub trait PersistenceProvider: 'static + Sync + Send {
    async fn persist(&mut self, payload: )
}

