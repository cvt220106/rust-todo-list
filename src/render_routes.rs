use migration::tests_cfg::json;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FlashMessage, FromRequest, Outcome};
use rocket::response::{Flash, Redirect, Responder};
use rocket::Request;
use rocket_dyn_templates::Template;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryOrder, QueryFilter
};
use sea_orm_rocket::Error::Config;
use sea_orm_rocket::{Connection, Database};

use entity::tasks::{self, Entity as Tasks};

use crate::{pool::Db, task_routes, user_routes::AuthenticatedUser};

pub struct DatabaseError(sea_orm::DbErr);

impl<'r> Responder<'r, 'r> for DatabaseError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'r> {
        Err(Status::InternalServerError)
    }
}

impl From<sea_orm::DbErr> for DatabaseError {
    fn from(value: sea_orm::DbErr) -> Self {
        DatabaseError(value)
    }
}

#[get("/?<page>&<tasks_per_page>")]
pub async fn index(
    conn: Connection<'_, Db>,
    flash: Option<FlashMessage<'_>>,
    page: Option<usize>,
    tasks_per_page: Option<usize>,
    _user: AuthenticatedUser,
) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();
    let page = page.unwrap_or(0);
    let tasks_per_page = tasks_per_page.unwrap_or(5);

    let pageinator = Tasks::find()
        .filter(tasks::Column::UserId.eq(_user.user_id))
        .order_by_asc(tasks::Column::Id)
        .paginate(db, tasks_per_page);
    let number_pf_pages = pageinator.num_pages().await?;
    let tasks = pageinator.fetch_page(page).await?;

    Ok(Template::render(
        "todo_list",
        json!({
            "tasks": tasks,
            "flash": flash.map(FlashMessage::into_inner),
            "number_of_pages": number_pf_pages,
            "current_page": page
        }),
    ))
}

#[get("/?<page>&<tasks_per_page>", rank = 2)]
pub async fn index_redirect(page: Option<usize>, tasks_per_page: Option<usize>) -> Redirect {
    task_routes::redirect_to_login()
}

#[get("/edit/<id>")]
pub async fn edit_task_page(conn: Connection<'_, Db>, id: i32, _user: AuthenticatedUser) -> Result<Template, DatabaseError> {
    let db = conn.into_inner();

    let task = Tasks::find_by_id(id).one(db).await?.unwrap();

    Ok(Template::render("edit_task_form", json!({ "task": task })))
}

#[post("/edit/<id>", rank = 2)]
pub async fn edit_task_page_redirect(id: i32) -> Redirect {
    task_routes::redirect_to_login()
}

#[get("/signup")]
pub async fn signup_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "signup_page",
        json!({ "flash": flash.map(FlashMessage::into_inner) }),
    )
}

#[get("/login")]
pub async fn login_page(flash: Option<FlashMessage<'_>>) -> Template {
    Template::render(
        "login_page",
        json!({ "flash": flash.map(FlashMessage::into_inner) }),
    )
}
