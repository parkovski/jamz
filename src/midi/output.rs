use std::sync::mpsc;
use midir::{MidiOutput, MidiOutputConnection};
use super::MidiError;

pub struct OutputManager {
    pub receiver: mpsc::Receiver<u8>,
    connections: Vec<MidiOutputConnection>,
}

impl OutputManager {
    pub fn new(receiver: mpsc::Receiver<u8>) -> Self {
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

    pub fn work(&self) {
        println!("begin worker thread");
        loop {
            let data = match self.receiver.recv() {
                Ok(data) => data,
                Err(_) => return,
            };

            print!(" {:X}", data);
        }
    }
}
