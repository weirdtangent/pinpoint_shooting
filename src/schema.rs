use diesel::*;

table! {
    likes (like_id) {
        like_id -> Int4,
        photo_id -> Int4,
        user_id -> Int4,
        status -> Varchar,
        comments -> Nullable<Text>,
        create_time -> Timestamp,
        post_time -> Nullable<Timestamp>,
        unpost_time -> Nullable<Timestamp>,
        modify_time -> Timestamp,
    }
}

table! {
    locations (location_id) {
        location_id -> Int4,
        user_id -> Int4,
        visit_id -> Int4,
        status -> Varchar,
        title -> Nullable<Varchar>,
        lat -> Numeric,
        long -> Numeric,
        description -> Nullable<Text>,
        create_time -> Timestamp,
        post_time -> Nullable<Timestamp>,
        unpost_time -> Nullable<Timestamp>,
        modify_time -> Timestamp,
    }
}

table! {
    photos (photo_id) {
        photo_id -> Int4,
        location_id -> Int4,
        user_id -> Int4,
        visit_id -> Int4,
        status -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        create_time -> Timestamp,
        post_time -> Nullable<Timestamp>,
        unpost_time -> Nullable<Timestamp>,
        modify_time -> Timestamp,
    }
}

table! {
    ratings (rating_id) {
        rating_id -> Int4,
        location_id -> Int4,
        user_id -> Int4,
        visit_id -> Int4,
        status -> Varchar,
        rating -> Int4,
        comments -> Nullable<Text>,
        create_time -> Timestamp,
        post_time -> Nullable<Timestamp>,
        unpost_time -> Nullable<Timestamp>,
        modify_time -> Timestamp,
    }
}

table! {
    users (user_id) {
        user_id -> Int4,
        user_name -> Varchar,
        password -> Varchar,
        status -> Varchar,
        email -> Varchar,
        real_name -> Varchar,
        create_time -> Timestamp,
        active_time -> Nullable<Timestamp>,
        inactive_time -> Nullable<Timestamp>,
        remove_time -> Nullable<Timestamp>,
        modify_time -> Timestamp,
    }
}

table! {
    visits (visit_id) {
        visit_id -> Int4,
        location_id -> Int4,
        user_id -> Int4,
        visit_time -> Nullable<Timestamp>,
        create_time -> Timestamp,
        modify_time -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(likes, locations, photos, ratings, users, visits,);
