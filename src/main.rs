use clap::Arg;

mod midi;
mod term;
mod worker;

fn list_devices() {
    println!("midi in:");
    for name in midi::Manager::inp_devices() {
        println!(" - {}", name);
    }
    println!("midi out:");
    for name in midi::Manager::out_devices() {
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

    let input = matches.value_of("input");
    let output = matches.value_of("output");

    let (tx, rx) = std::sync::mpsc::channel();
    worker::start_worker_thread(rx);

    let mut man = midi::Manager::new(tx);
    if let Some(name) = input {
        man.open_inp(name).unwrap();
    }
    if let Some(name) = output {
        man.open_out(name).unwrap();
    }

    print!("Press enter to exit.");
    term::wait_for_return();
}
