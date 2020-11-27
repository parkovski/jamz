use std::sync::mpsc;
use std::collections::HashMap;
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
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;

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
        let mut notes_on: [u8; 256] = [0; 256];

        let bpm: i64 = 135;
        let ppq: i64 = 96;

        let message_time = |i: usize|
            pulses_to_us(messages[i].0 as i64, bpm, ppq);

        let mut update_notes_on = |msg: &[u8; 4], notes_on: &mut [u8]| {
            if msg[3] == 3 && msg[0] == MIDI_NOTE_ON {
                notes_on[msg[1] as usize] = msg[2];
            } else if msg[3] == 3 && msg[0] == MIDI_NOTE_OFF {
                notes_on[msg[1] as usize] = 0;
            }
        };

        let send_pause_messages = |notes_on: &[u8]| {
            for i in 0..notes_on.len() {
                if notes_on[i] > 0 {
                    self.sender.send([MIDI_NOTE_OFF, i as u8, 0, 3]).unwrap();
                }
            }
        };

        let send_resume_messages = |notes_on: &[u8]| {
            for i in 0..notes_on.len() {
                let velocity = notes_on[i];
                if velocity > 0 {
                    self.sender.send([MIDI_NOTE_ON, i as u8, velocity, 3])
                        .unwrap();
                }
            }
        };

        let mut reset_notes_on = |notes_on: &mut [u8]| {
            for i in 0..notes_on.len() {
                notes_on[i] = 0;
            }
        };

        loop {
            if playing {
                if target > now {
                    // The next message is still in the future, see if there
                    // is an incoming message
                    match self.receiver.try_recv() {
                        Ok(in_msg) => {
                            // Process the message.
                            if in_msg[3] == 1 && in_msg[0] == MIDI_STOP {
                                playing = false;
                                pause_offset = monotonic_now() -
                                    (target - message_time(out_msg_i));
                                send_pause_messages(&notes_on);
                            }
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            return;
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            // Sleep while the next message is still queued.
                            now = monotonic_sleep(
                                std::cmp::max(target - now, MAX_SLEEP));
                        }
                    }
                } else {
                    // It's time to send the message.
                    update_notes_on(&messages[out_msg_i].1, &mut notes_on);
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
                match self.receiver.try_recv() {
                    Ok(in_msg) if in_msg[3] == 1 => {
                        if in_msg[0] == MIDI_START {
                            playing = true;
                            out_msg_i = 0;
                            now = monotonic_now();
                            target = now + message_time(0);
                            reset_notes_on(&mut notes_on);
                        } else if in_msg[0] == MIDI_CONTINUE {
                            playing = true;
                            now = monotonic_now();
                            target = now + message_time(out_msg_i)
                                - pause_offset;
                            send_resume_messages(&notes_on);
                        }
                    }
                    Ok(_) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        // Sleep a bit before checking for messages again.
                        monotonic_sleep(MAX_SLEEP);
                    }
                }
            }
        }
    }
}
