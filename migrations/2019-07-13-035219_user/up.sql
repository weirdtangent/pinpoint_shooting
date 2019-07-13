CREATE TABLE users (
  user_id serial PRIMARY KEY,
  user_name VARCHAR(64) UNIQUE NOT NULL CHECK (user_name <> ''),
  password VARCHAR(64) NOT NULL CHECK (password <> ''),
  email VARCHAR(355) UNIQUE NOT NULL CHECK (email <> ''),
  real_name VARCHAR(128) NOT NULL CHECK (real_name <> ''),
  create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  modify_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);
