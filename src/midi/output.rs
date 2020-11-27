use std::sync::mpsc;
use midir::{MidiOutput, MidiOutputConnection};
use super::{Message, MidiError};

pub struct OutputManager {
    pub receiver: mpsc::Receiver<Message>,
    connections: Vec<MidiOutputConnection>,
}

impl OutputManager {
    pub fn new(receiver: mpsc::Receiver<Message>) -> Self {
        OutputManager {
            receiver,
            connections: Vec::new(),
        }
    }

    pub fn devices() -> Vec<String> {
        let midi_out = match MidiOutput::new("jamz") {
            Ok(x) => x,
            Err(_) => return Vec::new(),
        };
        let mut v = Vec::new();
        for p in midi_out.ports() {
            if let Ok(port_name) = midi_out.port_name(&p) {
                v.push(port_name);
            }
        }
        v
    }

    pub fn open(&mut self, name: &str) -> Result<usize, MidiError> {
        let midi_out = MidiOutput::new("jamz")?;
        let next = self.connections.len();
        for p in midi_out.ports() {
            let port_name = midi_out.port_name(&p)?;
            if name == port_name {
                let conn = midi_out.connect(&p, &port_name)?;
                self.connections.push(conn);
                return Ok(next);
            }
        }
        Err(MidiError::PortNotFound)
    }

    pub fn run(mut self) {
        println!("begin output thread");
        loop {
            let data = match self.receiver.recv() {
                Ok(data) => data,
                Err(_) => return,
            };

            let bytes = &data[0..data[3] as usize];

            if self.connections.len() > 0 {
                self.connections[0].send(bytes)
                    .expect("Send error!");
            } else {
                print!("Received");
                for &b in bytes {
                    print!(" {:X}", b);
                }
                println!();
            }
        }
    }
}
