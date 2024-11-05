use sea_orm::{
    sea_query::OnConflict, DatabaseConnection, EntityTrait, QueryOrder, QuerySelect, Set,
};
use sparker_core::{InsertTrade, Trade};
use sparker_entity::trade::{self, Entity as TradeEntity};

use crate::error::Error;

pub struct Query;
impl Query {
    pub async fn find(
        db: &DatabaseConnection,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Trade>, Error> {
        let trades = TradeEntity::find()
            .order_by_desc(trade::Column::Timestamp)
            .offset(offset)
            .limit(limit)
            .all(db)
            .await?;
        let trades = trades.into_iter().map(Trade::from).collect();

        Ok(trades)
    }
}

pub struct Mutation;
impl Mutation {
    pub async fn insert(db: &DatabaseConnection, data: InsertTrade) -> Result<(), Error> {
        let trade = trade::ActiveModel {
            tx_id: Set(data.tx_id),
            trade_id: Set(data.trade_id),
            order_id: Set(data.order_id),
            limit_type: Set(data.limit_type.into()),
            size: Set(data.size as i64),
            price: Set(data.price as i64),
            timestamp: Set(data.timestamp),
            market_id: Set(data.market_id),
            ..Default::default()
        };
        let on_conflict = OnConflict::column(trade::Column::TradeId)
            .do_nothing()
            .to_owned();
        TradeEntity::insert(trade)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn insert_many(db: &DatabaseConnection, data: Vec<InsertTrade>) -> Result<(), Error> {
        let len = data.len();
        if len == 0 {
            return Ok(());
        }

        let trades = data
            .into_iter()
            .map(|trade| trade::ActiveModel {
                tx_id: Set(trade.tx_id),
                trade_id: Set(trade.trade_id),
                order_id: Set(trade.order_id),
                limit_type: Set(trade.limit_type.into()),
                size: Set(trade.size as i64),
                price: Set(trade.price as i64),
                timestamp: Set(trade.timestamp),
                market_id: Set(trade.market_id),
                ..Default::default()
            })
            .collect::<Vec<trade::ActiveModel>>();

        let on_conflict = OnConflict::column(trade::Column::TradeId)
            .do_nothing()
            .to_owned();
        let res = TradeEntity::insert_many(trades)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(db)
            .await?;

        log::debug!("DB | CREATE_TRADES: {} | {:?}", len, res);

        Ok(())
    }
}
