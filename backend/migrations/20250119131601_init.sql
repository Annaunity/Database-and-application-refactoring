create table users (
    username text primary key,
    email text not null unique,
    password_hash text not null,
    favourite_animal text not null
);

create table sessions (
    token bytea primary key,
    token_id bytea not null unique,
    username text not null references users (username),
    last_used_at timestamp not null,
    expires_at timestamp not null,
    user_agent text not null,
    ip_address text not null
);

create table drawings (
    id serial primary key,
    name text not null,
    owner text not null references users (username),
    width integer not null check (width > 0),
    height integer not null check (height > 0),
    image_id text not null,
    thumbnail_image_id text not null,
    created_at timestamp not null,
    updated_at timestamp not null
);
