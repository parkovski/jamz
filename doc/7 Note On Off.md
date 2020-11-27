# Part 7: Note On/Off Resolution

Let's recap some of the problems here:

- Pausing doesn't send a note off message and resuming doesn't send a note on.
- Quitting is broken because the sequencer thread never exits.
- The loop sequence is hard coded and can't be modified.

On the other hand, this is still pretty cool even if it's not that useful right
now. I successfully made threads communicate which is basically a dark art. But
back to work. I've been ignoring the quit issue, but I may be able to solve it
the same way as the output thread - just return when `recv`/`try_recv` returns
a disconnected error.... And this works! Hooray! That loop is getting messy
though, I'll have to deal with that soon.

The other tasks are not quite as simple though. The last one is kind of the
eventual goal of this series, so let's look at the first one. Basically, all
we have to do is keep track of note on messages and send a corresponding note
off when paused, then resend the note on when resumed. A few minutes ago I
thought this needed a hash map, but then as I typed `u8` I remembered, there
are only 256 possible notes! This *will* change when we start supporting more
channels, but for now we can keep this simple.

The thing about allocation is it is not a real time operation, and I have to
remember that the sequencer thread is not intended to be real time. But in
order for it to not be real time, I need to restructure a couple things:

- The output thread needs a cache.
- The sequencer needs to feed messages to the output thread in advance.

I'd like to do these soon so I'm not trying to retrofit them onto a complicated
program.

I'm adding a `notes_on` variable - this contains the velocity of the active
notes. If the velocity is 0, the notes are not active. I'm going to accomplish
the updating and checking this with some closures for now, but I think it's
about time to refactor this big function in the next part.

I just have to update `notes_on` when a note on or off message is sent, and
send the appropriate messages by checking it when pausing/resuming. And it
works. Amazing.
