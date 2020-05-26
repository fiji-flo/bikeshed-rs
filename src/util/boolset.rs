use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::hash::Hash;

// Like a hash table, the bool set is a data structure where keys
// can be explicitly set to true or false.

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BoolSet<K>
where
    K: Hash + Eq + Default + Clone,
{
    map: HashMap<K, bool>,
    default_val: bool,
}

impl<K> BoolSet<K>
where
    K: Hash + Eq + Default + Clone,
{
    pub fn new_with_default(default_val: bool) -> Self {
        BoolSet {
            map: HashMap::new(),
            default_val,
        }
    }

    // Maps the key to the value in bool set, overwriting any existing mapping for the key.
    pub fn insert(&mut self, key: K, val: bool) {
        self.map.insert(key, val);
    }

    // Returns the value for the key in bool set.
    // If no value is found for the key, then default value is returned.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        *self.map.get(key).unwrap_or(&self.default_val)
    }

    pub fn update(&mut self, other: &Self) {
        self.map.extend(other.map.clone());
        self.default_val = other.default_val;
    }
}

#[cfg(test)]
mod tests {
    use super::BoolSet;

    #[test]
    fn test_bool_set() {
        {
            let bs = BoolSet::<&str>::new_with_default(true);
            assert_eq!(bs.get("a"), true);
        }
        {
            let bs = BoolSet::<&str>::new_with_default(false);
            assert_eq!(bs.get("a"), false);
        }
        {
            let mut bs = BoolSet::<&str>::new_with_default(true);
            bs.insert("a", true);
            assert_eq!(bs.get("a"), true);
            assert_eq!(bs.get("b"), true);
        }
        {
            let mut bs = BoolSet::<&str>::new_with_default(false);
            bs.insert("a", true);
            assert_eq!(bs.get("a"), true);
            assert_eq!(bs.get("b"), false);
        }
        {
            let mut bs = BoolSet::<&str>::new_with_default(false);
            bs.insert("a", true);
            assert_eq!(bs.get("a"), true);
            assert_eq!(bs.get("b"), false);

            let mut other = BoolSet::<&str>::new_with_default(true);
            other.insert("a", false);
            other.insert("b", true);
            bs.update(&other);
            assert_eq!(bs.get("a"), false);
            assert_eq!(bs.get("b"), true);
        }
    }
}
