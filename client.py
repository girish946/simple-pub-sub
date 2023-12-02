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
        self.sock = None
        self.recv_thread = None

    def parse_header(self, buf: bytes, s: socket.socket):
        topic = ""
        msg: bytes|str = ""
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
                    print(": ")
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
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((HOST, PORT))
        self.recv_thread = threading.Thread(target=self.recv, args=(self.sock,))
        self.recv_thread.start()

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
                data = self.get_packet(data, topic, type_=type_)
                self.sock.send(data)


if __name__ == "__main__":
    cli = pubsub_client(HOST, PORT)
    print(cli.port, cli.host)
    cli.connect()
