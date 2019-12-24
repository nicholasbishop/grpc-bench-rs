mod bench;
mod bench_grpc;
use grpc_bench::gen_random_bytes;
use std::io::Read;
use std::sync::Arc;
use std::{io, thread};

use futures::sync::oneshot;
use futures::Future;
use grpcio::{
    ChannelBuilder, Environment, ResourceQuota, RpcContext, ServerBuilder,
    UnarySink,
};

use bench::{RandomBytesReply, RandomBytesRequest};
use bench_grpc::{create_bench, Bench};

#[derive(Clone)]
struct BenchService {
    data: Vec<u8>,
}

impl Bench for BenchService {
    fn get_random_bytes(
        &mut self,
        ctx: RpcContext<'_>,
        req: RandomBytesRequest,
        sink: UnarySink<RandomBytesReply>,
    ) {
        let resp = RandomBytesReply {
            data: self.data.clone(),
            ..Default::default()
        };
        let f = sink
            .success(resp)
            .map_err(move |e| eprintln!("failed to reply {:?}: {:?}", req, e));
        ctx.spawn(f)
    }
}

fn main() {
    let env = Arc::new(Environment::new(1));
    let num_bytes = 64 * 1024 * 1024; // 64MiB
    let service = create_bench(BenchService {
        data: gen_random_bytes(num_bytes),
    });

    let quota = ResourceQuota::new(Some("BenchServerQuota"))
        .resize_memory(1024 * 1024 * 1024);
    let ch_builder = ChannelBuilder::new(env.clone()).set_resource_quota(quota);

    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("[::1]", 50_051)
        .channel_args(ch_builder.build_args())
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
}
