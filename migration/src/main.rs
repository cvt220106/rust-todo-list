use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
    let key = "DATABASE_URL";
    if std::env::var(key).is_err() {
        let figment = rocket::Config::figment();
        let database_url: String = figment.extract_inner("databases.todo.url").expect("Cannot find DATABASE_URL in Rocket.toml");
        std::env::set_var(key, &database_url);
    }

    cli::run_cli(migration::Migrator).await;
}
