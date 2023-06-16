fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "./proto/omnipaxos.proto";
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&[proto_file], &["proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
    Ok(())
}
