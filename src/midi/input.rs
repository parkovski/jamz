use std::sync::mpsc;
use midir::{MidiInput, MidiInputConnection};
use super::MidiError;

pub struct InputManager {
    sender: mpsc::Sender<[u8; 4]>,
    connections: Vec<MidiInputConnection<mpsc::Sender<[u8; 4]>>>,
}

impl InputManager {
    pub fn new(sender: mpsc::Sender<[u8; 4]>) -> Self {
        InputManager {
            sender,
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
                    &p, &port_name, message_received, self.sender.clone()
                )?;
                self.connections.push(conn);
                return Ok(next);
            }
        }
        Err(MidiError::PortNotFound)
    }
}

fn message_received(timestamp: u64, bytes: &[u8],
    sender: &mut mpsc::Sender<[u8; 4]>)
{
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

    sender.send(out_message).expect("Couldn't process incoming MIDI message.");
}
