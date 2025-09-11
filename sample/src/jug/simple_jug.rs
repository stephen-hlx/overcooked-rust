use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::jug::{Jug, JugError};

#[derive(Debug)]
struct SimpleJug {
    inner: Arc<RwLock<InMemJug>>,
}

#[derive(Debug, PartialEq, Clone, Default)]
struct InMemJug {
    pub capacity: u8,
    pub occupancy: u8,
}

#[async_trait]
impl Jug for SimpleJug {
    async fn add_to(&self, other: Arc<dyn Jug>) -> Result<(), JugError> {
        let occupancy = self.inner.read().await.occupancy;
        let other_available_space = other.available_space().await?;

        other
            .add(std::cmp::min(other_available_space, occupancy))
            .await?;

        self.inner.write().await.occupancy = occupancy.saturating_sub(other_available_space);

        Ok(())
    }
    async fn add(&self, volume: u8) -> Result<(), JugError> {
        self.inner.write().await.occupancy += volume;
        Ok(())
    }
    async fn empty(&self) -> Result<(), JugError> {
        self.inner.write().await.occupancy = 0;
        Ok(())
    }
    async fn fill(&self) -> Result<(), JugError> {
        let capacity = self.inner.read().await.capacity;
        self.inner.write().await.occupancy = capacity;
        Ok(())
    }
    async fn available_space(&self) -> Result<u8, JugError> {
        let occupancy = self.inner.read().await.occupancy;
        Ok(self.inner.read().await.capacity - occupancy)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use test_case::test_case;
    use tokio::sync::RwLock;

    use crate::jug::{
        Jug,
        simple_jug::{InMemJug, SimpleJug},
    };

    #[tokio::test]
    async fn add_works() {
        let jug = SimpleJug {
            inner: Arc::new(RwLock::new(InMemJug {
                capacity: 10,
                occupancy: 4,
            })),
        };
        jug.add(2).await.unwrap();

        assert_eq!(jug.inner.read().await.occupancy, 6);
    }

    #[tokio::test]
    async fn available_space_works() {
        let jug = SimpleJug {
            inner: Arc::new(RwLock::new(InMemJug {
                capacity: 10,
                occupancy: 4,
            })),
        };

        assert_eq!(jug.available_space().await.unwrap(), 6);
    }

    #[tokio::test]
    async fn empty_empties_jug() {
        let jug = SimpleJug {
            inner: Arc::new(RwLock::new(InMemJug {
                capacity: 10,
                occupancy: 4,
            })),
        };
        jug.empty().await.unwrap();

        assert_eq!(jug.inner.read().await.occupancy, 0);
    }

    #[tokio::test]
    async fn fill_fills_up_jug() {
        let jug = SimpleJug {
            inner: Arc::new(RwLock::new(InMemJug {
                capacity: 10,
                occupancy: 4,
            })),
        };
        jug.fill().await.unwrap();

        assert_eq!(jug.inner.read().await.occupancy, 10);
    }

    #[test_case(jug3(0), jug5(0), jug3(0), jug5(0))]
    #[test_case(jug3(0), jug5(1), jug3(0), jug5(1))]
    #[test_case(jug3(0), jug5(5), jug3(0), jug5(5))]
    #[test_case(jug3(1), jug5(1), jug3(0), jug5(2))]
    #[test_case(jug3(1), jug5(4), jug3(0), jug5(5))]
    #[test_case(jug3(2), jug5(4), jug3(1), jug5(5))]
    #[test_case(jug3(1), jug5(5), jug3(1), jug5(5))]
    #[tokio::test]
    async fn add_to_works(
        from: InMemJug,
        to: InMemJug,
        expected_from: InMemJug,
        expected_to: InMemJug,
    ) {
        let from_in_mem = Arc::new(RwLock::new(from));
        let to_in_mem = Arc::new(RwLock::new(to));

        let from_simple_jug = SimpleJug {
            inner: from_in_mem.clone(),
        };
        let to_simple_jug = Arc::new(SimpleJug {
            inner: to_in_mem.clone(),
        });

        from_simple_jug.add_to(to_simple_jug).await.unwrap();

        assert_eq!(from_in_mem.read().await.clone(), expected_from);
        assert_eq!(to_in_mem.read().await.clone(), expected_to);
    }

    fn jug3(used: u8) -> InMemJug {
        InMemJug {
            capacity: 3,
            occupancy: used,
        }
    }

    fn jug5(used: u8) -> InMemJug {
        InMemJug {
            capacity: 5,
            occupancy: used,
        }
    }
}
