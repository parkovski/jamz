use std::io::{self, Read};
use clap::Arg;

mod midi;
mod os;

fn list_devices() {
    println!("midi in:");
    for name in midi::InputManager::devices() {
        println!(" - {}", name);
    }
    println!("midi out:");
    for name in midi::OutputManager::devices() {
        println!(" - {}", name);
    }
}

fn main() {
    let matches = clap::App::new("jamz")
        .version("0.1")
        .about("midi stuff")
        .arg(Arg::with_name("list")
            .short("l")
            .long("list")
            .help("list devices")
        )
        .arg(Arg::with_name("input")
            .short("i")
            .takes_value(true)
            .help("input device name")
        )
        .arg(Arg::with_name("output")
            .short("o")
            .takes_value(true)
            .help("output device name")
        )
        .arg(Arg::with_name("play")
            .short("p")
        )
        .get_matches();

    if matches.is_present("list") {
        list_devices();
        return;
    }

    let (tx_out, rx_out) = std::sync::mpsc::channel();
    let (tx_seq, rx_seq) = std::sync::mpsc::channel();
    let mut sequencer = midi::Sequencer::new(rx_seq, tx_out.clone());
    let mut input_manager = midi::InputManager::new(tx_out, tx_seq.clone());
    let mut output_manager = midi::OutputManager::new(rx_out);

    if let Some(name) = matches.value_of("input") {
        match input_manager.open(name) {
            Ok(_) => {}
            Err(_) => {
                println!("Couldn't open MIDI input port '{}'.", name);
                return;
            }
        }
    }

    if let Some(name) = matches.value_of("output") {
        match output_manager.open(name) {
            Ok(_) => {}
            Err(_) => {
                println!("Couldn't open MIDI output port '{}'.", name);
                return;
            }
        }
    }

    crossbeam::scope(move |s| {
        s.spawn(move |_| output_manager.run());
        s.spawn(move |_| sequencer.run());

        println!("Options:");
        println!("  q > quit");
        println!("  p > start playing");
        println!("  x > stop playing");
        println!("  r > resume from last stopped point");
        let mut stdin = io::stdin();
        let mut b = [0u8];
        loop {
            match stdin.read(&mut b) {
                Ok(1) => {
                    match b[0] & !0x20 {
                        //13 => break,
                        81 => break,
                        80 => { tx_seq.send([0xFA, 0, 0, 1]).unwrap(); },
                        82 => { tx_seq.send([0xFB, 0, 0, 1]).unwrap(); },
                        88 => { tx_seq.send([0xFC, 0, 0, 1]).unwrap(); },
                        _ => {}
                    }
                }
                Err(_) => break,
                _ => continue,
            }
        }

        // Sequence of events here: all senders are destroyed -> receiver
        // returns error -> worker thread finishes -> worker thread joins
        // main thread -> program ends.
        drop(input_manager);
    }).expect("Thread error!");
}
