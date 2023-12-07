import argparse
import socket
import threading

HOST = "127.0.0.1"  # The server's hostname or IP address
PORT = 6480  # The port used by the server

HEADER_BYTE = 0x0F
TYPE_REGISTER = 0x01
TYPE_PUBLISH = 0x02
TYPE_SUBSCRIBE = 0x03
TYPE_UNSUBSCRIBE = 0x04
TYPE_QUERY = 0x05


class pubsub_client:
    def __init__(self, host: str, port: int) -> None:
        self.host = host
        self.port = port
        self.recv_thread = None
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    def parse_header(self, buf: bytes, s: socket.socket):
        topic = ""
        msg: bytes | str = ""
        if buf[0] == 0x0F and buf[7] == 0x00:
            if buf[1] == 0x00 and buf[2] == 0x01:
                pkt_type = int(buf[3])
                topic_len = int(buf[4])
                msg_len = int(buf[5]) << 8 | int(buf[6])
                # print(pkt_type, topic_len, msg_len)
                topic = s.recv(topic_len).decode()
                if pkt_type == TYPE_PUBLISH:
                    msg = s.recv(msg_len)
                    return topic, msg
        return (topic, msg)

    def recv(self, s: socket.socket):
        try:
            while True:
                x = s.recv(8)
                if x:
                    topic, msg = self.parse_header(x, s)
                    print(f"\ntpic: {topic}, msg: {msg}")
                else:
                    break
        except Exception as e:
            print("exception occured ", e)
        except KeyboardInterrupt:
            print("exitting")

    def get_packet(self, data: str | None, topic: None | str, type_: int = 0x02):
        length: bytes = bytes([0, 0])
        topic_len = 0.1
        topic_data = []
        msg_data = []
        if data:
            length = len(data).to_bytes(2, "big")
            msg_data = list(bytearray(data.encode()))
        if topic:
            topic_len = len(topic)
            topic_data = list(bytearray(topic.encode()))
        s = bytes(
            [HEADER_BYTE, 0x00, 0x01, type_, topic_len, length[0], length[1], 0x00]
            + topic_data
            + msg_data
        )
        return s

    def connect(self):
        self.sock.connect((HOST, PORT))

    def start_receving_thread(self):
        self.recv_thread = threading.Thread(target=self.recv, args=(self.sock,))
        self.recv_thread.start()

    def start_console(self):
        self.start_receving_thread()
        while True:
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
                data = self.get_packet(data, topic, type_=type_)
                self.sock.send(data)

    def publish(self, pkt: bytes):
        self.sock.send(pkt)
        response = self.sock.recv(8)
        if response:
            topic, _ = self.parse_header(response, self.sock)
            print(f"\ntpic: {topic}")


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
        "--consloe" "-c", type=bool, help="start the console for the client"
    )
    args = parser.parse_args()
    # print(args)
    cli = pubsub_client(HOST, PORT)
    print(cli.port, cli.host)
    cli.connect()
    if args.publish:
        if args.message:
            print(f"publishing to {args.publish} the message is: {args.message}")
            pkt = cli.get_packet(args.message, args.publish, TYPE_PUBLISH)
            cli.publish(pkt)
    if args.subscribe:
        pkt = cli.get_packet("", args.subscribe, TYPE_SUBSCRIBE)
        cli.publish(pkt)
        cli.start_receving_thread()
