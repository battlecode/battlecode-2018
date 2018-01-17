"""
Battlecode 2018 Matchmaker

PURPOSE:
The matchmaker is an automated system that matches similarly-ranked teams
against each other in ranked scrimmages. The matchmaker uses the team's rating
and most current submission at the time the match is enqueued. A match consists
of NUM_MAPS_PER_MATCH rounds on different maps.

To generate a match for enqueueing,
1. Randomly choose one team to be the Red Team.
2. Randomly choose up to MAX_SAMPLE_SIZE teams, and pick the closest ranked
   team to the Red Team to be the Blue Team. Ties in rank are broken by the
   team that was sampled first.
3. Randomly choose NUM_MAPS_PER_MATCH distinct maps.

Once the matchmaker has started, it checks the queue every MINUTES_PER_INTERVAL
minutes. It aims to enqueue approximately one match per team every
MINUTES_PER_MATCH_PER_TEAM minutes. This is only an average expected outcome
due to randomization. Finally, the matchmaker will not enqueue more matches if
the queue exceeds the MAX_QUEUE_LENGTH.
"""

import os
import random
import sys
import time
import logging
import psycopg2

from typing import Tuple, List, NewType

Map   = NewType('Map', int)
Team  = NewType('Team', Tuple[int, int, int])
Match = NewType('Match', Tuple[bool, List[int], int, int, int, int])

MINUTES_PER_MATCH_PER_TEAM = int(os.environ.get('MINUTES_PER_MATCH_PER_TEAM'))
MINUTES_PER_INTERVAL = int(os.environ.get('MINUTES_PER_INTERVAL'))
MAX_SAMPLE_SIZE = int(os.environ.get('MAX_SAMPLE_SIZE'))
NUM_MAPS_PER_MATCH = int(os.environ.get('NUM_MAPS_PER_MATCH'))
MAX_QUEUE_LENGTH = int(os.environ.get('MAX_QUEUE_LENGTH'))

DEFAULT_RATING = 1200

def db_connect(host=os.environ.get('POSTGRES_HOST'),
               port=os.environ.get('POSTGRES_PORT'),
               dbname=os.environ.get('POSTGRES_DATABASE'),
               user=os.environ.get('POSTGRES_USERNAME'),
               password=os.environ.get('POSTGRES_PASSWORD')):
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
                                    host=host,
                                    port=port)
        except Exception as e:
            logging.error('Error connecting to DB, retrying in 10 seconds...')
            logging.exception(e)
            time.sleep(10)

    logging.info('Connected to DB {0} on {1}:{2}'.format(dbname, host, port))
    return conn


def get_teams(conn, ranking=True) -> List[Team]:
    """
    Returns a list of teams that can be entered into the next round of
    matchmaking, along with their most recently submitted submission
    and, optionally, the ranking.
    """
    cur = conn.cursor()

    query = 'SELECT teams1.id, subs.id AS submission_id, ranking             \
        FROM battlecode_teams teams1 INNER JOIN scrimmage_submissions subs   \
        ON teams1.id = subs.team                                             \
        WHERE subs.uploaded_time = (                                         \
            SELECT MAX(uploaded_time)                                        \
            FROM battlecode_teams teams2 INNER JOIN scrimmage_submissions s  \
            ON teams2.id = s.team                                            \
            WHERE teams1.id = teams2.id                                      \
        ) ORDER BY ranking;'
    cur.execute(query);
    teams = cur.fetchall()
    cur.close()

    if ranking:
        teams = list(filter(lambda x: x[2] != 0, teams))
    else:
        teams = list(map(lambda x: (x[0], x[1]), teams))

    return teams


def get_maps(conn, tag=None) -> List[Map]:
    """
    Returns a list of potential map IDs with the given tag.
    """
    cur = conn.cursor()

    if tag is not None:
        filter_query = 'WHERE tag=\'{}\''.format(tag)
    else:
        filter_query = ''

    query = 'SELECT id FROM scrimmage_maps {} ORDER BY id;'.format(filter_query)
    cur.execute(query)
    maps = cur.fetchall()
    cur.close()

    return list(map(lambda x: x[0], maps))


def get_queue_length(conn) -> int:
    """
    Returns the length of the match queue.
    """
    cur = conn.cursor()

    query = 'SELECT COUNT(*) FROM scrimmage_matches WHERE \
        status=\'queued\' or status=\'running\';'
    cur.execute(query)
    queue_length = cur.fetchone()[0]
    cur.close()

    return queue_length


def update_rankings(conn) -> None:
    """
    Reads the most recent scrimmage match data and updates the rankings in the
    teams table. Teams without any prior rating will default to a constant.
    """
    cur = conn.cursor()
    cur.execute('SELECT red_team, red_rating_after, \
                        blue_team, blue_rating_after \
                 FROM scrimmage_matches WHERE ranked=TRUE AND finish_time!=NULL\
                 ORDER BY finish_time, id ASC;')
    pairs = cur.fetchall()

    team_to_rating = {}
    teams = get_teams(conn, ranking=False)
    for team_id, _ in teams:
        team_to_rating[team_id] = DEFAULT_RATING
    for red_id, red_rating, blue_id, blue_rating in pairs:
        team_to_rating[red_id] = red_rating
        team_to_rating[blue_id] = blue_rating

    sorted_team_ratings = sorted(team_to_rating.items(), key=lambda x: -x[1])
    sorted_team_rankings = [
        (sorted_team_ratings[i][0], i + 1) for i in range(len(sorted_team_ratings))
    ]

    for team, ranking in sorted_team_rankings:
        cur.execute('UPDATE battlecode_teams SET ranking={0} WHERE id={1}'
            .format(ranking, team))
    conn.commit()
    cur.close()


def queue_matches(conn, matches: List[Match]) -> None:
    """
    Queues the list of matches in the database.
    """
    logging.info('Queueing {0} matches...'.format(len(matches)))
    if len(matches) == 0:
        return

    for match in matches:
        logging.debug(match)

    cur = conn.cursor()
    cur.execute('INSERT INTO scrimmage_matches(matchmaker, ranked, maps, ' +
        'red_team, red_submission, blue_team, blue_submission) '+
        'VALUES {0};'.format(', '.join(matches)))
    conn.commit()
    cur.close()


def fmt_match(match: Match) -> str:
    """
    Formats a match for insertion via SQL query.
    """
    maps = 'ARRAY{0}'.format(str(match[1]))
    fmt = '(True, {0}, {1}, {2}, {3}, {4}, {5})'.format(
        match[0], maps, match[2], match[3], match[4], match[5]
    )
    return fmt


def get_num_matches(teams: List[Team]) -> int:
    """
    Returns the number of matches to run using the queue length and number of
    eligible teams.
    """
    queue_length = get_queue_length(conn)
    num_matches = int(len(teams) * MINUTES_PER_INTERVAL / MINUTES_PER_MATCH_PER_TEAM / 2)
    num_matches = max(1, num_matches) if len(teams) >= 2 else 0
    num_matches = min(num_matches, MAX_QUEUE_LENGTH - queue_length)

    num_extra_matches = num_matches - (MAX_QUEUE_LENGTH - queue_length)
    if num_extra_matches > 0:
        logging.warn('We want to enqueue {0} more matches, but the queue is ' +
            'already full. Should we increase the compute power of the ' +
            'match manager?'.format(num_extra_matches))

    return num_matches


def make_matches(num_matches: int, teams: List[Team], maps: List[Map], ranked: bool=True):
    """
    Generates the next batch of matches to run from the list of teams and maps.
    """
    if len(maps) < NUM_MAPS_PER_MATCH:
        return []
    if len(teams) < 2:
        return []

    matches = []
    sample_size = min(MAX_SAMPLE_SIZE + 1, len(teams))

    random.seed(time.time())

    for _ in range(num_matches):
        match_maps = random.sample(maps, NUM_MAPS_PER_MATCH)
        enemies = random.sample(teams, sample_size)

        red = enemies.pop(0)
        blue = min(enemies, key=lambda blue: abs(blue[2] - red[2]))
        matches.append(
            fmt_match((ranked, match_maps, red[0], red[1], blue[0], blue[1]))
        )

    return matches


def run(conn) -> None:
    """
    Wake up every MINUTES_PER_INTERVAL minutes to check the queue and run tasks.
    """
    while True:
        try:
            update_rankings(conn)

            teams = get_teams(conn, ranking=True)
            maps = get_maps(conn)

            logging.info('Fetched {0} teams and {1} maps'
                .format(len(teams), len(maps)))
            num_matches = get_num_matches(teams)
            matches = make_matches(num_matches, teams, maps)

            queue_matches(conn, matches)
        except Exception as e:
            logging.exception(e)

        time.sleep(60 * MINUTES_PER_INTERVAL)


if __name__ == '__main__':
    logging.getLogger().setLevel(logging.DEBUG)
    enable_mm = os.environ.get('ENABLE_MATCHMAKER') == 'True'
    if enable_mm:
        conn = db_connect()
        run(conn)
    logging.info('Matchmaker is exiting...')
