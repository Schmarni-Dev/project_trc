use derive_more::{Deref, DerefMut};
use rand::random;
use std::collections::HashMap;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Session(u32);

#[derive(Clone, Default)]
pub struct Sessions {
    used_session_ids: Arc<Mutex<HashSet<u32>>>,
}

impl Sessions {
    pub async fn acquire_session(&self) -> Session {
        let mut new_id: u32 = random();
        let mut ids = self.used_session_ids.lock().await;
        while ids.contains(&new_id) {
            new_id = random();
        }
        ids.insert(new_id);
        Session(new_id)
    }

    pub async fn free_session(&self, session: Session) -> bool {
        let mut ids = self.used_session_ids.lock().await;
        ids.remove(&session.0)
    }
}

#[derive(Deref, DerefMut)]
pub struct SessionMap<T>(HashMap<Session, T>);
impl<T> Default for SessionMap<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T> SessionMap<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}
