use crate::os::{RealTimeThreadContext, monotonic_now};

pub struct Scheduler {
    rtthread: RealTimeThreadContext,
    // Is the scheduler active or paused?
    active: bool,
    // Time when started plus sum of lengths of all pauses.
    clock_offset: i64,
    // Clock time when the last pause started.
    pause_time: i64,
    // Time in pulses of the song.
    pulse_count: u32,
    // Microbeats per minute - allows three decimal places without precision
    // loss.
    mbpm: u32,
    // Pulses per quarter (beat).
    ppq: u32,
    // Previous pulse time in microseconds.
    prev_pulse: i64,
    // Microseconds per pulse.
    us_per_pulse: i64,
}

const fn pulses_to_us(pulses: u32, mbpm: u32, ppq: u32) -> i64 {
    // quarters = pulses / ppq
    // minutes = quarters * 1000 / mbpm
    // seconds = minutes * 60
    // us = seconds * 1_000_000
    // us = sec * 60_000_000_000 * ((pulses / ppq) / mbpm)
    let pulses = pulses as i64;
    let mbpm = mbpm as i64;
    let ppq = ppq as i64;
    // luckily i64s can store really huge numbers
    60_000_000_000 * pulses / (mbpm * ppq)
}

impl Scheduler {
    pub const MAX_SLEEP: i64 = 50_000;

    pub fn new(mbpm: u32, ppq: u32) -> Self {
        Scheduler {
            rtthread: RealTimeThreadContext::new(),
            active: false,
            clock_offset: 0,
            pause_time: 0,
            pulse_count: 0,
            mbpm,
            ppq,
            prev_pulse: 0,
            us_per_pulse: pulses_to_us(1, mbpm, ppq),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn song_time_us(&self) -> i64 {
        pulses_to_us(self.pulse_count, self.mbpm, self.ppq)
    }

    pub fn time(&self) -> u32 {
        self.pulse_count
    }

    pub fn pause(&mut self) {
        self.pause_time = monotonic_now();
        self.active = false;
    }

    pub fn resume(&mut self) {
        let now = monotonic_now();
        self.clock_offset = now - self.song_time_us();
        self.prev_pulse = now;
        self.active = true;
    }

    /// Set song time counter to 0.
    pub fn reset(&mut self) {
        self.pulse_count = 0;
        self.pause_time = 0;
    }

    pub fn wait_for_next_pulse(&mut self) {
        debug_assert!(self.active);
        let now = monotonic_now();
        let target = self.prev_pulse + self.us_per_pulse;
        let time = target - now;
        if time > 0 {
            self.rtthread.sleep(time);
        }
        self.prev_pulse += self.us_per_pulse;
        self.pulse_count += 1;
    }

    pub fn wait_paused(&mut self) {
        debug_assert!(!self.active);
        self.rtthread.sleep(Self::MAX_SLEEP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MBPM: u32 = 135000;
    const PPQ: u32 = 96;
    const ERR_MARGIN: i64 = 1500;
    const PULSE_LENGTH: i64 = pulses_to_us(1, MBPM, PPQ);

    #[test]
    fn test_pulses_to_us() {
        assert!(PULSE_LENGTH == 4629);
    }

    #[test]
    fn test_scheduler_resolution() {
        let mut times = Vec::with_capacity(100);
        let mut scheduler = Scheduler::new(MBPM, PPQ);

        scheduler.resume();
        let start_time = monotonic_now();
        for i in 0..times.capacity() {
            assert!(scheduler.time() == i as u32);
            scheduler.wait_for_next_pulse();
            times.push(monotonic_now());
        }

        for i in 0..times.len() {
            let target = start_time + (i + 1) as i64 * PULSE_LENGTH;
            let error = times[i] - target;
            println!("{}: {}", i, error);
            assert!(error > -ERR_MARGIN && error < ERR_MARGIN);
        }

        let total = times[times.len() - 1] - times[0];
        let total_target = PULSE_LENGTH * (times.len() as i64 - 1);
        let error = total - total_target;
        println!("--\n{}", error);
        assert!(error > -ERR_MARGIN && error < ERR_MARGIN);
    }

    #[test]
    fn test_scheduler_pause() {
        let mut times = Vec::with_capacity(100);
        let mut scheduler = Scheduler::new(MBPM, PPQ);

        for i in 0..10 {
            assert!(!scheduler.is_active());
            scheduler.resume();
            assert!(scheduler.is_active());
            for j in 0..10 {
                times.push(monotonic_now());
                assert!(scheduler.time() == i * 10 + j);
                scheduler.wait_for_next_pulse();
            }
            scheduler.pause();
            let pause_time = monotonic_now();
            scheduler.wait_paused();
            assert!(monotonic_now() > pause_time + PULSE_LENGTH);
        }

        for i in 0..10 {
            for j in 1..10 {
                let k = i * 10 + j;
                let difference = times[k] - times[k - 1];
                assert!(
                    difference > PULSE_LENGTH - ERR_MARGIN &&
                    difference < PULSE_LENGTH + ERR_MARGIN
                );
            }
        }
    }

    #[test]
    fn test_scheduler_reset() {
        let mut times = Vec::with_capacity(100);
        let mut scheduler = Scheduler::new(MBPM, PPQ);

        scheduler.resume();
        for _ in 0..10 {
            for i in 0..10 {
                times.push(monotonic_now());
                assert!(scheduler.time() == i);
                scheduler.wait_for_next_pulse();
            }
            scheduler.reset();
        }

        for i in 1..times.len() {
            let difference = times[i] - times[i - 1];
            assert!(
                difference > PULSE_LENGTH - ERR_MARGIN &&
                difference < PULSE_LENGTH + ERR_MARGIN
            );
        }

        let total = times[times.len() - 1] - times[0];
        let total_target = PULSE_LENGTH * (times.len() as i64 - 1);
        assert!(
            total > total_target - ERR_MARGIN &&
            total < total_target + ERR_MARGIN
        );
    }
}
