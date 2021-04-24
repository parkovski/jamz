# Part 11: Patterns

It occurs to me that one way to do recording and playback at the same time is
to separate them. We could have any number of loops currently recording and any
number of loops playing, but a given loop cannot be both. Once recording is
finished, the loop moves from the recording list to the playback list.

My last attempt at implementing recording didn't go that well. One issue was
trying to record to and play from the same queue. Another issue was how to
remember the recording end time.

I'm going to call a block of messages a pattern, because that's what it's
called. I'm going to call a position index a cursor, also because that's what
it's called. We need to remember the previous and next times in microseconds,
although this is an approach that won't work with tempo modulation. Changing
the tempo is gonna really mess things up and will take some thought, so I'm
going to engage in the celebrated tradition of pretending the problem doesn't
exist for now. Kind of like sysex. 640, um, bpm, oughta be enough for everyone
right?
