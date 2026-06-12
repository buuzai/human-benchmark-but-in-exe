# Reaction Time

A native Windows reaction time tester — no browser, no install, just you and the clock.

## Why a native app?

Browser-based tests run inside a rendering pipeline, an event loop, and a JS layer that all sit between your click and the measurement. A native app cuts out the browser and JS layers, so there's less *between* you and the timer.

To be clear about what this does **not** fix: any GUI test — this one included — still has display latency. The clock can't know exactly when the green pixels actually light up your monitor; there's repaint, compositor, and panel response time in the way. The only way to measure that precisely is with a photodiode and external hardware. So treat your number here the same way you'd treat a Human Benchmark score: good for tracking yourself over time, not an absolute measure of human nerve speed.

What this version does do is start the timer *after* the repaint is queued rather than before it, so the clock isn't running while the frame is still being drawn. It's a small honesty fix, not a magic latency eliminator.

101 lines of Python. Compiled to a standalone `.exe`.

---

## Download

Grab the latest `.exe` from the [Releases](https://github.com/buuzai/human-benchmark-but-in-exe/releases) page. No Python required.

---

## Build it yourself

```
py -m pip install pyinstaller
py -m PyInstaller --onefile --noconsole --icon=reactiontimeicon.ico reaction_time.py
```

Output lands in `dist/`.

---

## License

Open source. Do whatever.
