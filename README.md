# grpc-bench

Very simple benchmark of [tonic](https://github.com/hyperium/tonic)
and [grpc-rs (aka grpcio)](https://github.com/tikv/grpc-rs).

## Usage

    cargo build --release && cargo run --release --bin test <num-requests>
    
## Implementation

A single grpc service is implemented, with a single method. This
method takes an empty request message and responds with a large
message containing 2 MiB of random data. (This emulates a case I
happen to care about, where a grpc server is acting in part as a file
cache.) The random data is generated in advance so it is the same for
all calls to the server; the intent here is to have the server method
not take any significant time other than having to send 2 MiB of data.

The test is run four times with all the combinations of grpcio and
tonic for client and server.

## Results

```
cargo run --release --bin test 1000

test Grpcio server with Grpcio client
1.797s

test Grpcio server with Tonic client
3.327s

test Tonic server with Grpcio client
7.246s

test Tonic server with Tonic client
6.278s
```

Perhaps also of interest is that with higher values of
`<num-requests>`, e.g. 2000, tonic doesn't work.
