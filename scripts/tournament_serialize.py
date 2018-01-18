"""
Battlecode 2018 Tournament Serializer

Outputs a JSON tournament format for the Unity viewer:

{
    "tournament": "sprint",
    "rounds": [{
        "round": 0,
        "matches": [{
            "index": 0",
            "Red": {
                "id": 1,
                "name": "Team 1",
                "avatar": "avatar/1.png"
            },
            "Blue": {
                "id": 2,
                "name": "Team 2",
                "avatar": "avatar/2.png"
            },
            "replays": ["replay/1.bc18z", "replay/2.bc18z", "replay/3.bc18z"],
            "winner_ids": [1, 2, 2],
            "winner_id": 2
        }]
    }]
}
"""

import json
import re

from tournament_single_elim import db_connect

# Hardcoded relative paths to local files
AVATAR_PREFIX = 'avatar/'
REPLAY_PREFIX = 'replay/'

# Indexed information, avatar is formatted like "avatar/1.png"
TEAM_ID_TO_NAME = {}
TEAM_ID_TO_AVATAR = {}

# Other constants
DEFAULT_AVATAR = AVATAR_PREFIX + 'default.png'

def index_team_info(cur) -> None:
    '''
    Fill in TEAM_ID_TO_NAME and TEAM_ID_TO_AVATAR with the information of all
    teams that have submitted a robot.
    '''
    query = 'SELECT DISTINCT teams.id, teams.name, teams.avatar \
        FROM battlecode_teams teams INNER JOIN scrimmage_submissions subs \
        ON teams.id = subs.team;'
    cur.execute(query);

    teams = cur.fetchall()
    for team_id, name, avatar in teams:
        TEAM_ID_TO_NAME[team_id] = name
        if avatar is None:
            TEAM_ID_TO_AVATAR[team_id] = DEFAULT_AVATAR
        else:
            TEAM_ID_TO_AVATAR[team_id] = AVATAR_PREFIX  + re.search('/(\d+\..+)$', avatar).group(1)


def serialize_team(team_id):
    '''
    Returns the deserialized version of a team.
    '''
    return {
        'id': team_id,
        'name': TEAM_ID_TO_NAME[team_id],
        'avatar': TEAM_ID_TO_AVATAR[team_id]
    }


def serialize_match(cur, round_num: int, match_index: int):
    '''
    Returns the deserialized version of a single match.
    '''
    match = {}
    match['index'] = match_index
    match['replays'] = []
    match['winner_ids'] = []

    cur.execute('SELECT red_team, blue_team, status, replay \
        FROM {} WHERE round={} AND index={} \
        AND (status=\'redwon\' or status=\'bluewon\');'
        .format(TABLE_NAME, round_num, match_index))

    matches = cur.fetchall()
    if len(matches) != 3:
        return None

    red_team, blue_team, _, _ = matches[0]

    if red_team is None or blue_team is None:
        return None

    match['Red'] = serialize_team(red_team)
    match['Blue'] = serialize_team(blue_team)

    match_winners = {
        red_team: 0,
        blue_team: 0
    }

    for _, _, status, replay in matches:
        match['replays'].append(REPLAY_PREFIX + re.search('replays/(.+\.bc18z)$', replay).group(1))

        if status == 'redwon':
            match['winner_ids'].append(red_team)
            match_winners[red_team] += 1
        elif status == 'bluewon':
            match['winner_ids'].append(blue_team)
            match_winners[blue_team] += 1
        else:
            raise Exception("Match should be finished.")

        if match_winners[red_team] == 2:
            match['winner_id'] = red_team
            break
        if match_winners[blue_team] == 2:
            match['winner_id'] = blue_team
            break
    
    return match

def serialize_round(cur, round_num: int):
    '''
    Returns the deserialized version of a single round.
    '''
    t_round = {}
    t_round['round'] = round_num
    t_round['matches'] = []

    cur.execute('SELECT MAX(index) FROM {} WHERE round=%s;'.format(TABLE_NAME), (round_num,))
    max_index = cur.fetchone()[0]
    for index in range(0, max_index + 1):
        match = serialize_match(cur, round_num, index)
        if match is None:
            continue
        t_round['matches'].append(match)

    return t_round


def serialize_tournament(cur):
    index_team_info(cur)

    tournament = {}
    tournament['tournament'] = TOURNAMENT_NAME
    tournament['rounds'] = []

    cur.execute('SELECT MAX(round) FROM {};'.format(TABLE_NAME))
    max_round = cur.fetchone()[0]
    for round_num in range(0, max_round + 1):
        tournament['rounds'].append(serialize_round(cur, round_num))

    return tournament


if __name__ == '__main__':
    conn = db_connect()
    cur = conn.cursor()
    TOURNAMENT_NAME = input('Tournament name? ')
    TABLE_NAME = input('DB table (i.e. tournament_sprint): ')
    tournament = serialize_tournament(cur)
    cur.close()

    filename = '{}.bc18r'.format(TOURNAMENT_NAME)
    f = open(filename, 'w')
    f.write(json.dumps(tournament, indent=4))
    f.close()

    print('Wrote JSON to {}'.format(filename))
