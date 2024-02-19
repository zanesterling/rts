# Development log

## 2024-02-19
Today I'll make some obstacles, and have the units bump into them.

## 2024-02-18
Today I want to swap out the rendering of units to use sprites from the sprite
sheet.

---

Did it! Boom! There's definitely some cleanup to be done though. The interface
is a little grody.

---

What do I want to add now? I've got sprites that can move around...
Maybe next I'll make it so you can pan around by holding middle-mouse.

So let's think about the state machine here. If you hold middle-mouse, it
should cancel any ongoing drag interaction.

## 2024-02-17
Wrote the code to slice up a sprite sheet and blit just one sprite to the
screen. Woo!

## 2024-02-16
Coming back a couple days later. On the 14th I ran into some difficulty getting
SDL2 to build with image support. I've come back to it now, and found an easy
explanation for why:

I needed to `sudo apt install libsdl2-image-dev`, not `libsdl2-image-2.0.0`.
After doing that, I can build with the `"image"` feature enabled, no problem.

---

Managed to load a PNG file and blit it to the canvas. Boom! Next steps will be
dividing that image into a sprite sheet, and showing blitting just one sprite
from the sheet. Then after that I can try to do animations :o

## 2024-02-14
Let's try loading and rendering sprites in place of the units today.

## 2024-02-13
Written the morning after.
 
I achieved what I intended. There are a few units and I can box select and even
tell them to move.

## 2024-02-12
Back again today. Let's make tonight's mission trying to get mouse input
working.  We'll react to a click event, and when it happens we move the only
rectangle to that spot.

---

Nice! It's time for bed now. Tomorrow I want to have units and be able to box
select them.

## 2024-02-11
I'm developing this on WSL. To get a window rendering with SDL2 I needed to go
through a few hoops.

First, `apt` only had `libsdl2-2.0-0` up to version `2.0.10`. To get around the
issue I had to use the "bundled" feature of `rust-sdl2`, as suggested [in their
README](https://github.com/Rust-SDL2/rust-sdl2?tab=readme-ov-file#bundled-feature).  
Once I fixed that I could build successfully. I still get the following
warnings, which I'll look into at some point.

> warning: dependency (sdl2) specified without providing a local path, Git repository, version, or workspace dependency to use. This will be considered an error in future versions
> warning: unused manifest key: dependenciess

But! When I ran the demo, no window popped up and I saw a non-terminating error:
`error: XDG_RUNTIME_DIR not set in the environment.`

Some searching led to
[these instructions](https://dev.to/winebaths/getting-up-and-running-with-the-windows-subsystem-for-linux-8oc),
which also suggested that I install a better terminal emulator with access to
colors. That's real nice! Big ups to mintty. After following the instructions
through step 6 I had XFCE (X Window Server) installed and connected to XMing.
At this point I could `cargo run` and see a window with an animating color.
wahoo!
