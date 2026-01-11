//! Diesel models for database tables.

use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use crate::schema::{courses, exams, modules, notes, videos};

/// Diesel model for the courses table.
#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = courses)]
#[diesel(check_for_backend(Sqlite))]
pub struct CourseRow {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub playlist_id: String,
    pub description: Option<String>,
    pub created_at: String, // SQLite stores TIMESTAMP as TEXT
}

/// Insertable model for courses.
#[derive(Insertable)]
#[diesel(table_name = courses)]
pub struct NewCourse<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub source_url: &'a str,
    pub playlist_id: &'a str,
    pub description: Option<&'a str>,
}

/// Diesel model for the modules table.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(table_name = modules)]
#[diesel(belongs_to(CourseRow, foreign_key = course_id))]
#[diesel(check_for_backend(Sqlite))]
pub struct ModuleRow {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub sort_order: i32,
}

/// Insertable model for modules.
#[derive(Insertable)]
#[diesel(table_name = modules)]
pub struct NewModule<'a> {
    pub id: &'a str,
    pub course_id: &'a str,
    pub title: &'a str,
    pub sort_order: i32,
}

/// Diesel model for the videos table.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(table_name = videos)]
#[diesel(belongs_to(ModuleRow, foreign_key = module_id))]
#[diesel(check_for_backend(Sqlite))]
pub struct VideoRow {
    pub id: String,
    pub module_id: String,
    pub youtube_id: String,
    pub title: String,
    pub duration_secs: i32,
    pub is_completed: bool,
    pub sort_order: i32,
    pub description: Option<String>,
}

/// Insertable model for videos.
#[derive(Insertable)]
#[diesel(table_name = videos)]
pub struct NewVideo<'a> {
    pub id: &'a str,
    pub module_id: &'a str,
    pub youtube_id: &'a str,
    pub title: &'a str,
    pub duration_secs: i32,
    pub is_completed: bool,
    pub sort_order: i32,
    pub description: Option<&'a str>,
}

/// Diesel model for the exams table.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(table_name = exams)]
#[diesel(belongs_to(VideoRow, foreign_key = video_id))]
#[diesel(check_for_backend(Sqlite))]
pub struct ExamRow {
    pub id: String,
    pub video_id: String,
    pub question_json: String,
    pub score: Option<f32>,
    pub passed: Option<bool>,
    pub user_answers_json: Option<String>,
}

/// Insertable model for exams.
#[derive(Insertable)]
#[diesel(table_name = exams)]
pub struct NewExam<'a> {
    pub id: &'a str,
    pub video_id: &'a str,
    pub question_json: &'a str,
    pub user_answers_json: Option<&'a str>,
}

/// Diesel model for the notes table.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(table_name = notes)]
#[diesel(belongs_to(VideoRow, foreign_key = video_id))]
#[diesel(check_for_backend(Sqlite))]
pub struct NoteRow {
    pub id: String,
    pub video_id: String,
    pub content: String,
    pub updated_at: String, // SQLite stores TIMESTAMP as TEXT
}

/// Insertable model for notes.
#[derive(Insertable)]
#[diesel(table_name = notes)]
pub struct NewNote<'a> {
    pub id: &'a str,
    pub video_id: &'a str,
    pub content: &'a str,
}
