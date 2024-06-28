#![deny(unsafe_code)]

pub mod actor;
pub mod errors;
pub mod system;

#[cfg(feature = "persistence")]
pub mod persistence;
mod identifier;

#[cfg(feature = "re-export")]
pub use async_trait::async_trait;