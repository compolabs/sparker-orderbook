use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"
                CREATE OR REPLACE FUNCTION notify_order_update()
                RETURNS TRIGGER AS $$
                BEGIN
                  PERFORM pg_notify('order_updates', row_to_json(NEW)::text);
                  RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#
                .to_owned(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"
                CREATE TRIGGER order_update_trigger
                AFTER INSERT OR UPDATE ON "order" FOR EACH ROW
                EXECUTE PROCEDURE notify_order_update();
                "#
                .to_owned(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"DROP TRIGGER IF EXISTS order_update_trigger ON "order";"#.to_owned(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                r#"DROP FUNCTION IF EXISTS notify_order_update;"#.to_owned(),
            ))
            .await?;

        Ok(())
    }
}
