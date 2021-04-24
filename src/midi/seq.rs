use std::sync::mpsc;
use super::Message;
use crate::sched::Scheduler;

const MIDI_UNDEF_TOGGLE_RECORD: u8 = 0xF9;
const MIDI_START: u8 = 0xFA;
const MIDI_CONTINUE: u8 = 0xFB;
const MIDI_STOP: u8 = 0xFC;
// const MIDI_RESET: u8 = 0xFF;
const MIDI_NOTE_ON: u8 = 0x90;
const MIDI_NOTE_OFF: u8 = 0x80;

pub type MessageWithTime = (u32, Message);

struct Pattern {
    pub messages: Vec<MessageWithTime>,
    pub length: u32,
}

impl Pattern {
    pub fn new(length: u32) -> Self {
        Pattern {
            messages: vec![],
            length,
        }
    }

    pub fn hard_coded_1() -> Self {
        Pattern {
            messages: vec![
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
            ],
            length: 384,
        }
    }

    pub fn hard_coded_2() -> Self {
        Pattern {
            messages: vec![
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
            ],
            length: 768,
        }
    }
}

struct Cursor {
    pub pattern_index: usize,
    pub message_index: usize,
    // The master song time that corresponds to the pattern start.
    pub offset: u32,
}

impl Cursor {
    pub fn new(pattern_index: usize) -> Self {
        Cursor {
            pattern_index,
            message_index: 0,
            offset: 0,
        }
    }
}

pub struct Sequencer {
    receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<Message>,
    patterns: Vec<Pattern>,
    play_cursors: Vec<Cursor>,
    record_cursors: Vec<Cursor>,
    scheduler: Scheduler,
    notes_on: [u8; 128],
}

impl Sequencer {
    pub fn new(receiver: mpsc::Receiver<Message>,
        sender: mpsc::Sender<Message>) -> Self
    {
        Sequencer {
            receiver,
            sender,
            patterns: vec![
                Pattern::hard_coded_1(),
                Pattern::hard_coded_2(),
                Pattern::new(384),
            ],
            play_cursors: vec![Cursor::new(0), Cursor::new(1)],
            record_cursors: vec![Cursor::new(2)],
            scheduler: Scheduler::new(135000, 96),
            notes_on: [0; 128],
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

    fn reset_notes_on(&mut self) {
        for i in 0..self.notes_on.len() {
            self.notes_on[i] = 0;
        }
    }

    fn reset_cursors(&mut self) {
        for cursor in &mut self.play_cursors {
            cursor.message_index = 0;
            cursor.offset = 0;
        }
    }

    fn pause(&mut self) {
        self.send_pause_messages();
        self.scheduler.pause();
    }

    fn play(&mut self, reset: bool) {
        if reset {
            self.reset_notes_on();
            self.reset_cursors();
            self.scheduler.reset();
        } else {
            self.send_resume_messages();
        }
        self.scheduler.resume();
    }

    fn on_playing_message(&mut self, msg: &Message) {
        if msg[3] == 1 && msg[0] == MIDI_STOP {
            /*if self.recording {
                self.recording = false;
            }*/
            self.pause();
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
            self.play(true);
        } else if msg[3] == 1 && msg[0] == MIDI_CONTINUE {
            self.play(false);
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
            Err(mpsc::TryRecvError::Empty) => {}
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
                self.scheduler.wait_paused();
            }
        };
        true
    }

    fn send_messages_at_cursor(&mut self, cursor_index: usize) {
        let ref mut cursor = self.play_cursors[cursor_index];
        let ref pattern = self.patterns[cursor.pattern_index];
        let mut pattern_time = self.scheduler.time() - cursor.offset;

        loop {
            let (message_time, ref message) =
                pattern.messages[cursor.message_index];
            if message_time > pattern_time {
                break;
            }

            self.sender.send(message.clone()).unwrap();

            if message[3] == 3 && message[0] == MIDI_NOTE_ON {
                self.notes_on[message[1] as usize] = message[2];
            } else if message[3] == 3 && message[0] == MIDI_NOTE_OFF {
                self.notes_on[message[1] as usize] = 0;
            }

            cursor.message_index += 1;
            if cursor.message_index == pattern.messages.len() {
                cursor.message_index = 0;
                cursor.offset += pattern_time;
                pattern_time = 0;
            }
        }
    }

    fn send_outgoing_messages(&mut self) {
        for cursor_index in 0..self.play_cursors.len() {
            self.send_messages_at_cursor(cursor_index);
        }
    }

    pub fn run(mut self) {
        loop {
            if self.scheduler.is_active() {
                self.scheduler.wait_for_next_pulse();
                self.send_outgoing_messages();
                if !self.check_playing_messages() {
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
