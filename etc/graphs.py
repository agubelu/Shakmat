from json import loads
import matplotlib.pyplot as plt

score_data = loads(open("out/scores.json", "r").read())
versions = list(score_data) 
labels = versions + ["Draws"]

v1, v2 = versions
blacks = [score_data[v1]["black"]["win"], score_data[v2]["black"]["win"], score_data[v1]["black"]["draw"]]
whites = [score_data[v1]["white"]["win"], score_data[v2]["white"]["win"], score_data[v1]["white"]["draw"]]

totals = [score_data[v1]["black"]["win"] + score_data[v1]["white"]["win"],
          score_data[v2]["black"]["win"] + score_data[v2]["white"]["win"],
          score_data[v1]["black"]["draw"] + score_data[v1]["white"]["draw"]
]

WIDTH = 0.4

fig, ax = plt.subplots()
ax.bar(labels, blacks, width=WIDTH, label='Black', color="black", edgecolor="black")
ax.bar(labels, whites, width=WIDTH, bottom=blacks, label='White', color="white", edgecolor="black")

for index,data in enumerate(totals):
    plt.text(x=index , y =data+1 , s=f"{data}" , fontdict=dict(fontsize=20), color="red")

ax.set_ylabel('Games')
ax.legend()
plt.show()

'''
time_data = loads(open("out/times.json", "r").read())
max_time = max(max(int(i) for i in time_data[v1]), max(int(i) for i in time_data[v2]))

def get_time(ver, epoch):
    epoch = str(epoch)
    if epoch not in time_data[ver]:
        return None
    return sum(time_data[ver][epoch]) / len(time_data[ver][epoch])

x = [i for i in range(1, max_time + 1)]
times_v1 = [get_time(v1, i) for i in x]
times_v2 = [get_time(v2, i) for i in x]

fig, ax = plt.subplots()
line1, = ax.plot(x, times_v1, label=v1)
line2, = ax.plot(x, times_v2, label=v2)
ax.legend()
plt.show()
'''