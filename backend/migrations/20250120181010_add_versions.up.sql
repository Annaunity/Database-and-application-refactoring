create table drawing_versions (
    drawing_id integer not null,
    version_id integer not null,
    width integer not null check (width > 0),
    height integer not null check (height > 0),
    image_id text not null,
    thumbnail_image_id text not null,
    created_at timestamp not null,
    primary key (drawing_id, version_id)
);
