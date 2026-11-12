# human-benchmark-but-in-exe

A native desktop clone of [Human Benchmark](https://humanbenchmark.com)'s Reaction Time test, built to add as little delay as possible. Wait for green, click fast, see your milliseconds.

There are two versions in this repo:

- **Rust (main version):** fullscreen, drawn by the GPU, raw mouse input. Builds to one small standalone Windows `.exe`.
- **Python + tkinter:** the original windowed version. Simpler, but slower to show green.

## What it does

- Clean red / green / blue UI in the style of humanbenchmark
- Random 2 to 5 second hold before the green flash so you can't time it
- **Too-soon penalty:** click before green and you eat a flat **350 ms** on your record. No free retries for guessing
- Live stats after every click: attempts, average, and best
- **R** wipes your scores and starts fresh

## Rust version

### Build

Needs [Rust](https://rustup.rs) on Windows.

```
cargo build --release
```

Find the build at `target/release/reaction_time.exe`. It is a single file with nothing to install.

### Controls

| Input | Action |
| --- | --- |
| Left click | Start a round / react |
| R | Reset scores |
| Esc | Quit |

The line at the bottom of the screen shows the refresh rate Windows is using and whether tearing and raw input are active.

### Why it is faster

- **Fullscreen with vsync off:** a flip-model Direct3D swap chain with tearing allowed, so green does not wait for the next screen refresh.
- **Green is drawn ahead of time:** while red is showing, the green frame is already finished. At the random moment the program only presents it, then starts the clock.
- **Raw mouse input:** the click is read directly and timestamped the moment it arrives, while the loop spins instead of sleeping.
- **High process priority** so Windows is less likely to delay it.

## Python version

Python 3 (tkinter is bundled with it on Windows):

```
py reaction_time.py
```

Build it as an `.exe`:

```
py -m pip install pyinstaller
pyinstaller --onefile --windowed --icon=reactiontimeicon.ico reaction_time.py
```

Find the build in `dist/`.

## How to play

1. Click anywhere to start.
2. Sit tight on red.
3. When green hits, click as fast as you can.
4. Keep clicking to log more attempts, or hit **R** to reset.

Jump the gun before green and it logs a 350 ms penalty into your average. Patience pays.

## Timing accuracy

The clock starts right after the green frame is handed to the screen, and the click is stamped as soon as it is read. Monitor response time and mouse delay are still inside every result. No software can measure those (you would need a photodiode), so readings sit a little above a hardware-measured time.

The Python version draws through the normal desktop path, so green can show up to about one refresh later and its results read higher than the Rust version.
