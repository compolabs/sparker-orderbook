use std::sync::Arc;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub type Sender<T> = Arc<UnboundedSender<T>>;
pub type Receiver<T> = Arc<Mutex<UnboundedReceiver<T>>>;
