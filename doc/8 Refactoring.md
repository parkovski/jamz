# Part 8: Refactoring

Aw yeah, the fun part. Actually, it's kind of a rite of passage or maturity
thing to me. Acknowledging that you can't just hit keys all day and expect to
make something cool - sometimes you have to stop and think about the
organization of what you're making.

The first step will be to sketch out the main loop - rewrite it with the main
`if playing` left alone, but the bodies of the branches replaced with calls to
(currently) non-existant functions. I like to split the window so I can view
two different places in the same file. I also wrap my code at 80 columns -
some people think this is unnecessary, but I work on a Surface Pro
(unfortunately I've had tons of issues with it) and this is the only way I can
see two files at the same time.

So I guess I'll call the two branches logic `play_loop_step` and
`check_paused_messages`. Then I'll create those functions and start filling
them in, recursively doing the same process until I have a bunch of small
functions that don't work because all the variables they need to access are
only in the main function. You know that joke, there's two hard problems in
computer science: cache invalidation, naming things, and off by one errors?
We're definitely going to run into all of those problems in this project.
Although we could also rephrase the joke to be there's only one hard problem
in computer science - all of it.

One thing to watch out for is not to lose the comments when copying code.
Spending a lot of time in this code, it's easy to pass them over, especially if
I'm retyping something instead of copy/pasting. With no sarcasm at all,
commenting the correct things is definitely a sign of maturity in programming
skills. Now with sarcasm, another sign of maturity is not passing `std::string`
by value all over the place. Actually, if someone knows about
`std::string_view`, you should probably hire them immediately. Btw I know about
`std::string_view`.

Another thing to watch out for is `return` statements - when you move these to
a new function, they don't accomplish the same thing anymore. So I've
decided to write about this process, and this is what I'm doing right now, but
I really am not trying to give refactoring lessons so I'm just gonna get this
done and move on.

It works, but I ran into this bug: 

```rust
// Sleep a bit before checking for messages again.
self.now = monotonic_sleep(
  std::cmp::max(self.target_time - self.now, MAX_SLEEP));
```

I had forgotten the assignment to `self.now` when I refactored this part,
causing the current time to not get updated and the loop to get stuck on the
first note. Refactoring is hard and this is not the end of it.
