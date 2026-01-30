fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/proto")
        .compile(
            &[
                "../protos/file_service.proto",
                "../protos/command_service.proto",
                "../protos/git_service.proto",
                "../protos/snapshot_service.proto",
                "../protos/system_service.proto",
                "../protos/policy_service.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}
