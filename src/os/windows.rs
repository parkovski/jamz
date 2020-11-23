use std::mem::MaybeUninit;

use lazy_static::lazy_static;

use winapi::um::profileapi::{
    QueryPerformanceCounter, QueryPerformanceFrequency
};
use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};
use winapi::um::processthreadsapi::{
    GetCurrentProcess, GetCurrentThread, SetPriorityClass, SetThreadPriority
};
use winapi::um::synchapi::Sleep;
use winapi::um::winbase::{
    REALTIME_PRIORITY_CLASS, THREAD_PRIORITY_HIGHEST
};

pub struct InitRealTimeThread;

impl InitRealTimeThread {
    pub fn new() -> Self {
        unsafe {
            timeBeginPeriod(1);
            SetPriorityClass(GetCurrentProcess(), REALTIME_PRIORITY_CLASS);
            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST as _);
        }
        InitRealTimeThread
    }
}

impl Drop for InitRealTimeThread {
    fn drop(&mut self) {
        unsafe { timeEndPeriod(1); }
    }
}

lazy_static! {
    static ref PERFORMANCE_FREQUENCY: i64 = {
        unsafe {
            let mut freq = MaybeUninit::uninit().assume_init();
            QueryPerformanceFrequency(&mut freq);
            *freq.QuadPart()
        }
    };
}

fn qpf_units_to_us(qpf: i64) -> i64 {
    qpf * 1_000_000 / *PERFORMANCE_FREQUENCY
}

pub fn monotonic_now() -> i64 {
    unsafe {
        let mut c = MaybeUninit::uninit().assume_init();
        QueryPerformanceCounter(&mut c);
        qpf_units_to_us(*c.QuadPart())
    }
}

// Time specified in microseconds.
pub fn monotonic_sleep(mut time: i64) -> i64 {
    unsafe {
        let mut c0 = MaybeUninit::uninit().assume_init();
        let mut c1 = MaybeUninit::uninit().assume_init();
        QueryPerformanceCounter(&mut c0);
        loop {
            // We assume sleep has a 1-2ms precision.
            if time > 5000 {
                // > 5ms = coarse sleep, rounded to the nearest millisecond,
                // leave the last 3ms for finer control.
                Sleep((time / 1000 - 3) as u32);
            } else if time > 2000 {
                // 1ms .. 5ms = 1ms sleep
                Sleep(1);
            } else if time > 10 {
                Sleep(0);
            } else {
                return qpf_units_to_us(*c0.QuadPart());
            }
            QueryPerformanceCounter(&mut c1);
            time -= qpf_units_to_us(c1.QuadPart() - c0.QuadPart());
            c0 = c1;
        }
    }
}

#[test]
fn test_monotonic_sleep() {
    let _rtthread = InitRealTimeThread::new();

    let targets = &[
        100000, 75000, 50000, 30000, 20000, 15000, 12500, 11000, 10000, 7500,
        5000, 4000, 3000, 2500, 2000, 1500, 1400, 1300, 1200, 1100, 1000, 750,
        500, 300, 250, 100, 50,
    ];

    let mut reals = Vec::new();
    reals.reserve(targets.len());

    let mut c0 = monotonic_now();

    for &t in targets {
        let c1 = monotonic_sleep(t);
        reals.push(c1 - c0);
        c0 = c1;
    }

    for i in 0..targets.len() {
        // Note: assumes 10MHz QPF.
        println!("Target = {}, real = {}", targets[i], reals[i] / 10);
    }
}
