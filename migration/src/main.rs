use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let key = "DATABASE_URL";
    if std::env::var(key).is_err() {
        let figment = rocket::Config::figment();
        let database_url = figment.extract_inner("databases.todo.url")?;
        std::env::set_var(key, database_url);
    }

    cli::run_cli(migration::Migrator).await;
}
