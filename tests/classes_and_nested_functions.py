import socket
from typing import Callable

class Server:
    def __init__(self, ip, port) -> None:
        self._socket = socket.socket()

        # for debug purposes on linux [OsError 98] address is already in use
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._socket.bind((ip, port))
        self._socket.listen()
        self._endpoints: dict[str, Callable] = dict()

    def run(self):
        while True:
            client, addr = self._socket.accept()
            print("Client connected from:", addr)
            self._handle_client(client)

    def _extract_request(self, client: socket.socket) -> str:

        # very not gpt code
        request_data = b'' 
        while True:
            chunk = client.recv(4096)  

            if not chunk:
                break  
            
            request_data += chunk 
            
            if b'\r\n\r\n' in request_data:
                break

        return request_data.decode()

    def _handle_client(self, client: socket.socket):
        request = self._extract_request(client)
        endpoint = request.split()[1]
        res = self._endpoints[endpoint]()
        client.send(res.encode())
        client.close()

    def route(self, ep: str):
        def inner(method: Callable):
            if ep in self._endpoints:
                raise ValueError
            self._endpoints[ep] = method
            return method
        return inner

server = Server('0.0.0.0', 1337)


@server.route('/')
def root():
    return "Hello, World"


server.run()