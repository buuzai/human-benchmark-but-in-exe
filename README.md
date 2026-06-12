# Reaction Time

A native Windows reaction time tester — no browser, no overhead, just you and the clock.

## Why not just use Human Benchmark?

Browser-based tests have a problem: there's a layer of rendering pipeline between your click and the measurement. The score you get reflects your browser as much as it reflects you. This runs native, so what you measure is what you are.

91 lines of Python. Compiled to a standalone `.exe`.

---

## Download

Grab the latest `.exe` from the [Releases](../../releases) page. No Python required.

---

## Build it yourself

```bash
py -m pip install pyinstaller
py -m PyInstaller --onefile --noconsole --icon=reactiontimeicon.ico reaction_time.py
```

Output lands in `dist/`.

---

## License

Open source. Do whatever.
