#![allow(unused)]
use std::sync::Arc;

use async_trait::async_trait;

mod simple_jug;

#[async_trait]
trait Jug: Send + Sync {
    async fn add_to(&self, other: Arc<dyn Jug>) -> Result<(), JugError>;
    async fn add(&self, volume: u8) -> Result<(), JugError>;
    async fn empty(&self) -> Result<(), JugError>;
    async fn fill(&self) -> Result<(), JugError>;
    async fn available_space(&self) -> Result<u8, JugError>;
}

#[derive(Debug, thiserror::Error)]
#[error("JugError {0}")]
struct JugError(String);
