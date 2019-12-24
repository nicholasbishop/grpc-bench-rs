fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/bench.proto")?;

    let proto_root = "proto";
    let output = "src";
    println!("cargo:rerun-if-changed={}", proto_root);
    protoc_grpcio::compile_grpc_protos(
        &["bench.proto"],
        &[proto_root],
        output,
        None,
    )?;

    Ok(())
}
