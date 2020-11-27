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
const MIDI_CONTINUE: u8 = 0xFB;
const MIDI_STOP: u8 = 0xFC;
// const MIDI_RESET: u8 = 0xFF;

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
            (0,  [0x90, 0x2D, 0x64, 3]),
            (96, [0x80, 0x2D, 0,    3]),

            (0,  [0x90, 0x34, 0x64, 3]),
            (48, [0x80, 0x34, 0,    3]),

            (0,  [0x90, 0x2D, 0x64, 3]),
            (48, [0x80, 0x2D, 0,    3]),

            (0,  [0x90, 0x30, 0x64, 3]),
            (48, [0x80, 0x30, 0,    3]),

            (0,  [0x90, 0x36, 0x64, 3]),
            (48, [0x80, 0x36, 0,    3]),

            (0,  [0x90, 0x34, 0x64, 3]),
            (96, [0x80, 0x34, 0,    3]),
        ];

        let mut playing = false;
        // Current time
        let mut now = 0;
        // Target time for next message
        let mut target = 0;
        // Time to the next message previously covered
        let mut pause_offset = 0;
        // Index of the next message
        let mut out_msg_i = 0;

        let bpm: i64 = 135;
        let ppq: i64 = 96;

        let message_time = |i: usize|
            pulses_to_us(messages[i].0 as i64, bpm, ppq);

        loop {
            if playing {
                if target > now {
                    // The next message is still in the future, see if there
                    // is an incoming message
                    if let Ok(in_msg) = self.receiver.try_recv() {
                        if in_msg[3] == 1 && in_msg[0] == MIDI_STOP {
                            playing = false;
                            pause_offset = monotonic_now() -
                                (target - message_time(out_msg_i));
                            continue;
                        }
                    }

                    // Sleep while the next message is still queued.
                    now = monotonic_sleep(
                        std::cmp::max(target - now, MAX_SLEEP));
                } else {
                    // It's time to send the message.
                    self.sender.send(messages[out_msg_i].1.clone()).unwrap();
                    // Point to the next message
                    out_msg_i += 1;
                    if out_msg_i == messages.len() {
                        // Reached the end, restart from the beginning.
                        out_msg_i = 0;
                    }
                    // Update target time for this message.
                    target += message_time(out_msg_i);
                }
            } else {
                // Not playing; check if there is a start message.
                if let Ok(in_msg) = self.receiver.try_recv() {
                    if in_msg[3] == 1 && in_msg[0] == MIDI_START {
                        playing = true;
                        out_msg_i = 0;
                        now = monotonic_now();
                        target = now + message_time(0);
                        continue;
                    } else if in_msg[3] == 1 && in_msg[0] == MIDI_CONTINUE {
                        playing = true;
                        now = monotonic_now();
                        target = now + message_time(out_msg_i) - pause_offset;
                        continue;
                    }
                }

                // Sleep a bit before checking for start messages again.
                monotonic_sleep(MAX_SLEEP);
            }
        }
    }
}
