import logging
import single_elimination
import double_elimination

from tournament_helper import *

if __name__ == '__main__':
    logging.getLogger().setLevel(logging.DEBUG)

    map_tag = input('Map tag in DB (i.e. seeding): ')
    table_name = input('Tournament table in DB (i.e. tournament_seeding): ')
    elim_style = input('Elimination style ("single" or "double"): ')

    if elim_style == ELIM_SINGLE:
        single_elimination.run(map_tag, table_name)
    elif elim_style == ELIM_DOUBLE:
        double_elimination.run(map_tag, table_name)
    else:
        raise Exception('No such elimination style: {}'.format(elim_style))

    logging.info('Tournament runner is exiting...')
