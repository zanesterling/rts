# Development log

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
