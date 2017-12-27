'''
Testing suite for the engine
'''

import unittest
import socket
import os
import json
import server

SOCK_FILE = "/tmp/battlecode-test"

def create_socket():
    '''
    Creats a single socket wraps
    '''

    conn = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    conn.connect(SOCK_FILE)
    return conn

def login(conn, client_id):
    '''
    Log a socket in with a client id
    '''
    login_dict = {}
    login_dict['client_id'] = client_id
    login_str = json.dumps(login_dict) + "\n"
    conn.sendall(login_str.encode())

def recv_next_message(conn):
    '''
    Read the next message comming from this socket
    '''
    with conn.makefile('rb', 2) as wrapped_conn:
        data = next(wrapped_conn)
    data = data.decode("utf-8").strip()
    unpacked_data = json.loads(data)
    return unpacked_data

class TestLoginSocketServer(unittest.TestCase):
    '''
    Testing suite to test the login function of the socket server.
    '''

    def setUp(self):
        '''
        This function setusp up the unix stream server for all the other tests.
        '''
        num_players = 2
        self.game = server.Game(num_players, "NULL")
        dockers = {}
        try:
            os.unlink(SOCK_FILE)
        except OSError:
            if os.path.exists(SOCK_FILE):
                print("File exists at connection point")
                raise

        self.server = server.start_server(SOCK_FILE, self.game, dockers)


    def test_login(self):
        '''
        Test a single login works
        '''
        conn = create_socket()
        try:
            secret_key = self.game.player_ids[0]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["client_id"], secret_key)


    def test_login_any_order(self):
        '''
        Check login in any order works
        '''
        conn = create_socket()
        try:
            secret_key = self.game.player_ids[1]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["client_id"], secret_key)

    def test_two_login(self):
        '''
        Test that two logins work
        '''
        conn = create_socket()
        try:
            secret_key = self.game.player_ids[0]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["client_id"], secret_key)

        conn = create_socket()
        try:
            secret_key = self.game.player_ids[1]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["client_id"], secret_key)

    def test_failed_login(self):
        '''
        Test that a wrong login fails to work
        '''
        conn = create_socket()
        wrong_key = 0
        while wrong_key in self.game.player_ids:
            wrong_key += 1

        try:
            print(wrong_key)
            secret_key = wrong_key
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertFalse(unpacked_data["logged_in"])
        self.assertNotEqual(unpacked_data["client_id"], secret_key)

    def test_start_game(self):
        '''
        Test that all successful logins start the game
        '''
        conn = create_socket()
        try:
            secret_key = self.game.player_ids[0]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["client_id"], secret_key)

        conn = create_socket()
        try:
            secret_key = self.game.player_ids[1]
            login(conn, secret_key)
            # login check
            unpacked_data = recv_next_message(conn)
            self.assertTrue(unpacked_data["logged_in"])
            self.assertEqual(unpacked_data["client_id"], secret_key)

            # Game start check
            unpacked_data = recv_next_message(conn)
            self.assertTrue(unpacked_data['game_started'])
        except:
            conn.close()
            raise
        finally:
            conn.close()

    def tearDown(self):
        self.server.server_close()
        os.unlink(SOCK_FILE)

if __name__ == "__main__":
    unittest.main()
