use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::ops::{Bound, Range, RangeBounds};

/// Leaks because of the iterator
#[derive(Clone, Copy)]
pub struct Interval<K> {
    start: K,
    end: K,
}

impl<K: Ord> Interval<K> {
    fn overlaps<Q>(&self, other: &Interval<Q>) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        // find which starts first, that will be 1 the other 2
        // get the end of 1 and the start of 2
        let (e1, s2) = match self.start.borrow() < &other.start {
            true => (self.end.borrow(), &other.start),
            _ => (&other.end, self.start.borrow()),
        };

        // if 2 starts before 1 ends then the intervals overlap
        s2 < e1
    }

    fn contains(&self, pt: &K) -> bool {
        &self.start <= pt && pt < &self.end
    }
}

impl<K> Borrow<K> for Interval<K> {
    fn borrow(&self) -> &K {
        &self.start
    }
}

impl<K> From<Range<K>> for Interval<K> {
    fn from(r: Range<K>) -> Self {
        Interval {
            start: r.start,
            end: r.end,
        }
    }
}

impl<K: PartialEq> PartialEq for Interval<K> {
    fn eq(&self, other: &Self) -> bool {
        self.start.eq(&other.start)
    }
}

impl<K: Eq> Eq for Interval<K> {}

impl<K: PartialOrd> PartialOrd for Interval<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start.partial_cmp(&other.start)
    }
}

impl<K: Ord> Ord for Interval<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

enum RefBound<'a, K> {
    Lower(&'a K),
    Upper(&'a K),
}

impl<K: Ord> RangeBounds<K> for RefBound<'_, K> {
    fn start_bound(&self) -> Bound<&K> {
        match self {
            RefBound::Lower(i) => Bound::Included(i),
            RefBound::Upper(_) => Bound::Unbounded,
        }
    }

    fn end_bound(&self) -> Bound<&K> {
        match self {
            RefBound::Lower(_) => Bound::Unbounded,
            RefBound::Upper(i) => Bound::Included(i),
        }
    }
}

pub struct IntervalMap<K, V> {
    tree: BTreeMap<Interval<K>, V>,
}

#[derive(Debug)]
pub struct Entry<K, V> {
    pub start: K,
    pub end: K,
    pub value: V,
}

pub struct EntryIterator<I> {
    iter: I,
}

impl<'a, K, V, I> Iterator for EntryIterator<I>
where
    K: 'a,
    V: 'a,
    I: Iterator<Item = (&'a Interval<K>, &'a V)>,
{
    type Item = Entry<&'a K, &'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(Interval { start, end }, value)| Entry { start, end, value })
    }
}

impl<K: Ord, V> IntervalMap<K, V> {
    pub fn new() -> Self {
        IntervalMap {
            tree: BTreeMap::new(),
        }
    }

    fn non_overlapping<'a>(&'a self, interval: &Interval<K>) -> Result<(), ()> {
        // First ensure that this doesn't overlap with an existing range
        let prev_point = self
            .tree
            .range(RefBound::Upper(interval))
            .rev()
            .next()
            .map(|(k, _)| k)
            .filter(|p| p.overlaps(&interval));
        if let Some(_) = prev_point {
            return Err(());
        }

        let next_point = self
            .tree
            .range(RefBound::Lower(interval))
            .next()
            .map(|(k, _)| k)
            .filter(|p| p.overlaps(&interval));
        if let Some(_) = next_point {
            return Err(());
        }

        Ok(())
    }

    pub fn insert(&mut self, key: Range<K>, value: V) -> Result<(), ()> {
        let interval = key.into();
        self.non_overlapping(&interval)?;
        self.tree.insert(interval, value);
        Ok(())
    }

    pub fn get_entry(&self, key: &K) -> Option<Entry<&K, &V>> {
        self.tree
            .range(RefBound::Upper(key))
            .rev()
            .next()
            .filter(|(i, _)| i.contains(key))
            .map(|(Interval { start, end }, v)| Entry {
                start,
                end,
                value: v,
            })
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_entry(key).map(|e| e.value)
    }

    pub fn contains_key(&self, key: &K) -> bool{
        self.get_entry(key).is_some()
    }

    pub fn remove_by_start(&mut self, key: &K) -> Option<V> {
        self.tree.remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = Entry<&K, &V>> {
        EntryIterator{iter: self.tree.iter()}
    }
}

impl<K: Ord + Clone, V> IntervalMap<K, V> {
    pub fn remove_containing_interval(&mut self, key: &K) -> Option<V> {
        let i = self
            .tree
            .range(RefBound::Upper(key))
            .rev()
            .next()
            .map(|(i, _)| i)
            .filter(|i| i.contains(key));

        if let Some(i) = i {
            let s = i.start.clone();
            self.remove_by_start(&s)
        } else {
            None
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    #[test]
    fn insert_success() {
        let mut map = IntervalMap::new();
        map.insert(0..10, 1).expect("first");
        map.insert(11..12, 2).expect("non overlapping");
    }

    #[test]
    fn insert_duplicate() {
        let mut map = IntervalMap::new();
        map.insert(0..10, 1).expect("first");
        map.insert(0..10, 2).expect_err("identical");
    }

    #[test]
    fn insert_overlapping() {
        let mut map = IntervalMap::new();
        map.insert(0..10, 1).expect("first");
        map.insert(1..1, 2).expect_err("overlapping");
    }

    #[test]
    fn lookups() {
        let mut map = IntervalMap::new();
        map.insert(0..10, 1).expect("first");
        map.insert(11..12, 2).expect("non overlapping");

        assert_eq!(1, *map.get_entry(&3).expect("valid key").value);
        assert_eq!(2, *map.get_entry(&11).expect("valid key").value);

        map.insert(14..22, 3).expect("non overlapping");
        assert_eq!(1, *map.get_entry(&3).expect("valid key").value);
        assert_eq!(2, *map.get_entry(&11).expect("valid key").value);
        assert_eq!(3, *map.get_entry(&15).expect("valid key").value);

        assert!(map.get_entry(&22).is_none());
    }

    #[test]
    fn remove() {
        let mut map = IntervalMap::new();
        map.insert(0..10, 1).expect("first");
        map.insert(11..12, 2).expect("non overlapping");

        assert_eq!(1, *map.get_entry(&3).expect("valid key").value);
        assert_eq!(2, *map.get_entry(&11).expect("valid key").value);

        assert_eq!(1, map.remove_containing_interval(&3).expect("exists"));
        assert!(map.get_entry(&5).is_none());

        map.insert(0..10, 1).expect("non overlapping");
        assert_eq!(2, map.remove_by_start(&11).expect("valid key"));
        assert!(map.get_entry(&11).is_none());
    }
}
