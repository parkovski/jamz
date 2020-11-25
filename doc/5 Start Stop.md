# Part 5: Start/Stop

At this point, we have a sequencer with a hard coded snippet from a song. I'm
changing the hard coded track to the bass line from "She's Not There" for now
for reasons. It'd be good to be able to start and stop it on demand instead of
starting the program at the exact right moment.

Starting and stopping introduces a lot of messy logic around time and it still
doesn't cover everything. Pausing in the middle and resuming is still not
supported.

I have to say that for this particular task, a programming language that
supported units would help a lot. I messed up the piece of logic converting
pulses to microseconds a few times before writing the equations on paper with
units and uncovering my errors. I remembered taking chemistry in high school
and thinking I was smart enough to do this in my head. Now I think I'm smart
enough to just write it out because that's easier.

One important thing I changed here is adding a maximum sleep time. Maybe
you've seen those pieces of orchestra music where the first part is "rest for
130 bars"? If the program got into this scenario, we'd have to wait a long
time for it to exit. So any thread that sleeps needs to have a limit of how
long before it checks for messages. There's a nice analogy here for social
media addiction - some people are blocking on `recv` when they should be
calling `try_recv` every once in a while and then going about other business
in the mean time. Too nerdy?

I need to clarify something at this point. There is a sequencer thread and an
output thread, and I've been kind of unclear on the difference between them.
Right now the output thread is just a middleman type thing that really doesn't
add any value, and the sequencer is doing what I'd probably expect the output
thread to do.

Here's how I envision these threads to work together. The input and output
threads are the only ones that should need to respond in real time. The output
thread is going to keep a small cache (ring buffer) of upcoming messages and
send them at the appropriate time. Then the sequencer, not being real time,
is free to allocate memory and do more complicated things. The sequencer will
have its own command system letting it act sort of as a compiler from higher
level commands to MIDI.

The way I think of this program is, the way vim manipulates text, this should
manipulate MIDI.
