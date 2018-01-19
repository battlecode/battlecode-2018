"""
Battlecode 2018 Tournament Runner
"""

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


def get_queue_length(conn) -> int:
    """
    Returns the length of the match queue.
    """
    cur = conn.cursor()

    query = 'SELECT COUNT(*) FROM {} WHERE \
        status=\'queued\' or status=\'running\';'.format(TABLE_NAME)
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


def get_all_teams_and_index_submissions(conn) -> List[Team]:
    """
    Returns a list of teams that can be entered into the next round of
    matchmaking, along with their most recently submitted submission.
    The teams are ranked from best to least best.
    """
    cur = conn.cursor()

    query = 'SELECT teams1.id, subs.source_code                              \
        FROM battlecode_teams teams1 INNER JOIN scrimmage_submissions subs   \
        ON teams1.id = subs.team                                             \
        WHERE subs.uploaded_time = (                                         \
            SELECT MAX(uploaded_time)                                        \
            FROM battlecode_teams teams2 INNER JOIN scrimmage_submissions s  \
            ON teams2.id = s.team                                            \
            WHERE teams1.id = teams2.id                                      \
        ) ORDER BY teams1.mu DESC;'
    cur.execute(query);
    teams = cur.fetchall()
    cur.close()

    for team in teams:
        team_id = team[0]
        team_sub = team[1]
        team_submission[team_id] = team_sub
    team_submission[None] = None

    return list(map(lambda x: x[0], teams))


def get_next_team_and_from(conn, next_round_num, next_index, color):
    """
    Returns (team_id, match_id) of the winner at this round. The match_id
    can be any of the three matches in the previous game.
    """
    prev_round_num = next_round_num - 1
    if color == COLOR_RED:
        prev_index = next_index * 2
    if color == COLOR_BLUE:
        prev_index = next_index * 2 + 1

    cur = conn.cursor()
    cur.execute('SELECT status, id, red_team, blue_team FROM {} \
                 WHERE round=%s AND index=%s;'
                .format(TABLE_NAME),
                (prev_round_num, prev_index))

    matches = cur.fetchall()
    if len(matches) != 3:
        print("{} {} {}", next_round_num, next_index, color)
        print(matches)

    _, match_id, red_team, blue_team = matches[0]
    match_winners = {
        red_team: 0,
        blue_team: 0,
    }

    for status, _, _, _ in matches:
        if status == 'redwon':
            match_winners[red_team] += 1
        elif status == 'bluewon':
            match_winners[blue_team] += 1
        else:
            raise Exception("Match should be finished.")

    if match_winners[red_team] > match_winners[blue_team]:
        match_winner = red_team
    else:
        match_winner = blue_team

    return (match_winner, match_id)


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


def wait_for_empty_queue(conn) -> None:
    """
    Checks every 10 seconds until the queue is empty. This method is used to
    ensure that rankings are final after we recalculate them.
    """
    queue_length = get_queue_length(conn)
    while queue_length != 0:
        time.sleep(10)
        logging.info('Waiting for the queue to empty: {} left...'
            .format(queue_length))
        queue_length = get_queue_length(conn)
    logging.debug('Queue is empty.')


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


def queue_match(conn, round_num, index, red, blue, maps):
    """
    Red and blue are passed as (team_id, from)
    """
    red_team, red_from = red
    blue_team, blue_from = blue
    red_sub = team_submission[red_team]
    blue_sub = team_submission[blue_team]

    if red_team is not None and blue_team is not None:
        cur = conn.cursor()

        status = 'queued'
        query = 'INSERT INTO {} (red_team, blue_team, red_key, blue_key, \
            map, round, index, red_from, blue_from, status) \
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)'.format(TABLE_NAME)

        for game_map in maps:
            params = (red_team, blue_team, red_sub, blue_sub,
                game_map, round_num, index, red_from, blue_from, status)
            cur.execute(query, params)

        logging.debug('Round {}, index {}: queued {} vs {}'.format(
            round_num, index, red_team, blue_team))

        cur.close()
    else:
        cur = conn.cursor()

        winner_id = None
        status = 'cancelled'

        if red_team is not None:
            red_team, red_from = red
            red_sub = team_submission[red_team]
            winner_id = red_team
            status = 'redwon'

        if blue_team is not None:
            blue_team, blue_from = blue
            blue_sub = team_submission[blue_team]
            winner_id = blue_team
            status = 'bluewon'

        query = 'INSERT INTO {} (red_team, blue_team, red_key, blue_key, \
            map, round, index, red_from, blue_from, status) \
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s)'.format(TABLE_NAME)

        for game_map in maps:
            params = (red_team, blue_team, red_sub, blue_sub,
                game_map, round_num, index, red_from, blue_from, status)
            cur.execute(query, params)

        logging.debug('Round {}, index {}: BYE {} vs {}'.format(
            round_num, index, red_team, blue_team))

        cur.close()


def queue_round(conn, round_num, maps):
    assert len(maps) == NUM_MAPS_PER_GAME
    cur = conn.cursor()
    cur.execute('SELECT COUNT(*) FROM {} WHERE round=%s;'.format(TABLE_NAME),
        (round_num - 1,))
    max_index = int(cur.fetchone()[0] / 3 / 2)

    for index in range(max_index):
        red = get_next_team_and_from(conn, round_num, index, COLOR_RED)
        blue = get_next_team_and_from(conn, round_num, index, COLOR_BLUE)
        queue_match(conn, round_num, index, red, blue, maps)
    conn.commit()
    cur.close()


def queue_initial_round(conn, teams, maps):
    assert len(maps) == NUM_MAPS_PER_GAME
    round_num = 0
    cur = conn.cursor()
    max_index = int(len(teams) / 2)

    logging.debug('Queuing Round 0...')
    for index in range(max_index):
        red = (teams[2 * index], None)
        blue = (teams[2 * index + 1], None)
        queue_match(conn, round_num, index, red, blue, maps)
    conn.commit()
    cur.close()


def run_tournament(conn, maps: List[Map], teams: List[Team]):
    """
    The tournament is run in single elimination format. Maps are used in sets
    of 3, where the 3rd map of one round overlaps with the 1st map of the next
    round. So if the set of maps is [1, 2, 3, 4, 5], Round 1 will use maps
    [1, 2, 3] and Round 2 will use maps [3, 4, 5].

    maps  - A list of maps for the tournament, run in the order they are given.
    teams - A power of two number of teams, where some of the teams may be BYEs.
    """
    logging.debug('Generating a bracket for {} teams and {} maps'
        .format(len(teams), len(maps)))

    teams = generate_bracket(teams)
    num_rounds = int(math.log(len(teams), 2))

    logging.debug('We want {} maps, and we have {}'
        .format(num_rounds * 2 + 1, len(maps)))
    assert len(maps) >= num_rounds * 2 + 1

    initial_round = int(input('Start at which round? (0 to {}):\n'
        .format(num_rounds - 1)))

    logging.debug('Running rounds {} to {}... here we go!'
        .format(initial_round, num_rounds - 1))
    for round_num in range(initial_round, num_rounds):
        if round_num == 0:
            queue_initial_round(conn, teams, maps[:NUM_MAPS_PER_GAME])
            continue

        wait_for_empty_queue(conn)
        logging.debug('Queuing Round {} out of {}...'
            .format(round_num, num_rounds - 1))
        round_maps = maps[(NUM_MAPS_PER_GAME - 1) * round_num : 
                          (NUM_MAPS_PER_GAME - 1) * (round_num + 1) + 1]
        logging.debug('Using these maps: {}'.format(round_maps))
        queue_round(conn, round_num, round_maps)


def run(conn) -> None:
    """
    No more ranked matches should be run at this point, nor should ratings be
    updated, nor new bots submitted.
    """
    maps = get_tournament_maps(conn, tag=MAP_TAG)
    teams = get_all_teams_and_index_submissions(conn)

    num_teams = len(teams)
    if num_teams < 2:
        logging.warn('Only {} teams in the tournament, exiting...'
            .format(num_teams))
        return

    teams = pad_teams_power_of_two(teams)
    logging.info('Fetched {0} teams and {1} maps'
        .format(num_teams, len(maps)))

    run_tournament(conn, maps, teams)


if __name__ == '__main__':
    logging.getLogger().setLevel(logging.DEBUG)

    MAP_TAG = input('Map tag in DB (i.e. sprint2018):\n')
    TABLE_NAME = input('Tournament table in DB (i.e. tournament_sprint):\n')

    conn = db_connect()
    run(conn)
    logging.info('Tournament runner is exiting...')
