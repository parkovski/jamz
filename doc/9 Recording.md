# Part 9: Recording

Side note: I opened the MIDI spec, there are actually 128 notes, not 256. The
MSB is always 0 for a message parameter it seems.

Back in part 7, I outlined 3 problems and solved two of them. The last one is
more complicated than the others so it needs to be broken down further.

The eventual goal is to be able to play some amount of music, have that be
recorded and then replayed. Two things:

- The computer doesn't understand "some amount".
- Recording implies allocating.

Let's just call "some amount" one bar for now. Four beats. 384 "pulses".
1777778 (rounded) microseconds, as the BPM value is also hard coded for now.

Now for the second point, we gotta draw some boundaries here. This is not
airplane software. It's not even synthesizing audio, which would have glitches
if real time constraints were not met. However, we'd still like to do our best
to get things happening on time. For now, I'm just going to pre-allocate a
bunch of message spaces and panic if we use more than that. I think that at
some point, we could create a custom vector-type object that does an allocation
far in advance of when it is needed during downtimes. That, combined with
measuring allocation time and memory usage would probably be enough for this
program.

For now, recording will have a time limit, but in the future we might want to
support recording until a switch is flipped. To support this, I'm adding
`recording` and `recording_limit` variables to the `Sequencer` struct. No limit
is represented by `i64::MAX` because this simplifies logic - we will never pass
this value. The recording limit is in microseconds from
the monotonic clock. Really somebody's gotta figure out a good way to handle
units in programming languages - I think this would nearly get rid of a whole
class of bugs, but as the great prophet Mick Jagger said, you can't always get
what you want.

Now we have to figure out how to communicate that recording should be enabled.
There is a standard way to do this, but it's sysex and hahaha no. I do however
want this to be entirely run by MIDI, so for now I'll just take the undefined
message before start (0xF9). I'm unsure about this goal because I'd like to be
able to input numbers to set a recording time, but I have an idea how this
might be done.

Getting recording to work reliably is going to take a bit more work. We can
send note on/off messages to the sequencer and push them onto `messages`, but
we can only record this way one time and one track. There are some places that
assume `messages` is not empty, which is also a problem. It may be time for
some design.
