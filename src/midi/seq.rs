use std::sync::mpsc;
use super::Message;

use crate::os::{
    monotonic_now, monotonic_sleep, InitRealTimeThread
};

const MIDI_UNDEF_TOGGLE_RECORD: u8 = 0xF9;
const MIDI_START: u8 = 0xFA;
const MIDI_CONTINUE: u8 = 0xFB;
const MIDI_STOP: u8 = 0xFC;
// const MIDI_RESET: u8 = 0xFF;
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;

const MAX_SLEEP: i64 = 50_000;

pub type MessageWithTime = (i32, Message);

#[derive(Debug)]
enum PlayState {
    Paused,
    Playing,
    Quit,
}

pub struct Sequencer {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    messages: Vec<MessageWithTime>,
    message_index: usize,
    now: i64,
    target_time: i64,
    prev_target: i64,
    pause_offset: i64,
    // When paused, this is the length of time that will be spent recording.
    // Once recording starts, this is shifted to become the time counter
    // value at which recording ends.
    recording_limit: i64,
    notes_on: [u8; 128],
    bpm: i32,
    ppq: i32,
    recording: bool,
}

impl Sequencer {
    pub fn new(receiver: mpsc::Receiver<Message>,
        sender: mpsc::Sender<Message>) -> Self
    {
        Sequencer {
            receiver,
            sender,
            messages: Self::hard_coded_messages(),
            message_index: 0,
            now: 0,
            target_time: i64::MAX,
            prev_target: 0,
            pause_offset: 0,
            recording_limit: i64::MAX,
            notes_on: [0; 128],
            bpm: 135,
            ppq: 96,
            recording: false,
        }
    }

    fn hard_coded_messages() -> Vec<MessageWithTime> {
        let messages: &[MessageWithTime] = &[
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

        let mut vec = Vec::with_capacity(250);
        for m in messages {
            vec.push(*m);
        }
        vec
    }

    // If recording is not bounded, record_end_time should be set to i64::MAX.
    fn play_loop_step(&mut self) -> PlayState {
        if self.recording && self.recording_limit < self.now {
            println!("done recording");
            self.recording = false;
        }

        if self.target_time > self.now {
            // The next message is still in the future, see if there
            // is an incoming message
            self.check_playing_messages()
        } else {
            // It's time to send the next message.
            self.send_message_and_advance();
            PlayState::Playing
        }
    }

    fn send_pause_messages(&self) {
        for i in 0..self.notes_on.len() {
            if self.notes_on[i] > 0 {
                self.sender.send([MIDI_NOTE_OFF, i as u8, 0, 3]).unwrap();
            }
        }
    }

    fn send_resume_messages(&self) {
        for i in 0..self.notes_on.len() {
            if self.notes_on[i] > 0 {
                let velocity = self.notes_on[i];
                self.sender.send([MIDI_NOTE_ON, i as u8, velocity, 3])
                    .unwrap();
            }
        }
    }

    fn update_notes_on(&mut self) {
        let message = &self.messages[self.message_index].1;
        if message[3] == 3 && message[0] == MIDI_NOTE_ON {
            self.notes_on[message[1] as usize] = message[2];
        } else if message[3] == 3 && message[0] == MIDI_NOTE_OFF {
            self.notes_on[message[1] as usize] = 0;
        }
    }

    fn send_message_and_advance(&mut self) {
        self.update_notes_on();
        self.sender.send(self.messages[self.message_index].1.clone()).unwrap();
        // Point to the next message
        self.message_index += 1;
        if self.message_index == self.messages.len() {
            // Reached the end, restart from the beginning.
            self.message_index = 0;
        }
        // Update target time for this message.
        self.prev_target = self.target_time;
        self.target_time += self.message_time();
    }

    fn reset_notes_on(&mut self) {
        for i in 0..self.notes_on.len() {
            self.notes_on[i] = 0;
        }
    }

    fn pulses_to_us(&self, pulses: i32) -> i64 {
        // quarters = pulses / ppq
        // minutes = quarters / bpm
        // seconds = minutes * 60
        // us = seconds * 1_000_000
        // us = sec * 60_000_000 * ((pulses / ppq) / bpm)
        60_000_000 * (pulses as i64) / ((self.bpm * self.ppq) as i64)
    }

    fn us_to_pulses(&self, us: i64) -> i32 {
        let pulses = us * ((self.bpm * self.ppq) as i64) / 60_000_000;
        debug_assert!(pulses <= i32::MAX as i64);
        pulses as i32
    }

    fn message_time(&self) -> i64 {
        self.pulses_to_us(self.messages[self.message_index].0)
    }

    fn on_playing_message(&mut self, msg: &Message) -> PlayState {
        if msg[3] == 1 && msg[0] == MIDI_STOP {
            self.now = monotonic_now();
            self.pause_offset = self.now -
                (self.target_time - self.message_time());
            if self.recording && self.recording_limit != i64::MAX {
                self.recording_limit -= self.now;
                if self.recording_limit < 0 {
                    self.recording_limit = 0;
                    self.recording = false;
                }
            }
            self.send_pause_messages();
            PlayState::Paused
        } else if self.recording && msg[3] == 3 &&
            (msg[0] == MIDI_NOTE_ON || msg[0] == MIDI_NOTE_OFF)
        {
            self.now = monotonic_now();
            let time = self.us_to_pulses(self.now - self.prev_target);
            self.prev_target = self.now;
            //self.messages.push((time, msg.clone()));
            println!("note {} @ {}",
                if msg[0] == MIDI_NOTE_ON { "on" } else { "off" }, time);
            PlayState::Playing
        } else {
            PlayState::Playing
        }
    }

    fn on_paused_message(&mut self, msg: &Message) -> PlayState {
        if msg[3] == 1 && msg[0] == MIDI_START {
            self.message_index = 0;
            self.now = monotonic_now();
            self.prev_target = self.target_time;
            self.target_time = self.now + self.message_time();
            self.reset_notes_on();
            if self.recording {
                //self.messages.clear();
                if self.recording_limit != i64::MAX {
                    self.recording_limit += self.now;
                }
            }
            PlayState::Playing
        } else if msg[3] == 1 && msg[0] == MIDI_CONTINUE {
            self.now = monotonic_now();
            self.prev_target = self.target_time;
            self.target_time = self.now + self.message_time()
                - self.pause_offset;
            self.send_resume_messages();
            if self.recording {
                //self.messages.clear();
                if self.recording_limit != i64::MAX {
                    self.recording_limit += self.now;
                }
            }
            PlayState::Playing
        } else if msg[3] == 1 && msg[0] == MIDI_UNDEF_TOGGLE_RECORD {
            self.recording = !self.recording;
            self.recording_limit = self.pulses_to_us(384);
            println!("recording {}",
                if self.recording { "enabled" } else { "disabled" });
            PlayState::Paused
        } else {
            PlayState::Paused
        }
    }

    fn check_playing_messages(&mut self) -> PlayState {
        match self.receiver.try_recv() {
            Ok(msg) => self.on_playing_message(&msg),
            // Disconnected means nothing else will be coming on this
            // channel. The program must be shutting down.
            Err(mpsc::TryRecvError::Disconnected) => PlayState::Quit,
            Err(mpsc::TryRecvError::Empty) => {
                // Sleep a bit before checking for messages again.
                self.now = monotonic_sleep(
                    std::cmp::max(self.target_time - self.now, MAX_SLEEP));
                PlayState::Playing
            }
        }
    }

    fn check_paused_messages(&mut self) -> PlayState {
        match self.receiver.try_recv() {
            Ok(msg) => self.on_paused_message(&msg),
            // Disconnected means nothing else will be coming on this
            // channel. The program must be shutting down.
            Err(mpsc::TryRecvError::Disconnected) => PlayState::Quit,
            Err(mpsc::TryRecvError::Empty) => {
                // Sleep a bit before checking for messages again.
                monotonic_sleep(MAX_SLEEP);
                PlayState::Paused
            }
        }
    }

    pub fn run(mut self) {
        let _rtthread = InitRealTimeThread::new();

        let mut state = PlayState::Paused;
        loop {
            state = match state {
                PlayState::Playing => self.play_loop_step(),
                PlayState::Paused => self.check_paused_messages(),
                PlayState::Quit => return,
            }
        }
    }
}
