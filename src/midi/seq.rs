use std::sync::mpsc;
use super::Message;

use crate::os::{
    monotonic_now, RealTimeThreadContext
};

const MIDI_UNDEF_TOGGLE_RECORD: u8 = 0xF9;
const MIDI_START: u8 = 0xFA;
const MIDI_CONTINUE: u8 = 0xFB;
const MIDI_STOP: u8 = 0xFC;
// const MIDI_RESET: u8 = 0xFF;
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;

pub type MessageWithTime = (i32, Message);

struct Scheduler {
    rtthread: Option<RealTimeThreadContext>,
    // Is the scheduler active or paused?
    active: bool,
    // Current time
    current_time: i64,
    // Time to execute the next event
    target_time: i64,
    // Time the previous event was executed
    prev_target: i64,
    // The amount of the current time between messages that passed before
    // the pause. In the diagram, with t0 = the previous message time,
    // t1 = the next message time, and tp = the pause time, this is tp - t0.
    // |---+---|
    // t0  tp  t1
    pause_offset: i64,
}

impl Scheduler {
    pub const MAX_SLEEP: i64 = 50_000;

    pub fn new() -> Self {
        Scheduler {
            rtthread: None,
            active: false,
            current_time: 0,
            target_time: 0,
            prev_target: 0,
            pause_offset: 0,
        }
    }

    pub fn init_real_time_thread(&mut self) {
        self.rtthread = Some(RealTimeThreadContext::new());
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Save the amount of elapsed time since the last message and unset the
    /// active flag. This updates the cached current time.
    pub fn pause(&mut self) {
        self.pause_offset = self.now() - self.prev_target;
        self.active = false;
    }

    /// Update relevant time variables as if the pause didn't happen and set
    /// the active flag. This updates the cached current time.
    pub fn resume(&mut self) {
        let target_delta = self.target_time - self.prev_target;
        self.prev_target = self.now() - self.pause_offset;
        self.target_time = self.prev_target + target_delta;
        self.active = true;
    }

    /// Reset target times and pause offset.
    pub fn reset(&mut self) {
        self.pause_offset = 0;
        self.prev_target = 0;
        self.target_time = 0;
    }

    /// Updates and returns the cached current time. For use when the time may
    /// not have been recently updated.
    pub fn now(&mut self) -> i64 {
        self.current_time = monotonic_now();
        self.current_time
    }

    /// True if the cached current time is past the next target time.
    pub fn is_target_time(&self) -> bool {
        self.current_time > self.target_time
    }

    /// Sleep and update the cached current time. The sleep time is
    /// `MAX_SLEEP` unless the upcoming target time is sooner.
    pub fn sleep_for_target(&mut self) {
        let time = std::cmp::max(self.target_time - self.now(),
            Self::MAX_SLEEP);
        //self.current_time = monotonic_sleep(time);
        self.rtthread.as_ref().unwrap().sleep(time);
        self.current_time = monotonic_now();
    }

    /// Sleep for `MAX_SLEEP` and updates the cached current time.
    pub fn sleep_for_wait(&mut self) {
        //self.current_time = monotonic_sleep(Self::MAX_SLEEP);
        self.rtthread.as_ref().unwrap().sleep(Self::MAX_SLEEP);
        self.current_time = monotonic_now();
    }

    /// Update the target time to the current target time plus the target
    /// delta.
    pub fn next_target(&mut self, target_delta: i64) {
        self.prev_target = self.target_time;
        self.target_time += target_delta;
    }
}

pub struct Sequencer {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    messages: Vec<MessageWithTime>,
    message_index: usize,
    scheduler: Scheduler,
    notes_on: [u8; 128],
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
            scheduler: Scheduler::new(),
            notes_on: [0; 128],
            bpm: 135,
            ppq: 96,
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
        let message_time = self.message_time();
        self.scheduler.next_target(message_time);
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

    fn on_playing_message(&mut self, msg: &Message) {
        if msg[3] == 1 && msg[0] == MIDI_STOP {
            /*if self.recording {
                self.recording = false;
            }*/
            self.send_pause_messages();
            self.scheduler.pause();
        } /*else if self.recording && msg[3] == 3 &&
            (msg[0] == MIDI_NOTE_ON || msg[0] == MIDI_NOTE_OFF)
        {
            let time = self.us_to_pulses(now - self.prev_target);
            //self.prev_target = now;
            //self.messages.push((time, msg.clone()));
            println!("note {} @ {}",
                if msg[0] == MIDI_NOTE_ON { "on" } else { "off" }, time);
        }*/
    }

    fn on_paused_message(&mut self, msg: &Message) {
        if msg[3] == 1 && msg[0] == MIDI_START {
            self.message_index = 0;
            self.reset_notes_on();
            self.scheduler.reset();
            self.scheduler.resume();
        } else if msg[3] == 1 && msg[0] == MIDI_CONTINUE {
            self.send_resume_messages();
            self.scheduler.resume();
        } /*else if msg[3] == 1 && msg[0] == MIDI_UNDEF_TOGGLE_RECORD {
            self.recording = !self.recording;
            println!("recording {}",
                if self.recording { "enabled" } else { "disabled" });
        }*/
    }

    /// Checks for messages that are valid when playing. Return value indicates
    /// whether the message loop should continue (true) or exit (false).
    fn check_playing_messages(&mut self) -> bool {
        match self.receiver.try_recv() {
            Ok(msg) => self.on_playing_message(&msg),
            // Disconnected means nothing else will be coming on this
            // channel. The program must be shutting down.
            Err(mpsc::TryRecvError::Disconnected) => return false,
            Err(mpsc::TryRecvError::Empty) => {
                self.scheduler.sleep_for_target();
            }
        };
        true
    }

    /// Checks for messages that are valid when paused. Return value indicates
    /// whether the message loop should continue (true) or exit (false).
    fn check_paused_messages(&mut self) -> bool {
        match self.receiver.try_recv() {
            Ok(msg) => self.on_paused_message(&msg),
            // Disconnected means nothing else will be coming on this
            // channel. The program must be shutting down.
            Err(mpsc::TryRecvError::Disconnected) => return false,
            Err(mpsc::TryRecvError::Empty) => {
                // Sleep a bit before checking for messages again.
                self.scheduler.sleep_for_wait();
            }
        };
        true
    }

    pub fn run(mut self) {
        self.scheduler.init_real_time_thread();

        loop {
            if self.scheduler.is_active() {
                if self.scheduler.is_target_time() {
                    self.send_message_and_advance();
                } else if !self.check_playing_messages() {
                    return;
                }
            } else {
                if !self.check_paused_messages() {
                    return;
                }
            }
        }
    }
}
