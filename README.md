# Astria Conductor

Coordinates blocks between the data layer and the execution layer.

### Running for development

* create `ConductorConfig.toml` in the repo root and populate accordingly, e.g.

Note: I've been generating random namespace ids for development. [See how here](https://go.dev/play/p/7ltvaj8lhRl)

```
celestia_node_url = "http://localhost:26659"
namespace_id = "b860ccf0e97fdf6c"
rpc_address = "https://[::1]:50051"
```

* run `cargo run`

### Tests

To run the tests, you need to build and run [`sequencer-relayer`](https://github.com/astriaorg/sequencer-relayer.git) as well as a Celestia cluster and Metro.

Run [metro](https://github.com/astriaorg/metro.git):
```bash
git clone https://github.com/astriaorg/metro.git
cd metro
git checkout noot/msg-type
make install
bash scripts/single-node.sh
```

Run a Celestia cluster:
```bash
git clone https://github.com/astriaorg/sequencer-relayer.git
cd sequencer-relayer
docker compose -f docker/test-docker-compose.yml up -d bridge0
```

To run the relayer, inside `sequencer-relayer/`:
```bash
cargo build
./target/debug/relayer
```

Then, you can run the tests:
```bash
cargo test
```