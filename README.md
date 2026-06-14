# human-benchmark-but-in-exe

A native desktop clone of [Human Benchmark](https://humanbenchmark.com)'s Reaction Time test. Python + tkinter, packaged as a Windows `.exe`. Wait for green, click fast, see your milliseconds.

## What it does

- Clean red / green / blue UI in the style of humanbenchmark
- Random 2–5 second hold before the green flash so you can't time it
- **Too-soon penalty:** click before green and you eat a flat **350 ms** on your record — no free retries for guessing
- Live stats after every click: attempts, average, and best
- **R** wipes your scores and starts fresh

## Run

Python 3 (tkinter is bundled with it on Windows):

```
py reaction_time.py
```

## Build the .exe

```
py -m pip install pyinstaller
pyinstaller --onefile --windowed --icon=reactiontimeicon.ico reaction_time.py
```

Find the build in `dist/`.

## How to play

1. Click anywhere to start.
2. Sit tight on red.
3. Green hits — click as fast as you can.
4. Keep clicking to log more attempts, or hit **R** to reset.

Jump the gun before green and it logs a 350 ms penalty into your average. Patience pays.

## Timing accuracy

The clock arms the instant tkinter flushes the green repaint — the nearest thing the toolkit offers to "green is actually on screen." It can't measure monitor/compositor presentation lag (you'd need a photodiode for that), so readings may sit a few ms above a hardware-measured time. Plenty consistent for beating your own scores, just not lab-grade.
