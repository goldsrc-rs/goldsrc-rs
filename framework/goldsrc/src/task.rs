//! Foolproof asynchronous task dispatch and background worker synchronization.
//!
//! Bridges background worker threads (thread pools, I/O, database) with the GoldSrc
//! main engine thread:
//! - [`spawn`] runs heavy work on a background worker thread, then dispatches the result
//!   back to the main game loop (`Stage::Frame`).
//! - [`dispatch`] enqueues a closure directly for execution on the main game thread.
//! - [`after`] schedules a one-shot task after discrete ticks or wall-clock duration.
//! - [`every`] schedules a recurring task repeating at the specified interval.
//! - [`TaskHandle::bound_to`] binds task lifetime to a player session, eliminating Slot Recycling Hazards.
//! - Main engine handles ([`crate::Player`], [`crate::Client`], [`crate::Entity`]) are `!Send`,
//!   preventing accidental data races. Background closures must use [`crate::PlayerSlot`].

#[cfg(feature = "task")]
use crossbeam_channel::{Receiver, Sender, unbounded};
pub use goldsrc_api::client::PlayerSessionToken;
pub use goldsrc_api::timer::{
    IntoScheduleDelay, ScheduleDelay, Ticks, TimerAction, TimerId, TimerMode,
};
#[cfg(feature = "task")]
use std::collections::HashSet;
#[cfg(feature = "task")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "task")]
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "task")]
type TaskCallback = Box<dyn FnOnce() + Send + 'static>;
#[cfg(feature = "task")]
type RecurringTaskCallback = Box<dyn FnMut() -> TimerAction + Send + 'static>;

#[cfg(feature = "task")]
struct TaskQueue {
    tx: Sender<TaskCallback>,
    rx: Receiver<TaskCallback>,
}

#[cfg(feature = "task")]
static TASK_QUEUE: OnceLock<TaskQueue> = OnceLock::new();

#[cfg(feature = "task")]
fn queue() -> &'static TaskQueue {
    TASK_QUEUE.get_or_init(|| {
        let (tx, rx) = unbounded();
        TaskQueue { tx, rx }
    })
}

#[cfg(feature = "task")]
struct ScheduledTask {
    id: u64,
    target_tick: Option<u64>,
    target_time_secs: Option<f64>,
    mode: TimerMode,
    bound: Option<PlayerSessionToken>,
    callback: RecurringTaskCallback,
}

#[cfg(feature = "task")]
struct SchedulerState {
    tasks: Vec<ScheduledTask>,
    cancelled: HashSet<u64>,
    current_tick: u64,
}

#[cfg(feature = "task")]
static SCHEDULER: OnceLock<Mutex<SchedulerState>> = OnceLock::new();
#[cfg(feature = "task")]
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "task")]
fn scheduler() -> &'static Mutex<SchedulerState> {
    SCHEDULER.get_or_init(|| {
        Mutex::new(SchedulerState {
            tasks: Vec::new(),
            cancelled: HashSet::new(),
            current_tick: 0,
        })
    })
}

#[cfg(feature = "task")]
fn uptime_secs() -> f64 {
    goldsrc_api::timer::host_time() as f64
}

/// Handle to a scheduled task or timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskHandle {
    id: u64,
}

impl TaskHandle {
    /// Creates a handle from a raw task ID.
    pub const fn new(id: u64) -> Self {
        Self { id }
    }

    /// Returns the unique numeric ID of the task.
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Binds this task to a specific player session token.
    ///
    /// If the player disconnects or reconnects before the task completes, the task
    /// is automatically discarded without running, eliminating Slot Recycling Hazards.
    pub fn bound_to(self, session: PlayerSessionToken) -> Self {
        #[cfg(feature = "task")]
        {
            let mut state = scheduler().lock().unwrap_or_else(|e| e.into_inner());
            if let Some(task) = state.tasks.iter_mut().find(|t| t.id == self.id) {
                task.bound = Some(session);
            }
        }
        self
    }

    /// Cancels the task if it has not yet completed.
    pub fn cancel(&self) -> bool {
        #[cfg(feature = "task")]
        {
            let mut state = scheduler().lock().unwrap_or_else(|e| e.into_inner());
            state.cancelled.insert(self.id)
        }
        #[cfg(not(feature = "task"))]
        {
            false
        }
    }
}

/// Dispatches a closure to execute strictly on the GoldSrc main engine thread during the next frame.
#[cfg(feature = "task")]
pub fn dispatch<F>(callback: F)
where
    F: FnOnce() + Send + 'static,
{
    let _ = queue().tx.send(Box::new(callback));
}

/// Schedules a one-shot task to execute after the specified delay (`Ticks` or `Duration`).
#[cfg(feature = "task")]
pub fn after<D, F>(delay: D, callback: F) -> TaskHandle
where
    D: IntoScheduleDelay,
    F: FnOnce() + Send + 'static,
{
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    let delay = delay.into_schedule_delay();
    let mut cb = Some(callback);

    let (target_tick, target_time_secs) = match delay {
        ScheduleDelay::Ticks(t) => {
            let current = scheduler()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .current_tick;
            (Some(current.saturating_add(t)), None)
        }
        ScheduleDelay::Duration(d) => (None, Some(uptime_secs() + d.as_secs_f64())),
    };

    let task = ScheduledTask {
        id,
        target_tick,
        target_time_secs,
        mode: TimerMode::Once,
        bound: None,
        callback: Box::new(move || {
            if let Some(f) = cb.take() {
                f();
            }
            TimerAction::Stop
        }),
    };

    scheduler()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .tasks
        .push(task);

    TaskHandle::new(id)
}

/// Schedules a recurring task repeating at the specified interval (`Ticks` or `Duration`).
#[cfg(feature = "task")]
pub fn every<D, F>(interval: D, callback: F) -> TaskHandle
where
    D: IntoScheduleDelay,
    F: FnMut() -> TimerAction + Send + 'static,
{
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
    let delay = interval.into_schedule_delay();

    let (target_tick, target_time_secs) = match delay {
        ScheduleDelay::Ticks(t) => {
            let current = scheduler()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .current_tick;
            (Some(current.saturating_add(t)), None)
        }
        ScheduleDelay::Duration(d) => (None, Some(uptime_secs() + d.as_secs_f64())),
    };

    let task = ScheduledTask {
        id,
        target_tick,
        target_time_secs,
        mode: TimerMode::Recurring(delay),
        bound: None,
        callback: Box::new(callback),
    };

    scheduler()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .tasks
        .push(task);

    TaskHandle::new(id)
}

/// Spawns a background task on a worker thread, then safely dispatches its result
/// to a callback on the GoldSrc main engine thread.
///
/// # Compile-Time Safety
/// Because [`crate::Player`], [`crate::Client`], and [`crate::Entity`] do not implement [`Send`],
/// they cannot be accidentally moved into the `work` closure. Pass [`crate::PlayerSlot`] or [`crate::EntityId`]
/// instead, and resolve them inside the `callback` closure on the main thread!
#[cfg(feature = "task")]
pub fn spawn<F, R, Res>(work: F, callback: R)
where
    F: FnOnce() -> Res + Send + 'static,
    R: FnOnce(Res) + Send + 'static,
    Res: Send + 'static,
{
    std::thread::Builder::new()
        .name("goldsrc-worker".to_string())
        .spawn(move || {
            let res = work();
            dispatch(move || {
                callback(res);
            });
        })
        .expect("failed to spawn goldsrc worker thread");
}

/// Drains and executes pending tasks and timers on the GoldSrc main engine thread.
///
/// Automatically invoked by the frame dispatcher or ECS during `Stage::Frame`.
/// Limits execution to `max_tasks` per frame to prevent tick rate starvation.
#[cfg(feature = "task")]
pub fn drain_main_tasks(max_tasks: usize) -> usize {
    let mut count = 0;

    // 1. Drain immediate tasks from background channels
    let q = queue();
    while count < max_tasks {
        match q.rx.try_recv() {
            Ok(task) => {
                task();
                count += 1;
            }
            Err(_) => break,
        }
    }

    // 2. Advance local frame tick and drain expired scheduled timers
    let now = uptime_secs();
    let (ready, current_tick) = {
        let mut state = scheduler().lock().unwrap_or_else(|e| e.into_inner());
        state.current_tick = state.current_tick.saturating_add(1);
        let cur_tick = state.current_tick;

        let mut ready = Vec::new();
        let mut remaining = Vec::new();
        let tasks = std::mem::take(&mut state.tasks);

        for task in tasks {
            if state.cancelled.remove(&task.id) {
                continue;
            }

            let is_ready = match (task.target_tick, task.target_time_secs) {
                (Some(t), _) => t <= cur_tick,
                (_, Some(secs)) => secs <= now,
                (None, None) => true,
            };

            if is_ready {
                ready.push(task);
            } else {
                remaining.push(task);
            }
        }

        state.tasks = remaining;
        (ready, cur_tick)
    };

    let mut reschedule = Vec::new();

    // 3. Execute expired callbacks without holding scheduler lock
    for mut task in ready {
        if count >= max_tasks {
            reschedule.push(task);
            continue;
        }

        // Validate session binding if present
        if let Some(token) = task.bound {
            use goldsrc_api::ClientExt;
            let current_token = goldsrc_api::client::Player::new(token.slot).session_token();
            if current_token != Some(token) {
                // Session expired or player reconnected; skip task
                continue;
            }
        }

        count += 1;
        let action = (task.callback)();

        if action == TimerAction::Continue
            && let TimerMode::Recurring(interval) = task.mode
        {
            match interval {
                ScheduleDelay::Ticks(t) => {
                    task.target_tick = Some(current_tick.saturating_add(t));
                }
                ScheduleDelay::Duration(d) => {
                    task.target_time_secs = Some(uptime_secs() + d.as_secs_f64());
                }
            }
            reschedule.push(task);
        }
    }

    // 4. Put repeating and postponed tasks back
    if !reschedule.is_empty() {
        let mut state = scheduler().lock().unwrap_or_else(|e| e.into_inner());
        state.tasks.extend(reschedule);
    }

    count
}

#[cfg(all(test, feature = "task"))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_task_dispatch_and_drain() {
        let _guard = TEST_LOCK.lock().unwrap();
        static HIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

        dispatch(|| {
            HIT_COUNTER.fetch_add(1, Ordering::SeqCst);
        });
        dispatch(|| {
            HIT_COUNTER.fetch_add(10, Ordering::SeqCst);
        });

        let drained = drain_main_tasks(10);
        assert!(drained >= 2);
        assert_eq!(HIT_COUNTER.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_task_after_ticks() {
        let _guard = TEST_LOCK.lock().unwrap();
        let fired = Arc::new(AtomicBool::new(false));
        let f_clone = fired.clone();

        after(Ticks(2), move || {
            f_clone.store(true, Ordering::SeqCst);
        });

        // Frame 1: tick + 1, not ready yet
        drain_main_tasks(10);
        assert!(!fired.load(Ordering::SeqCst));

        // Frame 2: tick + 2, fires
        drain_main_tasks(10);
        assert!(fired.load(Ordering::SeqCst));
    }

    #[test]
    fn test_task_every_ticks_recurring_and_cancel() {
        let _guard = TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c_clone = counter.clone();

        let handle = every(Ticks(1), move || {
            c_clone.fetch_add(1, Ordering::SeqCst);
            TimerAction::Continue
        });

        drain_main_tasks(10);
        assert!(counter.load(Ordering::SeqCst) >= 1);

        drain_main_tasks(10);
        assert!(counter.load(Ordering::SeqCst) >= 2);

        handle.cancel();

        let prev = counter.load(Ordering::SeqCst);
        drain_main_tasks(10);
        assert_eq!(counter.load(Ordering::SeqCst), prev);
    }
}
