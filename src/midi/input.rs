use std::sync::mpsc;
use midir::{MidiInput, MidiInputConnection};
use super::{Message, MidiError};

struct InputState {
    pub out: mpsc::Sender<Message>,
    pub seq: mpsc::Sender<Message>,
}

impl InputState {
    pub fn new(out: mpsc::Sender<Message>, seq: mpsc::Sender<Message>) -> Self
    {
        InputState {
            out,
            seq
        }
    }
}

pub struct InputManager {
    out: mpsc::Sender<Message>,
    seq: mpsc::Sender<Message>,
    connections: Vec<MidiInputConnection<InputState>>,
}

impl InputManager {
    pub fn new(out: mpsc::Sender<Message>, seq: mpsc::Sender<Message>) -> Self
    {
        InputManager {
            out,
            seq,
            connections: Vec::new(),
        }
    }

    pub fn devices() -> Vec<String> {
        let midi_inp = match MidiInput::new("jamz") {
            Ok(x) => x,
            Err(_) => return Vec::new(),
        };
        let mut v = Vec::new();
        for p in midi_inp.ports() {
            if let Ok(port_name) = midi_inp.port_name(&p) {
                v.push(port_name);
            }
        }
        v
    }

    pub fn open(&mut self, name: &str) -> Result<usize, MidiError> {
        let midi_inp = MidiInput::new("jamz")?;
        let next = self.connections.len();
        for p in midi_inp.ports() {
            let port_name = midi_inp.port_name(&p)?;
            if name == port_name {
                let conn = midi_inp.connect(
                    &p, &port_name, message_received,
                    InputState::new(self.out.clone(), self.seq.clone())
                )?;
                self.connections.push(conn);
                return Ok(next);
            }
        }
        Err(MidiError::PortNotFound)
    }
}

fn message_received(timestamp: u64, bytes: &[u8], state: &mut InputState) {
    print!("[{:>10}]", timestamp);
    for &b in bytes {
        print!(" {:X}", b);
    }
    println!();

    if bytes.len() > 3 {
        return;
    }

    let mut out_message = [0u8; 4];
    out_message[0..bytes.len()].copy_from_slice(bytes);
    out_message[3] = bytes.len() as u8;

    state.out.send(out_message)
        .expect("Couldn't process incoming MIDI message.");
}
