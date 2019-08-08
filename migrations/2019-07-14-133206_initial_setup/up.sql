CREATE TABLE shooter (
      shooter_id serial PRIMARY KEY,
      shooter_name VARCHAR(64) UNIQUE NOT NULL CHECK (shooter_name <> ''),
      shooter_password VARCHAR(64) NOT NULL CHECK (shooter_password <> ''),
      shooter_status VARCHAR(25) NOT NULL DEFAULT 'pending',
      shooter_email VARCHAR(355) UNIQUE NOT NULL CHECK (shooter_email <> ''),
      shooter_real_name VARCHAR(128) NOT NULL CHECK (shooter_real_name <> ''),
      shooter_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      shooter_active_time TIMESTAMP,
      shooter_inactive_time TIMESTAMP,
      shooter_remove_time TIMESTAMP,
      shooter_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE location (
      location_id serial PRIMARY KEY,
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      visit_id INTEGER NOT NULL CHECK (visit_id >= 0),
      location_status VARCHAR(25) NOT NULL DEFAULT 'draft',
      location_title VARCHAR(128),
      location_lat NUMERIC(11,8) NOT NULL,
      location_long NUMERIC(11,8) NOT NULL,
      location_description TEXT,
      location_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      location_post_time TIMESTAMP,
      location_unpost_time TIMESTAMP,
      location_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE rating (
      rating_id serial PRIMARY KEY,
      location_id INTEGER NOT NULL CHECK (location_id >= 0),
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      visit_id INTEGER NOT NULL CHECK (visit_id >= 0),
      rating_status VARCHAR(25) NOT NULL DEFAULT 'draft',
      rating_score INTEGER NOT NULL CHECK (rating_score >= 0 AND rating_score <=5),
      rating_comments TEXT,
      rating_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      rating_post_time TIMESTAMP,
      rating_unpost_time TIMESTAMP,
      rating_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE visit (
      visit_id serial PRIMARY KEY,
      location_id INTEGER NOT NULL CHECK (location_id >= 0),
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      visit_time TIMESTAMP,
      visit_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      visit_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE photo (
      photo_id serial PRIMARY KEY,
      location_id INTEGER NOT NULL CHECK (location_id >= 0),
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      visit_id INTEGER NOT NULL CHECK (visit_id >= 0),
      photo_status VARCHAR(25) NOT NULL DEFAULT 'active',
      photo_title VARCHAR(128) NOT NULL,
      photo_description TEXT,
      photo_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      photo_post_time TIMESTAMP,
      photo_unpost_time TIMESTAMP,
      photo_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE thumbsup (
      thumbsup_id serial PRIMARY KEY,
      photo_id INTEGER NOT NULL CHECK (photo_id >= 0),
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      thumbsup_status VARCHAR(25) NOT NULL DEFAULT 'active',
      thumbsup_comments TEXT,
      thumbsup_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      thumbsup_post_time TIMESTAMP,
      thumbsup_unpost_time TIMESTAMP,
      thumbsup_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE oauth (
      oauth_id serial PRIMARY KEY,
      oauth_vendor VARCHAR(64) NOT NULL CHECK (oauth_vendor <> ''),
      oauth_user VARCHAR(512) NOT NULL CHECK (oauth_user <> ''),
      shooter_id INTEGER NOT NULL CHECK (shooter_id >= 0),
      oauth_status VARCHAR(25) NOT NULL DEFAULT 'active',
      oauth_create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      oauth_last_use_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
      oauth_modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);
