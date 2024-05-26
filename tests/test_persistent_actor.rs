use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::{NoContext, Timestamp, Uuid};
use diazene::actor::{Context, Handler, Message};
use diazene::persistence::{PersistenceBehavior, PersistentActor};
use diazene::system::ActorSystem;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct PersonId(Uuid);

impl Display for PersonId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Person({})", self.0)
    }
}

impl Default for PersonId {
    fn default() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Book {
    id: Uuid,
    title: String,
    rental: HashSet<PersonId>
}

#[derive(Debug, Clone)]
pub enum BookCommand {
    Rental { id: PersonId },
    Return { id: PersonId },
    Archive,
}

#[derive(Debug, Clone)]
pub enum Error {
    AlreadyExist { reason: String },
    NotFound { reason: String }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyExist { reason } => write!(f, "{}", reason),
            Error::NotFound { reason } => write!(f, "{}", reason),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BookEvent {
    Rental { id: PersonId },
    Returned { id: PersonId },
    Archived,
}

impl Message for BookCommand {}

impl PersistentActor for Book {}

impl Handler<BookCommand> for Book {
    type Accept = BookEvent;
    type Rejection = Error;

    async fn handle(&mut self, msg: BookCommand, ctx: &mut Context) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            BookCommand::Rental { id } => {
                if !self.rental.insert(id) {
                    return Err(Error::AlreadyExist {
                        reason: format!("The book is already on loan by {}.", id)
                    })
                }
                tracing::debug!("rental={}", id);
                Ok(BookEvent::Rental { id })
            }
            BookCommand::Return { id } => {
                if !self.rental.remove(&id) {
                    return Err(Error::NotFound {
                        reason: format!("This book is not on loan from {}.", id)
                    })
                }
                tracing::debug!("return={}", id);
                Ok(BookEvent::Returned { id })
            },
            BookCommand::Archive => {
                tracing::info!("book archived. (self shutdown)");
                ctx.shutdown();
                Ok(BookEvent::Archived)
            },
        }
    }
}


fn create_book() -> (Uuid, Book) {
    let id = Uuid::new_v4();
    let book = Book {
        id,
        title: "Charlie and the Chocolate Factory".to_string(),
        rental: Default::default(),
    };

    (id, book)
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,diazene=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    
    let system = ActorSystem::new();
    
    let (id, book) = create_book();
    
    let refs = system.spawn(id, book).await?;
    
    
    let ev = refs.ask(BookCommand::Rental { id: PersonId::default() }).await??;
    let ser = serde_json::to_string(&ev)?;
    tracing::debug!("{:?}", ser);
    
    
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    Ok(())
}