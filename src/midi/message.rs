pub type Message = [u8; 4];

pub struct TimedMessage {
    pub message: Message,
    pub ticks: u32,
}

impl TimedMessage {
    pub fn new(message: Message, ticks: u32) -> Self {
        TimedMessage {
            message,
            ticks,
        }
    }
}

impl PartialEq for TimedMessage {
    fn eq(&self, other: &TimedMessage) -> bool {
        self.ticks == other.ticks
    }
}

impl PartialOrd for TimedMessage {
    fn partial_cmp(&self, other: &TimedMessage) -> Option<std::cmp::Ordering> {
        self.ticks.partial_cmp(&other.ticks)
    }
}
