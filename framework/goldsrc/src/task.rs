//! Foolproof asynchronous task dispatch and background worker synchronization.
//!
//! Bridges background worker threads (thread pools, I/O, database) with the GoldSrc
//! main engine thread:
//! - [`spawn`] runs heavy work on a background worker thread, then dispatches the result
//!   back to the main game loop (`Stage::Frame`).
//! - [`dispatch`] enqueues a closure directly for execution on the main game thread.
//! - Main engine handles ([`crate::Player`], [`crate::Client`], [`crate::Entity`]) are `!Send`,
//!   preventing accidental data races. Background closures must use [`crate::PlayerSlot`].

#[cfg(feature = "task")]
use crossbeam_channel::{Receiver, Sender, unbounded};
#[cfg(feature = "task")]
use std::sync::OnceLock;

#[cfg(feature = "task")]
type TaskCallback = Box<dyn FnOnce() + Send + 'static>;

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

/// Dispatches a closure to execute strictly on the GoldSrc main engine thread during the next frame.
#[cfg(feature = "task")]
pub fn dispatch<F>(callback: F)
where
    F: FnOnce() + Send + 'static,
{
    let _ = queue().tx.send(Box::new(callback));
}

/// Spawns a background task on a worker thread, then safely dispatches its result
/// to a callback on the GoldSrc main engine thread.
///
/// # Compile-Time Safety
/// Because [`crate::Player`], [`crate::Client`], and [`crate::Entity`] do not implement [`Send`],
/// they cannot be accidentally moved into the `work` closure. Pass [`crate::PlayerSlot`] or [`crate::EntityId`]
/// instead, and resolve them inside the `callback` closure on the main thread!
///
/// # Example
/// ```rust,ignore
/// let slot = player.slot();
/// goldsrc::task::spawn(
///     move || {
///         // Runs in background thread (heavy I/O, DB, calculation)
///         fetch_vip_tier(slot.index())
///     },
///     move |tier| {
///         // Runs safely on the GoldSrc main thread!
///         if let Some(mut player) = slot.resolve() {
///             player.set(Health(tier.bonus_hp));
///             player.print_chat("VIP activated!");
///         }
///     },
/// );
/// ```
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

/// Drains and executes pending tasks on the GoldSrc main engine thread.
///
/// Automatically invoked by the frame dispatcher or ECS during `Stage::Frame`.
/// Limits execution to `max_tasks` per frame to prevent tick rate starvation.
#[cfg(feature = "task")]
pub fn drain_main_tasks(max_tasks: usize) -> usize {
    let q = queue();
    let mut count = 0;
    while count < max_tasks {
        match q.rx.try_recv() {
            Ok(task) => {
                task();
                count += 1;
            }
            Err(_) => break,
        }
    }
    count
}

#[cfg(all(test, feature = "task"))]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_task_dispatch_and_drain() {
        static HIT_COUNTER: AtomicUsize = AtomicUsize::new(0);

        dispatch(|| {
            HIT_COUNTER.fetch_add(1, Ordering::SeqCst);
        });
        dispatch(|| {
            HIT_COUNTER.fetch_add(10, Ordering::SeqCst);
        });

        let drained = drain_main_tasks(10);
        assert_eq!(drained, 2);
        assert_eq!(HIT_COUNTER.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_task_spawn_and_drain() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c_clone = counter.clone();

        spawn(
            move || {
                // Background calculation
                42usize
            },
            move |val| {
                // Main thread callback
                c_clone.store(val, Ordering::SeqCst);
            },
        );

        // Wait a tiny bit for the worker thread to finish
        std::thread::sleep(std::time::Duration::from_millis(50));

        let drained = drain_main_tasks(10);
        assert_eq!(drained, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }
}
