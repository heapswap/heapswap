# Subfield

Subfield is at its core a distribuited hash table, but it works differently from other implementations, like Libp2p's Kad.

## How it works

There are two types of requests:
- Server Requests are 1:1 between servers
- Client Requests are handed off between servers until it finds the closest server, where it gets handled.

```
[client]        [server]          [server]
                     <-[server_req]->  <-[... 
   <-[client_req]->--<-[client_req]->--<-[...
```

## Server messages
- Chord
	-
	
## Client messages
- Record
	- Open(Key) - open a record and listen for updates
	- Write(Key, Record) - publish a record
	- Close(Key) - close a record