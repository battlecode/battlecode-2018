"""
Battlehack Tournament Runner
TODO: fix this and make it work with Battlecode 2017...
There was definitely some bad stuff in here...
"""

import math
import os
import random
import sys
import time
import logging
import psycopg2
import matchmaker as mm

from typing import Tuple, List, NewType

MAP_TAG = 'tournament_battlehack'
TABLE_NAME = 'tournament_battlehack'

COLOR_RED = 'RED'
COLOR_BLUE = 'BLUE'

team_submission = {}


def get_queue_length(conn) -> int:
    """
    Returns the length of the match queue.
    """
    logging.info('Waiting for the queue to empty...')

    cur = conn.cursor()

    query = 'SELECT COUNT(*) FROM {} WHERE \
        status=\'queued\' or status=\'running\';'.format(TABLE_NAME)
    cur.execute(query)
    queue_length = cur.fetchone()[0]
    cur.close()

    return queue_length


def get_team_match(conn, round_num, index, color):
    """
    Returns (team_id, match_id).
    """
    next_round_num = round_num - 1
    if color == COLOR_RED:
        next_index = index * 2
    if color == COLOR_BLUE:
        next_index = index * 2 + 1

    cur = conn.cursor()
    cur.execute('SELECT winner_id, id FROM {} \
                 WHERE round={} AND index={};'
                .format(TABLE_NAME, next_round_num, next_index))

    team_match = cur.fetchone()
    return team_match


def set_team_submissions(teams):
    for team in teams:
        team_id = team[0]
        team_sub = team[1]
        team_submission[team_id] = team_sub


def generate_bracket(teams: List[mm.Team]) -> List[mm.Team]:
    n = len(teams)
    full_bracket = bracket(n)
    sortedTeams = []

    logging.debug('The bracket was and then will be')
    logging.debug(list(map(lambda x: x[0], teams)))
    for i in range(n):
        sortedTeams.append(teams[full_bracket[i] - 1])
    logging.debug(list(map(lambda x: x[0], sortedTeams)))
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
        queue_length = get_queue_length(conn)
    logging.debug('Queue is empty.')


def pad_teams_power_of_two(teams: List[mm.Team]) -> List[mm.Team]:
    """
    Pads the teams list to a power of two by adding BYEs to the end.
    """
    assert len(teams) > 0
    num_teams_orig = len(teams)
    num_teams_goal = 2**math.ceil(math.log(num_teams_orig, 2))
    for ranking in range(len(teams) + 1, num_teams_goal + 1):
        teams.append((None, 'NULL', ranking))

    logging.debug('Padded {} teams to {} teams (goal {})'.format(
        num_teams_orig, len(teams), num_teams_goal))

    assert len(teams) & (len(teams) - 1) == 0
    return teams


def queue_match(conn, round_num, index, red, blue, maps):
    """
    Red and blue are passed as (team_id, from)
    """
    fmt_maps = 'ARRAY{0}'.format(str(maps))

    if red[0] is not None and blue[0] is not None:
        cur = conn.cursor()

        status = '\'queued\''

        red_team, red_from = red
        blue_team, blue_from = blue
        red_sub = team_submission[red_team]
        blue_sub = team_submission[blue_team]

        query = 'INSERT INTO {0}(maps, status, red_team, red_submission, \
            blue_team, blue_submission, round, index, red_from, blue_from) \
            VALUES ({1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}, {10});'.format(
                TABLE_NAME, fmt_maps, status, red_team, red_sub,
                blue_team, blue_sub, round_num, index, red_from, blue_from
            )

        logging.debug('queue_match(): {}'.format(query))

        cur.execute(query)
        cur.close()
    else:
        cur = conn.cursor()

        winner_id = 'NULL'
        status = '\'completed\''
        red_team, red_from, red_sub = ('NULL', 'NULL', 'NULL')
        blue_team, blue_from, blue_sub = ('NULL', 'NULL', 'NULL')

        if red[0] is not None:
            red_team, red_from = red
            red_sub = team_submission[red_team]
            winner_id = red_team

        if blue[0] is not None:
            blue_team, blue_from = blue
            blue_sub = team_submission[blue_team]
            winner_id = blue_team

        query = 'INSERT INTO {0}(maps, status, red_team, red_submission, \
            blue_team, blue_submission, round, index, red_from, blue_from, \
            winner_id, finish_time) \
            VALUES ({1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}, {10}, {11}, {12});'.format(
                TABLE_NAME, fmt_maps, status, red_team, red_sub,
                blue_team, blue_sub, round_num, index, red_from, blue_from,
                winner_id, 'CURRENT_TIMESTAMP'
            )

        logging.debug('queue_match() BYE: {}'.format(query))

        cur.execute(query)
        cur.close()


def queue_round(conn, round_num, maps):
    assert len(maps) == mm.NUM_MAPS_PER_MATCH
    cur = conn.cursor()
    cur.execute('SELECT COUNT(*) FROM {} WHERE round={};'
        .format(TABLE_NAME, round_num - 1))
    max_index = int(cur.fetchone()[0] / 2)

    logging.debug('Round {} has max index {}'.format(round_num, max_index))

    for index in range(max_index):
        red = get_team_match(conn, round_num, index, COLOR_RED)
        blue = get_team_match(conn, round_num, index, COLOR_BLUE)
        queue_match(conn, round_num, index, red, blue, maps)
    conn.commit()
    cur.close()


def queue_initial_round(conn, teams, maps):
    assert len(maps) == mm.NUM_MAPS_PER_MATCH
    round_num = 0
    cur = conn.cursor()
    max_index = int(len(teams) / 2)

    logging.debug('Initial round has max index {}'.format(max_index))

    for index in range(max_index):
        red = (teams[2 * index][0], 'NULL')
        blue = (teams[2 * index + 1][0], 'NULL')
        queue_match(conn, round_num, index, red, blue, maps)
    conn.commit()
    cur.close()


def run_tournament(conn, maps: List[mm.Map], teams: List[mm.Team]):
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
    assert len(maps) > num_rounds * 2 + 1

    logging.debug('Queuing Round 0...')

    queue_initial_round(conn, teams, maps[:mm.NUM_MAPS_PER_MATCH])
    for round_num in range(1, num_rounds):
        wait_for_empty_queue(conn)
        logging.debug('Queuing Round {} out of {}...'
            .format(round_num, num_rounds - 1))
        round_maps = maps[mm.NUM_MAPS_PER_MATCH * round_num : 
                          mm.NUM_MAPS_PER_MATCH * (round_num + 1)]
        queue_round(conn, round_num, round_maps)


def run(conn) -> None:
    """
    Wait for the queue to empty and update the rankings. No more ranked
    matches should be run at this point, nor should ratings be updated.
    """
    wait_for_empty_queue(conn)
    mm.update_rankings(conn)

    maps = mm.get_maps(conn, tag=MAP_TAG)
    teams = mm.get_teams(conn, ranking=True)
    num_teams = len(teams)
    if num_teams < 2:
        logging.warn('Only {} teams in the tournament, exiting...'
            .format(num_teams))
        return

    teams = pad_teams_power_of_two(teams)

    if len(teams) == 0:
        return

    set_team_submissions(teams)
    logging.info('Fetched {0} teams and {1} maps'
        .format(num_teams, len(maps)))

    run_tournament(conn, maps, teams)


if __name__ == '__main__':
    logging.getLogger().setLevel(logging.DEBUG)
    enable_mm = os.environ.get('ENABLE_MATCHMAKER') == 'True'
    if enable_mm:
        conn = mm.db_connect()
        run(conn)
    logging.info('Tournament runner is exiting...')
