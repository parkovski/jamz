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

const MIDI_START: u8 = 0xFA;
const MIDI_STOP: u8 = 0xFC;

const MAX_SLEEP: i64 = 50_000;

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
            (0,   [0x90, 0x2D, 0x64, 3]),
            (96,  [0x80, 0x2D, 0,    3]),

            (96,  [0x90, 0x34, 0x64, 3]),
            (144, [0x80, 0x34, 0,    3]),

            (144, [0x90, 0x2D, 0x64, 3]),
            (192, [0x80, 0x2D, 0,    3]),

            (192, [0x90, 0x30, 0x64, 3]),
            (240, [0x80, 0x30, 0,    3]),

            (240, [0x90, 0x36, 0x64, 3]),
            (288, [0x80, 0x36, 0,    3]),

            (288, [0x90, 0x34, 0x64, 3]),
            (384, [0x80, 0x34, 0,    3]),
        ];

        let mut playing = false;
        // Start time
        let mut t0 = 0;
        // Current time
        let mut t1 = 0;
        // Target time for next message
        let mut tt = 0;

        let mut out_msg_i = 0;
        loop {
            if playing {
                if tt > t1 {
                    // The next message is still in the future, see if there
                    // is an incoming message
                    if let Ok(in_msg) = self.receiver.try_recv() {
                        if in_msg[3] == 1 && in_msg[0] == MIDI_STOP {
                            playing = false;
                            continue;
                        }
                    }

                    // Sleep while the next message is still queued.
                    t1 = monotonic_sleep(std::cmp::max(tt - t1, MAX_SLEEP));
                } else {
                    // It's time to send the next message.
                    self.sender.send(messages[out_msg_i].1.clone());
                    out_msg_i += 1;
                    if out_msg_i == messages.len() {
                        // Reached the end, restart from the beginning.
                        out_msg_i = 0;
                        t0 = monotonic_now();
                        t1 = t0;
                        tt = t0 + pulses_to_us(messages[0].0 as i64, 135, 96);
                    } else {
                        // Move on to the next message.
                        let ts = messages[out_msg_i].0 as i64;
                        tt = t0 + pulses_to_us(ts, 135, 96);
                    }
                }
            } else {
                // Not playing; check if there is a start message.
                if let Ok(in_msg) = self.receiver.try_recv() {
                    if in_msg[3] == 1 && in_msg[0] == MIDI_START {
                        playing = true;
                        out_msg_i = 0;
                        t0 = monotonic_now();
                        t1 = t0;
                        tt = t0 + pulses_to_us(messages[0].0 as i64, 135, 96);
                        continue;
                    }
                }

                // Sleep a bit before checking for start messages again.
                monotonic_sleep(MAX_SLEEP);
            }
        }
    }
}
