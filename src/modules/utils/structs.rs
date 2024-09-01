use std::collections::VecDeque;

#[derive(Debug, Clone)]
/// This Set replace HashSet when data order matters
pub struct Set<T> {
    data: Vec<T>
}

impl<T> Set<T> 
    where T: PartialEq + Clone
{
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn insert(&mut self, value: T) {
        if !self.data.contains(&value) {
            self.data.push(value);
        }
    }
    pub fn to_vec(&self) -> Vec<T> {
        self.data.clone()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    pub fn position(&self, value: &T) -> Option<usize> {
        self.data.iter().position(|v| v == value)
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}


pub struct LimitedVec<T> {
    deque: VecDeque<T>,
    capacity: usize,
}

impl<T> LimitedVec<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            deque: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.deque.len() == self.capacity {
            self.deque.pop_front();
        }
        self.deque.push_back(item);
    }

    pub fn as_vecdeque(&self) -> &VecDeque<T> {
        &self.deque
    }

    pub fn len(&self) -> usize {
        self.deque.len()
    } 

    pub fn clear(&mut self) {
        self.deque.clear()
    }
}