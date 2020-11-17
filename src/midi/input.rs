use std::sync::mpsc;

pub fn message_received(timestamp: u64, bytes: &[u8],
    sender: &mut mpsc::Sender<u8>)
{
    print!("[{:>10}]", timestamp);
    for b in bytes {
        print!(" {:X}", b);
    }
    println!();
    for &b in bytes {
        sender.send(b).unwrap();
    }
}
