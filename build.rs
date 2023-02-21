fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("astria-proto/execution/execution.proto")?;
    Ok(())
}
