pub use timer::Timer;

#[cfg(not(target_arch = "wasm32"))]
mod timer {
    use std::time::{Duration, Instant};
    
    pub struct Timer {
        num_frames: usize,
        timestamps: Vec<Duration>,
        prev_time: Instant,
    }
    
    impl Timer {
        pub fn new(num_frames: usize) -> Self {
            Timer {
                num_frames,
                timestamps: Vec::with_capacity(num_frames),
                prev_time: Instant::now(),
            }
        }
    
        pub fn tick(&mut self) -> Option<Duration> {
            let now = Instant::now();
            self.timestamps.push(now - self.prev_time);
            let res = if self.timestamps.len() > self.num_frames {
                Some(
                    (self
                        .timestamps
                        .drain(..)
                        .take(self.num_frames)
                        .sum::<Duration>()
                        / self.num_frames as u32)
                        .max(Duration::from_nanos(1)),
                )
            } else {
                None
            };
            self.prev_time = now;
            res
        }
    }
}


#[cfg(target_arch = "wasm32")]
mod timer {
    pub struct Timer {}

    impl Timer {
        pub fn new(_: usize) -> Self {
            Timer {}
        }
        pub fn tick(&mut self) -> Option<()> {
            None
        }
        }

}
