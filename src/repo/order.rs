use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use sparker_core::{InsertOrder, Order, OrderType, UpdateOrder};
use sparker_entity::{
    order::{self, Entity as OrderEntity},
    sea_orm_active_enums::{OrderStatus as OrderStatusSea, OrderType as OrderTypeSea},
};

use crate::error::Error;

pub struct Query;
impl Query {
    pub async fn find_best_bid(
        db: &DatabaseConnection,
        user_ne: Option<String>,
    ) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(find_condition(OrderTypeSea::Buy, user_ne))
            .order_by_desc(order::Column::Price)
            .one(db)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn find_best_ask(
        db: &DatabaseConnection,
        user_ne: Option<String>,
    ) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(find_condition(OrderTypeSea::Sell, user_ne))
            .order_by_asc(order::Column::Price)
            .one(db)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        order_id: &str,
    ) -> Result<Option<Order>, Error> {
        let order = OrderEntity::find()
            .filter(order::Column::OrderId.eq(order_id))
            .one(db)
            .await?;

        let order = order.map(Order::from);

        Ok(order)
    }

    pub async fn find(
        db: &DatabaseConnection,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Order>, Error> {
        let orders = OrderEntity::find()
            .filter(is_active_condition())
            .order_by_desc(order::Column::Timestamp)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let orders = orders.into_iter().map(Order::from).collect();

        Ok(orders)
    }

    pub async fn find_by_user(
        db: &DatabaseConnection,
        user: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Order>, Error> {
        let orders = OrderEntity::find()
            .filter(order::Column::User.eq(user))
            .order_by_desc(order::Column::Timestamp)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let orders = orders.into_iter().map(Order::from).collect();

        Ok(orders)
    }

    pub async fn find_by_type(
        db: &DatabaseConnection,
        order_type: OrderType,
        limit: u64,
        offset: u64,
        user_ne: Option<String>,
    ) -> Result<Vec<Order>, Error> {
        let order_type = OrderTypeSea::from(order_type);
        let select = OrderEntity::find().filter(find_condition(order_type.clone(), user_ne));

        // Sort orders by price depending on order type
        let orders = match order_type {
            OrderTypeSea::Buy => select.order_by_desc(order::Column::Price),
            OrderTypeSea::Sell => select.order_by_asc(order::Column::Price),
        }
        .offset(offset)
        .limit(limit)
        .all(db)
        .await?;
        let orders = orders.into_iter().map(Order::from).collect();

        Ok(orders)
    }
}

pub struct Mutation;
impl Mutation {
    pub async fn insert(db: &DatabaseConnection, data: InsertOrder) -> Result<(), Error> {
        let order = order::ActiveModel {
            tx_id: Set(data.tx_id),
            order_id: Set(data.order_id),
            order_type: Set(data.order_type.into()),
            user: Set(data.user),
            asset: Set(data.asset),
            amount: Set(data.amount as i64),
            price: Set(data.price as i64),
            status: Set(data.status.into()),
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
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn insert_many(db: &DatabaseConnection, data: Vec<InsertOrder>) -> Result<(), Error> {
        let len = data.len();
        if len == 0 {
            return Ok(());
        }

        let orders = data
            .into_iter()
            .map(|order| order::ActiveModel {
                tx_id: Set(order.tx_id),
                order_id: Set(order.order_id),
                order_type: Set(order.order_type.into()),
                user: Set(order.user),
                asset: Set(order.asset),
                amount: Set(order.amount as i64),
                price: Set(order.price as i64),
                status: Set(order.status.into()),
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
            .exec(db)
            .await?;

        log::debug!("DB | CREATE_ORDERS: {} | {:?}", len, res);

        Ok(())
    }

    pub async fn update(db: &DatabaseConnection, data: UpdateOrder) -> Result<Order, Error> {
        let order = OrderEntity::find()
            .filter(order::Column::OrderId.eq(data.order_id))
            .one(db)
            .await?;
        let mut order: order::ActiveModel = order.unwrap().into();

        if let Some(amount) = data.amount {
            order.amount = Set(amount as i64);
        }
        order.status = Set(data.status.into());

        let order = OrderEntity::update(order).exec(db).await?;

        Ok(Order::from(order))
    }
}

fn is_active_condition() -> Condition {
    Condition::any()
        .add(order::Column::Status.eq(OrderStatusSea::New))
        .add(order::Column::Status.eq(OrderStatusSea::PartiallyMatched))
}

fn find_condition(order_type: OrderTypeSea, user_ne: Option<String>) -> Condition {
    // Filter orders by type and active status
    let condition = Condition::all()
        .add(is_active_condition())
        .add(order::Column::OrderType.eq(order_type));

    // Exclude orders by user
    match user_ne {
        Some(user) => condition.add(order::Column::User.ne(user)),
        None => condition,
    }
}
