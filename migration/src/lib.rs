pub use sea_orm_migration::prelude::*;

mod m20240517_061226_create_tasks_table;
mod m20240521_021114_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240521_021114_create_users_table::Migration),
            Box::new(m20240517_061226_create_tasks_table::Migration),
        ]
    }
}
