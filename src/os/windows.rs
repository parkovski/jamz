use std::mem::MaybeUninit;

use lazy_static::lazy_static;

use winapi::um::handleapi::CloseHandle;
use winapi::um::winbase::INFINITE;
use winapi::shared::ntdef::{HANDLE, LARGE_INTEGER};
use winapi::um::profileapi::{
    QueryPerformanceCounter, QueryPerformanceFrequency
};
use winapi::um::timeapi::{timeBeginPeriod, timeEndPeriod};
use winapi::um::processthreadsapi::{
    GetCurrentProcess, GetCurrentThread, SetPriorityClass, SetThreadPriority
};
use winapi::um::synchapi::{
    CreateWaitableTimerW, SetWaitableTimer, WaitForSingleObject
};
use winapi::um::winbase::{
    ABOVE_NORMAL_PRIORITY_CLASS, THREAD_PRIORITY_TIME_CRITICAL
};

pub struct RealTimeThreadContext {
    timer: HANDLE,
}

impl RealTimeThreadContext {
    pub fn new() -> Self {
        unsafe {
            timeBeginPeriod(1);
            SetPriorityClass(GetCurrentProcess(), ABOVE_NORMAL_PRIORITY_CLASS);
            SetThreadPriority(GetCurrentThread(),
                THREAD_PRIORITY_TIME_CRITICAL as _);

            let timer = CreateWaitableTimerW(
                std::ptr::null_mut(), false as _,
                std::ptr::null()
            );
            debug_assert!(!timer.is_null());
            RealTimeThreadContext {
                timer,
            }
        }
    }

    pub fn sleep(&self, microseconds: i64) {
        unsafe {
            let mut time: LARGE_INTEGER = MaybeUninit::uninit().assume_init();
            *time.QuadPart_mut() = -microseconds * 10;
            SetWaitableTimer(self.timer, &time, 0, None,
                std::ptr::null_mut(), false as _);
            WaitForSingleObject(self.timer, INFINITE);
        }
    }
}

impl Drop for RealTimeThreadContext {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.timer);
            timeEndPeriod(1);
        };
    }
}

unsafe impl Send for RealTimeThreadContext {}

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
