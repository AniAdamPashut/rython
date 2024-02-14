import socket
from typing import Callable

def endpoint(ep: str):
    def inner(method):
        setattr(method, "registered", True)
        setattr(method, "endpoint", ep)
        return method
    
    return inner

class Server:
    def __init__(self, ip, port) -> None:
        self._socket = socket.socket()
        self._socket.bind((ip, port))
        self._socket.listen()

        self._endpoints: dict[str, Callable] = dict()
        for item in self.__dir__():
            if callable(item) and getattr(item, 'registered', False):
                self._endpoints[getattr(item, 'endpoint')] = item

    def start(self):
        while True:
            client, addr = self._socket.accept()
            print("Client connected from:", addr)
            self._handle_client(client)

    def _handle_client(self, client):
        request = self._extract_request(client)
        endpoint = request.split()[1]
        self._endpoints[endpoint](request)

    @endpoint('/')
    def _root(self, req):
        return "Hello, World!"
