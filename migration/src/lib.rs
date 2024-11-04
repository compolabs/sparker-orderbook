pub use sea_orm_migration::prelude::*;

mod m20241101_130253_create_types;
mod m20241101_130314_create_orders;
mod m20241101_225432_create_trades;
mod m20241104_075814_create_state;
mod order;
mod state;
mod trade;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241101_130253_create_types::Migration),
            Box::new(m20241101_130314_create_orders::Migration),
            Box::new(m20241101_225432_create_trades::Migration),
            Box::new(m20241104_075814_create_state::Migration),
        ]
    }
}
