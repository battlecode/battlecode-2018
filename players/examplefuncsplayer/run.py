import battlecode as bc
import random

gc = bc.GameController()
directions = list(bc.Direction)

while True:
    print('round:', gc.round())
    try:
        for unit in gc.my_units():
            r = random.choice(directions)
            if gc.can_move(unit.id, r):
                gc.move_robot(unit.id, r)
    except Exception as e:
        print(e)
    gc.next_turn()

print("finished!")
