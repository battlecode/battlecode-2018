import math
import os
import time
import logging
import psycopg2

from typing import List, NewType

Map   = NewType('Map', str)
Team  = NewType('Team', int)

NUM_MAPS_PER_GAME = 3

COLOR_RED = 'RED'
COLOR_BLUE = 'BLUE'

ELIM_SINGLE = 'single'
ELIM_DOUBLE = 'double'

team_submission = {}


def db_connect(host=os.environ.get('DB_HOST'),
               dbname='battlecode',
               user='battlecode',
               password=os.environ.get('DB_PASS')):
    """
    Connect and return a connection to the database. The database credentials
    are read from environment variables, if not passed in the function call.
    """

    conn = None
    while conn is None:
        try:
            conn = psycopg2.connect(dbname=dbname,
                                    user=user,
                                    password=password,
                                    host=host)
        except Exception as e:
            logging.error('Error connecting to DB, retrying in 10 seconds...')
            logging.exception(e)
            time.sleep(10)

    logging.info('Connected to DB {0} on {1}'.format(dbname, host))
    return conn


def get_queue_length(conn, table) -> int:
    """
    Returns the length of the match queue.
    """
    cur = conn.cursor()

    query = 'SELECT COUNT(*) FROM {} WHERE \
        status=\'queued\' or status=\'running\';'.format(table)
    cur.execute(query)
    queue_length = cur.fetchone()[0]
    cur.close()

    return queue_length


def get_tournament_maps(conn, tag=None) -> List[Map]:
    """
    Returns a list of potential map names with the given tag.
    """
    cur = conn.cursor()

    if tag is not None:
        filter_query = 'WHERE tag=\'{}\''.format(tag)
    else:
        filter_query = ''

    query = 'SELECT name FROM scrimmage_maps {} ORDER BY id;'.format(filter_query)
    cur.execute(query)
    maps = cur.fetchall()
    cur.close()

    return list(map(lambda x: x[0], maps))


FILTER_CLAUSE = {
    '0': '',
    '1': 'AND qualifying != \'no\'',
    '2': 'AND qualifying = \'us\'',
    '3': 'AND qualifying = \'international\'',
    '4': 'AND newbie = \'yes\'',
    '5': 'AND high_school = \'yes\'',
}


def get_all_teams_and_index_submissions(conn) -> List[Team]:
    """
    Returns a list of teams that can be entered into the next round of
    matchmaking, along with their most recently submitted submission.
    The teams are ranked from best to least best.
    """
    cur = conn.cursor()

    print('(0) Sprint, (1) Seeding, (2) US Qualifying, (3) Intl Qualifying, (4) Newbie, (5) HS')
    clause = FILTER_CLAUSE[input('Which tournament eligibility? ')]

    query = 'SELECT teams1.id, subs.source_code                              \
        FROM battlecode_teams teams1 INNER JOIN scrimmage_submissions subs   \
        ON teams1.id = subs.team                                             \
        WHERE subs.uploaded_time = (                                         \
            SELECT MAX(uploaded_time)                                        \
            FROM battlecode_teams teams2 INNER JOIN scrimmage_submissions s  \
            ON teams2.id = s.team                                            \
            WHERE teams1.id = teams2.id                                      \
        ) {} ORDER BY teams1.mu DESC;'.format(clause)
    cur.execute(query);
    teams = cur.fetchall()
    cur.close()

    for team in teams:
        team_id = team[0]
        team_sub = team[1]
        team_submission[team_id] = team_sub
    team_submission[None] = None

    return list(map(lambda x: x[0], teams))


def wait_for_empty_queue(conn, table) -> None:
    """
    Checks every queue_length / 10 seconds until the queue is empty.
    """
    while True:
        queue_length = get_queue_length(conn, table)
        if queue_length == 0:
            break
        logging.info('Waiting for the queue to empty: {} left...'
            .format(queue_length))
        time.sleep(queue_length / 10)
    logging.debug('Queue is empty.')


def generate_bracket(teams: List[Team]) -> List[Team]:
    n = len(teams)

    # get the bracket by index
    full_bracket = bracket(n)

    # put the teams in the correct position in the bracket
    sortedTeams = []
    for i in range(n):
        sortedTeams.append(teams[full_bracket[i] - 1])
    logging.debug('The bracket for single elimination is...')
    logging.debug(list(sortedTeams))
    return sortedTeams


def bracket(n: int):
    if (n == 1):
        return [1]
    half_bracket = bracket(int(n / 2))
    full_bracket = n * [0]
    for i in range(int(n / 2)):
        full_bracket[2 * i] = half_bracket[i]
        full_bracket[2 * i + 1] = n + 1 - full_bracket[2 * i]
    return full_bracket


def pad_teams_power_of_two(teams: List[Team]) -> List[Team]:
    """
    Pads the teams list to a power of two by adding BYEs to the end.
    """
    assert len(teams) > 0
    num_teams_orig = len(teams)
    num_teams_goal = 2**math.ceil(math.log(num_teams_orig, 2))
    for ranking in range(len(teams) + 1, num_teams_goal + 1):
        teams.append(None)

    logging.debug('Padded {} teams to {} teams (goal {})'.format(
        num_teams_orig, len(teams), num_teams_goal))

    assert len(teams) & (len(teams) - 1) == 0
    return teams

