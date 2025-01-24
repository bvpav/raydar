use std::time::{Duration, Instant};

#[derive(Default)]
pub struct FrameTimer {
    last_frame_start: Option<Instant>,
    last_frame_duration: Option<Duration>,
    last_sample_start: Option<Instant>,
    last_sample_duration: Option<Duration>,
}

impl FrameTimer {
    pub fn start_frame(&mut self) {
        self.last_frame_start = Some(Instant::now());
    }

    pub fn start_sample(&mut self) {
        self.last_sample_start = Some(Instant::now());
    }

    pub fn end_frame(&mut self) {
        if let Some(last_frame_start) = self.last_frame_start {
            self.last_frame_duration = Some(last_frame_start.elapsed());
        }
    }

    pub fn end_sample(&mut self) {
        if let Some(last_sample_start) = self.last_sample_start {
            self.last_sample_duration = Some(last_sample_start.elapsed());
        }
    }

    pub fn last_frame_duration(&self) -> Option<Duration> {
        self.last_frame_duration
    }

    pub fn last_sample_duration(&self) -> Option<Duration> {
        self.last_sample_duration
    }
}
