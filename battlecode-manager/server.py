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
        # TODO handle loading state
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
        self.game_over = False

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

        # TODO sleep docker instance

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

        logging.debug("Client %s: entered start turn", client_id)
        while not self.game_over:
            time.sleep(0.05)
            if self.running_lock.acquire(timeout=0.1):
                if  self.this_turn_pid == client_id:
                    break
                self.running_lock.release()

        logging.debug("Client %s: exit start turn", client_id)
        # TODO Start Docker instance
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

class MessageSchema(object):
    '''
    This creates a message to send to the client.
    The point of this class is to ensure that all the fields are there, so there
    is no wierd parsing errors down the line
    '''

    def __init__(self, logged_in: bool, client_id: int, error: str, # pylint: disable=too-many-arguments
                 game_started: bool, state_diff: bytes):# pylint: disable=too-many-arguments
        '''
        This function intializes the dictionary
        '''
        self.message = {}
        assert isinstance(logged_in, bool), "logged_in wrong type"
        assert isinstance(client_id, int), "client_id wrong type"
        assert isinstance(error, str), "error wrong type"
        assert isinstance(game_started, bool), "game_started wrong type"
        self.message['logged_in'] = logged_in
        self.message['client_id'] = client_id
        self.message['error'] = error
        self.message['game_started'] = game_started
        self.message['state_diff'] = state_diff


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
            self.client_id = 0
            self.error = ""
            self.logged_in = False
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


        def handle(self):
            '''
            This does all the processing of the data we receive and we spend our
            time in this function.
            '''

            self.logged_in = False
            logging.debug("Client connected to server")

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

                log_success = MessageSchema(self.logged_in, self.client_id,
                                            self.error, False, 0)

                self.send_message(log_success.message)


            logging.debug("Client %s: Spinning waiting for game to start",
                          self.client_id)
            while not self.game.started and not self.game.game_over:
                # Spin while waiting for game to start
                # TODO probe the socket close
                time.sleep(0.5)

            game_start_dict = MessageSchema(self.logged_in, self.client_id,
                                            self.error, self.game.started,
                                            self.game.state)
            self.send_message(game_start_dict.message)

            # TODO sleep docker instance

            logging.info("Client %s: Game started", self.client_id)

            while self.game.started and not self.game.game_over:
                # This is the loop that the code will always remain in

                # Blocks until it this clients turn
                self.game.start_turn(self.client_id)

                if self.game.game_over:
                    # Cleanup
                    self.request.close()
                    return

                logging.debug("Client %s: Started turn", self.client_id)

                # Send state diff over

                unpacked_data = self.get_next_message()

                # Check client is who they claim they are
                if unpacked_data['client_id'] != self.client_id:
                    assert False, "Wrong Client id"

                # Get the moves to pass to the game
                moves = unpacked_data['moves']

                # Process action that the client is doing
                game.make_action(moves, self.client_id)

                # Send information back to client

                self.game.end_turn(self.client_id)

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
