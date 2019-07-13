use diesel::Queryable;

#[derive(Queryable)]

pub struct User {
    pub user_id: u32,
    pub user_name: String,
    pub password: String,
    pub email: String,
    pub real_name: String,
    pub create_time: String,
    pub modify_time: String,
}
