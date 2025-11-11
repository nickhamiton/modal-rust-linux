fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a vendored protoc binary so users don't need to install protoc system-wide.
    let protoc = protoc_bin_vendored::protoc_bin_path()
        .map_err(|e| format!("failed to locate vendored protoc: {}", e))?;
    std::env::set_var("PROTOC", protoc);

    // Compile the Modal proto into Rust types using tonic/prost.
    // Reference the proto files from the modal-client-main directory
    let proto_dir = "modal-client-main/modal_proto";
    let api_proto = format!("{}/api.proto", proto_dir);
    
    // Check if proto directory exists, if not try alternative path
    let proto_path = std::path::Path::new(&proto_dir);
    if !proto_path.exists() {
        // Try relative path from workspace root
        let alt_proto_dir = "../modal-client-main/modal_proto";
        let alt_proto_path = std::path::Path::new(alt_proto_dir);
        if alt_proto_path.exists() {
            let alt_api_proto = format!("{}/api.proto", alt_proto_dir);
            tonic_build::configure()
                .build_server(false)
                .compile(&[&alt_api_proto], &[alt_proto_dir])?;
            return Ok(());
        } else {
            return Err(format!(
                "Proto directory not found. Tried: {} and {}",
                proto_dir, alt_proto_dir
            ).into());
        }
    }
    
    tonic_build::configure()
        .build_server(false)
        .compile(&[&api_proto], &[proto_dir])?;
    Ok(())
}

