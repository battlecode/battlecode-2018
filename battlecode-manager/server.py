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
import ujson as json
import battlecode as bc

# TODO:
# Timing works, but still has to be checked
# We should also check that pausing doesn't hurt unix streams they don't
# have

INIT_TIME = 250
TIME_PER_TURN = 10


class Game(object): # pylint: disable=too-many-instance-attributes
    '''
    This function contains the game information, and is started at the begining
    of the process
    It handles talking to the rust engine, and sending data to the client.
    This class also processes the received data from the client, but the actual
    reception is done by the ReceiveHandler and socket server
    '''

    def __init__(self, game_map: bc.GameMap, logging_level=logging.DEBUG,
                 logging_file="server.log"):

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

        # Initialize the player_ids
        for _ in range(4):
            new_id = random.randrange(65536)
            self.players.append({'id':new_id})
            self.players[-1]['player'] = bc.Player(bc.Team.Red if index < 2 else bc.Team.Blue, bc.Planet.Earth if index % 2 == 0 else bc.Planet.Mars)
            self.player_logged[new_id] = False
            self.times[new_id] = INIT_TIME

        self.started = False
        self.game_over = False

        # Lock thread running player should hold
        self.running_lock = threading.RLock()
        self.this_turn_pid = 0 # The id of the player whose turn it is

        self.map = game_map

        self.manager = bc.GameController.new_manager(self.map)
        for player in self.players:
            player['start_message'] = self.manager.start_game(player['player']).to_json()
        self.viewer_messages = []
        self.last_message = self.manager.initial_start_turn_message().to_json()
        self.initialized = False

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

        client_id = unpacked_data['client_id']

        # Check if they are in our list of clients
        if client_id not in self.player_ids:
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
        self.this_turn_pid = self.players[0]['id']
        self.started = True
        return



    def end_turn(self):
        '''
        This function handles the release of all locks and moving the player to
        the next turn. It also handles sleeping the docker instances.
        Args:
            client_id: The int of the client that this thread is related to
        '''


        # Increment to the next player
        index = self.player_ids.index(self.this_turn_pid)
        index = (index + 1) % len(self.player_ids)
        self.this_turn_pid = self.player_ids[index]

        self.running_lock.release()

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




    def start_turn(self, client_id: int):
        '''
        This is a blocking function that waits until it client_id's turn to
        start the game. It attempts to take the game lock and then checks to see
        if the client_id matches the next player id. If it does it returns and
        the player can start running.

        This also handles waking the docker instances to start computing
        '''

        logging.debug("Client %s: entered start turn", client_id)
        while not self.game_over:
            time.sleep(0.05)
            if self.running_lock.acquire(timeout=0.1):
                if  self.this_turn_pid == client_id:
                    break
                self.running_lock.release()

        self.times[client_id] += TIME_PER_TURN
        logging.debug("Client %s: exit start turn", client_id)
        return




    def make_action(self, turn_message: bc.TurnMessage, client_id: int, diff_time: float):
        '''
        Take action data and give it to the engine
        Args:
            data: the data received from the stream

        '''
        # interact with the engine
        # TODO add to viewer_messages
        self.viewer_messages.append("")
        self.last_message = self.manager.apply_turn(turn_message).to_json()
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
                game.game_over = True
                sys.exit(0)
            except KeyboardInterrupt:
                wrapped_socket.close()
                recv_socket.close()
                game.game_over = True
                logging.warning("Client %s: Game Over", self.client_id)
                print("Cleaning up")
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

            message = json.dumps(obj) + "\n"
            encoded_message = message.encode()
            logging.debug("Client %s: Sending message %s", self.client_id,
                          encoded_message)

            wrapped_socket = send_socket.makefile('rwb', 1)
            try:
                wrapped_socket.write(encoded_message)
            except IOError:
                # TODO handle DCs better
                self.game.game_over = True
                wrapped_socket.close()
                send_socket.close()
                sys.exit(0)
            except KeyboardInterrupt:
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
            message = {}
            message['logged_in'] = self.logged_in
            message['client_id'] = self.client_id
            message['error'] = self.error
            message['message'] = state_diff
            return message

        def player_handler(self):
            '''
            This is the handler for socket connections from players
            '''
            self.logged_in = False
            logging.debug("Client connected to server")
            # TODO check if this is enough time out is generous enough
            self.request.settimeout(5)

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
                self.game.start_turn(self.client_id)

                game_over = game.manager.is_game_over()
                # check if game is over, if so, get winner in winner variables
                if game_over is True: #replace with actual checking
                    winner = 0 #report the winner
                    self.request.close()
                    logs = self.docker.destroy()
                    report_winner() # somehow alert people who won
                    return


                logging.debug("Client %s: Started turn", self.client_id)

                if game.initialized:
                    start_turn_msg = self.message(game.last_message)
                else:
                    for player in self.game.players:
                        if player['id'] == self.game.this_turn_pid:
                            start_turn_msg = player['start_message']

                """# Start player code computing
                if use_docker:
                    self.docker.unpause()
                """

                # TODO check this timer makes, sense it looks like the right one
                # but i'm getting wierd results when testing?
                start_time = time.perf_counter()
                self.send_message(start_turn_msg)

                if game.initialized:
                    unpacked_data = self.get_next_message()
                    end_time = time.perf_counter()
                    diff_time = end_time-start_time

                    # Check client is who they claim they are
                    if unpacked_data['client_id'] != self.client_id:
                        assert False, "Wrong Client id"

                    # Get the moves to pass to the game
                    turn_message = bc.TurnMessage.from_json(unpacked_data['turn_message'])

                    game.make_action(turn_message, self.client_id, diff_time)



                """
                if use_docker:
                    self.docker.pause()
                """
                self.game.end_turn()

        def viewer_handler(self):
            '''
            This handles the connection to the viewer
            '''
            pass

            while True:
                # TODO interact with engine and send next message
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

    # Start server
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

if __name__ == "__main__":
    print("Do not run this fuction call battlecode cli to start a game")
    sys.exit(1)
