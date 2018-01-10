#!/bin/sh
# build the program!
# note: there will eventually be a separate build step for your bot, but for now it counts against your runtime.

# we provide this env variable for you
if [ "$BC_PLATFORM" = 'LINUX' ]; then
    LIBRARIES="-lutil -ldl -lrt -lpthread -lgcc_s -lc -lm -L../battlecode/c/lib -lbattlecode-linux"
    INCLUDES="-I../battlecode/c/include -I."
elif [ "$BC_PLATFORM" = 'DARWIN' ]; then
    LIBRARIES="-lSystem -lresolv -lc -lm -L../battlecode/c/lib -lbattlecode-darwin"
    INCLUDES="-I../battlecode/c/include -I."
fi

echo "$ gcc extra.c -c -O -g $INCLUDES"
gcc extra.c -c -O -g $INCLUDES
echo "$ gcc main.c -c -O -g $INCLUDES"
gcc main.c -c -O -g $INCLUDES
echo "$ gcc main.o extra.o -o main $LIBRARIES"
gcc main.o extra.o -o main $LIBRARIES

# run the program!
./main
