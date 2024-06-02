use std::future::Future;
use crate::persistence::event::Event;
use crate::persistence::PersistError;

pub trait JournalProvider: 'static + Sync + Send {
    fn append(&mut self, event: &impl Event) -> impl Future<Output=Result<(), PersistError>> + Send;
}

pub(crate) trait ExtractJournalProvider: 'static + Sync + Send {
    fn journal_provider(&self);
}