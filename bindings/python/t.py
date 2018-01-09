import battlecode as bc
import time
t = time.time()
for _ in range(100000):
    bc.Direction.North.opposite()
print((time.time() - t) / 100000)
