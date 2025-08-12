use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/socket/protocol.proto"], &["src/socket"])?;
    Ok(())
}
