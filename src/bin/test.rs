use bench::{
    bench_client::BenchClient as BenchClientTonic,
    RandomBytesRequest as RandomBytesRequestTonic,
};
use chrono::Utc;
use futures03::future::join_all;
use grpc_bench::{
    bench::RandomBytesRequest as RandomBytesRequestGrpcio,
    bench_grpc::BenchClient as BenchClientGrpcio, BoxError,
};
use grpcio::{ChannelBuilder, EnvBuilder};
use std::{sync::Arc, time::Duration};
use structopt::StructOpt;
use tokio::{process, time::delay_for};

mod bench {
    tonic::include_proto!("bench");
}

#[derive(Clone, Copy, Debug)]
enum Lib {
    Grpcio,
    Tonic,
}

fn run_tonic_server() -> Result<process::Child, BoxError> {
    let child = process::Command::new("target/release/tonic_server")
        .kill_on_drop(true)
        .spawn()?;
    Ok(child)
}

fn run_grpcio_server() -> Result<process::Child, BoxError> {
    let child = process::Command::new("target/release/grpcio_server")
        .kill_on_drop(true)
        .spawn()?;
    Ok(child)
}

fn run_server(lib: Lib) -> Result<process::Child, BoxError> {
    match lib {
        Lib::Grpcio => run_grpcio_server(),
        Lib::Tonic => run_tonic_server(),
    }
}

pub fn format_duration(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();
    format!(
        "{}.{:03}s",
        seconds,
        duration.num_milliseconds() - (seconds * 1000)
    )
}

#[derive(Debug, StructOpt)]
struct Opt {
    num_requests: usize,
}

async fn send_request_grpcio() -> Result<(), BoxError> {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("[::1]:50051");
    let client = BenchClientGrpcio::new(ch);

    let req = RandomBytesRequestGrpcio::default();
    let _reply = client.get_random_bytes(&req)?;
    Ok(())
}

async fn send_request_tonic() -> Result<(), BoxError> {
    let mut client = BenchClientTonic::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(RandomBytesRequestTonic {});
    let _reply = client.get_random_bytes(request).await?.into_inner();
    Ok(())
}

async fn send_request(lib: Lib) -> Result<(), BoxError> {
    match lib {
        Lib::Grpcio => send_request_grpcio().await,
        Lib::Tonic => send_request_tonic().await,
    }
}

async fn run_test(
    server_lib: Lib,
    client_lib: Lib,
    num_requests: usize,
) -> Result<(), BoxError> {
    println!("test {:?} server with {:?} client", server_lib, client_lib);

    let _server = run_server(server_lib)?;
    // Give the server time to start
    delay_for(Duration::from_secs(1)).await;

    let start_time = Utc::now();

    let r = join_all(
        (0..num_requests).map(|_| tokio::spawn(send_request(client_lib))),
    )
    .await;

    for r in r {
        r??;
    }

    let end_time = Utc::now();

    println!("{}\n", format_duration(end_time - start_time));

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let opt = Opt::from_args();
    let num_requests = opt.num_requests;

    // Test with grpcio server
    run_test(Lib::Grpcio, Lib::Grpcio, num_requests).await?;
    run_test(Lib::Grpcio, Lib::Tonic, num_requests).await?;

    // Test with tonic server
    run_test(Lib::Tonic, Lib::Grpcio, num_requests).await?;
    run_test(Lib::Tonic, Lib::Tonic, num_requests).await?;

    Ok(())
}
