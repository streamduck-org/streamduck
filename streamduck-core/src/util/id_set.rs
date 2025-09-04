use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdSetError {
    #[error("specified '{0}' id was already taken!")]
    IdAlreadyReserved(String)
}

#[derive(Clone)]
pub struct IdSet<T> {
    set: HashSet<T>
}

trait IdSetCompatible: Hash + Eq + ToString {}
impl<T: Hash + Eq + ToString> IdSetCompatible for T {}

impl<T: Hash + Eq + ToString> IdSet<T> {
    pub fn new() -> IdSet<T> {
        IdSet {
            set: HashSet::default()
        }
    }

    pub fn reserve(&mut self, id: T) -> Result<(), IdSetError> {
        if self.set.contains(&id) {
            return Err(IdSetError::IdAlreadyReserved(id.to_string()))
        }

        self.set.insert(id);
        Ok(())
    }
}

impl<T: IdSetCompatible> Deref for IdSet<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.set
    }
}

impl<T: IdSetCompatible> DerefMut for IdSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set
    }
}