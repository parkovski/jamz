use std::sync::mpsc;
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};
use super::MidiError;
use super::message_received;

pub struct Manager {
    conn_inp: Vec<MidiInputConnection<mpsc::Sender<u8>>>,
    conn_out: Vec<MidiOutputConnection>,
    sender: mpsc::Sender<u8>,
}

impl Manager {
    pub fn new(sender: mpsc::Sender<u8>) -> Self {
        Manager {
            conn_inp: Vec::new(),
            conn_out: Vec::new(),
            sender,
        }
    }

    pub fn inp_devices() -> Vec<String> {
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

    pub fn out_devices() -> Vec<String> {
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

    pub fn open_inp(&mut self, name: &str) -> Result<usize, MidiError> {
        let midi_inp = MidiInput::new("jamz")?;
        let next = self.conn_inp.len();
        for p in midi_inp.ports() {
            let port_name = midi_inp.port_name(&p)?;
            if name == port_name {
                let conn = midi_inp.connect(
                    &p, &port_name, message_received, self.sender.clone()
                )?;
                self.conn_inp.push(conn);
                return Ok(next);
            }
        }
        Err(MidiError::PortNotFound)
    }

    pub fn open_out(&mut self, name: &str) -> Result<usize, MidiError> {
        let midi_out = MidiOutput::new("jamz")?;
        let next = self.conn_out.len();
        for p in midi_out.ports() {
            let port_name = midi_out.port_name(&p)?;
            if name == port_name {
                let conn = midi_out.connect(&p, &port_name)?;
                self.conn_out.push(conn);
                return Ok(next);
            }
        }
        Err(MidiError::PortNotFound)
    }
}
