use std::sync::mpsc;
use super::Message;

use crate::os::{
    monotonic_now, monotonic_sleep, InitRealTimeThread
};

fn pulses_to_us(pulses: i64, bpm: i64, ppq: i64) -> i64 {
    // quarters = pulses / ppq
    // minutes = quarters / bpm
    // seconds = minutes * 60
    // us = seconds * 1_000_000
    // us = sec * 60_000_000 * ((pulses / ppq) / bpm)
    60_000_000 * pulses / (bpm * ppq)
}

pub struct Sequencer {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    messages: Vec<Message>,
}

impl Sequencer {
    pub fn new(receiver: mpsc::Receiver<Message>,
        sender: mpsc::Sender<Message>) -> Self
    {
        Sequencer {
            receiver,
            sender,
            messages: Vec::new(),
        }
    }

    pub fn run(self) {
        let _rtthread = InitRealTimeThread::new();

        let messages: &[(i32, [u8; 4])] = &[
            (48,  [0x90, 0x39, 0x64, 3]),
            (96,  [0x80, 0x39, 0x64, 3]),

            (96,  [0x90, 0x3C, 0x64, 3]),
            (144, [0x80, 0x3C, 0x64, 3]),

            (144, [0x90, 0x40, 0x64, 3]),
            (192, [0x80, 0x40, 0x64, 3]),

            (192, [0x90, 0x3E, 0x64, 3]),
            (240, [0x80, 0x3E, 0x64, 3]),

            (240, [0x90, 0x3C, 0x64, 3]),
            (288, [0x80, 0x3C, 0x64, 3]),

            (288, [0x90, 0x39, 0x64, 3]),
            (336, [0x80, 0x39, 0x64, 3]),

            (336, [0x90, 0x3C, 0x64, 3]),
            (528, [0x80, 0x3C, 0x64, 3]),

            (528, [0x90, 0x3E, 0x64, 3]),
            (624, [0x80, 0x3E, 0x64, 3]),

            (624, [0x90, 0x40, 0x64, 3]),
            (768, [0x80, 0x40, 0x64, 3]),
        ];

        // Starting time
        let t0 = monotonic_now();
        // Current time
        let mut t1 = t0;
        for msg in messages {
            let ts = pulses_to_us(msg.0 as _, 135, 96);
            println!("> {}", ts);
            let target = t0 + ts;
            if target > t1 {
                t1 = monotonic_sleep(target - t1);
            } else {
                t1 = monotonic_now();
            }
            self.sender.send(msg.1.clone()).expect("seq send");
        }
    }
}
