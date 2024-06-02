#![allow(unused)]

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use uuid::Uuid;

use diazene::actor::{Actor, Context, Handler, Message, Terminate};
use diazene::actor::behavior::RegularBehavior;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct PersonId(Uuid);

impl Display for PersonId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Person({})", self.0)
    }
}

impl Default for PersonId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum BookEvent {
    Rental { id: PersonId },
    Returned { id: PersonId },
    Archived,
}

impl Message for BookCommand {}

impl Actor for Book {}

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

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
                  .with_filter(tracing_subscriber::EnvFilter::new("test=trace,diazene=trace"))
                  .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .init();
    let system = diazene::system::ActorSystem::new();

    let (id, book) = create_book();
    
    let book_ref = system.spawn(id, book).await?;

    tracing::debug!("=-=-=- Success -=-=-=");
    
    for _ in 0..5 {
        let id = PersonId::default();
        
        let res = book_ref.ask(BookCommand::Rental { id }).await?;
        tracing::debug!("{:?}", res);
        
        let res = book_ref.ask(BookCommand::Return { id }).await?;
        tracing::debug!("{:?}", res);
    }

    tracing::debug!("=-=-=- Failure -=-=-=");

    let person_id = PersonId::default();
    
    for _ in 0..3 {
        let res = book_ref.ask(BookCommand::Return { id: PersonId::default() }).await?;
        tracing::debug!("{:?}", res);
        
        let res = book_ref.ask(BookCommand::Rental { id: person_id }).await?;
        tracing::debug!("{:?}", res);
    }

    tracing::debug!("=-=-=- Shutdown -=-=-=");
    
    system.shutdown(id).await?;

    tokio::time::sleep(Duration::from_secs(3)).await;
    
    tokio::task::spawn_blocking(|| { drop(book_ref) }).await?;
    
    let (id, book) = create_book();
    
    let book_ref = system.spawn(id, book).await?;
    
    let ev = book_ref.ask(BookCommand::Archive).await?;
    
    tracing::debug!("{:?}", ev);

    let (id, book) = create_book();

    let book_ref = system.spawn(id, book).await?;

    book_ref.tell(Terminate).await??;
    
    
    Ok(())
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