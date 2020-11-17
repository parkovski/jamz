# Setup

So to start with I want three threads. Input, MIDI, and worker. The MIDI thread
is created for me and could be more than one thread as far as I'm concerned,
I just care that it's separate from the other threads.

I'll start by creating separate files for each thread. We'll have `term.rs`
for terminal user input, `worker.rs` for what right now will just copy the
data it receives, and `midi/input.rs` for MIDI input. A super over-engineered
memcpy. At least there's no `FactoryFactory`s though, right?

Mostly I want these in separate files to make me think about them as really
separate things. Each thread doesn't know what the other one is getting up to,
so when I'm working on one of the threads, I'm only looking at that one.

Maybe you noticed that I put the MIDI input file in its own directory. That's
because I like it the best. Remember, confident programmers pick favorites.
Anyways there is some other MIDI stuff that needs to go into that folder - this
is going to be a MIDI looper app after all. For now I'm going to put the other
stuff in `midi/manager.rs`. This file will be responsible for opening MIDI
ports and keeping them around. I'm also going to deal with the dumb BS that is
error handling in `midi/error.rs`.

Wow, that's like 7 source files and hardly any functionality! Looks like we're
well on our way to successful big tech employee status. Also check this shit
out. I used the abbreviation `inp` instead of `in` so that code dealing with
input and output will line up. So good.

At this point you can use `-l` to list available MIDI ports, `-i` to select an
input port, and `-o` to select an output port which currently doesn't do
anything but we're getting there. With an input port selected, you'll see all
the MIDI bytes printed to the console! Hooray!

Now, the console is kinda wack at this point and that's because threads are
cool. Luckily since I'm a pro I can tell that we just need to add the
`synchronized` keyword from Java to every function. Also luckily this is not
Java so let's come up with a better way.