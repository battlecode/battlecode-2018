#!/bin/python3
'''
This starts the socket server to which things connect to play the game
'''

import socketserver
import socket # pylint: disable=unused-import
import threading
import time
import random
import sys
import logging
import os.path
import ujson as json
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../bindings/python')))
import battlecode as bc

NUM_PLAYERS = 4

class Game(object): # pylint: disable=too-many-instance-attributes
    '''
    This function contains the game information, and is started at the begining
    of the process
    It handles talking to the rust engine, and sending data to the client.
    This class also processes the received data from the client, but the actual
    reception is done by the ReceiveHandler and socket server
    '''

    def __init__(self, game_map: bc.GameMap, logging_level=logging.DEBUG,
                 logging_file="server.log", time_pool=1000, time_additional=50):

        self.time_pool = time_pool/1000.
        self.time_additional = time_additional/1000.
        logging.basicConfig(filename=logging_file, level=logging_level)
        '''
        Initialize Game object
        Args:
            num_players: Number of players
            state:       Start state of game (Note can be snapshot
        '''
        self.players = [] # Array containing the player ids
        # Dict taking player id and giving bool of log in
        self.player_logged = {}
        # Dict taking player id and giving amount of time left as float
        self.times = {}

        self.disconnected = False

        # Initialize the players
        for index in range(NUM_PLAYERS):
            new_id = random.randrange(65536)
            self.players.append({'id':new_id})
            self.players[-1]['player'] = bc.Player(bc.Team.Red if index % 2 == 0 else bc.Team.Blue, bc.Planet.Earth if index < 2 else bc.Planet.Mars)
            self.player_logged[new_id] = False
            self.times[new_id] = self.time_pool

        self.started = False
        self.game_over = False

        # Lock thread running player should hold
        self.running_lock = threading.RLock()
        self.current_player_index = 0  # The index of the player whose turn it is
        self.turn_events = [threading.Event() for _ in range(len(self.players))]

        self.map = game_map

        self.manager = bc.GameController.new_manager(self.map)
        for player in self.players:
            player['start_message'] = self.manager.start_game(player['player']).to_json()
        self.viewer_messages = []
        manager_start_message = self.manager.initial_start_turn_message()
        self.last_message = manager_start_message.start_turn.to_json()
        self.viewer_messages.append(manager_start_message.viewer.to_json())
        self.initialized = 0


    def player_id2index (self, client_id):
        for i in range(len(self.players)):
            if self.players[i]['id'] == client_id:
                return i

        raise Exception("Invalid id")

    @property
    def num_log_in(self):
        '''
        Returns the number of people who have been logged in
        '''
        total = 0
        for key in self.player_logged:
            if self.player_logged[key]:
                total += 1
        return total

    def verify_login(self, unpacked_data: str):
        '''
            This function verifies the login and then logins in the player code.
            Adds them to the game state

            Args:
                data: A socket that we received data from the client on
            Return:
                Boolean if login was successful
        '''

        client_id = int(unpacked_data['client_id'])

        # Check if they are in our list of clients
        if client_id not in [player['id'] for player in self.players]:
            return "Client id Mismatch"

        # Check if they logged in already
        if self.player_logged[client_id]:
            return "Already Logged In"

        self.player_logged[client_id] = True

        # Check if all the players are logged in and then start the game
        logging.info("Player logged in: %s", self.player_logged)
        if len(self.players) == self.num_log_in:
            self.start_game()
        return client_id

    def start_game(self):
        '''
        This code handles starting the game. Anything that is meant to be
        triggered when a game starts is stored here.
        '''

        # Init the player who starts and then tell everyone we started
        self._set_player_turn(0)
        self.started = True
        return

    def _set_player_turn(self, player_index):
        self.current_player_index = player_index
        self.turn_events[player_index].set()



    def end_turn(self):
        '''
        This function handles the release of all locks and moving the player to
        the next turn. It also handles sleeping the docker instances.
        Args:
            client_id: The int of the client that this thread is related to
        '''


        # Increment to the next playe
        self.current_player_index = (self.current_player_index + 1) % len(self.players)
        self._set_player_turn(self.current_player_index)

    def get_viewer_messages(self):
        '''
        A generator for the viewer messages
        '''
        # TODO check this works with the way the engine works
        max_yield_item = 0
        while not self.game_over or max_yield_item != len(self.viewer_messages):
            if len(self.viewer_messages) > max_yield_item:
                new_max = len(self.viewer_messages)
                for i in range(max_yield_item, new_max):
                    yield self.viewer_messages[i]
                max_yield_item = new_max
            time.sleep(0.1)




    def start_turn(self, player_index: int):
        '''
        This is a blocking function that waits until it client_id's turn to
        start the game. It attempts to take the game lock and then checks to see
        if the client_id matches the next player id. If it does it returns and
        the player can start running.

        '''

        logging.debug("Client %s: entered start turn", player_index)
        while True:
            if self.turn_events[player_index].wait(timeout=0.1):
                self.turn_events[player_index].clear()
                assert(self.current_player_index == player_index)
                self.times[self.players[player_index]['id']] += self.time_additional
                logging.debug("Client %s: exit start turn", player_index)
                return True

            if self.game_over:
                return False

    def make_action(self, turn_message: bc.TurnMessage, client_id: int, diff_time: float):
        '''
        Take action data and give it to the engine
        Args:
            data: the data received from the stream

        '''
        # interact with the engine
        application = self.manager.apply_turn(turn_message)
        self.last_message = application.start_turn.to_json()
        self.viewer_messages.append(application.viewer.to_json())
        self.times[client_id] -= diff_time
        return


def create_receive_handler(game: Game, dockers, use_docker: bool,
                           is_unix_stream: bool)  \
                                    -> socketserver.BaseRequestHandler:
    '''
    Create a Class that will be used a receive handler

    Args:
        game: The game the receive handler should operate on
        dockers: A map of the docker files with the key being
        use_docker: if True sleep and wake with docker otherwise don't use
                    docker. Useful for testing the socket server
    Return:
        A ReceiveHandler class
    '''
    class ReceiveHandler(socketserver.BaseRequestHandler):
        '''
        This class overrides the default handling method in socketServer, so it
        calls what we want
        '''

        def __init__(self, *args, **kwargs):
            '''
            Hidden init
            '''
            self.game = game
            self.dockers = dockers
            self.client_id = 0
            self.error = ""
            self.logged_in = False
            self.is_unix_stream = is_unix_stream
            super(ReceiveHandler, self).__init__(*args, **kwargs)

        def get_next_message(self) -> object:
            '''
            Returns the json loaded object of the next string that is sent over the
            socket

            Returns:
                An object, for our purposes this will be a dictionary, of the json
                loaded string
            '''

            recv_socket = self.request
            game = self.game

            wrapped_socket = recv_socket.makefile('rwb', 1)
            logging.debug("Client %s: Waiting for next message", self.client_id)
            try:
                data = next(wrapped_socket)
            except (StopIteration, IOError):
                # TODO on DC assign winners and losers
                wrapped_socket.close()
                recv_socket.close()
                logging.warning("Client %s: Game Over", self.client_id)
                self.disconnected = True
                for i in range(NUM_PLAYERS):
                    if self.client_id == self.game.players[i]['id']:
                        if i < 2:
                            self.game.winner = 'player2'
                        else:
                            self.game.winner = 'player1'
                self.game.game_over = True
                sys.exit(0)
            except KeyboardInterrupt:
                wrapped_socket.close()
                recv_socket.close()
                self.disconnected = True
                for i in range(NUM_PLAYERS):
                    if self.client_id == self.game.players[i]['id']:
                        if i < 2:
                            self.game.winner = 'player2'
                        else:
                            self.game.winner = 'player1'
                logging.warning("Client %s: Game Over", self.client_id)
                self.game.game_over = True
                raise KeyboardInterrupt
            finally:
                wrapped_socket.close()

            data = data.decode("utf-8").strip()
            unpacked_data = json.loads(data)
            return unpacked_data

        def send_message(self, obj: object) -> None:
            '''
            Sends newline delimited message to socket
            The object desired to be sent will be converted to a json and then encoded
            and sent.

            Args:
                Obj: The object that wants to be serialized and sent over

            Returns:
                None
            '''


            send_socket = self.request
            if isinstance(obj, bytes):
                obj = obj.decode()

            message = obj + "\n"
            encoded_message = message.encode()
            logging.debug("Client %s: Sending message %s", self.client_id,
                          encoded_message)

            wrapped_socket = send_socket.makefile('rwb', 1)
            try:
                wrapped_socket.write(encoded_message)
            except IOError:
                self.disconnected = True
                for i in range(NUM_PLAYERS):
                    if self.client_id == self.game.players[i]['id']:
                        if i < 2:
                            self.game.winner = 'player2'
                        else:
                            self.game.winner = 'player1'
                logging.warning("Client %s: Game Over", self.client_id)
                self.game.game_over = True
                wrapped_socket.close()
                send_socket.close()
                sys.exit(0)
            except KeyboardInterrupt:
                self.disconnected = True
                for i in range(NUM_PLAYERS):
                    if self.client_id == self.game.players[i]['id']:
                        if i < 2:
                            self.game.winner = 'player2'
                        else:
                            self.game.winner = 'player1'
                self.game.game_over = True
                wrapped_socket.close()
                send_socket.close()
                sys.exit(0)
            finally:
                wrapped_socket.close()
            return


        def message(self, state_diff):
            '''
            Compress the current state into a message that will be sent to the
            client
            '''
            if self.error == "":
                error = "null"
            else:
                self.docker.destroy()

            if state_diff == "":
                state_diff = '""'
            if isinstance(state_diff, bytes):
                state_diff = state_diff.decode()

            if self.logged_in:
                logged_in = "true"
            else:
                logged_in = "false"

            message = f'{{"logged_in":{logged_in},"client_id":"{self.client_id}","error":{error},"message":{state_diff}}}'
            return message

        def player_handler(self):
            '''
            This is the handler for socket connections from players
            '''
            self.logged_in = False
            logging.debug("Client connected to server")
            # TODO check if this is enough time out is generous enough
            self.request.settimeout(50)

            # Handle Login phase
            while not self.logged_in:
                unpacked_data = self.get_next_message()

                logging.debug("Received %s when trying to login", unpacked_data)

                verify_out = self.game.verify_login(unpacked_data)

                self.error = ""
                if not isinstance(verify_out, int):
                    self.error = verify_out
                    logging.warning("Client failed to log in error: %s",
                                    self.client_id)
                else:
                    logging.info("Client %s: logged in succesfully", self.client_id)
                    self.logged_in = True
                    self.client_id = verify_out

                log_success = self.message("")

                self.send_message(log_success)

            """
            if use_docker:
                # Attribute defined here for ease of use.
                self.docker = self.dockers[self.client_id]#pylint: disable=W0201
                self.docker.pause()
            """

            logging.debug("Client %s: Spinning waiting for game to start",
                          self.client_id)

            while not self.game.started and not self.game.game_over:
                # Spin while waiting for game to start
                time.sleep(0.5)


            logging.info("Client %s: Game started", self.client_id)

            while self.game.started and not self.game.game_over:
                # This is the loop that the code will always remain in

                # Blocks until it this clients turn
                our_index = self.game.player_id2index(self.client_id)

                if not self.game.start_turn(our_index):
                    # Game is done
                    self.request.close()
                    return

                if self.game.manager.is_over():
                    self.game.game_over = True
                    self.game.end_turn()
                    self.request.close()
                    return

                logging.debug("Client %s: Started turn", self.client_id)

                if self.game.initialized > 3:
                    start_turn_msg = self.message(self.game.last_message)
                else:
                    start_turn_msg = self.message(self.game.players[self.game.current_player_index]['start_message'])

                """# Start player code computing
                if use_docker:
                    self.docker.unpause()
                """

                # TODO check this timer makes, sense it looks like the right one
                # but i'm getting wierd results when testing?
                start_time = time.perf_counter()
                self.send_message(start_turn_msg)

                if self.game.initialized > 3:
                    unpacked_data = self.get_next_message()
                    end_time = time.perf_counter()
                    diff_time = end_time-start_time

                    # Check client is who they claim they are
                    if int(unpacked_data['client_id']) != self.client_id:
                        assert False, "Wrong Client id"

                    # Get the moves to pass to the game
                    turn_message = bc.TurnMessage.from_json(json.dumps(unpacked_data['turn_message']))

                    game.make_action(turn_message, self.client_id, diff_time)
                else:
                    self.game.initialized += 1

                """
                if use_docker:
                    self.docker.pause()
                """
                self.game.end_turn()

        def viewer_handler(self):
            '''
            This handles the connection to the viewer
            '''
            for message in self.game.get_next_message():
                # TODO check this schema works for the viewer
                self.send_message(message)

        def handle(self):
            '''
            This does all the processing of the data we receive and we spend our
            time in this function.
            '''
            if self.is_unix_stream:
                self.player_handler()
            else:
                self.viewer_handler()

    return ReceiveHandler


def start_server(sock_file: str, game: Game, dockers, use_docker=True) -> socketserver.BaseServer:
    '''
    Start a socket server for the players to connect to
    Args:
        sock_file: This is a string name of the file that will be used for
                    as UnixStream

        game: The game information that is being run

        use_docker bool: whether to use docker or not

    Return:
        server_thread: The connection so it can be closed by parent functions at
                        the appropriate time
    '''

    # Create handler for mangaing each connections to server
    receive_handler = create_receive_handler(game, dockers, use_docker, True)

    if isinstance(sock_file, tuple):
        # tcp port
        server = socketserver.ThreadingTCPServer(sock_file, receive_handler)
    else:
        server = socketserver.ThreadingUnixStreamServer(sock_file, receive_handler)

    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    logging.info("Server Started at %s", sock_file)
    server_thread.start()

    return server

def start_viewer_server(port: int, game: Game) -> socketserver.BaseServer:
    '''
    Start a socket server for the players to connect to
    Args:
        port: port to connect to viewer on

        game: The game information that is being run

        use_docker bool: whether to use docker or not

    Return:
        server_thread: The connection so it can be closed by parent functions at
                        the appropriate time
    '''

    # Create handler for mangaing each connections to server
    receive_handler = create_receive_handler(game, {}, False, False)

    # Start server
    server = socketserver.ThreadingTCPServer(('localhost', port), receive_handler)
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()

    return server
