
import socket
import time
import threading
import sys

HOST = "127.0.0.1"  # The server's hostname or IP address
PORT = 6480# The port used by the server

HEADER_BYTE = 0x0F;
TYPE_REGISTER = 0x01;
TYPE_PUBLISH  = 0x02;
TYPE_SUBSCRIBE = 0x03;
TYPE_UNSUBSCRIBE = 0x04;
TYPE_QUERY = 0x05;


def get_packet(data, topic=None, type_=0x02):
    length = len(data).to_bytes(2, "big")
    print(data, topic, type_)
    s = bytearray(
        [HEADER_BYTE, 0x00, 0x01, type_, len(topic),length[0], length[1], 
        0x00 ] +
            list(bytearray(topic.encode()))+
            list(bytearray(data.encode()))
    ) #f"{HEADER_BYTE}-{TYPE_PUBLISH}-{len(data)}:{data}"
    print(s)
    return s

def recv(s):
    while True:
        x = s.recv(1)
        if x:
            sys.stdout.write(str(x))

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    t = threading.Thread(target=recv, args=(s,))
    t.start()
    while True:
        type_ =0x02
        topic = "test"
        data = input(": ")
        if data:
            # data = data
            topic_and_msg = data.split(":", 2)
            if len(topic_and_msg) >= 2:
                print(topic_and_msg)
                topic= topic_and_msg[1]
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
            data  = get_packet(data*2049, topic, type_=type_)
            print(data)
            n = s.send(data)
            print(n, "bytes sent")
