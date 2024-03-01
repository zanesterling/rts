Tinkering on a little RTS for fun.

Read along in [the log](LOG.md).

## Build & Run

### Linux
```
$ sudo apt install libsdl2-image-dev libsdl2-ttf-dev
$ cargo run -r
```

### macOS
```
$ brew install cmake
$ brew install sdl2 sdl2_image sdl2_ttf
$ export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
$ cargo run -r
```
