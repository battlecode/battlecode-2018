import logging
import single_elimination
import double_elimination

if __name__ == '__main__':
    logging.getLogger().setLevel(logging.DEBUG)

    map_tag = input('Map tag in DB (i.e. sprint2018): ')
    table_name = input('Tournament table in DB (i.e. tournament_sprint): ')
    elim_style = input('Elimination style ("single" or "double"): ')

    if elim_style == 'single':
        single_elimination.run(map_tag, table_name)
    elif elim_style == 'double':
        double_elimination.run(map_tag, table_name)
    else:
        logging.error('No such eliminiation style: {}'.format(elim_style))

    logging.info('Tournament runner is exiting...')
