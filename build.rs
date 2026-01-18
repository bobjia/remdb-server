use std::env;
use std::path::Path;

fn main() {
    // 定义proto文件目录
    let proto_dir = Path::new("proto");

    // 定义输出目录
    let out_dir = env::var("OUT_DIR").unwrap();

    // 收集所有proto文件
    let proto_files = vec![proto_dir.join("jdbc.proto")];

    // 编译protobuf文件
    prost_build::Config::new()
        .out_dir(out_dir)
        .compile_protos(&proto_files, &[proto_dir])
        .expect("Failed to compile protobuf files");

    // 告诉Cargo重新构建的条件
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=build.rs");
}
