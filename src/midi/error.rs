use midir::{InitError, PortInfoError, ConnectError};

#[derive(Debug)]
pub enum MidiError {
    PortNotFound,
    Init,
    Info,
    Connect
}

impl From<InitError> for MidiError {
    fn from(_: InitError) -> MidiError {
        MidiError::Init
    }
}

impl From<PortInfoError> for MidiError {
    fn from(_: PortInfoError) -> MidiError {
        MidiError::Info
    }
}

impl<T> From<ConnectError<T>> for MidiError {
    fn from(_: ConnectError<T>) -> MidiError {
        MidiError::Connect
    }
}
