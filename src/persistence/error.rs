use std::fmt::{Display, Formatter};
use crate::errors::ActorError;

#[derive(Debug)]
pub enum PersistError {
    
}

impl Display for PersistError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for PersistError {}

impl From<PersistError> for ActorError {
    fn from(e: PersistError) -> Self {
        Self::Persist(e)
    }
}