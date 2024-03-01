# Development log

## 2024-02-30
I have a day off today! I'm going to try to work on some big chunk.

Yesterday I added buildings that are selectable. I started to give them
abilities like "train a unit", but discovered that my abstraction for abilities
doesn't quite match this case. The abstraction as it stands is:

```rust
trait Ability {
    fn name(&self) -> &'static str;
    fn keycode(&self) -> Keycode;
    fn cast(&self, &mut game::State, WorldPoint);
}
```

This fits nicely with point- or area-targeted abilities, and could be made to
work with unit-targeted abilities. But it doesn't fit with non-targeted or
auto-self-targeted abilities, like training a unit. There are two issues.

The smaller issue is that the interaction code I've written around this trait
assumes that the trait is point/area-targeted, and so includes two states for
first selecting the ability and then casting it. That could be fixed by
adding another boolean to the trait, or using subtraits or enums.

The larger issue is that in order for an ability to make changes to a specific
unit, it must keep a `&mut` to that unit. That would break all sorts of things,
starting with tracking the ability's lifetime and continuing into blocking any
mutation of the unit by other code while the ability exists. A solution might
be to give each unit a UID. That way the ability can store just the UID, and
look up its caster or target when it's cast.

---

Done! Buildings can train units. I did it as described above, by introducing
UIDs for units and buildings. I also had to add a concept of UnitType, so that
the building knows what the unit's fields should look like.

There are still some rough edges that I'd like to sand off:
- buildings don't show the status of training in progress,
- you have to press the train button and then click, instead of just pressing
  the train button,
- and units all get put in the same spot. If you queue up a few units and then
  sit back, the game will put them all on top of each other. I might not fix
  this one until I have some idea how I want to handle collisions.

---

The first two issues mentioned above are now solved. Units are still all put in
the same spot, but that's okay for now.

## 2024-02-29
Today adding buildings. A lot of the logic should be same as units: they can be
selected, they have abilities ... probably other things are shared too.

## 2024-02-27
Today I just made a tiny improvement: if you can't pathfind to a location, move
directly towards it instead. Otherwise having a melt-brain day again.

## 2024-02-25
Today I did some refactoring, so now it's a bit harder to trip up when doing
coordinate space conversions. Basically just made a couple more helper
functions and switched the existing code to use those. Spring cleaning :)

---

Let's next do some of that crazy window-dragging viewport stuff. When you drag
a window to a new spot, you should see the new world location when you drop it
off.

---

Okay I'm going to back away from the novel window-based mechanic for now. It
sounds cool, and I think it's super neat to have a novel mechanic. But I'm
confident right now that I'm not very good at making something FUN. So I think
I should cut that from the MVP and instead try to make something normal that's
FUN.

---

I added sdl2::ttf as a dependency, so you have to also
`apt install libsdl2-ttf-dev`.

## 2024-02-24
What shall I do this morning? I think I'll try adding a simple, sub-optimal
form of pathfinding. I can improve on it in future steps, but it should get a
guy through the maze. I'll just do neighbors4 BFS, and not worry about
diagonals for now.

---

The other night I had a cool idea for a feature. Very few games do multiple OS
windows. Like, the only ones I can think of are Stellaris et al, which do
in-game windows (not OS windows), and some of Bram's spreadsheet games.  But
both of those just use windows for the same purpose as the OS, to show more
content and let you decide what to look at and how to organize it. I'm
interested in using windows in ways that are more directly tied to the
gameplay.

For instance, maybe you have some overlord-like unit that provides vision. But
rather than have fog of war, each overlord produces/is a different OS window.
To move them around you drag the OS window, and then it snaps back to where it
was and creeps across the screen to the destination.

Or maybe it's like a point&click puzzle game, and you move your flashlight or
something around the screen by dragging the window.

Should be fun to play with & explore ideas. I just need to figure out what the
APIs are like for moving windows around. Maybe SDL has some stuff?

---

Note: would be good to revisit any/all uses of `WorldPoint.x.0` et al and try
to replace them with natural point transformation helpers.

## 2024-02-23
What is an improvement I can make to the game?

- I could add pathfinding.
- I could add waypoints and shift-click.
- I could add production structures.
- I could add VIOLENCE.

Let's do waypoints first :-)

---

Finished up doing the waypoints and shift click. yeet!

## 2024-02-22
What can I do today? My brain is exTREMELY melty. Maybe I can find some nice
code cleaning to do.

---

Okay I think my brain is really actually a puddle. I'm not going to try to do
anything tonight, because I think it would come out my ears from pushing so
hard.

## 2024-02-21
The other night I fixed the bug with #1. My solution was to assume that the
unit is square, and find all tiles overlapping it. Then I check if any is a
wall, and fail the move if so.

## 2024-02-20
Tonight I'm gonna make collision with the walls. The steps are:
1. If you would move into a wall, don't move.
2. If you would move into a wall, move to touch it.
3. If you would move into a wall, move to touch it, then slide along it at a
   speed proportional to the component of your velocity that's parallel to the
   wall.

So far I've done #1, but there's a little bug. I didn't account for how the
units have a radius, so they move until their midpoint touches the wall. Fix!

---

Ooh, actually this is nontrivial in the general case. The issue is: which cell
do you check for collision with the circle?

## 2024-02-19
Today I'll make some obstacles, and have the units bump into them.

---

I've got the map rendering, but no collision yet. I just found a bug:
the unit selection doesn't work right if your camera is offset. Must fix!
Probably I should introduce separate world / camera / screen coords, and
have explicit transforms between them.

---

Fixed the bug by separating camera and world coordinates. Now the only easy way
to convert between them is with `.to_screen(camera)` and `.to_world(camera)`,
which do the appropriate transformations.

Thinking in coordinate spaces is fun :-)

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
