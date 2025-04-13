use std::cmp::Ordering;
use std::vec::IntoIter;
use thiserror::Error;

const FIRST_ELEMENT_INDEX: usize = 1;

pub struct PriorityQueue<P>
where
    P: Ord,
{
    heap: Vec<(usize, P)>,
    positions: Vec<Option<usize>>,
    size: usize,
    max: usize,
}

#[derive(Error, Debug)]
pub enum PriorityQueueError {
    #[error("Priority queue is full")]
    Full,
    #[error("Element already exists in the priority queue")]
    ElementAlreadyExists,
}

impl<P> PriorityQueue<P>
where
    P: Ord + Copy + Default,
{
    pub fn new(max: usize) -> Self {
        let mut heap = Vec::with_capacity(max + 1);
        heap.push((max + 1, P::default()));
        Self {
            heap,
            positions: vec![None; max + 1],
            size: 0,
            max,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn contains(&self, id: usize) -> bool {
        self.positions[id].is_some()
    }

    pub fn push(&mut self, id: usize, priority: P) -> Result<usize, PriorityQueueError> {
        if id >= self.max {
            panic!("ID {} out of bounds, max: {}", id, self.max)
        }

        if self.size == self.max {
            return Err(PriorityQueueError::Full);
        }

        if self.contains(id) {
            return Err(PriorityQueueError::ElementAlreadyExists);
        }

        self.size += 1;
        self.heap.push((id, priority));
        self.positions[id] = Some(self.size);
        self.sift_up(self.size);

        Ok(id)
    }

    pub fn peek(&self) -> Option<&(usize, P)> {
        self.heap.get(FIRST_ELEMENT_INDEX)
    }

    pub fn pop(&mut self) -> Option<(usize, P)> {
        if self.size == 0 {
            return None;
        }

        let (id, priority) = self.heap.swap_remove(FIRST_ELEMENT_INDEX);
        self.size -= 1;

        // Id is removed from the heap
        self.positions[id] = None;

        if self.size > 0 {
            // The last becomes the first element
            self.positions[self.heap[FIRST_ELEMENT_INDEX].0] = Some(FIRST_ELEMENT_INDEX);
            self.sift_down(FIRST_ELEMENT_INDEX);
        }

        Some((id, priority))
    }

    pub fn update_priority(&mut self, id: usize, priority: P) {
        if let Some(position) = self.positions[id] {
            let current_priority = self.heap[position].1;
            self.heap[position] = (id, priority);

            match priority.cmp(&current_priority) {
                Ordering::Greater => self.sift_down(position),
                Ordering::Less => self.sift_up(position),
                Ordering::Equal => {}
            }
        }
    }

    pub fn clear(&mut self) {
        self.positions.fill(None);
        self.heap.clear();
        self.heap.push((self.max + 1, P::default()));
        self.size = 0;
    }

    fn sift_up(&mut self, element_index: usize) {
        if element_index == FIRST_ELEMENT_INDEX {
            return;
        }

        let mut index = element_index;
        let priority = self.heap[index].1;
        while index >> 1 > 0 && priority < self.heap[index >> 1].1 {
            let parent_index = index >> 1;
            self.heap.swap(index, parent_index);

            // The position of the previous parent is updated
            self.positions[self.heap[index].0] = Some(index);

            index = parent_index;
        }

        self.positions[self.heap[index].0] = Some(index);
    }

    fn sift_down(&mut self, element_index: usize) {
        if self.size == 0 {
            return;
        }

        let mut index = element_index;

        let priority = self.heap[index].1;

        while index << 1 <= self.size {
            let left_child_index = index << 1;
            let right_child_index = left_child_index + 1;

            let mut child_index = left_child_index;
            if right_child_index <= self.size
                && self.heap[right_child_index].1 < self.heap[left_child_index].1
            {
                child_index = right_child_index;
            }

            if priority <= self.heap[child_index].1 {
                break;
            }

            self.heap.swap(index, child_index);

            // The position of the previous child is updated
            self.positions[self.heap[index].0] = Some(index);

            index = child_index;
        }

        self.positions[self.heap[index].0] = Some(index);
    }

    pub fn to_vec(&self) -> Vec<(usize, P)> {
        // Create a clone of the heap elements (excluding sentinel)
        let mut elements: Vec<(usize, P)> = self.heap[1..=self.size].to_vec();
        // Sort by priority
        elements.sort_by(|a, b| a.1.cmp(&b.1));

        elements
    }

    pub fn iter(&self) -> PriorityQueueIter<P> {
        PriorityQueueIter {
            inner: self.to_vec().into_iter(),
        }
    }
}

pub struct PriorityQueueIter<P> {
    inner: IntoIter<(usize, P)>,
}

// Implement Iterator trait
impl<P> Iterator for PriorityQueueIter<P> {
    type Item = (usize, P);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pop() {
        let mut queue = PriorityQueue::<usize>::new(100);
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn push_zero() {
        let mut queue = PriorityQueue::<usize>::new(100);
        assert!(queue.push(0, 1).is_ok());
    }

    #[test]
    fn test_size() {
        let mut queue = PriorityQueue::<usize>::new(100);
        assert!(queue.push(1, 5).is_ok());
        assert_eq!(queue.len(), 1);
        assert!(queue.push(2, 5).is_ok());
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn error_duplicate_id() {
        let mut queue = PriorityQueue::<usize>::new(100);
        assert!(queue.push(1, 5).is_ok());
        assert!(
            queue.push(1, 5).is_err(),
            "Element already exists in the priority queue"
        );
    }

    #[test]
    fn test_contains() {
        let mut queue = PriorityQueue::<usize>::new(100);
        let _ = queue.push(1, 5);
        assert!(queue.contains(1));
        assert!(!queue.contains(2));
    }

    #[test]
    fn test_push_and_peek() {
        let mut queue = PriorityQueue::<usize>::new(100);
        let _ = queue.push(1, 5);
        assert_eq!(queue.peek(), Some(&(1, 5)));
        let _ = queue.push(2, 3);
        assert_eq!(queue.peek(), Some(&(2, 3)));
        let _ = queue.push(3, 4);
        assert_eq!(queue.peek(), Some(&(2, 3)));
    }

    #[test]
    fn test_pop() {
        let mut queue = PriorityQueue::new(5);
        let _ = queue.push(1, 5);
        let _ = queue.push(2, 3);
        let _ = queue.push(3, 4);

        assert_eq!(queue.pop(), Some((2, 3)));
        assert_eq!(queue.pop(), Some((3, 4)));
        assert_eq!(queue.pop(), Some((1, 5)));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_iter() {
        let mut queue = PriorityQueue::new(5);
        let _ = queue.push(1, 5);
        let _ = queue.push(2, 3);
        let _ = queue.push(3, 4);

        let mut iter = queue.iter();

        assert_eq!(iter.next(), Some((2, 3)));
        assert_eq!(iter.next(), Some((3, 4)));
        assert_eq!(iter.next(), Some((1, 5)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_update_priority() {
        let mut queue = PriorityQueue::new(5);
        let _ = queue.push(1, 5);
        let _ = queue.push(2, 3);
        let _ = queue.push(3, 4);

        queue.update_priority(1, 2);
        assert_eq!(queue.pop(), Some((1, 2)));
        assert_eq!(queue.pop(), Some((2, 3)));
        assert_eq!(queue.pop(), Some((3, 4)));
    }

    #[test]
    fn test_clear() {
        let mut queue = PriorityQueue::new(5);
        let _ = queue.push(1, 5);
        let _ = queue.push(2, 3);
        let _ = queue.push(3, 4);

        queue.clear();
        assert_eq!(queue.len(), 0);
        assert!(!queue.contains(1));
        assert!(!queue.contains(2));
        assert!(!queue.contains(3));

        let _ = queue.push(1, 5);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_push_when_full() {
        let mut queue = PriorityQueue::new(2);
        let _ = queue.push(1, 5);
        let _ = queue.push(2, 3);
        assert!(queue.push(3, 4).is_err(), "Priority queue is full");
    }
}
