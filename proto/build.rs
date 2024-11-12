fn main() {
    tonic_build::configure()
        .type_attribute("orderbook.OrderType", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.OrderStatus", "#[derive(strum::FromRepr)]")
        .type_attribute("orderbook.LimitType", "#[derive(strum::FromRepr)]")
        .build_server(true)
        .compile_protos(&["proto/orderbook.proto"], &["proto"])
        .expect("Failed to compile protos");
}
