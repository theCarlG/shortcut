fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/wifi.proto")?;
    tonic_build::compile_protos("proto/ssh.proto")?;
    Ok(())
}
