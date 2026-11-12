import tkinter as tk
from random import randint
from time import perf_counter

RED = "#ce2636"
GREEN = "#4bdb6a"
BLUE = "#2b87d1"
WHITE = "#ffffff"

CLOCK = "\u23F1"
WARN = "\u26A0"
DOTS = "\u25CF \u25CF \u25CF"
INTRO = "When the red box turns green, click as quickly as you can.\nClick anywhere to start."
PENALTY = 350


class ReactionTime(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Reaction Time")
        self.geometry("700x400")
        self.minsize(300, 200)

        self.state = "idle"
        self.after_id = None
        self.start_time = 0.0
        self.count = 0
        self.total = 0
        self.best = 0

        self.frame = tk.Frame(self, bg=BLUE, cursor="hand2")
        self.frame.pack(fill="both", expand=True)

        self.icon = tk.Label(self.frame, text=CLOCK, font=("Segoe UI", 40), bg=BLUE, fg=WHITE)
        self.icon.pack(pady=(60, 10))

        self.title_lbl = tk.Label(self.frame, text="Reaction Time Test", font=("Segoe UI", 36, "bold"), bg=BLUE, fg=WHITE)
        self.title_lbl.pack()

        self.sub_lbl = tk.Label(self.frame, text=INTRO, font=("Segoe UI", 14), bg=BLUE, fg=WHITE)
        self.sub_lbl.pack(pady=10)

        self.bind("<Button-1>", self.on_click)
        self.bind("<r>", self.reset)
        self.bind("<R>", self.reset)

    def show(self, icon, title, sub, color):
        self.frame.config(bg=color)
        self.icon.config(text=icon, bg=color)
        self.title_lbl.config(text=title, bg=color)
        self.sub_lbl.config(text=sub, bg=color)

    def record(self, ms):
        self.count += 1
        self.total += ms
        if self.count == 1 or ms < self.best:
            self.best = ms

    def stats(self):
        return (f"Attempts: {self.count}   Avg: {self.total / self.count:.0f} ms   "
                f"Best: {self.best} ms\nPress R to reset")

    def cancel_timer(self):
        if self.after_id:
            self.after_cancel(self.after_id)
            self.after_id = None

    def reset(self, event=None):
        self.cancel_timer()
        self.count = self.total = self.best = 0
        self.state = "idle"
        self.show(CLOCK, "Reaction Time Test", INTRO, BLUE)

    def on_click(self, event=None):
        now = perf_counter()
        state = self.state
        if state == "ready":
            ms = int((now - self.start_time) * 1000)
            self.record(ms)
            self.state = "result"
            self.show(CLOCK, f"{ms} ms", "Click to keep going.\n" + self.stats(), BLUE)
        elif state == "waiting":
            self.cancel_timer()
            self.record(PENALTY)
            self.state = "toosoon"
            self.show(WARN, "Too soon!", f"+{PENALTY} ms penalty\nClick to try again.\n" + self.stats(), BLUE)
        else:
            self.start_wait()
        self.focus_set()

    def start_wait(self):
        self.state = "waiting"
        self.show(DOTS, "Wait for green", "", RED)
        self.after_id = self.after(randint(2000, 5000), self.go_green)

    def go_green(self):
        self.after_id = None
        self.state = "ready"
        self.frame.config(bg=GREEN)
        self.icon.config(bg=GREEN)
        self.title_lbl.config(text="Click!", bg=GREEN)
        self.sub_lbl.config(bg=GREEN)
        self.update_idletasks()
        self.start_time = perf_counter()


if __name__ == "__main__":
    ReactionTime().mainloop()
