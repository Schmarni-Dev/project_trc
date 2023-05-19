use serde::{Deserialize, Serialize, Serializer};
use std::sync::{Arc, Mutex};

pub struct ArcMutex<T: ?Sized>(pub Arc<Mutex<T>>);

impl<T> ArcMutex<T> {
    pub fn new(data: T) -> ArcMutex<T> {
        Self(Arc::new(Mutex::new(data)))
    }
    pub fn clone_arc(&self) -> ArcMutex<T> {
        Self(self.0.clone())
    }
}

impl<T: ?Sized + Serialize> Serialize for ArcMutex<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .lock()
            .expect("mutex is poisoned")
            .serialize(serializer)
    }
}

impl<'de, T: ?Sized + Deserialize<'de>> Deserialize<'de> for ArcMutex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(Arc::new(Mutex::new(T::deserialize(deserializer)?))))
    }
}
