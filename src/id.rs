use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dyn_hash::DynHash;

use crate::errors::ActorError;

pub trait Identifier: Any + DynHash + Debug + Sync + Send + Display + 'static {
    fn as_any(&self) -> &dyn Any;

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Sync + Send>;

    fn eq(&self, other: &dyn Identifier) -> bool;
}

dyn_hash::hash_trait_object!(Identifier);

#[derive(Debug)]
pub struct AnyId(Arc<dyn Identifier>);

impl AnyId {
    pub fn downcast_ref<T: Identifier>(&self) -> Result<&T, ActorError> {
        self.0
            .as_any()
            .downcast_ref::<T>()
            .ok_or(ActorError::DownCastFromAny)
    }

    pub fn downcast<T: Identifier + Copy>(self) -> Result<T, ActorError> {
        let id = self
            .0
            .as_any_arc()
            .downcast::<T>()
            .map_err(|_| ActorError::DownCastFromAny)?;
        Ok(*id)
    }
}

impl<T> Identifier for T
where
    T: Any + PartialEq + Eq + Display + Hash + Sync + Send + 'static + Copy + Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Sync + Send> {
        self
    }

    fn eq(&self, other: &dyn Identifier) -> bool {
        let Some(other) = other.as_any().downcast_ref::<T>() else {
            return false;
        };

        self.eq(other)
    }
}

impl Clone for AnyId {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Display for AnyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<Self> for AnyId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.0.as_ref())
    }
}

impl<T: Identifier> PartialEq<T> for AnyId {
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl Eq for AnyId {}

impl Hash for AnyId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: Identifier> From<T> for AnyId {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use uuid::Uuid;

    use crate::id::{AnyId, Identifier};

    #[test]
    fn init() {
        let uuid = Uuid::new_v4();
        let id: AnyId = uuid.into();

        println!("{}", id);

        let str_id = "abc123";
        let id: AnyId = str_id.into();

        println!("{}", id);
    }

    #[test]
    fn downcast() {
        let uuid = Uuid::new_v4();
        let copied = uuid;
        let id: AnyId = uuid.into();

        let id = id.downcast::<Uuid>().unwrap();

        assert_eq!(copied, id);

        let str_id = "abc123";
        let copied = str_id;
        let id: AnyId = str_id.into();

        let id = id.downcast::<&str>().unwrap();

        assert_eq!(copied, id);
    }

    #[test]
    #[should_panic]
    fn downcast_failure() {
        let uuid = Uuid::new_v4();
        let id: AnyId = uuid.into();

        let _panic = id.downcast::<&str>().unwrap();
    }

    #[test]
    fn downcast_ref() {
        let uuid = Uuid::new_v4();
        let id: AnyId = uuid.into();

        let id = id.downcast_ref::<Uuid>().unwrap();
        println!("{}", id);
    }

    #[test]
    #[should_panic]
    fn downcast_ref_failure() {
        let uuid = Uuid::new_v4();
        let id: AnyId = uuid.into();

        let _panic = id.downcast_ref::<&str>().unwrap();
    }

    #[test]
    fn eq() {
        let uuid = Uuid::new_v4();
        let copied = uuid;
        let id: AnyId = uuid.into();

        assert_eq!(id, copied);
    }

    #[test]
    fn eq_unmatched_value() {
        let uuid = Uuid::new_v4();
        let other_uuid = Uuid::new_v4();
        let id: AnyId = uuid.into();

        assert_ne!(id, other_uuid);

        let id = id.downcast::<Uuid>().unwrap();

        assert_ne!(id, other_uuid);
    }

    #[test]
    fn eq_refs() {
        let uuid = Uuid::new_v4();
        let copied = uuid;

        println!("copy: {}", copied);

        let id: AnyId = uuid.into();

        let mut set = HashSet::new();
        set.insert(id);
        let set = Set(set);

        let id = set.find(&copied);

        println!("find: {}", id);

        assert_eq!(id, &copied);

        pub struct Set(HashSet<AnyId>);

        impl Set {
            pub fn find(&self, id: &impl Identifier) -> &AnyId {
                self.0.iter().find(|item| PartialEq::eq(item, &id)).unwrap()
            }
        }
    }
}
