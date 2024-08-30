#[derive(Debug, Clone)]
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
