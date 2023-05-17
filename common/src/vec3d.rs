use std::collections::{hash_map::Entry, HashMap};

use crate::Pos3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Vec3D<T> {
    inner: HashMap<Pos3, T>,
}

impl<T> Vec3D<T> {
    pub fn get(&self, pos: &Pos3) -> Option<&T> {
        self.inner.get(pos)
    }
    pub fn insert(&mut self, pos: Pos3, data: T) {
        self.inner.insert(pos, data);
    }
    pub fn entry(&mut self, pos: Pos3) -> Entry<'_, Pos3, T> {
        self.inner.entry(pos)
    }
}
