use std::io::{self, Read};

pub fn wait_for_return() {
    let mut stdin = io::stdin();
    let mut b = [0u8];
    loop {
        match stdin.read(&mut b) {
            Ok(1) if b[0] == 13 => {
                return;
            }
            Err(_) => return,
            _ => continue,
        }
    }
}
