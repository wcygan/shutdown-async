//! A library for gracefully shutting down asynchronous applications
//!
//! This may be useful when you want to allow all in-flight processing
//! to complete before shutting down in order to maintain a consistent state.
//!
//! # Examples
//!
//! ```
//! use shutdown_async::ShutdownController;
//!
//! #[tokio::main]
//! async fn main() {
//!   let shutdown = ShutdownController::new();
//!   
//!   tokio::task::spawn({
//!     let mut monitor = shutdown.subscribe();
//!     async move {
//!       // Wait for something to happen
//!       tokio::select! {
//!        _ = monitor.recv() => { println!("shutdown initiated"); }
//!        _ = tokio::time::sleep(ONE_YEAR) => { println!("one year has passed!"); }
//!       }
//!     }
//!   });
//!
//!   shutdown.shutdown().await;
//! }
//!
//! static ONE_YEAR: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 365);
//! ```
use tokio::sync::{broadcast, mpsc};

/// A [`ShutdownController`] is used to control the shutdown of an application.
///
/// This is accomplished by creating a [`ShutdownMonitor`] instance for each task
/// that should be monitored. When [`ShutdownController::shutdown`] is called,
/// all [`ShutdownMonitor`] instances will be notified that shutdown has started.
///
/// # Examples
///
/// ```
/// use shutdown_async::ShutdownController;
///
/// #[tokio::main]
/// async fn main() {
///   let shutdown = ShutdownController::new();
///   
///   tokio::task::spawn({
///     let mut monitor = shutdown.subscribe();
///     async move {
///       // Wait for something to happen
///       tokio::select! {
///        _ = monitor.recv() => { println!("shutdown initiated"); }
///        _ = tokio::time::sleep(ONE_YEAR) => { println!("one year has passed!"); }
///       }
///     }
///   });
///
///   shutdown.shutdown().await;
/// }
///
/// static ONE_YEAR: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 365);
/// ```
pub struct ShutdownController {
    /// Used to tell all [`ShutdownMonitor`] instances that shutdown has started.
    notify_shutdown: broadcast::Sender<()>,

    /// Implicitly used to determine when all [`ShutdownMonitor`] instances have been dropped.
    task_tracker: mpsc::Sender<()>,

    /// Used to determine when all tasks have finished. Calling `recv()` on this channel
    /// will return when all of the send halves of the `task_tracker` channel have been dropped.
    task_waiter: mpsc::Receiver<()>,
}

impl ShutdownController {
    /// Create a new [`ShutdownController`].
    ///
    /// # Examples
    ///
    /// ```
    /// let shutdown = shutdown_async::ShutdownController::new();
    /// ```
    pub fn new() -> Self {
        let (notify_shutdown, _) = broadcast::channel::<()>(1);
        let (task_tracker, task_waiter) = mpsc::channel::<()>(1);

        Self {
            notify_shutdown,
            task_tracker,
            task_waiter,
        }
    }

    /// Create a new [`ShutdownMonitor`] instance that can listen for the shutdown signal.
    ///
    /// # Examples
    ///
    /// ```
    /// let shutdown = shutdown_async::ShutdownController::new();
    /// let monitor = shutdown.subscribe();
    pub fn subscribe(&self) -> ShutdownMonitor {
        ShutdownMonitor::new(self.notify_shutdown.subscribe(), self.task_tracker.clone())
    }

    /// Begin shutting down and wait for all [`ShutdownMonitor`] instances to be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///  let shutdown = shutdown_async::ShutdownController::new();
    ///
    ///  // ... do stuff ...
    ///
    ///  // Tell all tasks to shutdown
    ///  shutdown.shutdown().await;
    /// }
    /// ```
    pub async fn shutdown(mut self) {
        // Notify all tasks that shutdown has started
        drop(self.notify_shutdown);

        // Destroy our mpsc::Sender so that the mpsc::Receiver::recv() will return immediately
        // once all tasks have completed (i.e. dropped their mpsc::Sender)
        drop(self.task_tracker);

        // Wait for all tasks to finish
        let _ = self.task_waiter.recv().await;
    }
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::new()
    }
}

/// A [`ShutdownMonitor`] listens for the shutdown signal from a [`ShutdownController`] and
/// tracks that the signal has been received.
///
/// Callers may query for whether the shutdown signal has been received or not.
///
/// # Examples
///
/// ```
/// use shutdown_async::ShutdownMonitor;
///
/// async fn run(monitor: &mut ShutdownMonitor) {
///   while !monitor.is_shutdown() {
///       tokio::select! {
///        _ = monitor.recv() => { println!("shutdown initiated"); }
///        _ = async { /* do work */ } => { println!("one year has passed!"); }
///       }
///   }
/// }
/// ```
pub struct ShutdownMonitor {
    /// `true` if the shutdown signal has been received
    shutdown_received: bool,

    /// The receive half of the channel used to listen for shutdown.
    shutdown_notifier: broadcast::Receiver<()>,

    /// Implicitly used to help [`ShutdownController`] understand when the program
    /// has completed shutdown.
    _task_tracker: mpsc::Sender<()>,
}

impl ShutdownMonitor {
    fn new(
        shutdown_notifier: broadcast::Receiver<()>,
        _task_tracker: mpsc::Sender<()>,
    ) -> ShutdownMonitor {
        ShutdownMonitor {
            shutdown_received: false,
            shutdown_notifier,
            _task_tracker,
        }
    }

    /// Returns `true` if the shutdown signal has been received, and `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///   let shutdown = shutdown_async::ShutdownController::new();
    ///   let mut monitor = shutdown.subscribe();
    ///
    ///   // Assert that the monitor has not yet received the shutdown signal
    ///   assert!(!monitor.is_shutdown());
    /// }
    /// ```
    pub fn is_shutdown(&self) -> bool {
        self.shutdown_received
    }

    /// Receive the shutdown notice, waiting if necessary.
    ///
    /// # Examples
    ///
    /// ```
    /// async fn long_lived_task(mut monitor: shutdown_async::ShutdownMonitor) {
    ///    // Wait for the shutdown signal
    ///    monitor.recv().await;
    /// }
    /// ```
    pub async fn recv(&mut self) {
        // If the shutdown signal has already been received, then return
        // immediately.
        if self.shutdown_received {
            return;
        }

        // Cannot receive a "lag error" as only one value is ever sent.
        let _ = self.shutdown_notifier.recv().await;

        // Remember that the signal has been received.
        self.shutdown_received = true;
    }
}
