use std::sync::mpsc;

pub fn start_worker_thread(rx: mpsc::Receiver<u8>) {
    std::thread::spawn(move || work(rx));
}

fn work(rx: mpsc::Receiver<u8>) {
    println!("begin worker thread");
    loop {
        let data = match rx.recv() {
            Ok(data) => data,
            Err(_) => return,
        };
        print!(" {:X}", data);
    }
}
