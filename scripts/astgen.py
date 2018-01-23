import argparse
import sys
import os.path
import random
import io
from typing import *
parent = os.path.abspath(__file__)
sys.path.append(os.path.abspath(os.path.join(parent, '..', '..', 'bc', 'python')))
try:
    import battlecode as bc
except:
    print('yo run make')
    sys.exit(1)

p = argparse.ArgumentParser(prog='astgen', usage='Generate asteroids for a map.')
p.add_argument('-i','--inplace', action='store_true', help='strip asteroids from map file and add them in place')
p.add_argument('-g','--generator', default='random', help='options: random|stride|pattern')
p.add_argument('-f','--frequency', default='random', help='options: int|random')
p.add_argument('-k','--karbonite', default='normal', help='options: int|random|normal')
p.add_argument('--stride-by', type=int, default=3)
p.add_argument('--karbonite-min', type=int, default=20)
p.add_argument('--karbonite-max', type=int, default=200)
p.add_argument('--karbonite-normal-sigma', type=int, default=40)
p.add_argument('-s','--seed', type=int, default=6147)
p.add_argument('-p','--pattern', default=None)
p.add_argument('MAP', help='path to map file')
a = p.parse_args()
try:
    a.frequency = int(a.frequency)
except: pass
try:
    a.karbonite = int(a.karbonite)
except: pass

random.seed(a.seed)

with open(a.MAP, 'r') as f:
    src = f.read()
m: bc.GameMap = bc.GameMap.parse_text_map(src)
mars: bc.PlanetMap = m.mars_map

passable = []
for x in range(mars.width):
    for y in range(mars.height):
        if mars.is_passable_terrain_at(bc.MapLocation(bc.Planet.Mars, x, y)):
            passable.append((x, y))

if a.pattern:
    passable_ = set(passable)
    pattern = eval(a.pattern)
    pattern = [l for l in pattern if l in passable_]

content = io.StringIO()
if a.inplace:
    for line in src.splitlines():
        if not line.strip().startswith('*'):
            content.write(line)
            content.write('\n')

content.write('# generated with: \n# {}\n'.format(' '.join(sys.argv)))

asteroids = []
turn = 1
i = 0
while True:
    if a.generator == 'random':
        location = random.choice(passable)
    elif a.generator == 'stride':
        location = passable[(i+1) * a.stride_by % len(passable)]
    elif a.generator == 'pattern':
        location = pattern[i % len(pattern)]
    else:
        print('unknown generator:', a.generator)
        sys.exit(1)

    if a.karbonite == 'random':
        karbonite = random.randint(a.karbonite_min, a.karbonite_max)
    elif a.karbonite == 'normal':
        karbonite = abs(int(random.normalvariate(0, a.karbonite_normal_sigma)))
        karbonite = max(min(karbonite, 180), 20)
    else:
        karbonite = a.karbonite
    
    content.write('* {} {} {} {}\n'.format(turn, location[0], location[1], karbonite))

    if a.frequency == 'random':
        turn += random.randint(10, 20)
    else:
        turn += a.frequency
    if turn >= 1000:
        break
    i += 1

if a.inplace:
    with open(a.MAP, 'w') as f:
        f.write(content.getvalue())
else:
    print(content.getvalue())