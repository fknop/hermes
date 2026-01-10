use parking_lot::{Condvar, Mutex};

pub struct CancellableBarrier {
    state: Mutex<BarrierState>,
    cvar: Condvar,
    num_threads: usize,
}

struct BarrierState {
    count: usize,
    generation_id: u64,
    cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitResult {
    Leader,
    Follower,
    Cancelled,
}

impl WaitResult {
    pub fn is_leader(&self) -> bool {
        matches!(self, WaitResult::Leader)
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, WaitResult::Cancelled)
    }
}

impl CancellableBarrier {
    pub fn new(n: usize) -> Self {
        Self {
            state: Mutex::new(BarrierState {
                count: 0,
                generation_id: 0,
                cancelled: false,
            }),
            cvar: Condvar::new(),
            num_threads: n,
        }
    }

    pub fn wait(&self) -> WaitResult {
        let mut lock = self.state.lock();

        if lock.cancelled {
            return WaitResult::Cancelled;
        }

        let local_gen = lock.generation_id;
        lock.count += 1;

        if lock.count < self.num_threads {
            self.cvar.wait_while(&mut lock, |state| {
                state.generation_id == local_gen && !state.cancelled
            });

            if lock.cancelled {
                WaitResult::Cancelled
            } else {
                WaitResult::Follower
            }
        } else {
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1);
            self.cvar.notify_all();
            WaitResult::Leader
        }
    }

    pub fn cancel(&self) {
        println!("Cancel barrier");
        let mut state = self.state.lock();
        state.cancelled = true;
        self.cvar.notify_all();
    }
}
