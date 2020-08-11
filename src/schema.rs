table! {
    directories (id) {
        id -> Nullable<Integer>,
        series -> Nullable<Integer>,
        pattern -> Nullable<Text>,
        dir -> Nullable<Text>,
        volume -> Nullable<Integer>,
        recursive -> Nullable<Integer>,
    }
}

table! {
    episodes (id) {
        id -> Nullable<Integer>,
        series -> Nullable<Integer>,
        number -> Nullable<Integer>,
        name -> Nullable<Text>,
        file -> Nullable<Text>,
        date_of_read -> Nullable<Timestamp>,
        volume -> Nullable<Integer>,
    }
}

table! {
    media_types (id) {
        id -> Nullable<Integer>,
        name -> Nullable<Text>,
        base_dir -> Nullable<Text>,
        file_types -> Nullable<Text>,
        program -> Nullable<Text>,
    }
}

table! {
    serieses (id) {
        id -> Nullable<Integer>,
        media_type -> Nullable<Integer>,
        name -> Nullable<Text>,
        numbers_repeat_each_volume -> Nullable<Integer>,
        download_command_dir -> Nullable<Text>,
        download_command -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    directories,
    episodes,
    media_types,
    serieses,
);
