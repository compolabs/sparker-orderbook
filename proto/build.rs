use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .type_attribute("orderbook.types.OrderType", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.types.OrderStatus", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.types.LimitType", "#[derive(strum::FromRepr)]")
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("orderbook_descriptor.bin"))
        .compile_protos(&["proto/orderbook.proto", "proto/types.proto"], &["proto"])
        .expect("Failed to compile protos");
}
