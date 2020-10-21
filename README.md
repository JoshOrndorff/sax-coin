# Sax Coin

A Substrate-based blockchain node for Saxons. This Blockchain is an example and experiment for my presentation at the [Blockchain Meetup Saxony](https://www.meetup.com/de-DE/BlockchainMeetupSaxony/events/269153470/). The chain is Proof of Work based with simple onchain governance. During the meetup we will launch the network, transfer coins, mine coins, and explore the nature of on-chain governance.

Substrate Version: `2.0.0-alpha.6`. For educational purposes only.

## Using the Network

As an end-user of the network, the easiest way to begin is by launching the [hosted user-interface](https://polkadot.js.org/apps?rpc=wss://saxony.bootnodes.net/node).


## Getting the Node

### Download the Docker Image

```
docker pull joshyorndorff/saxcoin:alpha3
```

### Build it Yourself
You can also build the node yourself. This is the most well-trodden path, but has some prerequisites and takes some disk space.

```bash
# Install Rust
curl https://sh.rustup.rs -sSf | sh

# Initialize rust toolchain
./scripts/init.sh

# Build release node for your platform
cargo build --release
```

## Running a Node

Once you have your node, you can join the live network.

As a full node:
```
# Using the Docker Image
docker run joshyorndorff/saxcoin:alpha3 --name YOUR-NODE-NAME

# Using a Local Binary
./target/release/sax-coin --name YOUR-NODE-NAME
```

As a mining node:
```
# Using the Docker Image
docker run joshyorndorff/saxcoin:alpha3 --validator --name YOUR-NODE-NAME

# Using a Local Binary
./target/release/sax-coin --validator --name YOUR-NODE-NAME
```

There are not yet block rewards issued to miners.

### The UI

Once you have your own node running, you can connect the [user interface](https://polkadot.js.org/apps#settings) to your own node rather than the fairly centralized bootnode. On the UI Setting tab, choose the node you wish to connect to.

## Longevity

There are no guarantees that this network will live much beyond the meetup. Although that really is up to the participants. Let's go Cleveland!
