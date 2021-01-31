use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Perf {
    exec_us: RefCell<VecDeque<u64>>,
}

impl Perf {
    pub fn push_exec_us(&self, time: u64) {
        let mut exec_us = self.exec_us.borrow_mut();
        while exec_us.len() >= 100 {
            exec_us.pop_front();
        }
        exec_us.push_back(time);
    }

    pub fn average_exec_us(&self) -> f64 {
        let exec_us = self.exec_us.borrow();
        exec_us.iter().map(|&us| us as f64).sum::<f64>() / (exec_us.len() as f64)
    }
}