table! {
    users (user_id) {
        user_id -> Int4,
        user_name -> Varchar,
        password -> Varchar,
        email -> Varchar,
        real_name -> Varchar,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}
