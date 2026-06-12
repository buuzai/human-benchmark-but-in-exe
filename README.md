# human-benchmark-but-in-exe

A desktop clone of [Human Benchmark](https://humanbenchmark.com)'s Reaction Time test, built with Python + tkinter. Click when the box turns green, get your time in milliseconds.

## Features

- humanbenchmark-style red/green/blue UI
- Random 2–5 second delay before green so you can't cheat the timing
- "Too soon!" detection if you click before green
- Running stats: attempts, average, and best time
- Press **R** to reset your scores

## Run it

Requires Python 3 (tkinter ships with it on Windows).

```
py reaction_time.py
```

## Build the .exe

```
py -m pip install pyinstaller
pyinstaller --onefile --windowed --icon=reactiontimeicon.ico reaction_time.py
```

The executable lands in `dist/`.

## How to play

1. Click anywhere to start.
2. Wait on the red screen.
3. The moment it turns green, click as fast as you can.
4. Click to keep going, or press **R** to wipe your stats.

## A note on timing accuracy

The clock starts the instant tkinter flushes the green repaint — the closest point the toolkit gives you to "green is now on screen." It can't account for monitor/compositor presentation lag (only a photodiode rig could), so your numbers may read a few ms higher than a hardware-measured reaction. It's consistent enough to compare your own attempts, just don't treat it as lab-grade.
