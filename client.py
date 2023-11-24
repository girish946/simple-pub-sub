
import socket
import time

HOST = "127.0.0.1"  # The server's hostname or IP address
PORT = 6480# The port used by the server

HEADER_BYTE = 0x0F;
TYPE_REGISTER = 0x01;
TYPE_PUBLISH  = 0x02;
TYPE_SUBSCRIBE = 0x03;
TYPE_UNSUBSCRIBE = 0x04;
TYPE_QUERY = 0x05;


def get_packet(data, topic):
    length = len(data).to_bytes(2, "big")
    s = bytearray(
        [HEADER_BYTE, 0x00, 0x01, TYPE_PUBLISH, length[0], length[1], 
         len(topic), 0x00, 0x00 ] +
            list(bytearray(topic.encode()))+
            list(bytearray(data.encode()))
    ) #f"{HEADER_BYTE}-{TYPE_PUBLISH}-{len(data)}:{data}"
    print(s)
    return s

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    while True:
        data = input(": ")
        if data:
            # data = data
            topic_and_msg = data.split(":", 1)
            if len(topic_and_msg) > 1:
                print(topic_and_msg)
                data = get_packet(topic_and_msg[1], topic_and_msg[0])
            else:
                data  = get_packet(data, "test")
            print(data)
            n = s.send(data)
            print(n, "bytes sent")
            data = s.recv(len(data[8:]))
            

            print(f"Received {data!r}")
