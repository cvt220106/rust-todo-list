use argon2::Config;
use entity::users::{self, Entity as Users, USER_PASSWORD_SALT};
use rocket::{
    response::{Flash, Redirect},
    http::{CookieJar, Cookie},
    request::{FromRequest, Outcome},
    Request
};
use rocket::form::Form;
use rocket::http::Status;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sea_orm_rocket::Connection;
use crate::pool::Db;

pub struct AuthenticatedUser {
    pub user_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = anyhow::Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        let user_id_cookie = match get_user_id_cookie(cookies) {
            Some(result) => result,
            None => {
                return Outcome::Forward(Status::new(404));
            }
        };

        let logged_in_user_id = match user_id_cookie.value().parse::<i32>() {
            Ok(result) => result,
            Err(_) => {
                return Outcome::Forward(Status::new(404));
            }
        };

        Outcome::Success(AuthenticatedUser {
            user_id: logged_in_user_id,
        })
    }
}

fn get_user_id_cookie<'a>(cookies: &'a CookieJar) -> Option<Cookie<'a>> {
    cookies.get_private("user_id")
}

fn set_user_id_cookie(cookies: &CookieJar, user_id: i32) {
    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
}

fn remove_user_id_cookie(cookies: &CookieJar) {
    cookies.remove_private(Cookie::build("user_id"));
}

#[post("/logout")]
pub async fn logout(cookie: &CookieJar<'_>) -> Flash<Redirect> {
    remove_user_id_cookie(cookie);
    Flash::success(Redirect::to("/login"), "Logged out successfully")
}

fn login_error() -> Flash<Redirect> {
    Flash::error(Redirect::to("/login"), "Incorrect username or password")
}

#[post("/createaccount", data = "<user_form>")]
pub async fn create_account(
    conn: Connection<'_, Db>,
    user_form: Form<users::Model>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    let hash_config = Config::default();
    let hash =
        match argon2::hash_encoded(user.password.as_bytes(), USER_PASSWORD_SALT, &hash_config) {
            Ok(result) => result,
            Err(_) => return Flash::error(Redirect::to("/signup"), "Issue creating the account!"),
        };

    let active_user = users::ActiveModel {
        username: Set(user.username),
        password: Set(hash),
        ..Default::default()
    };

    match active_user.insert(db).await {
        Ok(result) => result,
        Err(_) => return Flash::error(Redirect::to("/signup"), "Issue creating the account!"),
    };

    Flash::success(Redirect::to("/login"), "Account created successfully!")
}

#[post("/verifyaccount", data = "<user_form>")]
pub async fn verify_account(
    conn: Connection<'_, Db>,
    cookies: &CookieJar<'_>,
    user_form: Form<users::Model>,
) -> Flash<Redirect> {
    let db = conn.into_inner();
    let user = user_form.into_inner();

    println!("user: {:?}", user);
    let stored_user = match Users::find()
        .filter(users::Column::Username.eq(&*user.username))
        .one(db)
        .await
    {
        Ok(model_or_null) => {
            match model_or_null {
                Some(model) => model,
                None => {
                    return login_error();
                }
            }
        },
        Err(_) => {
            return login_error();
        }
    };

    let is_password_correct = match argon2::verify_encoded(&stored_user.password, user.password.as_bytes()) {
        Ok(result) => result,
        Err(_) => {
            return Flash::error(Redirect::to("/login"), "Encountered an issue processing your account!");
        }
    };

    if !is_password_correct {
        return login_error();
    }

    set_user_id_cookie(cookies, stored_user.id);
    Flash::success(Redirect::to("/"), "Logged in successfully")
}
