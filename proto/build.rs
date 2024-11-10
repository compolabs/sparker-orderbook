fn main() {
    tonic_build::configure()
        .type_attribute("order.OrderType", "#[derive(strum::FromRepr)]")
        .type_attribute("order.OrderStatus", "#[derive(strum::FromRepr)]")
        .type_attribute("order.LimitType", "#[derive(strum::FromRepr)]")
        .build_server(true)
        .compile_protos(&["proto/order.proto"], &["proto"])
        .expect("Failed to compile protos");
}
