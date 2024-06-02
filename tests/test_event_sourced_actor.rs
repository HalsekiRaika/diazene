use std::collections::{BTreeMap, HashSet};
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;


use uuid::{NoContext, Timestamp, Uuid};
use diazene::actor::{Context, Handler, Message};
use diazene::errors::ActorError;
use diazene::persistence::PersistentActor;
use diazene::persistence::event::{Event, EventSourced, Replay};

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

impl Event for BookEvent {
    const VERSION: &'static str = "0.1.0";
    type Actor = Book;
    fn apply(self, actor: &mut Self::Actor) {
        match self {
            BookEvent::Rental { id } => {
                actor.rental.insert(id);
            }
            BookEvent::Returned { id } => {
                actor.rental.remove(&id);
            }
            BookEvent::Archived => {}
        }
    }
}

impl Message for BookCommand {}

#[async_trait::async_trait]
impl PersistentActor for Book {
    async fn persist<M: Message>(&self, _msg: M, _ctx: &Context) -> Result<(), ActorError> {
        todo!()
    }
}

#[async_trait::async_trait]
impl EventSourced for Book {
    async fn activate(&mut self, _ctx: &mut Context) {
    }
}

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

async fn create_events() -> BTreeMap<i32, BookEvent> {
    let mut events = BTreeMap::new();

    let mut i = 0;
    loop {
        if i >= 100 {
            events.insert(100, BookEvent::Archived);
            break;
        }

        let id = PersonId::default();
        
        events.insert(i, BookEvent::Rental { id });
        i += 1;
        events.insert(i, BookEvent::Returned { id });
        i += 1;
    }
    
    events
}

#[tokio::test]
async fn test_replay() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,diazene=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    
    let events = create_events().await;
    events.iter().for_each(|(id, event)| tracing::debug!("id: {:>4}, event: {}", id, serde_json::to_string(event).unwrap()));
    
    let (_, mut book) = create_book();
    
    let evs = events.into_values().collect::<Vec<_>>();
    
    let now = Instant::now();
    
    book.replay(evs).await;
    
    let elapsed = now.elapsed().as_micros();
    
    tracing::debug!(name: "replay", "(took time {}ms) book={:?}", elapsed, book);
}