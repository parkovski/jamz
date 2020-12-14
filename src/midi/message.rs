pub type Message = [u8; 4];

pub struct BasicMessage {
    pub data: [u8; 3],
    pub size: u8,
    pub port: u8,
    pub time: u32,
}

impl BasicMessage {
    pub fn new(data: &[u8], time: u32, port: u8) -> Self {
        debug_assert!(data.len() <= 3, "Basic messages cannot exceed 3 bytes");
        let mut new_data = [0u8; 3];
        new_data[0..data.len()].copy_from_slice(data);

        BasicMessage {
            data: new_data,
            size: data.len() as u8,
            port,
            time,
        }
    }
}
