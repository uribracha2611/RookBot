use std::time::{Duration, Instant};
const MOVE_OVERHEAD: u64 = 30;
const MAX_TIME_FRAC: u64 = 600;

pub struct DYNAMICTIME {
    our_clock: u64,
    our_inc: u64,
}

impl DYNAMICTIME {
    pub fn new(our_clock: u64, our_inc: u64) -> DYNAMICTIME {
        DYNAMICTIME { our_clock, our_inc }
    }
}
pub enum ClockOption {
    NODES(u64),
    MOVETIME(u64),
    TIME(DYNAMICTIME),
    DEPTH(u32),
    NOT_CHOSEN,
}
impl ClockOption {
    pub fn from_nodes(nodes: u64) -> ClockOption {
        Self::NODES(nodes)
    }
    pub fn from_movetime(move_time: u64) -> ClockOption {
        Self::MOVETIME(move_time)
    }
    pub fn form_time(time: DYNAMICTIME) -> ClockOption {
        Self::TIME(time)
    }
    pub fn from_depth(depth: u32) -> ClockOption {
        Self::DEPTH(depth)
    }
    pub fn compute_time_windows(our_clock: u64, our_inc: u64) -> (u64, u64) {
        let max_time = (our_clock * MAX_TIME_FRAC / 1000).saturating_sub(MOVE_OVERHEAD);
        let hard_time = (our_clock * 46 / 100).min(max_time);
        let compute_time = (our_clock) / 24 + (our_inc * 94) / 100 - MOVE_OVERHEAD;
        let soft_time = (compute_time * 73 / 100).min(hard_time);
        (hard_time, soft_time)
    }
}
pub struct TimeManager {
    clock: ClockOption,
    hard_time: Duration,
    soft_time: Duration,
    start_time: Instant,
}
impl Default for TimeManager {
    fn default() -> Self {
        TimeManager {
            clock: ClockOption::NOT_CHOSEN,
            hard_time: Duration::from_secs(0),
            soft_time: Duration::from_secs(0),
            start_time: Instant::now(),
        }
    }
}
impl TimeManager {
    pub fn set_clock(&mut self, clock: ClockOption) {
        self.clock = clock;
        if let ClockOption::TIME(t) = &self.clock {
            let (hard_time_millis, soft_time_milis) =
                ClockOption::compute_time_windows(t.our_clock, t.our_inc);
            self.hard_time = Duration::from_millis(hard_time_millis);
            self.soft_time = Duration::from_millis(soft_time_milis);
        }
    }
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }
    pub fn elapsed(&self) {
        self.start_time.elapsed();
    }
    pub fn check_if_only_nodes_done(&self, nodes_count: u64) -> bool {
        if let ClockOption::NODES(nodes) = self.clock {
            return nodes_count >= nodes;
        }
        false
    }
    pub fn check_if_soft_time_done(&self) -> bool {
        if let ClockOption::TIME(_) = &self.clock {
            return self.start_time.elapsed() >= self.soft_time;
        }

        false
    }
    pub fn check_if_done(&self, nodes_count: u64) -> bool {
        match self.clock {
            ClockOption::NODES(node) => nodes_count >= node,
            ClockOption::MOVETIME(time) => {
                let elapsed = self.start_time.elapsed();
                let elapsed_milis = elapsed.as_millis() as u64;
                return elapsed_milis >= time;
            }
            ClockOption::TIME(_) => self.start_time.elapsed() >= self.hard_time,
            ClockOption::DEPTH(_) => false,
            ClockOption::NOT_CHOSEN => unreachable!(),
        }
    }
    pub fn check_if_time_only_done(&self) -> bool {
        match self.clock {
            ClockOption::MOVETIME(time) => {
                let elapsed = self.start_time.elapsed();
                let elapsed_milis = elapsed.as_millis() as u64;
                return elapsed_milis >= time;
            }
            ClockOption::TIME(_) => self.start_time.elapsed() >= self.hard_time,
            _ => false,
        }
    }
    pub fn check_if_depth_done(&self, current_depth: u32) -> bool {
        match self.clock {
            ClockOption::DEPTH(depth) => current_depth > depth,

            _ => current_depth > 64,
        }
    }
}
