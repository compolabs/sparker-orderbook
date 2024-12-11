use sparker_core::Order;

#[derive(Debug, Clone)]
pub enum Event {
    OrderUpdate(Order),
}
