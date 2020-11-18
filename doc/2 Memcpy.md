# Part 2: Ridonculous memcpy

At this point I have successfully made a useless thing. Good job me.
Unfortunately the worker thread doesn't know about the output port, and fixing
this is not totally straightforward for two reasons:

1. Writing threaded code the right way is hard.
2. Rust really tries to prevent us from doing it the wrong way.

So we're suddenly forced to consider the lifetime of the stuff being passed
into the thread, which is not long enough as the worker thread outlives
`main`. There is more than one way to approach this problem, but I'm going to
add a dependency on [crossbeam](https://crates.io/crates/crossbeam) and use a
scoped thread. This has the nice side effect of making the program hang forever
when `main` ends, which is a bonus since users are dumb and deserve to be
annoyed.

Right, so I don't want to punish the users if it means I also have to punish
myself, so maybe let's find a way to signal the new thread to finish. Here's
my first attempt:

```rust
// in main
let running = std::sync::atomic::AtomicBool::new(true);
let ref_running = &running;
crossbeam::scope(move |s| {
    s.spawn(move |_| worker::work(rx, ref_running));

    print!("Press enter to exit.");
    term::wait_for_return();
    ref_running.store(false, std::sync::atomic::Ordering::Release);
}).expect("Thread error!");

// in worker::work
while running.load(std::sync::atomic::Ordering::Acquire) {
    let data = match rx.recv() {
        Ok(data) => data,
        Err(_) => return,
    };
    print!(" {:X}", data);
}
```

This... well it's possible to close this program, you just have to hit another
MIDI key after pressing enter. Of course I'm really smart and I knew this
wouldn't work - I only tried it cause some of you dumb dumbs might have thought
to do it this way and I wanted to disprove it. Jk this was the best I could do
first time.

So shortly after this I realized that the way this has gotta be done is to
interrupt the polling of the MPSC channel, which luckily happens automatically
when all the senders are closed. I can take out the `AtomicBool` and just
explicitly `drop` the `midi::Manager` after the user presses return (enter? why
is this not clear?) and it all works!

Anyways, see that `Err(_) => return` branch? That's how the pros do it - you
gotta just throw those errors away! Actually I'm working on a new language
which is the same as Java except every statement is wrapped in
`try { ... } catch { /* do nothing! */ }`. Gonna get that shit taught in all
the schools, not like student programs could actually get worse.

So. Now the hard part. Well, every part is hard but this is one of them. The
worker needs to somehow know where to send output. Currently it's just sending
it to the console. This is only good if we're performing for robots but usually
humans want the MIDI data turned into notes.

Now we gotta think about lifetimes again, because the nice `midi::Manager` that
I made does not and can not live long enough to be shared with the worker
without making the program hang. Here I need to remember how I was talking
about separating the input and output tasks and separate their data too. Once I
drop the input, the worker thread will clean up and eventually drop the output
too.

So I'm going to get rid of `midi/manager.rs` and split it up between
`midi/input.rs` and `midi/output.rs`. Ugh, using the full names 'input' and
'output' doesn't line up the columns anymore. Such are the sacrifices we make
for better readability.

As I'm rewriting this, I notice there's kind of a cool symmetry if I get rid of
`worker.rs` and move that code to the `OutputManager`, so I'm going to do that.

Awwwww yessssss, so close. Gonna just go into the work function where the data
is received, and put a `self.connections[0].send()`... wait... "has to be a
valid MIDI message". You mean we can't just send it one byte at a time? 😭

The program at this point does the exact same thing it did at the end of the
last post, but this time it's organized better. It's one of those really
rewarding times when you do a bunch of work to see nothing change.