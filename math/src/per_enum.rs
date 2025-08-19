use serde::{Deserialize, Serialize};
use std::{
    array,
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone)]
pub struct PerEnum<E, T, const N: usize> {
    data: [T; N],
    _phantom: PhantomData<E>,
}

impl<E, T, const N: usize> PerEnum<E, T, N> {
    pub fn into_inner(self) -> [T; N] {
        self.data
    }
}

impl<E, T, const N: usize> PerEnum<E, T, N>
where
    E: TryFrom<usize>,
    E::Error: Debug,
{
    pub fn from_fn<F>(func: F) -> Self
    where
        F: Fn(E) -> T,
    {
        let data = array::from_fn(|i| func(i.try_into().unwrap()));
        Self {
            data,
            _phantom: PhantomData,
        }
    }

    pub fn try_from_fn<F>(func: F) -> Option<Self>
    where
        F: Fn(E) -> Option<T>,
    {
        let data = array::from_fn(|i| func(i.try_into().unwrap()));
        if data.iter().any(|opt| opt.is_none()) {
            return None;
        }
        let data = data.map(|opt| opt.unwrap());
        Some(Self {
            data,
            _phantom: PhantomData,
        })
    }
}

impl<E, T, const N: usize> PerEnum<E, T, N>
where
    E: Into<usize>,
{
    pub fn get(&self, e: E) -> &T {
        &self.data[e.into()]
    }

    pub fn get_mut(&mut self, e: E) -> &mut T {
        &mut self.data[e.into()]
    }
}

impl<E, T, const N: usize> Index<E> for PerEnum<E, T, N>
where
    E: Into<usize>,
{
    type Output = T;

    fn index(&self, index: E) -> &Self::Output {
        &self.data[index.into()]
    }
}

impl<E, T, const N: usize> IndexMut<E> for PerEnum<E, T, N>
where
    E: Into<usize>,
{
    fn index_mut(&mut self, index: E) -> &mut Self::Output {
        &mut self.data[index.into()]
    }
}

impl<E, T, const N: usize> Serialize for PerEnum<E, T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de, E, T, const N: usize> Deserialize<'de> for PerEnum<E, T, N>
where
    [T; N]: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            data: <[T; N]>::deserialize(deserializer)?,
            _phantom: PhantomData,
        })
    }
}
