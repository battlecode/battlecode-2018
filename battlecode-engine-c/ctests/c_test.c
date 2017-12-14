#include "battlecode.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>

void check_error(bc_t *bc) {
    if (bc_has_error(bc)) {
        char* error = bc_extract_error(bc);
        printf("ERROR: %s\n", error);
        bc_free_error(bc, error);
        exit(1);
    }
}

int main() {
    printf("-- c test --\n"
           "loading battlecode...");
    bc_t * bc = bc_init();
    printf("loaded!\n");
    printf("-- error test --\n");
    bc_free_game_world(bc, NULL);
    if (!bc_has_error(bc)) {
        printf("no error??\n");
        exit(1);
    }
    char * error = bc_extract_error(bc);
    if (error == NULL) {
        printf("error is NULL?\n");
        exit(1);
    }
    printf("error extracted correctly.\n");
    printf("error text: \"%s\"\n", error);
    bc_free_error(bc, error);

    printf("-- world test --\n");
    printf("creating world...\n");
    bc_game_world_t *world = bc_new_game_world(bc);
    check_error(bc);

    printf("successful.\n");
    int32_t round = bc_get_round(bc, world);
    check_error(bc);

    printf("round: %d\n", round);

    printf("-- all checks passed --\n");

    printf("-- benchmarking (note: will be slow, debug mode) --\n");

    // debug mode on os x: 90ns
    // release mode on os x: 20ns
    // very reasonable.

    struct timespec start;
    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &start);

    for (int i = 0; i < 10000; i++) {
        bc_get_round(bc, world);
        check_error(bc);
    }

    struct timespec end;
    clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &end);

    double diff = (end.tv_sec - start.tv_sec) * 1000000000. + (end.tv_nsec - start.tv_nsec);

    printf("mean time / bc_get_round call: %lf ns\n", diff / 10000.);

    printf("-- finished benchmarks --\n");

    bc_free_game_world(bc, world);
    check_error(bc);
    bc_shutdown(bc);

    printf("-- done. --\n");
}