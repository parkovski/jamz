# Intro

Wow writing is hard. My mind is super scattered.

## Subtitle

You can't hold on to a thread and tell it what to do. You can't send a message
to a thread in real time. All you can do is put stuff in a shared location and
leave it there until the other task gets around to checking the place. Then you
can approach the second task independently and try to make it check the place
often and efficiently. You can sleep and wake the thread, so you can make it
suspend itself when it's not doing anything and then have another thread start
it again when there is work to do. But you still can't wake it and tell it what
to do - once you spawn it, it is its own thing.

## TODO: Put something here.

Ah yes, multithreading, one of the great <s>joys</s>terrors of programming. I'm
going to start by making a simple app that copies MIDI data from an input port
to an output port. This is going to require multiple threads. Or to be clear,
it really doesn't, but in the interest of self-inflicted suffering I'm gonna use
multiple threads. Also it sets up for later things that do need multithreading.

## When to use threads vs async or coroutines or something?

"Async/await is so hot right now" - not zoolander.

It is though, all the cool languages are doing it, proving once again that Java
is not cool. But anyways way back in the jurassic era we had to use threads.
Well, actually, back then we had to use cooperative multitasking but probably in
like the stone age they invented threads. Like, one day a guy came out with a
bowl and everyone was like wow cool you can put your food here, and the next day
he was like hey I got this idea for running multiple tasks concurrently by
saving their state and giving them alternating small slices of time. Everyone
was confused and we forgot about threads until like the 1960's or something.

And then somehow in like 2005 or 2010 or something that I can't be bothered to
look up, Anders Hejlsberg reinvented cooperative multitasking and everyone was
like wow this is so new and modern, I bet the dinosaurs didn't have this. If
only dinosaurs had also invented a nice state machine syntax. Oh well.

So let's start off with how these work. They both need some sort of scheduler,
but coroutines need to cooperate with the scheduler. Threads don't. There's a
lot of nuanced overlap here though.

## Is this document serious at all?

No, probably not.

## Back to our regularly scheduled programming

Ok so I'm making this simple app composed of multiple threads. It'd be good to
understand what they are and what they do.

1. The user input thread. Just sits around doing nothing and waiting for the
  user to press enter and end the program. We'll call this one Jimothy.
2. The MIDI input thread. Created by the OS or MIDI framework and should do a
  minimal amount of work before passing the data on. This one gets named
  Pshtephanie.
3. The work thread. Mysterious things will happen here. For now though, it's
  just going to write MIDI data to the output port. This one is named 🅱teve.

Hmm, so how do we get MIDI data from Pshtephanie to 🅱teve?

## X is for you to write and for me to not understand

Where X is any concurrent data structure. Some mythical beings write these
things and we set aside a day of the year to be thankful that there are people
in the world who actually understand the dark art of concurrency. This day is
in early April but some years it also inexplicably shows up in September.