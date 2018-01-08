'''
Testing suite for the engine
'''

import unittest
import socket
import os
import json
import subprocess
import signal
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

def send_message(conn, obj: object) -> None:
    '''
    Sends newline delimited message to socket
    The object desired to be sent will be converted to a json and then encoded
    and sent.

    Args:
        Obj: The object that wants to be serialized and sent over

    Returns:
        None
    '''

    send_socket = conn

    message = json.dumps(obj) + "\n"
    encoded_message = message.encode()

    wrapped_socket = send_socket.makefile('rwb', 1)
    try:
        wrapped_socket.write(encoded_message)
    except IOError:
        wrapped_socket.close()
        send_socket.close()
        raise IOError
    except KeyboardInterrupt:
        wrapped_socket.close()
        send_socket.close()
        raise KeyboardInterrupt
    finally:
        wrapped_socket.close()
    return

def recv_next_message(conn):
    '''
    Read the next message comming from this socket
    '''
    with conn.makefile('rb', 1) as wrapped_conn:
        data = next(wrapped_conn)
    data = data.decode("utf-8").strip()
    if data[0] != '{':
        data += '{' + data
    print("start: " + data)
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
        self.game = server.Game(num_players)
        dockers = {}
        try:
            os.unlink(SOCK_FILE)
        except OSError:
            if os.path.exists(SOCK_FILE):
                print("File exists at connection point")
                raise

        self.server = server.start_server(SOCK_FILE, self.game, dockers,
                                          use_docker=False)


    @unittest.skip("Skip because it has a thread that doesn't close well")
    def test_login(self):
        '''
        Check a single login works
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


    @unittest.skip("Skip because it has a thread that doesn't close well")
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

    @unittest.skip("Skip because it has a thread that doesn't close well")
    def test_already_logged_in(self):
        '''
        Check double logins don't work
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
            secret_key = self.game.player_ids[0]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertFalse(unpacked_data["logged_in"])
        self.assertNotEqual(unpacked_data["error"], "")

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
            secret_key = wrong_key
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertFalse(unpacked_data["logged_in"])
        self.assertNotEqual(unpacked_data["error"], "")

    def tearDown(self):
        self.server.server_close()
        os.unlink(SOCK_FILE)

class TestGameSocketServer(unittest.TestCase):
    '''
    Testing suite for everything after log in of the game
    '''

    def setUp(self):
        '''
        This function sets up up the unix stream server for all the other tests.
        It also logs both players in.
        '''
        num_players = 2
        self.game = server.Game(num_players, "")
        dockers = {}
        try:
            os.unlink(SOCK_FILE)
        except OSError:
            if os.path.exists(SOCK_FILE):
                print("File exists at connection point")
                raise

        self.server = server.start_server(SOCK_FILE, self.game, dockers,
                                          use_docker=False)

        self.conn_p1 = create_socket()
        login(self.conn_p1, self.game.player_ids[0])
        unpacked_data = recv_next_message(self.conn_p1)
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["error"], "")

        self.conn_p2 = create_socket()
        login(self.conn_p2, self.game.player_ids[1])
        unpacked_data = recv_next_message(self.conn_p2)
        self.assertTrue(unpacked_data["logged_in"])
        self.assertEqual(unpacked_data["error"], "")

    def test_game_starts(self):
        '''
        Check that the game starts. Aka the setup works correctly
        '''
        pass

    def test_game_correct_player(self):
        '''
        Check correct player order is used
        '''
        recv_next_message(self.conn_p1)
        send_message(self.conn_p1, {'client_id':self.game.player_ids[0],
                                    'moves': ''})

        recv_next_message(self.conn_p2)
        send_message(self.conn_p2, {'client_id':self.game.player_ids[1],
                                    'moves': ''})

        recv_next_message(self.conn_p1)
        send_message(self.conn_p1, {'client_id':self.game.player_ids[0],
                                    'moves': ''})



    def tearDown(self):
        self.server.server_close()
        self.conn_p1.close()
        self.conn_p2.close()
        os.unlink(SOCK_FILE)


class TestFourLoginSocketServer(unittest.TestCase):
    '''
    Testing suite to test the login function of the socket server.
    '''

    def setUp(self):
        '''
        This function setusp up the unix stream server for all the other tests.
        '''
        num_players = 4
        self.game = server.Game(num_players)
        dockers = {}
        try:
            os.unlink(SOCK_FILE)
        except OSError:
            if os.path.exists(SOCK_FILE):
                print("File exists at connection point")
                raise

        self.server = server.start_server(SOCK_FILE, self.game, dockers,
                                          use_docker=False)


    def test_four_login(self):
        '''
        Test that all four logins work
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

        conn = create_socket()
        try:
            secret_key = self.game.player_ids[2]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()

        conn = create_socket()
        try:
            secret_key = self.game.player_ids[3]
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()

    def test_failed_login(self):
        '''
        Test that a wrong login fails to work
        '''
        conn = create_socket()
        wrong_key = 0
        while wrong_key in self.game.player_ids:
            wrong_key += 1

        try:
            secret_key = wrong_key
            login(conn, secret_key)
            unpacked_data = recv_next_message(conn)
        except:
            conn.close()
            raise
        finally:
            conn.close()
        self.assertFalse(unpacked_data["logged_in"])
        self.assertNotEqual(unpacked_data["error"], "")

    def tearDown(self):
        self.server.server_close()
        os.unlink(SOCK_FILE)

class TestCli(unittest.TestCase):
    '''
    Tests for CLI make sure flags work and game is starting
    '''

    def test_no_docker(self):
        '''
        Test cli with no docker. Check it connects properly
        '''
        return
        process = subprocess.Popen(['python', './battlecode_cli.py', '-p1', './', '-p2',
                                    './'], stdout=subprocess.PIPE, shell=True)
        (stdout, _) = process.communicate()
        print(stdout)
        process.send_signal(signal.SIGINT)


if __name__ == "__main__":
    try:
        os.remove("./server.log")
    except OSError:
        pass
    unittest.main()
