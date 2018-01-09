#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <assert.h>

#include <bc.h>

/// See bc.h at xxx for the API you have access too.
/// Note: the API is not thread safe; don't use pthreads.
/// It's not very pretty, sorry. If you want a prettier low-level language, maybe consider rust?

// Any method in the API may set an error.
// Call check_errors() to get the most recent error.
bool check_errors() {
    /// Check if we have an error...
    if (bc_has_err()) {
        char *err;
        /// Note: this clears the current global error.
        int8_t code = bc_get_last_err(&err);
        printf("Engine error code %d: %s\n", code, err);
        bc_free_string(err);
        return true;
    } else {
        return false;
    }
}

int main() {
    printf("Player C bot starting\n");

    // It's good to try and make matches deterministic. It's not required, but it
    // makes debugging wayyy easier.
    // Now if you use random() it will produce the same output each map.
    srand(0);

    // we provide some helpful helpers :)
    bc_Direction dir = North;
    bc_Direction opposite = bc_Direction_opposite(dir);
    // you should basically call this after every function call.
    check_errors();

    printf("Opposite direction of %d: %d\n", dir, opposite);

    // Make sure that the world is sane!
    assert(opposite == South);

    printf("Connecting to manager...\n");

    // Most methods return pointers; methods returning integers or enums are the only exception.
    bc_GameController *gc = new_bc_GameController();

    if (check_errors()) {
        // If there was an error creating gc, just die.
        printf("Failed, dying.\n");
        exit(1);
    }
    printf("Connected!\n");

    // loop through the whole game.
    while (true) {
        // The API is "object-oriented" - most methods take pointers to some object.
        uint32_t round = bc_GameController_round(gc);
        printf("Round: %d\n", round);

        // Note that all operations perform copies out of their data structures, returning new objects.
        // You're responsible for freeing objects.
        bc_VecUnit *units = bc_GameController_my_units(gc);

        // it's good to cache things like this. Calls into the API have around a 20ns overhead, plus the cost of
        // copying the data out of the engine. not horrible, but good to avoid more than necessary.
        int len = bc_VecUnit_len(units);
        for (int i = 0; i < len; i++) {
            // Get the current unit. This also copies.
            bc_Unit *unit = bc_VecUnit_index(units, i);

            // Calls on the controller take unit IDs for ownership reasons.
            uint16_t id = bc_Unit_id(unit);
            if (bc_GameController_can_move(gc, id, North) && bc_GameController_is_move_ready(gc, id)) {
                bc_GameController_move_robot(gc, id, North);
                check_errors();
            }

            // don't want memory leaks!
            delete_bc_Unit(unit);
        }
        delete_bc_VecUnit(units);

        // this line helps the output logs make more sense by forcing output to be sent
        // to the manager.
        // it's not strictly necessary, but it helps.
        fflush(stdout);

        // pause and wait for the next turn.
        bc_GameController_next_turn(gc);
    }
    // Convinced you shouldn't use C yet?
}