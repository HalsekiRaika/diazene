use std::future::Future;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::persistence::error::PersistError;

pub struct Journal();



impl Journal {
    pub fn new() -> Journal {
        Self()
    }

    pub async fn persist<P>(&mut self, _payload: &P) -> Result<(), PersistError>
        where P: Serialize + DeserializeOwned
    {
        tracing::debug!("auto persistence");
        #[cfg(test)]
        {
            tracing::debug!("on test");
            let ser = serde_json::to_string(_payload);
            tracing::debug!(name: "unimplemented", "payload={:?}", ser);
        }
        Ok(())
    }
}

