use sea_orm_migration::prelude::*;
use crate::m20240521_021114_create_users_table::Users;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> & str {
        "m20240517_061226_create_tasks_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tasks::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tasks::Item).string().not_null())
                    .col(ColumnDef::new(Tasks::UserId).integer().default(Value::Int(None)))
                    .foreign_key(
                        ForeignKey::create()
                            .name("user_id")
                            .from(Tasks::Table, Tasks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Tasks {
    Table,
    Id,
    Item,
    UserId,
}
