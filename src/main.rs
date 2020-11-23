use clap::Arg;

mod midi;
mod os;
mod term;

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
    let mut input_manager = midi::InputManager::new(tx_out, tx_seq);
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

        print!("Press enter to exit.");
        term::wait_for_return();

        // Sequence of events here: all senders are destroyed -> receiver
        // returns error -> worker thread finishes -> worker thread joins
        // main thread -> program ends.
        drop(input_manager);
    }).expect("Thread error!");
}
