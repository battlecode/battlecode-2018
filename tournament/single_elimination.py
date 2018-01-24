"""
Battlecode 2018 Tournament Runner
"""
from tournament_helper import *

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

        wait_for_empty_queue(conn, TABLE_NAME)
        logging.debug('Queuing Round {} out of {}...'
            .format(round_num, num_rounds - 1))
        round_maps = maps[(NUM_MAPS_PER_GAME - 1) * round_num : 
                          (NUM_MAPS_PER_GAME - 1) * (round_num + 1) + 1]
        logging.debug('Using these maps: {}'.format(round_maps))
        queue_round(conn, round_num, round_maps)


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

