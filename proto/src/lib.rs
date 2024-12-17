pub mod api {
    tonic::include_proto!("orderbook.api");
}

pub mod types {
    tonic::include_proto!("orderbook.types");
}

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("orderbook_descriptor");
