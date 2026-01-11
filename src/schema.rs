// @generated automatically by Diesel CLI.

diesel::table! {
    courses (id) {
        id -> Text,
        name -> Text,
        source_url -> Text,
        playlist_id -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    exams (id) {
        id -> Text,
        video_id -> Text,
        question_json -> Text,
        score -> Nullable<Float>,
        passed -> Nullable<Bool>,
        user_answers_json -> Nullable<Text>,
    }
}

diesel::table! {
    modules (id) {
        id -> Text,
        course_id -> Text,
        title -> Text,
        sort_order -> Integer,
    }
}

diesel::table! {
    videos (id) {
        id -> Text,
        module_id -> Text,
        youtube_id -> Text,
        title -> Text,
        duration_secs -> Integer,
        is_completed -> Bool,
        sort_order -> Integer,
    }
}

diesel::joinable!(exams -> videos (video_id));
diesel::joinable!(modules -> courses (course_id));
diesel::joinable!(videos -> modules (module_id));

diesel::allow_tables_to_appear_in_same_query!(courses, exams, modules, videos,);
