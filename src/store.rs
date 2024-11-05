use std::collections::HashMap;
use sparker_core::{Order, Trade, UpdateOrder};

pub struct Store {
    orders: HashMap<String, Order>,
    trades: HashMap<String, Trade>,
}

impl Store {
    pub fn new(orders: Vec<Order>) -> Self {
        let orders: HashMap<String, Order> = orders
            .into_iter()
            .map(|order| (order.order_id.clone(), order))
            .collect();

        Self {
            orders,
            trades: HashMap::new(),
        }
    }

    pub fn order(&self, order_id: &str) -> Option<&Order> {
        self.orders.get(order_id)
    }

    pub fn trade(&self, trade_id: &str) -> Option<&Trade> {
        self.trades.get(trade_id)
    }

    pub fn insert_order(&mut self, order: Order) {
        self.orders.insert(order.order_id.clone(), order);
    }

    pub fn update_order(
        &mut self,
        UpdateOrder {
            order_id,
            status,
            amount,
        }: UpdateOrder,
    ) {
        let order = self.orders.get(&order_id).unwrap();
        let amount = amount.unwrap_or(order.amount);
        self.orders.insert(
            order_id,
            Order {
                status,
                amount,
                ..order.clone()
            },
        );
    }

    pub fn insert_trade(&mut self, trade: Trade) {
        self.trades.insert(trade.trade_id.clone(), trade);
    }
}
