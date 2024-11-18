use sparker_core::{Order, Trade};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    OrderUpdated(Order),
    Traded(Trade),
}

pub fn broadcast_channel() -> broadcast::Sender<Event> {
    broadcast::channel::<Event>(8).0
}
