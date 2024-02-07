import socket

server = socket.socket()
server.bind(("0.0.0.0", 1337))
server.listen()

with open("filename.txt") as f:
    f.read()