//! Bounded collection types to prevent unbounded memory growth.

use std::ops::{Deref, DerefMut};

/// A vector with a maximum capacity that evicts oldest elements when full.
///
/// When the vector reaches capacity, adding a new element removes the oldest
/// 10% of elements to make room efficiently (avoiding per-element eviction overhead).
#[derive(Debug, Clone)]
pub struct BoundedVec<T> {
    inner: Vec<T>,
    max_capacity: usize,
}

impl<T> BoundedVec<T> {
    /// Create a new bounded vector with the specified maximum capacity.
    ///
    /// # Panics
    /// Panics if `max_capacity` is 0.
    pub fn new(max_capacity: usize) -> Self {
        assert!(max_capacity > 0, "max_capacity must be greater than 0");
        Self {
            inner: Vec::new(),
            max_capacity,
        }
    }

    /// Create a new bounded vector with pre-allocated capacity.
    pub fn with_capacity(max_capacity: usize, initial_capacity: usize) -> Self {
        assert!(max_capacity > 0, "max_capacity must be greater than 0");
        Self {
            inner: Vec::with_capacity(initial_capacity.min(max_capacity)),
            max_capacity,
        }
    }

    /// Push an element, evicting oldest elements if at capacity.
    ///
    /// When at capacity, removes the oldest 10% of elements (minimum 1)
    /// to make room efficiently.
    pub fn push(&mut self, item: T) {
        if self.inner.len() >= self.max_capacity {
            // Remove oldest 10% (minimum 1) for efficiency
            let to_remove = (self.max_capacity / 10).max(1);
            self.inner.drain(0..to_remove);
        }
        self.inner.push(item);
    }

    /// Get the maximum capacity.
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }

    /// Get the current length.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get a reference to the last element.
    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }

    /// Get a mutable reference to the last element.
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.inner.last_mut()
    }

    /// Iterate over elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.inner.iter()
    }

    /// Iterate mutably over elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.inner.iter_mut()
    }

    /// Get element by index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Get mutable element by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.inner.get_mut(index)
    }

    /// Pop the last element.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    /// Replace contents with items from a Vec, keeping only the last `max_capacity` items.
    pub fn replace_from_vec(&mut self, items: Vec<T>) {
        self.inner.clear();
        let skip = items.len().saturating_sub(self.max_capacity);
        self.inner.extend(items.into_iter().skip(skip));
    }

    /// Create from a Vec, keeping only the last `max_capacity` items.
    pub fn from_vec(items: Vec<T>, max_capacity: usize) -> Self {
        let skip = items.len().saturating_sub(max_capacity);
        let inner: Vec<T> = items.into_iter().skip(skip).collect();
        Self { inner, max_capacity }
    }

    /// Extend with items from an iterator.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

// Allow transparent access to inner vec methods via Deref
impl<T> Deref for BoundedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for BoundedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Default for BoundedVec<T> {
    fn default() -> Self {
        // Default to a reasonable capacity
        Self::new(1000)
    }
}

// Allow indexing
impl<T> std::ops::Index<usize> for BoundedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<usize> for BoundedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push() {
        let mut vec = BoundedVec::new(10);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[2], 3);
    }

    #[test]
    fn test_eviction_at_capacity() {
        let mut vec = BoundedVec::new(10);

        // Fill to capacity
        for i in 0..10 {
            vec.push(i);
        }
        assert_eq!(vec.len(), 10);
        assert_eq!(vec[0], 0);

        // Push one more - should evict oldest 10% (1 element)
        vec.push(10);
        assert_eq!(vec.len(), 10);
        assert_eq!(vec[0], 1); // 0 was evicted
        assert_eq!(vec[9], 10);
    }

    #[test]
    fn test_eviction_removes_ten_percent() {
        let mut vec = BoundedVec::new(100);

        // Fill to capacity
        for i in 0..100 {
            vec.push(i);
        }
        assert_eq!(vec.len(), 100);

        // Push one more - should evict oldest 10% (10 elements)
        vec.push(100);
        assert_eq!(vec.len(), 91); // 100 - 10 + 1
        assert_eq!(vec[0], 10); // 0-9 were evicted
    }

    #[test]
    fn test_last() {
        let mut vec = BoundedVec::new(10);
        assert!(vec.last().is_none());

        vec.push(1);
        assert_eq!(vec.last(), Some(&1));

        vec.push(2);
        assert_eq!(vec.last(), Some(&2));
    }

    #[test]
    fn test_last_mut() {
        let mut vec = BoundedVec::new(10);
        vec.push(1);

        if let Some(last) = vec.last_mut() {
            *last = 42;
        }

        assert_eq!(vec[0], 42);
    }

    #[test]
    fn test_clear() {
        let mut vec = BoundedVec::new(10);
        vec.push(1);
        vec.push(2);

        vec.clear();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut vec = BoundedVec::new(10);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_deref_slice_methods() {
        let mut vec = BoundedVec::new(10);
        vec.push(1);
        vec.push(2);
        vec.push(3);

        // Test slice methods via Deref
        assert_eq!(vec.first(), Some(&1));
        assert!(vec.contains(&2));
        assert!(!vec.contains(&99));
    }

    #[test]
    #[should_panic(expected = "max_capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        let _vec: BoundedVec<i32> = BoundedVec::new(0);
    }

    #[test]
    fn test_small_capacity_eviction() {
        // With capacity 3, 10% = 0, but we ensure minimum 1 eviction
        let mut vec = BoundedVec::new(3);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 2); // 1 was evicted
    }
}
