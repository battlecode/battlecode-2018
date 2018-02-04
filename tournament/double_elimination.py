"""
Battlecode 2018 Double Elimination Tournament Runner
"""

import random

from tournament_helper import *


def get_prev_round(round_num, subround):
    """
    Prev (round, subround) to go.
    """
    if round_num == 1 and subround == 'A':
        return (0, 'A')
    if subround == 'A':
        return (round_num - 1, 'C')
    if subround == 'B':
        return (round_num, 'A')
    if subround == 'C':
        return (round_num, 'B')
    return (None, None)


def round_num_split(round_num_abs):
    """
    Returns (round_num, subround).
    """
    if round_num_abs == 0:
        return (int(0), 'A')

    round_num = int((round_num_abs + 2) / 3)
    if round_num_abs % 3 == 1:
        subround = 'A'
    if round_num_abs % 3 == 2:
        subround = 'B'
    if round_num_abs % 3 == 0:
        subround = 'C'

    assert subround is not None
    return (round_num, subround)


def match_result(conn, round_num, subround, index):
    """
    Gets the (match_id, winner_id, loser_id) of this match, if it has finished.
    Otherwise returns (None, None, None).
    """
    cur = conn.cursor()
    cur.execute('SELECT status, id, red_team, blue_team FROM {} \
                 WHERE round=%s AND subround=%s AND index=%s;'
                .format(TABLE_NAME),
                (round_num, subround, index))

    matches = cur.fetchall()
    assert len(matches) == 3

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
        return (match_id, red_team, blue_team)
    else:
        return (match_id, blue_team, red_team)
    return (None, None, None)


def get_next_team_and_from(conn, round_num, subround, index, color):
    """
    Returns (team_id, match_id) of the winner at this round. The match_id
    can be any of the three matches in the previous game.

    round n subround A index i color red: round n-1 subround A index 2*i winner
    round n subround A index i color blue: round n-1 subround A index 2*i+1 winner
    round n subround B index i color red: round n-1 subround C index 2*i winner
    round n subround B index i color blue: round n-1 subround C index 2*i+1 winner
    round n subround C index i color red: round n subround A index i loser
    round n subround C index i color blue: round n subround B index i winner
    """
    if (round_num, subround) == (0, 'A'):
        raise Exception("Round 0A is not allowed in this method")
    if (round_num, subround) == (1, 'B'):
        raise Exception("Round 1B is not allowed in this method")

    if subround == 'A':
        if color == COLOR_RED:
            match_id, team_id, _ = match_result(conn, round_num - 1, 'A', index * 2)
        if color == COLOR_BLUE:
            match_id, team_id, _ = match_result(conn, round_num - 1, 'A', index * 2 + 1)
    if subround == 'B':
        if color == COLOR_RED:
            match_id, team_id, _ = match_result(conn, round_num - 1, 'C', index * 2)
        if color == COLOR_BLUE:
            match_id, team_id, _ = match_result(conn, round_num - 1, 'C', index * 2 + 1)
    if subround == 'C':
        if color == COLOR_RED:
            match_id, _, team_id = match_result(conn, round_num, 'A', index)
        if color == COLOR_BLUE:
            match_id, team_id, _ = match_result(conn, round_num, 'B', index)

    return (team_id, match_id)


def queue_match(conn, round_num, subround, index, red, blue, maps):
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
            map, round, subround, index, red_from, blue_from, status) \
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)'.format(TABLE_NAME)

        for game_map in maps:
            params = (red_team, blue_team, red_sub, blue_sub, game_map,
                round_num, subround, index, red_from, blue_from, status)
            cur.execute(query, params)

        logging.debug('Round {}{}, index {}: {} vs {}'.format(
            round_num, subround, index, red_team, blue_team))

        cur.close()
    else:
        cur = conn.cursor()

        winner_id = None
        status = 'redwon'

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
            map, round, subround, index, red_from, blue_from, status) \
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)'.format(TABLE_NAME)

        for game_map in maps:
            params = (red_team, blue_team, red_sub, blue_sub, game_map,
                round_num, subround, index, red_from, blue_from, status)
            cur.execute(query, params)

        logging.debug('Round {}{}, index {}: {} vs {} (BYE)'.format(
            round_num, subround, index, red_team, blue_team))

        cur.close()


def queue_round(conn, round_num, subround, maps):
    assert len(maps) == NUM_MAPS_PER_GAME
    cur = conn.cursor()

    if subround == 'A':
        cur.execute('SELECT COUNT(*) FROM {} WHERE round=%s AND subround=%s;'
            .format(TABLE_NAME), (round_num - 1, 'A'))
        max_index = int(cur.fetchone()[0] / 3 / 2)
    if subround == 'B':
        cur.execute('SELECT COUNT(*) FROM {} WHERE round=%s AND subround=%s;'
            .format(TABLE_NAME), (round_num - 1, 'C'))
        max_index = int(cur.fetchone()[0] / 3 / 2)
    if subround == 'C':
        cur.execute('SELECT COUNT(*) FROM {} WHERE round=%s AND subround=%s;'
            .format(TABLE_NAME), (round_num, 'B'))
        max_index = int(cur.fetchone()[0] / 3)

    permutation = list(range(max_index))
    if subround == 'C' and round_num % 2 == 1:
        permutation.reverse()

    for index in range(max_index):
        red = get_next_team_and_from(conn, round_num, subround, permutation[index], COLOR_RED)
        blue = get_next_team_and_from(conn, round_num, subround, index, COLOR_BLUE)
        queue_match(conn, round_num, subround, index, red, blue, maps)

    # last round
    if max_index == 0:
        match_id, team_id, _ = match_result(conn, round_num - 1, 'A', 0)
        red = (team_id, match_id)
        match_id, team_id, _ = match_result(conn, round_num - 1, 'C', 0)
        blue = (team_id, match_id)
        queue_match(conn, round_num, subround, 0, red, blue, maps)

    conn.commit()
    cur.close()


def queue_round_0A(conn, bracket, maps):
    assert len(maps) == NUM_MAPS_PER_GAME
    round_num = 0
    subround = 'A'
    cur = conn.cursor()
    max_index = int(len(bracket) / 2)

    for index in range(max_index):
        red = (bracket[2 * index], None)
        blue = (bracket[2 * index + 1], None)
        queue_match(conn, round_num, subround, index, red, blue, maps)
    conn.commit()
    cur.close()


def queue_round_1B(conn, teams, maps):
    assert len(maps) == NUM_MAPS_PER_GAME
    round_num = 1
    subround = 'B'
    cur = conn.cursor()

    losers_from = { None: None }
    losers = []

    for index in range(int(len(teams) / 2)):
        match_from, _, team_id = match_result(conn, 0, 'A', index)
        if team_id is not None:
            losers_from[team_id] = match_from
            losers.append(team_id)

    losers = pad_teams_power_of_two(losers)
    while len(losers) < int(len(teams) / 2):
        losers.append(None)
        losers = pad_teams_power_of_two(losers)

    bracket = generate_bracket(losers)
    j = 0
    for i in range(len(bracket)):
        if i is not None:
            bracket[i] = losers[j]
            j += 1

    for index in range(int(len(teams) / 4)):
        red = (losers[2 * index], losers_from[losers[2 * index]])
        blue = (losers[2 * index + 1], losers_from[losers[2 * index + 1]])
        queue_match(conn, round_num, subround, index, red, blue, maps)

    conn.commit()
    cur.close()


def get_maps(maps: List[Map], round_num: int, subround: str):
    if round_num == 0:
        return maps[:NUM_MAPS_PER_GAME]
    if subround == 'A' or subround == 'B':
        base = 4 * round_num - 2
        return maps[base:base + NUM_MAPS_PER_GAME]
    if subround == 'C':
        base = 4 * round_num
        return maps[base:base + NUM_MAPS_PER_GAME]
    raise Exception('invalid subround')


def run_tournament(conn, maps: List[Map], teams: List[Team]):
    """
    This tournament is run in double elimination format, which means teams have
    to lose two matches before they are eliminated from the tournament. There
    are separate winners and losers brackets.

    Maps are used in sets of 3, where the 3rd map of one round is the 1st map
    of the next round, just like in single elimination.

    maps  - A list of maps for the tournament, run in the order they are given.
    teams - A power of two number of teams, where some of the teams may be BYEs.
    """
    logging.debug('Generating a bracket for {} teams and {} maps'
        .format(len(teams), len(maps)))

    bracket = generate_bracket(teams)
    num_rounds = 3 * int(math.log(len(bracket), 2)) - 1

    # there may be an extra round if there is an upset in the final round
    # goal_num_maps = 2 * (int(math.log(len(bracket), 2)) + 1) + 1
    # logging.debug('We want at least {} maps, and we have {}'
    #     .format(goal_num_maps, len(maps)))
    # assert len(maps) >= goal_num_maps
    logging.debug('We have {} maps'.format(len(maps)))
    logging.info(maps)

    initial_round = int(input('Start at which round? (0 to {}):\n'
        .format(num_rounds - 1)))

    logging.debug('Running rounds {} to {}... here we go!'
        .format(initial_round, num_rounds - 1))

    for round_abs in range(initial_round, num_rounds):
        round_num, subround = round_num_split(round_abs)

        wait_for_empty_queue(conn, TABLE_NAME)
        logging.debug('Queuing Round {} out of {} ({}{})...'
            .format(round_abs, num_rounds - 1, round_num, subround))
        round_maps = get_maps(maps, round_num, subround)
        logging.debug('Using these maps: {}'.format(round_maps))

        # first round of the winners bracket
        if round_num == 0 and subround == 'A':
            queue_round_0A(conn, bracket, round_maps)
            continue

        # first round of the losers bracket
        if round_num == 1 and subround == 'B':
            queue_round_1B(conn, teams, round_maps)
            continue

        queue_round(conn, round_num, subround, round_maps)

    wait_for_empty_queue(conn, TABLE_NAME)
    logging.info('You may need to queue an extra round if Blue loses at the end.')


def run(map_tag, table_name) -> None:
    """
    No more ranked matches should be run at this point, nor should ratings be
    updated, nor new bots submitted.
    """
    global TABLE_NAME
    TABLE_NAME = table_name

    conn = db_connect()
    maps = get_tournament_maps(conn, tag=map_tag)
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

