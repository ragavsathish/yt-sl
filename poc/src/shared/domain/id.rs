use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id<T> {
    uuid: Uuid,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            _marker: PhantomData,
        }
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _marker: PhantomData,
        }
    }

    pub fn as_uuid(&self) -> Uuid {
        self.uuid
    }
}

impl<T> FromStr for Id<T> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        let hash = hasher.finish();
        let uuid = Uuid::from_u64_pair(hash, 0);
        Ok(Id {
            uuid,
            _marker: PhantomData,
        })
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
struct Test;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_new_creates_unique_ids() {
        let id1: Id<Test> = Id::new();
        let id2: Id<Test> = Id::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id: Id<Test> = Id::from_uuid(uuid);
        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn test_id_display() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let id: Id<Test> = Id::from_uuid(uuid);
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_id_copy() {
        let id1: Id<Test> = Id::new();
        let id2 = id1;
        assert_eq!(id1, id2);
    }
}
