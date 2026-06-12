import tkinter as tk
import random
import time

# humanbenchmark-style colors
RED = "#ce2636"
GREEN = "#4bdb6a"
BLUE = "#2b87d1"
WHITE = "#ffffff"

class ReactionTime(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Reaction Time")
        self.geometry("700x400")
        self.minsize(300, 200)

        self.state = "idle"  # idle, waiting, ready, result, toosoon
        self.after_id = None
        self.start_time = 0
        self.clock_armed = False
        self.times = []

        self.frame = tk.Frame(self, bg=BLUE, cursor="hand2")
        self.frame.pack(fill="both", expand=True)

        self.icon = tk.Label(self.frame, text="\u23F1", font=("Segoe UI", 40), bg=BLUE, fg=WHITE)
        self.icon.pack(pady=(60, 10))

        self.title_lbl = tk.Label(self.frame, text="Reaction Time Test", font=("Segoe UI", 36, "bold"), bg=BLUE, fg=WHITE)
        self.title_lbl.pack()

        self.sub_lbl = tk.Label(self.frame, text="When the red box turns green, click as quickly as you can.\nClick anywhere to start.", font=("Segoe UI", 14), bg=BLUE, fg=WHITE)
        self.sub_lbl.pack(pady=10)

        for w in (self.frame, self.icon, self.title_lbl, self.sub_lbl):
            w.bind("<Button-1>", self.on_click)

        self.bind("<r>", self.reset)
        self.bind("<R>", self.reset)

    def reset(self, event=None):
        if self.after_id:
            self.after_cancel(self.after_id)
            self.after_id = None
        self.times = []
        self.state = "idle"
        self.show("\u23F1", "Reaction Time Test",
                  "When the red box turns green, click as quickly as you can.\nClick anywhere to start.", BLUE)

    def set_bg(self, color):
        for w in (self.frame, self.icon, self.title_lbl, self.sub_lbl):
            w.config(bg=color)

    def show(self, icon, title, sub, color):
        self.set_bg(color)
        self.icon.config(text=icon)
        self.title_lbl.config(text=title)
        self.sub_lbl.config(text=sub)

    def on_click(self, event=None):
        if self.state in ("idle", "result", "toosoon"):
            self.start_wait()
        elif self.state == "waiting":
            # clicked before green
            if self.after_id:
                self.after_cancel(self.after_id)
                self.after_id = None
            self.state = "toosoon"
            self.show("\u26A0", "Too soon!", "Click to try again.", BLUE)
        elif self.state == "ready":
            if not self.clock_armed:
                return  # green is showing but the clock hasn't started yet; ignore
            ms = int((time.perf_counter() - self.start_time) * 1000)
            self.times.append(ms)
            avg = sum(self.times) / len(self.times)
            self.state = "result"
            self.show("\u23F1", f"{ms} ms", f"Click to keep going.\nAttempts: {len(self.times)}   Avg: {avg:.0f} ms   Best: {min(self.times)} ms\nPress R to reset", BLUE)

    def start_wait(self):
        self.state = "waiting"
        self.clock_armed = False
        self.show("\u25CF \u25CF \u25CF", "Wait for green", "", RED)
        delay = random.randint(2000, 5000)
        self.after_id = self.after(delay, self.go_green)

    def go_green(self):
        self.after_id = None
        self.state = "ready"
        self.show("\u25CF \u25CF \u25CF", "Click!", "", GREEN)
        self.update_idletasks()  # force the repaint flush
        # Arm the clock right after the flush — the closest point in tkinter to
        # "green is now showing." This still can't account for compositor/monitor
        # presentation time (only a photodiode could), but it puts no extra slack
        # between the paint and the clock start.
        self.start_time = time.perf_counter()
        self.clock_armed = True

if __name__ == "__main__":
    ReactionTime().mainloop()
