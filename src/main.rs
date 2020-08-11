extern crate diesel;

extern crate chapter_tracker;

// use diesel::prelude::*;

// use chapter_tracker::models::*;

fn main() {
    // let con = chapter_tracker::establish_connection();
    // use chapter_tracker::schema;
    // use chapter_tracker::schema::serieses::dsl::*;
    // // use chapter_tracker::schema::media_types::dsl::*;
    // use chapter_tracker::schema::episodes::dsl::*;

    // let result = media_types
        // // .filter(name.eq("anime"))
        // .load::<MediaType>(&con).unwrap();

    // for media_type in media_types.load::<MediaType>(&con).unwrap() {
        // println!("{:?}", media_type);
    // }

    // for (series, media_type) in serieses.filter(chapter_tracker::schema::serieses::dsl::name.eq("Supernatural")).inner_join(media_types).load::<(Series, MediaType)>(&con).unwrap() {
        // println!("{:?}", series);
        // println!("{:?}", media_type);
    // }
    // let result = serieses
        // // .filter(name.eq("Supernatural"))
        // .filter(media_type_id.eq(2))
        // .load::<Series>(&con).unwrap();

    // for series in result {
        // println!("{:?}", series);
    // }

    // for directory in directories.load::<Directory>(&con).unwrap() {
        // println!("{:?}", directory);
    // }
    // for episode in episodes.load::<Episode>(&con).unwrap() {
        // println!("{:?}", episode);
    // }
    //
    // for episode in episodes.filter(date_of_read.eq(None)).load::<Episode>(&con) {
    // let unread_serieses_query = episodes.filter(date_of_read.is_null()).select(series_id).distinct();
    // for series in serieses.filter(schema::serieses::dsl::id.eq_any(unread_serieses_query)).load::<Series>(&con).unwrap() {
        // println!("{:?}", series.name);
    // }
    // chapter_tracker::scan::scan();

}
