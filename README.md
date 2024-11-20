## For Simplicity

- Only ED25519 key type is supported
- Uses IPv4 & TCP for connection.

## Requirements

- Docker: version 20.10 or later
- Docker Compose: v2

## Steps

#### 1. Initiate config and files

```sh
./cmd.sh init
```

It will create

- `./.data/ipfs`: for persistent storage of ipfs config
- `./.data/rust`: for caching the libraries on rust build,
- `./.env`: for passing $UID down to the containers
- `./ed25519.pem`: the key for handshake code

#### 2. Bring up the target node

```sh
./cmd.sh target-node-up
```

#### 3. Run the handshake code

```sh
# default to local mode
./cmd.sh handshake
# is equivalent to
./cmd.sh handshake local

# connect as remote mode
./cmd.sh handshake remote
```

This will call the [handshake code](./handshake/src/main.rs) using `./ed25519.pem` as the private key.
It connects to `/ip4/127.0.0.1/tcp/4001` or `/dns4/ipfs/tcp/4001` depends on the mode.
A file `./peerid` will be generated for verifying the handshake easier.

#### 4. Verify the connection on IPFS

**!! OPEN A NEW TERMINAL !!**

```sh
# use
./cmd.sh target-node-peer
# or
./cmd.sh target-node-log

# or both (each on a new terminal)
./cmd.sh target-node-peer
./cmd.sh target-node-log
```

`target-node-peer` uses IPFS command to show connected peers, and filter by the content in `./peerid` that generated in Step 3.
`target-node-log` shows logs of IPFS node and filter by the `./peerid`.
They show different information so recommend to use both commands to verify.

#### 5. Call handshake repeatedly

Repeat running the command in Step 3 and stopping it by `Ctrl-C`. Monitor the output on the terminals opened in Step 4.

You can also modify the `RUST_LOG` to `debug` level in `./.env` to see more information for the handshake code.
