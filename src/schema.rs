// @generated automatically by Diesel CLI.

diesel::table! {
    course_tags (course_id, tag_id) {
        course_id -> Text,
        tag_id -> Text,
    }
}

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
    notes (id) {
        id -> Text,
        video_id -> Text,
        content -> Text,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    tags (id) {
        id -> Text,
        name -> Text,
        color -> Text,
    }
}

diesel::table! {
    transcript_chunk_index (rowid) {
        rowid -> Integer,
        video_id -> Nullable<Binary>,
        chunk_index -> Nullable<Binary>,
        content -> Nullable<Binary>,
        token_count -> Nullable<Binary>,
        char_count -> Nullable<Binary>,
        #[sql_name = "transcript_chunk_index"]
        transcript_chunk_index_ -> Nullable<Binary>,
        rank -> Nullable<Binary>,
    }
}

diesel::table! {
    transcript_chunk_index_config (k) {
        k -> Binary,
        v -> Nullable<Binary>,
    }
}

diesel::table! {
    transcript_chunk_index_content (id) {
        id -> Nullable<Integer>,
        c0 -> Nullable<Binary>,
        c1 -> Nullable<Binary>,
        c2 -> Nullable<Binary>,
        c3 -> Nullable<Binary>,
        c4 -> Nullable<Binary>,
    }
}

diesel::table! {
    transcript_chunk_index_data (id) {
        id -> Nullable<Integer>,
        block -> Nullable<Binary>,
    }
}

diesel::table! {
    transcript_chunk_index_docsize (id) {
        id -> Nullable<Integer>,
        sz -> Nullable<Binary>,
    }
}

diesel::table! {
    transcript_chunk_index_idx (segid, term) {
        segid -> Binary,
        term -> Binary,
        pgno -> Nullable<Binary>,
    }
}

diesel::table! {
    user_preferences (id) {
        id -> Text,
        ml_boundary_enabled -> Integer,
        cognitive_limit_minutes -> Integer,
        right_panel_visible -> Integer,
        onboarding_completed -> Integer,
        right_panel_width -> Integer,
    }
}

diesel::table! {
    videos (id) {
        id -> Text,
        module_id -> Text,
        youtube_id -> Nullable<Text>,
        title -> Text,
        duration_secs -> Integer,
        is_completed -> Bool,
        sort_order -> Integer,
        description -> Nullable<Text>,
        transcript -> Nullable<Text>,
        summary -> Nullable<Text>,
        source_type -> Text,
        source_ref -> Text,
        key_points -> Nullable<Text>,
        key_terms -> Nullable<Text>,
    }
}

diesel::joinable!(course_tags -> courses (course_id));
diesel::joinable!(course_tags -> tags (tag_id));
diesel::joinable!(exams -> videos (video_id));
diesel::joinable!(modules -> courses (course_id));
diesel::joinable!(notes -> videos (video_id));
diesel::joinable!(videos -> modules (module_id));

diesel::allow_tables_to_appear_in_same_query!(
    course_tags,
    courses,
    exams,
    modules,
    notes,
    tags,
    transcript_chunk_index,
    transcript_chunk_index_config,
    transcript_chunk_index_content,
    transcript_chunk_index_data,
    transcript_chunk_index_docsize,
    transcript_chunk_index_idx,
    user_preferences,
    videos,
);
