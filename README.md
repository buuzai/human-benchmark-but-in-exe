# human-benchmark-but-in-exe

A native desktop clone of [Human Benchmark](https://humanbenchmark.com)'s Reaction Time test, built to add as little delay as possible. Wait for the screen to turn green, click as quickly as you can, and see your reaction time in milliseconds.

There are two versions in this repository:

- **Rust (main version):** fullscreen, rendered by the GPU, with raw mouse input. Builds to one small standalone Windows `.exe`.
- **Python + tkinter:** the original windowed version. Simpler, but slower to display the green screen.

## Download

Download the latest `.exe` from the [Releases page](https://github.com/buuzai/human-benchmark-but-in-exe/releases/latest). It is a single file and requires no installation.

## Features

- Clean red, green and blue interface in the style of Human Benchmark
- Random 2 to 5 second delay before the screen turns green, so the change cannot be anticipated
- **Early-click penalty:** clicking before the screen turns green records a fixed **350 ms** result, so guessing is not rewarded
- Live statistics after every click: attempts, average and best time
- Press **R** to clear your scores and start over

## Rust version

### Build

Requires [Rust](https://rustup.rs) on Windows.

```
cargo build --release
```

The executable is created at `target/release/reaction_time.exe`.

### Controls

| Input | Action |
| --- | --- |
| Left click | Start a round / react |
| R | Reset scores |
| Esc | Quit |

The line at the bottom of the screen shows the refresh rate Windows is using and whether tearing and raw input are active.

### Why it is faster

- **Fullscreen with vsync off:** a flip-model Direct3D swap chain with tearing allowed, so the green screen does not wait for the next display refresh.
- **Green is drawn ahead of time:** while the screen is red, the green frame is already finished. At the random moment the program only presents it, then starts the clock.
- **Raw mouse input:** each click is read directly and timestamped the moment it arrives, while the loop spins instead of sleeping.
- **High process priority,** so Windows is less likely to delay the program.

## Python version

Requires Python 3 (tkinter is included with it on Windows):

```
py reaction_time.py
```

To build it as an `.exe`:

```
py -m pip install pyinstaller
pyinstaller --onefile --windowed --icon=reactiontimeicon.ico reaction_time.py
```

The executable is created in `dist/`.

## How to play

1. Click anywhere to start.
2. Wait while the screen is red.
3. When the screen turns green, click as quickly as you can.
4. Click again to record another attempt, or press **R** to reset your scores.

Clicking before the screen turns green adds a 350 ms penalty to your average.

## Timing accuracy

The clock starts immediately after the green frame is sent to the display, and each click is timestamped as soon as it is read. Monitor response time and mouse delay are still included in every result. Software cannot measure these (doing so requires external hardware such as a photodiode), so results are slightly higher than a hardware-measured reaction time.

The Python version renders through the standard desktop path, so the green screen can appear up to about one refresh later and its results are higher than those of the Rust version.
