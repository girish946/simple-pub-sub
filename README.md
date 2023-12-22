# Simple-pub-sub

A simple message broker implemented in rust.

The message frame looks like

|header|version|pkt type|topic length|message length|padding|topic|message|
|------|-------|--------|------------|--------------|-------|-----|-------|
|1 byte|2 bytes|1 byte|1 byte|2 bytes|1 byte|.....|.....|

So it's a 8 byte header followed by the topic and message.

## Cli Usage

- Server:
```bash
simple-pub-sub server 0.0.0.0 6480 --log-level trace
```

- Client:
    - subscribe:
    
    ```bash
    simple-pub-sub client 0.0.0.0 6480 subscribe the_topic --log-level trace
    ```
    
    - publish:

    ```bash
    simple-pub-sub client 0.0.0.0 6480 publish the_topic the_message --log-level info
    ```
    
    - query:

    ```bash
    simple-pub-sub client 0.0.0.0 6480 query the_topic --log-level trace
    ```
