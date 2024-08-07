use std::collections::HashMap;
use std::time;

pub struct PerformanceTracker {
    tracking: HashMap<&'static str, (Option<time::Instant>, LimitedVec<u128>)>
}

impl PerformanceTracker {

    pub fn new() -> Self {
        Self { tracking: HashMap::new() }
    }

    pub fn call_start(&mut self, tracking_name: &'static str) {
        let now = Some(time::Instant::now());
        if let Some((instant_option, _elapsed_times)) = self.tracking.get_mut(tracking_name) {
            *instant_option = now;
        } else {
            self.tracking.insert(tracking_name, (now, LimitedVec::new(100)));
        }
    }

    pub fn call_end(&mut self, tracking_name: &'static str) {
        if let Some((instant_option, elapsed_times)) = self.tracking.get_mut(tracking_name) {
            if let Some(instant) = instant_option {
                elapsed_times.push(instant.elapsed().as_nanos());
                *instant_option = None;
            }
        }
    }

    pub fn get_report(&mut self) -> Report {
        let report = Report {
            calls: self.tracking
                .iter()
                .filter(|(_, (_, elapsed_times))| elapsed_times.len() > 0)
                .map(|(name, (instant, elapsed_times))| {
                    let total = elapsed_times
                        .as_vecdeque()
                        .iter()
                        .fold(0, |acc, v| acc + v);
                    (*name, (total as f64) / elapsed_times.len() as f64)
                })
                .collect::<Vec<_>>()
        };
        self.clear();
        report
    }
    
    fn clear(&mut self) {
        for tracker in &mut self.tracking {
            tracker.1.0 = None;
            tracker.1.1.clear();
        }
    }
}

#[derive(Debug)]
pub struct Report {
    calls: Vec<(&'static str, f64)>
}

use std::collections::VecDeque;

struct LimitedVec<T> {
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