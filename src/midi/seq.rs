use std::sync::mpsc;
use std::collections::HashMap;
use super::Message;

use crate::os::{
    monotonic_now, monotonic_sleep, InitRealTimeThread
};

const MIDI_START: u8 = 0xFA;
const MIDI_CONTINUE: u8 = 0xFB;
const MIDI_STOP: u8 = 0xFC;
// const MIDI_RESET: u8 = 0xFF;
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;

const MAX_SLEEP: i64 = 50_000;

pub type MessageWithTime = (i64, Message);

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
    pause_offset: i64,
    notes_on: [u8; 256],
    bpm: i32,
    ppq: i32,
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
            target_time: 0,
            pause_offset: 0,
            notes_on: [0; 256],
            bpm: 135,
            ppq: 96,
        }
    }

    fn hard_coded_messages() -> Vec<MessageWithTime> {
        vec![
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
        ]
    }

    fn play_loop_step(&mut self) -> PlayState {
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
        self.target_time += self.message_time();
    }

    fn reset_notes_on(&mut self) {
        for i in 0..self.notes_on.len() {
            self.notes_on[i] = 0;
        }
    }

    fn pulses_to_us(&self, pulses: i64) -> i64 {
        // quarters = pulses / ppq
        // minutes = quarters / bpm
        // seconds = minutes * 60
        // us = seconds * 1_000_000
        // us = sec * 60_000_000 * ((pulses / ppq) / bpm)
        60_000_000 * pulses / ((self.bpm * self.ppq) as i64)
    }

    fn message_time(&self) -> i64 {
        self.pulses_to_us(self.messages[self.message_index].0 as i64)
    }

    fn on_playing_message(&mut self, msg: &Message) -> PlayState {
        if msg[3] == 1 && msg[0] == MIDI_STOP {
            self.pause_offset = monotonic_now() -
                (self.target_time - self.message_time());
            self.send_pause_messages();
            PlayState::Paused
        } else {
            PlayState::Playing
        }
    }

    fn on_paused_message(&mut self, msg: &Message) -> PlayState {
        if msg[3] == 1 && msg[0] == MIDI_START {
            self.message_index = 0;
            self.now = monotonic_now();
            self.target_time = self.now + self.message_time();
            self.reset_notes_on();
            PlayState::Playing
        } else if msg[3] == 1 && msg[0] == MIDI_CONTINUE {
            self.now = monotonic_now();
            self.target_time = self.now + self.message_time()
                - self.pause_offset;
            self.send_resume_messages();
            PlayState::Playing
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
