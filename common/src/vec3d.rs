use std::{
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData, fmt::{self, Debug},
};

use crate::Pos3;
use serde::{ser::SerializeSeq, Deserialize, Serialize, de::{SeqAccess, Visitor}, Deserializer};

pub struct Vec3D<T> {
    inner: HashMap<Pos3, T>,
}

impl<T> Vec3D<T> {
    pub fn new() -> Vec3D<T> {
        Vec3D {
            inner: HashMap::new(),
        }
    }
    pub fn get(&self, pos: &Pos3) -> Option<&T> {
        self.inner.get(pos)
    }
    pub fn insert(&mut self, pos: Pos3, data: T) {
        self.inner.insert(pos, data);
    }
    pub fn entry(&mut self, pos: Pos3) -> Entry<'_, Pos3, T> {
        self.inner.entry(pos)
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Pos3, T> {
        self.inner.iter()
    }
}

impl<T: Clone> Clone for Vec3D<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<T: Debug> Debug for Vec3D<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
#[derive(Serialize, Deserialize)]
struct Storage<T>(Pos3, T);

struct Vec3DVisitor<T> {
    marker: PhantomData<fn() -> Vec3D<T>>,
}

impl<T> Vec3DVisitor<T> {
    fn new() -> Self {
        Vec3DVisitor {
            marker: PhantomData,
        }
    }
}

impl<T> Serialize for Vec3D<T>
where
    T: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.inner.len()))?;
        for (pos, data) in self.inner.iter() {
            let storage = Storage(pos.clone(), data.clone());
            seq.serialize_element(&storage)?;
        }
        seq.end()
    }
}
impl<'de, T> Visitor<'de> for Vec3DVisitor<T>
where
    T: Deserialize<'de>,
{
    // The type that our Visitor is going to produce.
    type Value = Vec3D<T>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: SeqAccess<'de>,
    {
        let mut map = Vec3D { inner: HashMap::with_capacity(access.size_hint().unwrap_or(0))};

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some(Storage::<T>(pos, data)) = access.next_element()? {
            map.insert(pos, data);
        }

        Ok(map)
    }
}

// This is the trait that informs Serde how to deserialize MyMap.
impl<'de, T> Deserialize<'de> for Vec3D<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_seq(Vec3DVisitor::new())
    }
}
