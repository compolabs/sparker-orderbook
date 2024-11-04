use ::entity::order::{self, Entity as OrderEntity};
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use crate::{
    error::Error,
    models::{CreateOrder, Order, OrderStatus, OrderType, UpdateOrder},
};

pub struct OrderRepository {
    conn: DatabaseConnection,
}

impl OrderRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn create_order(&self, data: CreateOrder) -> Result<(), Error> {
        let order = order::ActiveModel {
            tx_id: Set(data.tx_id),
            order_id: Set(data.order_id),
            order_type: Set(data.order_type),
            user: Set(data.user),
            asset: Set(data.asset),
            amount: Set(data.amount as i64),
            price: Set(data.price as i64),
            status: Set(data.status),
            timestamp: Set(data.timestamp),
            market_id: Set(data.market_id),
            ..Default::default()
        };
        let on_conflict = OnConflict::column(order::Column::OrderId)
            .do_nothing()
            .to_owned();
        OrderEntity::insert(order)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(&self.conn)
            .await?;

        Ok(())
    }

    pub async fn create_orders(&self, data: Vec<CreateOrder>) -> Result<(), Error> {
        let len = data.len();
        if len == 0 {
            return Ok(());
        }

        let orders = data
            .into_iter()
            .map(|order| order::ActiveModel {
                tx_id: Set(order.tx_id),
                order_id: Set(order.order_id),
                order_type: Set(order.order_type),
                user: Set(order.user),
                asset: Set(order.asset),
                amount: Set(order.amount as i64),
                price: Set(order.price as i64),
                status: Set(order.status),
                timestamp: Set(order.timestamp),
                market_id: Set(order.market_id),
                ..Default::default()
            })
            .collect::<Vec<order::ActiveModel>>();

        let on_conflict = OnConflict::column(order::Column::OrderId)
            .do_nothing()
            .to_owned();
        let res = OrderEntity::insert_many(orders)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(&self.conn)
            .await?;

        log::debug!("DB | CREATE_ORDERS: {} | {:?}", len, res);

        Ok(())
    }

    pub async fn update_order(&self, order_id: &str, data: UpdateOrder) -> Result<Order, Error> {
        let order = OrderEntity::find()
            .filter(order::Column::OrderId.eq(order_id))
            .one(&self.conn)
            .await?;
        let mut order: order::ActiveModel = order.unwrap().into();

        if let Some(amount) = data.amount {
            order.amount = Set(amount as i64);
        }
        order.status = Set(data.status);

        let order = OrderEntity::update(order).exec(&self.conn).await?;

        Ok(Order::from(order))
    }

    pub async fn best_bid(&self) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(is_active_condition())
            .order_by_desc(order::Column::Price)
            .one(&self.conn)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn best_ask(&self) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(is_active_condition())
            .order_by_asc(order::Column::Price)
            .one(&self.conn)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn order_by_id(&self, order_id: &str) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(order::Column::OrderId.eq(order_id))
            .one(&self.conn)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn orders(&self, limit: u64, offset: u64) -> Result<Vec<Order>, Error> {
        let orders = OrderEntity::find()
            .filter(is_active_condition())
            .order_by_desc(order::Column::Timestamp)
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await?;
        let orders = orders.into_iter().map(Order::from).collect();

        Ok(orders)
    }

    pub async fn orders_by_type(
        &self,
        order_type: OrderType,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Order>, Error> {
        let select = OrderEntity::find().filter(
            Condition::all()
                .add(is_active_condition())
                .add(order::Column::OrderType.eq(order_type.clone())),
        );
        let orders = match order_type {
            OrderType::Buy => select.order_by_desc(order::Column::Price),
            OrderType::Sell => select.order_by_asc(order::Column::Price),
        }
        .offset(offset)
        .limit(limit)
        .all(&self.conn)
        .await?;
        let orders = orders.into_iter().map(Order::from).collect();

        Ok(orders)
    }
}

fn is_active_condition() -> Condition {
    Condition::any()
        .add(order::Column::Status.eq(OrderStatus::New))
        .add(order::Column::Status.eq(OrderStatus::PartiallyMatched))
}
