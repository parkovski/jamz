# Part 10: Scheduler

The sequencer file is getting a little hard to understand, and in my experience
this is the point when you want to stop and rethink what this program is
supposed to do and how you will achieve that. It'd be helpful to break things
down into smaller testable components. Yeah, testing. So fun.

First thing is to create a new `Scheduler` type and replace the `PlayState`
with an `active` field on the `Scheduler`. Then I will gradually move time
calculations out of the `Sequencer` into the `Scheduler`.

Now I really want to just stop here and not write about this anymore. I did it.
It's done. At the time I'm writing this, I've already spent a bunch of time
being like "oh no I'm so lost, how am I ever going to save this unwieldy
program?" In my experience, this is the nature of software development. You
write a little bit, then it gets all confusing, then you kinda freak out for
a bit, then you calm down and rewrite stuff in a cleaner way with some
comments.

I guess for most of my career I've hoped that one day I'd just become
a design master and like sit down and come up with this awesome diagram of how
everything fits together really neatly and cleanly. But my real experience is
that you kind of vomit code and then come through and pick the tasty parts out
of the vomit (you're welcome) and throw the rest away and then cover that in
more layers of vomit. You end up at the end with some thing that's like mostly
not vomit but there's definitely some sketchy parts in there that you missed.

Anyways in conclusion we're all janitors and the best we can hope for is that
somebody else cleans up their own vomit instead of leaving it for me.

As I was doing this, I realized that as far as recording goes, I already have
a scheduling framework here and there's no need to complicate things further.
It'd be best to create an "end recording" message, insert it at the appropriate
time in the message queue, and let that work itself out. I removed a lot of the
recording code - it was mostly vomit anyways. Now the issue is the queue. Which
like, if you ever thought, "what do I need binary trees for?" Well, for one,
everything. But yeah, efficient random access buffers. Well, trees or skip
lists or some other non-trivial data structure that probably everyone dreads.

Alright. In addition to all this stuff, I've found a (probably) better way to
manage sleeping on Windows as well, using the [waitable timer API](https://docs.microsoft.com/en-us/windows/win32/sync/waitable-timer-objects).
At least, it's more CPU friendly. I gotta say, using the Windows API from Rust
is an absolutely awful experience, but it is doable.
