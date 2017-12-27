#!/bin/python3
'''
This starts the socket server to which things connect to play the game
'''

import socketserver
import socket # pylint: disable=unused-import
import threading
import time
import argparse
import random
import sys
# import engine
import logging
import ujson as json



class Game(object): # pylint: disable=too-many-instance-attributes
    '''
    This function contains the game information, and is started at the begining
    of the process
    It handles talking to the rust engine, and sending data to the client.
    This class also processes the received data from the client, but the actual
    reception is done by the ReceiveHandler and socket server
    '''

    def __init__(self, num_players: int, state: bytes,
                 logging_level=logging.DEBUG, logging_file="server.log"):
        logging.basicConfig(filename=logging_file, level=logging_level)
        '''
        Initialize Game object
        Args:
            num_players: Number of players
            state:       Start state of game (Note can be snapshot
        '''
        self.num_players = num_players
        self.player_ids = [] # Array containing the player ids

        self.player_logged = {}
        # Dict taking player id and giving bool of log in

        # Initialize the player_ids
        for _ in range(num_players):
            new_id = random.randrange(65536)
            self.player_ids.append(new_id)
            self.player_logged[new_id] = False

        self.state = state
        self.started = False

        # Lock thread running player should hold
        self.running_lock = threading.RLock()

        self.this_turn_pid = 0 # The id of the player whose turn it is

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
        if len(self.player_ids) == self.num_log_in:
            self.start_game()
        return client_id

    def start_game(self):
        '''
        This code handles starting the game. Anything that is meant to be
        triggered when a game starts is stored here.
        '''

        #Init the player who starts and then tell everyone we started
        self.this_turn_pid = self.player_ids[0]
        self.started = True
        return



    def end_turn(self):
        '''
        This function handles the release of all locks and moving the player to
        the next turn. It also handles sleeping the docker instances.
        Args:
            client_id: The int of the client that this thread is related to
        '''

        # sleep docker instance

        # Increment to the next player
        index = self.player_ids.index(self.this_turn_pid)
        index = (index + 1) % len(self.player_ids)
        self.this_turn_pid = self.player_ids[index]

        self.running_lock.release()


    def start_turn(self, client_id: int):
        '''
        This is a blocking function that waits until it client_id's turn to
        start the game. It attempts to take the game lock and then checks to see
        if the client_id matches the next player id. If it does it returns and
        the player can start running.

        This also handles waking the docker instances to start computing
        '''

        while True:
            time.sleep(0.05)
            self.running_lock.acquire()
            if  self.this_turn_pid == client_id:
                break
            self.running_lock.release()

        # Start Docker instance
        return




    def make_action(self, data: bytes):
        # TODO Write this function
        '''
        Take action data and give it to the engine
        Args:
            data: the data received from the stream

        Return:
            State diff to be send back to the client
        '''
        # interact with the engine

        return data


def get_next_message(recv_socket: socket.socket) -> object:
    '''
    Returns the json loaded object of the next string that is sent over the
    socket

    Args:
        socket: The socket that is trying to be read from

    Returns:
        An object, for our purposes this will be a dictionary, of the json
        loaded string
    '''

    wrapped_socket = recv_socket.makefile('rb', 2)
    try:
        data = next(wrapped_socket)
    except (StopIteration, IOError):
        wrapped_socket.close()
        recv_socket.close()
        sys.exit(0)
    except KeyboardInterrupt:
        wrapped_socket.close()
        recv_socket.close()
        print("Cleaning up")
        raise KeyboardInterrupt
    finally:
        wrapped_socket.close()

    data = data.decode("utf-8").strip()
    unpacked_data = json.loads(data)
    return unpacked_data

def send_message(send_socket: socket.socket, obj: object) -> None:
    '''
    Sends newline delimited message to socket
    The object desired to be sent will be converted to a json and then encoded
    and sent.

    Args:
        socket: socket to send it on must be wrapped in a fd
        message: any object that wishes to be sent.

    Returns:
        None
    '''

    message = json.dumps(obj) + "\n"
    encoded_message = message.encode()

    wrapped_socket = send_socket.makefile('wb', 2)
    try:
        wrapped_socket.write(encoded_message)
    except IOError:
        wrapped_socket.close()
        send_socket.close()
        sys.exit(0)
    except KeyboardInterrupt:
        wrapped_socket.close()
        raise KeyboardInterrupt
    finally:
        wrapped_socket.close()
    return


# TODO Maybe change this factory function to a class method?
def create_receive_handler(game: Game, dockers) -> socketserver.BaseRequestHandler:
    '''
    Create a Class that will be used a receive handler

    Args:
        game: The game the receive handler should operate on
        dockers: A map of the docker files with the key being

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
            super(ReceiveHandler, self).__init__(*args, **kwargs)

        def handle(self):
            '''
            This does all the processing of the data we receive and we spend our
            time in this function.
            '''

            logged_in = False
            logging.debug("Client connected to server")

            # Handle Login phase
            while not logged_in:
                unpacked_data = get_next_message(self.request)

                logging.debug("Received %s when trying to login", unpacked_data)

                client_id = self.game.verify_login(unpacked_data)

                error = ""
                if not isinstance(client_id, int):
                    # TODO Here we send the error message back
                    # TODO Need to figure out what to do here
                    error = client_id
                    logging.warning("Client failed to log in error: %s",
                                    client_id)
                else:
                    logging.info("Client %d logged in succesfully", client_id)
                    logged_in = True
                log_success = {}
                log_success['logged_in'] = logged_in
                log_success['error'] = error
                log_success['client_id'] = client_id

                send_message(self.request, log_success)


            logging.debug("Spinning waiting for game to start")
            while not self.game.started:
                # Spin while waiting for game to start
                # TODO Get this to close properly if the conn closes
                time.sleep(0.5)

            game_start_dict = {}
            game_start_dict['game_started'] = True
            send_message(self.request, game_start_dict)

            # sleep docker instance

            logging.info("Game started")

            # TODO Change the while true to a more meaningful statement
            while self.game.started:
                # This is the loop that the code will always remain in

                # Blocks until it this clients turn
                self.game.start_turn(client_id)

                print("Started Turn")

                unpacked_data = get_next_message(self.request)

                # Check client is who they claim they are
                if unpacked_data['client_id'] != client_id:
                    assert False, "Wrong Client id"

                # Get the moves to pass to the game
                moves = unpacked_data['moves']

                # Process action that the client is doing
                game.make_action(moves, client_id)

                # Send information back to client

                self.game.end_turn(client_id)

    return ReceiveHandler


def start_server(sock_file: str, game: Game, dockers) -> socketserver.BaseServer:
    '''
    Start a socket server for the players to connect to
    Args:
        sock_file: This is a string name of the file that will be used for
                    as UnixStream

        game: The game information that is being run

    Return:
        server_thread: The connection so it can be closed by parent functions at
                        the appropriate time
    '''

    # Create handler for mangaing each connections to server
    receive_handler = create_receive_handler(game, dockers)

    # Start server
    server = socketserver.ThreadingUnixStreamServer(sock_file, receive_handler)
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    logging.info("Server Started at %s", sock_file)
    server_thread.start()

    return server

if __name__ == "__main__":
    PARSER = argparse.ArgumentParser(description="Starts and runs a battlecode"+
                                     "game")

    PARSER.add_argument('--socket-file', help='file used as socket connection'+ \
            ' for players', dest='sock_file', default='/tmp/battlecode-1')

    ARGS = PARSER.parse_args()

    print(ARGS.sock_file)
    NUM_PLAYERS = 2
    GAME = Game(NUM_PLAYERS, "NULL")
    DOCKERS = {}
    SERVER_CONN = start_server(ARGS.sock_file, GAME, DOCKERS)
    try:
        while not GAME.started:
            time.sleep(1)
            print("Waiting for game to start")
    except KeyboardInterrupt:
        print("Exiting gracefully")
        SERVER_CONN.shutdown()
        SERVER_CONN.socket.close()
