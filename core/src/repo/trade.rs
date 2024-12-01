use sea_orm::{
    sea_query::OnConflict, ColumnTrait, Condition, DatabaseConnection, DbErr as Error, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set
};
use sparker_entity::trade::{self, Entity as TradeEntity};

use crate::types::Trade;

pub struct Query;
impl Query {
    pub async fn find(
        db_conn: &DatabaseConnection,
        market_id: String,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<Trade>, Error> {
        let trades = TradeEntity::find()
            .filter(trade::Column::MarketId.eq(market_id))
            .order_by_desc(trade::Column::Timestamp)
            .offset(offset)
            .limit(limit)
            .all(db_conn)
            .await?;
        let trades = trades.into_iter().map(Trade::from).collect();

        Ok(trades)
    }
}

pub struct Mutation;
impl Mutation {
    pub async fn insert(db_conn: &DatabaseConnection, data: Trade) -> Result<(), Error> {
        let trade = trade::ActiveModel {
            tx_id: Set(data.tx_id),
            trade_id: Set(data.trade_id),
            order_id: Set(data.order_id),
            limit_type: Set(data.limit_type.into()),
            user: Set(data.user),
            size: Set(data.size as i64),
            price: Set(data.price as i64),
            timestamp: Set(data.timestamp),
            market_id: Set(data.market_id),
            block_number: Set(data.block_number as i64),
            ..Default::default()
        };
        let on_conflict = OnConflict::column(trade::Column::TradeId)
            .do_nothing()
            .to_owned();
        TradeEntity::insert(trade)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(db_conn)
            .await?;

        Ok(())
    }

    pub async fn insert_many(db_conn: &DatabaseConnection, data: Vec<Trade>) -> Result<(), Error> {
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
                user: Set(trade.user),
                size: Set(trade.size as i64),
                price: Set(trade.price as i64),
                timestamp: Set(trade.timestamp),
                market_id: Set(trade.market_id),
                block_number: Set(trade.block_number as i64),
                ..Default::default()
            })
            .collect::<Vec<trade::ActiveModel>>();

        let on_conflict = OnConflict::column(trade::Column::TradeId)
            .do_nothing()
            .to_owned();

        TradeEntity::insert_many(trades)
            .on_conflict(on_conflict)
            .do_nothing()
            .exec(db_conn)
            .await?;

        Ok(())
    }

    pub async fn delete_many(
        db_conn: &DatabaseConnection,
        market_id: String,
        from_block: i64,
    ) -> Result<u64, Error> {
        let res = TradeEntity::delete_many()
            .filter(
                Condition::all()
                    .add(trade::Column::MarketId.eq(market_id))
                    .add(trade::Column::BlockNumber.gte(from_block)),
            )
            .exec(db_conn)
            .await?;

        Ok(res.rows_affected)
    }
}
