#![allow(dead_code)]

use nos::actor::{Actor, Handler, Message};
use nos::system::ActorSystem;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug)]
pub struct Book {
    id: Uuid,
    title: String,
    stock: u32,
}

#[derive(Debug, Clone)]
pub struct User {
    id: Uuid,
    name: String,
    rental: HashSet<Uuid>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            rental: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub enum KernelError {
    InvalidValue,
}

impl Display for KernelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KernelError")
    }
}

impl Error for KernelError {}

pub enum UserCommand {
    Rental { book: Uuid },
}

#[derive(Debug)]
pub enum UserEvent {
    Rental { book: Uuid },
}

impl Actor for Book {}

impl Actor for User {}

impl Message for UserCommand {}

impl Handler<UserCommand> for User {
    type Accept = UserEvent;
    type Rejection = KernelError;
    
    async fn handle(&mut self, msg: UserCommand) -> Result<Self::Accept, Self::Rejection> {
        match msg {
            UserCommand::Rental { book } => {
                self.rental.insert(book);
                println!("{:?}", self);
                Ok(UserEvent::Rental { book })
            }
        }
    }
}

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let system = ActorSystem::default();
    let user = User::default();
    let id = user.id;
    
    let actor = system
        .spawn::<User>(id, user.clone())
        .await?;
    
    let res = actor
        .ask(UserCommand::Rental {
            book: Uuid::new_v4(),
        })
    
        .await??;
    
    let already = system
        .spawn::<User>(id, user.clone())
        .await;
    
    assert!(already.is_err(), "user actor id = {} was already activate.", id);
    
    println!("{:?}", res);
    
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    Ok(())
}
