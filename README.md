# Astria Conductor

Coordinates blocks between the data layer and the execution layer.

### Running for development

* create `ConductorConfig.toml` in the repo root and populate accordingly, e.g.

```
celestia_node_url = "http://localhost:26659"
namespace_id = "b860ccf0e97fdf6c"
rpc_address = "https://[::1]:50051"
```

Note: I've been generating random namespace ids for development. [See how here](https://go.dev/play/p/7ltvaj8lhRl)

* run `cargo run`

### Protos and Buf Build

[Buf Build](https://buf.build/) is a platform and registry for sharing Protocol Buffers between team members. It also comes with a set of tools to generate gRPC servers and clients in a range of languages.

[Astria's Buf Build organization](https://buf.build/astria)

First, install Buf CLI and authenticate yourself:

* `$ brew install bufbuild/buf/buf` - using homebrew
    * [other ways to install](https://docs.buf.build/installation)
* `$ buf registry login` - [must first create an API token](https://docs.buf.build/tutorials/getting-started-with-bsr#create-an-api-token)

#### Building and pushing after making changes in `astria-rpc/proto`

* `$ cd astria-rpc` - must be in same directory as `buf.yaml`
* `$ buf build` - [builds the proto files into a single binary file](https://docs.buf.build/build/explanation#what-are-buf-images)
* `$ buf push` - pushes a module to the registry

#### Generating clients and servers

* `$ cd astria-rpc` - must be in same directory as `buf.gen.yaml`
* `$ buf generate` - generate clients and servers according to the configuration in `buf.gen.yaml`
