use std::{
	sync::LazyLock,
	time::{Duration, Instant},
};

static TIME_START: LazyLock<Instant> = LazyLock::new(Instant::now);

pub fn get_millis() -> u64 {
    TIME_START.elapsed().as_millis() as u64
}

pub fn get_micros() -> u64 {
    TIME_START.elapsed().as_micros() as u64
}

pub struct Rate {
    rate: u16,
    start_ms: u64,
    finish_ms: u64,
}

impl Rate {
    pub fn new(rate: u16) -> Self {
        Self {
            rate,
            start_ms: 0,
            finish_ms: 0,
        }
    }

    pub fn start(&mut self) {
        self.start_ms = get_millis();
    }

    pub fn end(&mut self) {
        self.finish_ms = get_millis();

        let diff = self.finish_ms - self.start_ms;

        let frametime_micros = (1000.0_f32 / self.rate as f32) * 1000.0;
        let delay = frametime_micros as i64 - (diff * 1000) as i64;

        if delay > 0 {
            std::thread::sleep(Duration::from_micros(delay as u64));
        }
    }
}
