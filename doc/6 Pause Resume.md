# Part 6: Pause/Resume

In the last part, we got the loop to play and stop on command, but it resets
fully each time it is stopped. It would be nice if we could pause and resume
instead.

The first thing is to change the time keeping to use offsets relative to the
previous message. I don't know why, but it seemed like a thing that could be
done. I'm also giving variables more descriptive names because I'm not a real
h4xx0r.

This part is pretty simple but I want to comment on how much I hate simple math
(addition, subtraction, multiplication, division). I always tell people that
programming doesn't contain that much math and usually that's true, but
sometimes when dealing with time or space as in graphics or music, you have to
do a lot of simple math and it easily gets overwhelming to me. I'd much rather
do trig or calculus and that's not bragging, I'm just that bad at keeping track
of more than one or two additions.

Next I'll add a variable `pause_offset`, representing the amount of time
passed since the last message was sent when the stream is paused. There are
three messages in the MIDI spec which are slightly different from most music
UIs: start, continue, and stop. So pause and stop both become MIDI stop, play
becomes either start or continue.
