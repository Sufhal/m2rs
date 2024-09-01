use std::collections::HashMap;

pub struct PerformanceTracker {
    tracking: HashMap<&'static str, (Option<std::time::Instant>, LimitedVec<u128>)>
}

impl PerformanceTracker {

    pub fn new() -> Self {
        Self { tracking: HashMap::new() }
    }

    pub fn call_start(&mut self, tracking_name: &'static str) {
        if is_browser() { return }
        let now = Some(std::time::Instant::now());
        if let Some((instant_option, _elapsed_times)) = self.tracking.get_mut(tracking_name) {
            *instant_option = now;
        } else {
            self.tracking.insert(tracking_name, (now, LimitedVec::new(100)));
        }
    }

    pub fn call_end(&mut self, tracking_name: &'static str) {
        if is_browser() { return }
        if let Some((instant_option, elapsed_times)) = self.tracking.get_mut(tracking_name) {
            if let Some(instant) = instant_option {
                elapsed_times.push(instant.elapsed().as_nanos());
                *instant_option = None;
            }
        }
    }

    pub fn get_report(&mut self) -> Report {
        if is_browser() { return Report::empty() }
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
impl Report {
    fn empty() -> Self { Self { calls: Vec::new() } }
}

use std::collections::VecDeque;

use super::functions::is_browser;

