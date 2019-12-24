use bench::{
    bench_server::{Bench, BenchServer},
    RandomBytesReply, RandomBytesRequest,
};
use grpc_bench::{gen_random_bytes, BoxError};
use tonic::{transport::Server, Request, Response, Status};

pub mod bench {
    tonic::include_proto!("bench");
}

pub struct BenchService {
    data: Vec<u8>,
}

#[tonic::async_trait]
impl Bench for BenchService {
    async fn get_random_bytes(
        &self,
        _request: Request<RandomBytesRequest>,
    ) -> Result<Response<RandomBytesReply>, Status> {
        let reply = RandomBytesReply {
            data: self.data.clone(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let addr = "[::1]:50051".parse()?;
    let data = gen_random_bytes();
    let bench = BenchService { data };

    Server::builder()
        .add_service(BenchServer::new(bench))
        .serve(addr)
        .await?;

    Ok(())
}
