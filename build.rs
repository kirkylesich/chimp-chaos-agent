fn main() {
    let protoc_path = protoc_bin_vendored::protoc_bin_path().expect("protoc");
    std::env::set_var("PROTOC", protoc_path);
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/agent.proto"], &["proto"]).expect("compile proto");
}

