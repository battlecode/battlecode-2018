#!/bin/python3
"""
Simple Client to test out speed of server connection
"""
import socket
import argparse
import time


def highest_prime_factor(nums):
    '''
    find largest prime factor of number
    '''
    if isprime(nums):
        return nums
    for test_int in range(2, int(nums ** 0.5) + 1):
        if not nums % test_int:
            return_val = highest_prime_factor(nums/test_int)
            return return_val
    return 0

def isprime(nums):
    '''
    Calculate if number is prime
    '''
    for test_int in range(2, int(nums ** 0.5) + 1):
        if not nums % test_int:
            return False
    return True


def main():
    parser = argparse.ArgumentParser(description="Battlecode client.")

    parser.add_argument('--socket-file', help='file used as socket connection'+ \
            ' for players', dest='sock_file', default='/tmp/battlecode-socket')

    args = parser.parse_args()
    print(args.sock_file)
    conn = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    conn.connect(args.sock_file)
    conn.recv(2)
    time1 = time.perf_counter()
    highest_prime_factor(123456789101112131416171820212223)
    time2 = time.perf_counter()
    bytes_send = str(time1) + " " + str(time2) + "\n"
    conn.sendall(bytes_send.encode())

if __name__ == "__main__":
    main()
