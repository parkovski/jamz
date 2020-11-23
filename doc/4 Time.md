# Part 4: Time

Oh God. Remember when I said "this is the hard part"? Yeah, actually this is
the hard part. At least, until probably the next section when something harder
shows up. In making a music app, we gotta keep track of time, no way around it.
The nice part is the threading will start to make sense here.

It's been about 2 days since I wrote that paragraph. Research stuff. You know.
What I found is that timing things accurately is hard. It's not immediately
apparent how that should even work but the best I can figure is this:

- Set up a separate thread at a high priority level. On Windows this is done
  using `SetPriorityClass` and `SetThreadPriority`.
- On Windows, you need to call `timeBeginPeriod` and `timeEndPeriod` to make
  `Sleep` precise enough. This affects a global Windows setting and seems like
  a really bad design but that's apparently how it's done.
- On Linux you can use `nanosleep`, but on Windows you have to write your own.
- Use the high priority thread to sleep until the next action.

So once again we've learned that Windows sucks, but it's what I'm stuck with
at the moment. Now this app becomes platform specific, but I will cover Linux
and maybe macOS later on. Hoping to get this thing running on a Raspberry Pi,
so definitely Linux.

I am going to try to keep platform-specific stuff separate, so I'll make a new
directory for that. I'm calling this thing the `Sequencer`. This is a thread
whose job is to keep a list of upcoming messages and play them at the
appropriate time. It currently has a hard coded list of messages - the melody
of the first bar of "She's Not There" by The Zombies. I recreated a bit of the
intro and I can start that playing and then run the program at the right time
and it will play the melody!

Side note here, there are very few places in the Windows API where functions
don't use `PascalCase`, and the `time*` functions are one of them. Others are
WinMM MIDI, and the functions `keybd_event` and `mouse_event`. Makes me wonder
how these came to be. Sockets are also this way, but it makes more since as
they just copied BSD.

A "cool" and "fun" side project would be benchmarking this abomination I wrote
against `nanosleep` on Linux. Well, whatever, it works.