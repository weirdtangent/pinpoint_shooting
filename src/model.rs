use crate::schema::*;

#[derive(Identifiable, Queryable, Debug, PartialEq)]
#[table_name = "shooter"]
#[primary_key("shooter_id")]
pub struct Shooter {
    pub shooter_id: i32,
    pub shooter_name: String,
    pub shooter_password: String,
    pub shooter_status: String,
    pub shooter_email: String,
    pub shooter_real_name: String,
    pub shooter_create_time: chrono::NaiveDateTime,
    pub shooter_active_time: Option<chrono::NaiveDateTime>,
    pub shooter_inactive_time: Option<chrono::NaiveDateTime>,
    pub shooter_remove_time: Option<chrono::NaiveDateTime>,
    pub shooter_modify_time: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "shooter"]
pub struct NewShooter {
    pub shooter_name: String,
    pub shooter_password: String,
    pub shooter_status: String,
    pub shooter_email: String,
    pub shooter_real_name: String,
}

#[derive(Identifiable, Associations, Queryable, Debug, PartialEq)]
#[belongs_to(Shooter)]
#[table_name = "oauth"]
#[primary_key("oauth_id")]
pub struct Oauth {
    pub oauth_id: i32,
    pub oauth_vendor: String,
    pub oauth_user: String,
    pub shooter_id: i32,
    pub oauth_status: String,
    pub oauth_create_time: chrono::NaiveDateTime,
    pub oauth_last_use_time: chrono::NaiveDateTime,
    pub oauth_modify_time: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "oauth"]
pub struct NewOauth {
    pub oauth_vendor: String,
    pub oauth_user: String,
    pub shooter_id: i32,
}
