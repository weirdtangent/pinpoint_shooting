use diesel::Queryable;

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "users"]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub password: String,
    pub status: String,
    pub email: String,
    pub real_name: String,
    pub create_time: chrono::NaiveDateTime,
    pub active_time: Option<chrono::NaiveDateTime>,
    pub inactive_time: Option<chrono::NaiveDateTime>,
    pub remove_time: Option<chrono::NaiveDateTime>,
    pub modify_time: chrono::NaiveDateTime,
}

use super::schema::users;
 
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub user_name: &'a str,
    pub password: &'a str,
    pub email: &'a str,
    pub real_name: &'a str,
}
