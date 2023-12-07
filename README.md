# Simple-pub-sub

A simple message broker implemented in rust.

The message frame looks like

| header (1 byte) | version (2 bytes) | pkt type (1 byte) | topic length (1 byte) | message length (2 bytes) | padding (1 byte) | topic | message |

So it's a 8 byte header followed by the topic and message.

