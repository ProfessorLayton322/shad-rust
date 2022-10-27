#![forbid(unsafe_code)]

use std::mem::replace;
use std::{borrow::Borrow, iter::FromIterator, ops::Index};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        FlatMap(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    fn get_position<Q: ?Sized>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.0.binary_search_by(|pair| pair.0.borrow().cmp(key))
    }

    fn resort(&mut self) {
        self.0.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Returns None if key was not present, or Some(prev_value) if it was.
        match self.get_position(&key) {
            Ok(position) => Some(replace(&mut self.0[position].1, value)),
            Err(position) => {
                self.0.insert(position, (key, value));
                self.resort();
                None
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        match self.get_position(key) {
            Ok(position) => Some(&self.0[position].1),
            Err(_) => None,
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        match self.get_position(key) {
            Ok(position) => Some(self.0.remove(position).1),
            Err(_) => None,
        }
    }

    pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        match self.get_position(key) {
            Ok(position) => Some(self.0.remove(position)),
            Err(_) => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<Q: ?Sized + Ord, K: Ord + Borrow<Q>, V> Index<&Q> for FlatMap<K, V> {
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        match self.get_position(index) {
            Ok(position) => &self.0[position].1,
            Err(_) => panic!("No such key"),
        }
    }
}

impl<K: Ord, V> Extend<(K, V)> for FlatMap<K, V> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        self.0.extend(iter);
        self.resort();
        self.0.reverse();
        self.0.dedup_by(|a, b| a.0 == b.0);
        self.0.reverse();
    }
}

impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(mut input: Vec<(K, V)>) -> Self {
        input.sort_by(|a, b| a.0.cmp(&b.0));
        input.reverse();
        input.dedup_by(|a, b| a.0 == b.0);
        input.reverse();
        FlatMap::<K, V>(input)
    }
}

impl<K: Ord, V> From<FlatMap<K, V>> for Vec<(K, V)> {
    fn from(input: FlatMap<K, V>) -> Self {
        input.0
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let data: Vec<(K, V)> = iter.into_iter().collect();
        FlatMap::<K, V>::from(data)
    }
}

impl<K: Ord, V> IntoIterator for FlatMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
