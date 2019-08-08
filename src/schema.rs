table! {
    location (location_id) {
        location_id -> Int4,
        shooter_id -> Int4,
        visit_id -> Int4,
        location_status -> Varchar,
        location_title -> Nullable<Varchar>,
        location_lat -> Numeric,
        location_long -> Numeric,
        location_description -> Nullable<Text>,
        location_create_time -> Timestamp,
        location_post_time -> Nullable<Timestamp>,
        location_unpost_time -> Nullable<Timestamp>,
        location_modify_time -> Timestamp,
    }
}

table! {
    oauth (oauth_id) {
        oauth_id -> Int4,
        oauth_vendor -> Varchar,
        oauth_user -> Varchar,
        shooter_id -> Int4,
        oauth_status -> Varchar,
        oauth_create_time -> Timestamp,
        oauth_last_use_time -> Timestamp,
        oauth_modify_time -> Timestamp,
    }
}

table! {
    photo (photo_id) {
        photo_id -> Int4,
        location_id -> Int4,
        shooter_id -> Int4,
        visit_id -> Int4,
        photo_status -> Varchar,
        photo_title -> Varchar,
        photo_description -> Nullable<Text>,
        photo_create_time -> Timestamp,
        photo_post_time -> Nullable<Timestamp>,
        photo_unpost_time -> Nullable<Timestamp>,
        photo_modify_time -> Timestamp,
    }
}

table! {
    rating (rating_id) {
        rating_id -> Int4,
        location_id -> Int4,
        shooter_id -> Int4,
        visit_id -> Int4,
        rating_status -> Varchar,
        rating_score -> Int4,
        rating_comments -> Nullable<Text>,
        rating_create_time -> Timestamp,
        rating_post_time -> Nullable<Timestamp>,
        rating_unpost_time -> Nullable<Timestamp>,
        rating_modify_time -> Timestamp,
    }
}

table! {
    shooter (shooter_id) {
        shooter_id -> Int4,
        shooter_name -> Varchar,
        shooter_password -> Varchar,
        shooter_status -> Varchar,
        shooter_email -> Varchar,
        shooter_real_name -> Varchar,
        shooter_create_time -> Timestamp,
        shooter_active_time -> Nullable<Timestamp>,
        shooter_inactive_time -> Nullable<Timestamp>,
        shooter_remove_time -> Nullable<Timestamp>,
        shooter_modify_time -> Timestamp,
    }
}

table! {
    thumbsup (thumbsup_id) {
        thumbsup_id -> Int4,
        photo_id -> Int4,
        shooter_id -> Int4,
        thumbsup_status -> Varchar,
        thumbsup_comments -> Nullable<Text>,
        thumbsup_create_time -> Timestamp,
        thumbsup_post_time -> Nullable<Timestamp>,
        thumbsup_unpost_time -> Nullable<Timestamp>,
        thumbsup_modify_time -> Timestamp,
    }
}

table! {
    visit (visit_id) {
        visit_id -> Int4,
        location_id -> Int4,
        shooter_id -> Int4,
        visit_time -> Nullable<Timestamp>,
        visit_create_time -> Timestamp,
        visit_modify_time -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    location,
    oauth,
    photo,
    rating,
    shooter,
    thumbsup,
    visit,
);
