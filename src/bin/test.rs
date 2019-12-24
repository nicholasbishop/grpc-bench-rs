use bench::{bench_client::BenchClient, RandomBytesReply, RandomBytesRequest};
use chrono::Utc;
use futures03::future::join_all;
use grpc_bench::BoxError;
use std::time::Duration;
use structopt::StructOpt;
use tokio::{process, time::delay_for};

mod bench {
    tonic::include_proto!("bench");
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

async fn send_request() -> Result<RandomBytesReply, BoxError> {
    let mut client = BenchClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(RandomBytesRequest {});
    let reply = client.get_random_bytes(request).await?.into_inner();
    Ok(reply)
}

async fn bench(num_requests: usize) -> Result<(), BoxError> {
    let start_time = Utc::now();

    let r =
        join_all((0..num_requests).map(|_| tokio::spawn(send_request()))).await;

    for r in r {
        r??;
    }

    let end_time = Utc::now();

    println!("{}", format_duration(end_time - start_time));

    Ok(())
}

async fn test_tonic(num_requests: usize) -> Result<(), BoxError> {
    println!("test_tonic");
    let _server = run_tonic_server()?;
    // Give the server time to start
    delay_for(Duration::from_secs(1)).await;

    bench(num_requests).await
}

async fn test_grpcio(num_requests: usize) -> Result<(), BoxError> {
    println!("test_grpcio");
    let _server = run_grpcio_server()?;
    // Give the server time to start
    delay_for(Duration::from_secs(1)).await;

    bench(num_requests).await
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let opt = Opt::from_args();

    test_tonic(opt.num_requests).await?;
    test_grpcio(opt.num_requests).await?;

    Ok(())
}
