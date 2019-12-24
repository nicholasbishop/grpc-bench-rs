use futures::Future;
use grpc_bench::{
    bench::{RandomBytesReply, RandomBytesRequest},
    bench_grpc::{create_bench, Bench},
    gen_random_bytes,
};
use grpcio::{
    ChannelBuilder, Environment, ResourceQuota, RpcContext, ServerBuilder,
    UnarySink,
};
use std::sync::Arc;

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
    let service = create_bench(BenchService {
        data: gen_random_bytes(),
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
    futures::future::empty::<(), ()>().wait().unwrap();
}
