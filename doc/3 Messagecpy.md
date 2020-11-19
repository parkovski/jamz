# Messagecpy

Well, we were gonna have to get into the details of MIDI eventually. Since we
can't just copy one byte at a time, we're gonna need to know a little bit about
what we're actually sending. Luckily, I had been writing an async MIDI library
in C++, proving that my sanity is lacking but also giving me a decent
background on how MIDI works.

So the first thing to understand about MIDI is sysex doesn't exist, it's a
conspiracy by some music device programmers looking for job security. The
second thing to understand is that a message can be a maximum of three (3)
(iii) (0b11) bytes. Sweet, thanks gods of MIDI for making this format really
easy to use and definitely not complicated! We can take advantage of this
and change the MPSC channel to send `[u8; 4]` and use the last byte for the
message length. This way the... um... well, in C++ we use pointers a lot, and
it seemed kinda cool to have the message plus the length fit into the size of
an int and have the pointer to that whole thing be equivalent to the pointer to
the message bytes if that makes sense. Could just be my insanity.

Now at this point you might be asking why I'm not using a slice or a vector or
some other idiomatic Rust thing, and to that I respond that I'm too cool for
school and in hacker culture one's social status is determined by how cryptic
of an encoding one can create. But really it's just that Rust doesn't have
something that meets this weird niche requirement.

- Don't want to allocate.
- Can't pass a borrowed reference to a different thread.
- Don't know the exact size of the array but do know within a couple bytes.

When I eventually come to terms with the reality that sysex is a thing, I will
have to change this a bit, but for now let's just keep living in sweet
ignorance. Sweet, sweet `[u8; 4]`.