use std::time::{Duration, Instant};

/// A profiler utility for measuring the duration of operations with manual start/stop control.
#[derive(Default)]
pub struct Profiler {
    /// The timer for measuring the total duration of the last frame
    pub(super) frame_timer: Timer,
    /// The timer for measuring the duration of the last sample
    pub(super) sample_timer: Timer,
}

impl Profiler {
    pub fn frame_timer(&self) -> &Timer {
        &self.frame_timer
    }

    pub fn sample_timer(&self) -> &Timer {
        &self.sample_timer
    }
}

/// A timer utility for measuring the duration of operations with manual start/stop control.
///
/// The `Timer` struct provides a simple way to measure elapsed time between two points
/// in your code. It's particularly useful for:
/// * Performance profiling
/// * Benchmarking specific operations
/// * Measuring average operation time across multiple iterations
///
/// # Example
///
/// ```
/// let mut timer = Timer::default();
/// timer.start();
/// // ... perform some operation ...
/// timer.end();
///
/// if let Some(duration) = timer.duration() {
///     println!("Operation took: {:?}", duration);
/// }
/// ```
///
/// # Note
///
/// The timer must be explicitly started using [`Timer::start`] before it can measure duration.
/// If you try to end a timer that hasn't been started, the duration will remain `None`.
#[derive(Default)]
pub struct Timer {
    /// The instant when the timer was started, if it has been started
    start: Option<Instant>,
    /// The measured duration between start and end, if the timer has completed
    duration: Option<Duration>,
}

impl Timer {
    /// Starts the timer by recording the current instant.
    ///
    /// Any previous duration is preserved until the timer is ended again.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Ends the timer and calculates the duration since it was started.
    ///
    /// If the timer wasn't started, the duration remains `None`.
    pub fn end(&mut self) {
        if let Some(start) = self.start {
            self.duration = Some(start.elapsed());
        }
    }

    /// Ends the timer and calculates the average duration per iteration over a specified number of iterations.
    ///
    /// This is useful when timing operations that are repeated multiple times
    /// but cannot be timed individually, as it automatically divides the total
    /// elapsed time by the number of iterations to get the average time per
    /// iteration.
    pub fn end_multiple(&mut self, count: u32) {
        if let Some(start) = self.start {
            self.duration = Some(start.elapsed() / count);
        }
    }

    /// Returns the measured duration, if the timer has been started and ended.
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }
}
