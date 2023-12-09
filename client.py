import argparse
import socket
import threading
from typing import Callable
from typing import Tuple

HOST = "127.0.0.1"  # The server's hostname or IP address
PORT = 6480  # The port used by the server

"""
REGISTER: u8 = 0x01;
PUBLISH: u8 = 0x02;
SUBSCRIBE: u8 = 0x03;
UNSUBSCRIBE: u8 = 0x04;
QUERY: u8 = 0x05;
REGISTERACK: u8 = 0x0A;
PUBLISHACK: u8 = 0x0B;
SUBSCRIBEACK: u8 = 0x0C;
UNSUBSCRIBEACK: u8 = 0x0D;
QUERYRESP: u8 = 0x0E;
"""

HEADER_BYTE = 0x0F
TYPE_REGISTER = 0x01
TYPE_PUBLISH = 0x02
TYPE_SUBSCRIBE = 0x03
TYPE_UNSUBSCRIBE = 0x04
TYPE_QUERY = 0x05

PUBSUB_VERSION = [0x00, 0x01]

SUPPORTED_PUBSUB_VERSIONS = [PUBSUB_VERSION]


class Header:
    HEADER_BYTE = 0x0F
    PADDING = 0x00
    REGISTER = 0x01
    PUBLISH = 0x02
    SUBSCRIBE = 0x03
    UNSUBSCRIBE = 0x04
    QUERY = 0x05
    REGISTERACK = 0x0A
    PUBLISHACK = 0x0B
    SUBSCRIBEACK = 0x0C
    UNSUBSCRIBEACK = 0x0D
    QUERYRESP = 0x0E

    type_dict = {
        REGISTER: "REGISTER",
        PUBLISH: "PUBLISH",
        SUBSCRIBE: "SUBSCRIBE",
        UNSUBSCRIBE: "UNSUBSCRIBE",
        QUERY: "QUERY",
        REGISTERACK: "REGISTERACK",
        PUBLISHACK: "PUBLISHACK",
        SUBSCRIBEACK: "SUBSCRIBEACK",
        UNSUBSCRIBEACK: "UNSUBSCRIBEACK",
        QUERYRESP: "QUERYRESP",
    }

    def __init__(
        self,
        pkt_type: int = PUBLISH,
        topic_length: int = 0,
        message_length: int = 0,
        bytes: bytes = bytes([]),
    ):
        if bytes:
            if not (bytes[0] == Header.HEADER_BYTE and bytes[7] == Header.PADDING):
                raise Exception("invalid paket Header or Padding")
            if not [bytes[1], bytes[2]] in SUPPORTED_PUBSUB_VERSIONS:
                raise Exception("unsupported pubsub version")
            if not bytes[3] in Header.type_dict:
                raise Exception("Invalid Packet type")
            self.pkt_type = bytes[3]
            self.topic_length = bytes[4]
            self.message_length = [bytes[5], bytes[6]]
        else:
            if self.validate(pkt_type):
                self.pkt_type = pkt_type
                self.topic_length = topic_length
                self.message_length = message_length.to_bytes(2, "big")

            else:
                raise Exception("Invalid Packet type")

    def bytes(self) -> list:
        return [
            HEADER_BYTE,
            PUBSUB_VERSION[0],
            PUBSUB_VERSION[1],
            self.pkt_type,
            self.topic_length,
            self.message_length[0],
            self.message_length[1],
            self.PADDING,
        ]

    def validate(self, pkt_type) -> bool:
        if pkt_type in Header.type_dict:
            return True
        else:
            return False

    def length(self):
        return int(self.message_length[0]) << 8 | int(self.message_length[1])

    def __repr__(self):
        msg_len = self.length()
        s = f"Header: {HEADER_BYTE}, VERSION: {PUBSUB_VERSION[0]}.{PUBSUB_VERSION[1]}, "
        s += f"Pkt Type: {self.type_dict[self.pkt_type]}, Topic Length: {self.topic_length}, "
        s += f"Message Length: {msg_len}"


class Pkt:
    def __init__(
        self,
        pkt_type: int = Header.PUBLISH,
        topic: bytes = bytearray([]),
        message: bytes = bytearray([]),
    ):
        self.topic: bytes = topic  # .decode("utf-8")
        self.message: bytes = message
        self.header = Header(
            pkt_type=pkt_type, topic_length=len(topic), message_length=len(message)
        )

    def bytes(self) -> bytes:
        msg_bytes = list(self.message)
        topic_bytes = list(self.topic)
        return bytes(self.header.bytes() + topic_bytes + msg_bytes)

    def __repr__(self):
        return f"Header: {self.header}, topic: {self.topic}, message: {self.message}"


class pubsub_client:
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.recv_thread = threading.Thread(target=self.recv, args=(self.sock,))
        self.should_stop = False

    def recv_pkt(self, buf: bytes, s: socket.socket) -> Pkt:
        topic = ""
        msg: bytes = bytes()
        header = Header(bytes=buf)
        topic = s.recv(header.topic_length)
        if header.pkt_type == TYPE_PUBLISH:
            msg = s.recv(header.length())
        pkt = Pkt(pkt_type=header.pkt_type, topic=bytes(topic), message=msg)
        return pkt


    def recv(self, s: socket.socket, callback: Callable[[str, bytes], None]) -> None:
        try:
            while True:
                x = s.recv(8)
                if x:
                    pkt = self.recv_pkt(x, s)
                    callback(pkt.topic.decode("utf-8"), pkt.message)
                else:
                    break
                if self.should_stop:
                    break
        except Exception as e:
            print("exception occured ", e)


    def connect(self) -> None:
        self.sock.connect((HOST, PORT))

    def start_receving_thread(self, callback: Callable[[str, bytes], None]) -> None:
        self.recv_thread = threading.Thread(
            target=self.recv, args=(self.sock, callback)
        )
        self.recv_thread.start()

    def start_console(self) -> None:
        def recv_callback(topic, msg):
            print(f"topic: {topic} msg: {msg}")

        self.start_receving_thread(recv_callback)
        while True:
            try:
                type_ = 0x02
                topic = "test"
                data = input(": ")
                if data:
                    topic_and_msg = data.split(":", 2)
                    if len(topic_and_msg) >= 2:
                        topic = topic_and_msg[1]
                        data = topic_and_msg[2]
                        type_ = topic_and_msg[0]
                    else:
                        topic = "test"
                        data = topic_and_msg[0]
                    if type_ == "pub":
                        type_ = 0x02
                    elif type_ == "sub":
                        type_ = 0x03
                    if type_ == 0x03:
                        data = ""
                    if type_ == "usub":
                        type_ = TYPE_UNSUBSCRIBE
                    data = Pkt(
                        pkt_type=type_,
                        topic=bytearray(topic.encode()),
                        message=bytearray(data.encode()),
                    ).bytes()
                    self.sock.send(data)
            except KeyboardInterrupt:
                print("exitting")
                break

    def publish(self, topic: str, message: bytes) -> None:
        pkt = Pkt(
            pkt_type=TYPE_PUBLISH,
            topic=bytes(list(bytearray(topic.encode()))),
            message=message,
        ).bytes()
        self.sock.send(pkt)
        response = self.sock.recv(8)
        if response:
            pkt = self.recv_pkt(response, self.sock)
            print(f"topic: {pkt.topic}")

    def subscribe(self, topic: str) -> None:
        def recv_callback(topic, msg):
            print(f"topic: {topic} msg: {msg}")

        self.start_receving_thread(recv_callback)
        pkt = Pkt(
            pkt_type=TYPE_SUBSCRIBE,
            topic=bytearray(topic.encode()),
            message=bytearray([]),
        ).bytes()
        self.sock.send(pkt)
        self.recv_thread.join()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="client for `simple-pub-sub` implemented in python."
    )
    parser.add_argument("--publish", "-p", type=str, help="publish to the given topic")
    parser.add_argument(
        "--subscribe", "-s", type=str, help="subscribe to the given topic"
    )
    parser.add_argument("--message", "-m", type=str, help="message to be published")
    parser.add_argument(
        "--consloe", "-c", type=bool, help="start the console for the client"
    )
    args = parser.parse_args()

    cli = pubsub_client(HOST, PORT)
    print(cli.port, cli.host)
    cli.connect()
    if args.publish:
        if args.message:
            print(f"publishing to {args.publish} the message is: {args.message}")
            cli.publish(args.publish, bytes(list(bytearray(args.message.encode()))))
    if args.subscribe:
        print("subscribing to: ", args.subscribe)
        cli.subscribe(args.subscribe)
    if args.consloe:
        cli.start_console()
