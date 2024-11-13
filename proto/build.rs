fn main() {
    tonic_build::configure()
        .type_attribute("orderbook.types.OrderType", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.types.OrderStatus", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.types.LimitType", "#[derive(strum::FromRepr)]")
        .build_server(true)
        .compile_protos(&["proto/orderbook.proto", "proto/types.proto"], &["proto"])
        .expect("Failed to compile protos");
}
