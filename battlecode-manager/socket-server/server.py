#!/bin/python
import socket
import socketserver
import threading
import time
import argparse
try:
    import ujson as json
except:
    import json



class Game(object):
    '''
    This function contains the game information, and is started at the begining
    of the process
    It handles talking to the rust engine, and sending data to the client.
    This class also processes the received data from the client, but the actual
    reception is done by the ReceiveHandler and socket server
    '''

    def __init__(self, num_players, map_string):
        # map = open(map_string, 'r')
        self.num_players = num_players
        self.players = [] # Array contains sockets for the different clients
        pass

    '''
    False before the number of players expected to have joined have joined.
    After that it will become true
    '''
    @property
    def started(self):
        return len(self.players) >= self.num_players

    def add_player(self, client):
        if client in self.players:
            return
        print("Player loged in")
        self.players.append(client)


    def send_data_to_client(self, index, data):
        if index >= len(self.players):
            return
        player = self.players[index]
        player.send(data.encode('utf-8'))

    '''
        This function verifies the login and then logins in the player code.
        Adds them to the game state

        client (Socket): A socket that we received data from the client on
    '''
    def handle_login(self, client):

        socket = client.makefile('rwb', 2**10)

        try:
            data = next(socket)
        except:
            # Print DC message
            print("Failed Login: " + str(client))

        #implement our login information
        login_info = json.loads(data)
        print(login_info)

        game.add_client(client)


    def make_action(self, data):
        action = json.loads(data)
        # interact with the engine


class ReceiveHandler(socketserver.BaseRequestHandler):
    '''
    This class overrides the default handling method in socketServer, so it
    calls what we want
    '''

    def handle(self):
        '''
        This does all the processing of the data we receive and we spend our
        time in this function.
        '''
        game.handle_login(self.request)
        print(game.players)
        while True: # Change the while true to a more meaningful statement
            self._socket = self.request.makefile('rwb', 2)
            # We if the try fails the client dced
            try:
                self.data = next(self._socket)
            except:
                # Print DC message
                print("Client" + str(self.client_address) +  "Disconnected")
                break

            #Confirm that the client is allowed to make an action

            # Process action that the client is doing
            game.make_action(data)

            # Sleep the docker instance and wake up the other docker instance



if __name__=="__main__":
    parser = argparse.ArgumentParser(description="Starts and runs a battlecode"+
                                     "game")

    parser.add_argument('--socket-file', help='file used as socket connection'+ \
            ' for players', dest='sock_file', default='battlecode-1')

    args = parser.parse_args()

    sys.exit()
     
    NUM_PLAYERS = 2
    game = Game(NUM_PLAYERS, "NULL")

    server = socketserver.ThreadingUnixStreamServer(args.sock_file, ReceiveHandler)

    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()
    #This is the game thread so it handles state and position of the game
    while(not game.started):
        time.sleep(1)
        print("Waiting for game to start")
    game.send_data_to_client(1,"hi")
