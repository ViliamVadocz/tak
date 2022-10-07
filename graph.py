import matplotlib.pyplot as plt
import numpy as np
import re

BACKGROUND = "#404040"
EVALUATION = "#fb8b24"
WIDTH_PER_PLY = 0.2

with open("analysis.ptn", "r", encoding="utf-8") as file:
    evals = np.array(
        [
            float(match)
            for match in re.findall("{evaluation: ([+-]\d.\d*)}", file.read())
        ]
    )
    plies = evals.size

# plotting
fig = plt.figure(figsize=(WIDTH_PER_PLY * plies, 5), tight_layout=True, dpi=200)

ax = plt.axes()
ax.set_facecolor(BACKGROUND)

less = evals < 0
black = less | np.roll(less, 1)
white = ~less | np.roll(~less, 1)
b_evals = evals.clip(max=0)
w_evals = evals.clip(min=0)

x = 1 + np.arange(plies) / 2

ax.plot(x, np.zeros(plies), color="gray")
ax.plot(x, evals, drawstyle="steps-post", color=EVALUATION)
ax.fill_between(x, b_evals, step="post", where=black, color="black")
ax.fill_between(x, w_evals, step="post", where=white, color="white")

ax.set_title("Evaluation Graph")
ax.set_xlabel("Move Number")
ax.set_ylabel("Evaluation")

ax.set_xbound(1, (plies + 1) / 2)
ax.set_ybound(-1, 1)
ax.set_xticks(x[::2])

plt.savefig("graph.png")
