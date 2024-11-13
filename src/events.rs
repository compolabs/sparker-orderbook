use sparker_core::Order;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    OrderUpdated(Order),
}

pub fn broadcast_channel() -> broadcast::Sender<Event> {
    broadcast::channel::<Event>(8).0
}
