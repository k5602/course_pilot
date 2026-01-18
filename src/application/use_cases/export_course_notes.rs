//! Export course notes as Markdown.

use std::sync::Arc;

use crate::domain::{
    entities::{Module, Note, Tag, Video},
    ports::{
        CourseRepository, ModuleRepository, NoteRepository, RepositoryError, TagRepository,
        VideoRepository,
    },
    value_objects::CourseId,
};

/// Input for exporting course notes.
pub struct ExportCourseNotesInput {
    pub course_id: CourseId,
}

/// Error type for exporting course notes.
#[derive(Debug, thiserror::Error)]
pub enum ExportCourseNotesError {
    #[error("Course not found")]
    CourseNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

/// Use case to export all notes for a course as Markdown.
pub struct ExportCourseNotesUseCase<CR, MR, VR, NR, TR>
where
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    NR: NoteRepository,
    TR: TagRepository,
{
    course_repo: Arc<CR>,
    module_repo: Arc<MR>,
    video_repo: Arc<VR>,
    note_repo: Arc<NR>,
    tag_repo: Arc<TR>,
}

impl<CR, MR, VR, NR, TR> ExportCourseNotesUseCase<CR, MR, VR, NR, TR>
where
    CR: CourseRepository,
    MR: ModuleRepository,
    VR: VideoRepository,
    NR: NoteRepository,
    TR: TagRepository,
{
    /// Creates a new export use case.
    pub fn new(
        course_repo: Arc<CR>,
        module_repo: Arc<MR>,
        video_repo: Arc<VR>,
        note_repo: Arc<NR>,
        tag_repo: Arc<TR>,
    ) -> Self {
        Self { course_repo, module_repo, video_repo, note_repo, tag_repo }
    }

    /// Exports all notes for a course as Markdown.
    pub fn execute(&self, input: ExportCourseNotesInput) -> Result<String, ExportCourseNotesError> {
        let course = self
            .course_repo
            .find_by_id(&input.course_id)
            .map_err(map_repo_error)?
            .ok_or(ExportCourseNotesError::CourseNotFound)?;

        let modules = self.module_repo.find_by_course(&input.course_id).map_err(map_repo_error)?;

        let videos = self.video_repo.find_by_course(&input.course_id).map_err(map_repo_error)?;

        let tags = self.tag_repo.find_by_course(&input.course_id).map_err(map_repo_error)?;

        let markdown = build_markdown(
            course.name(),
            course.description(),
            &tags,
            &modules,
            &videos,
            |video| self.note_repo.find_by_video(video.id()).map_err(map_repo_error),
        )?;

        Ok(markdown)
    }
}

fn map_repo_error(err: RepositoryError) -> ExportCourseNotesError {
    ExportCourseNotesError::Repository(err.to_string())
}

fn build_markdown<F>(
    course_name: &str,
    course_description: Option<&str>,
    tags: &[Tag],
    modules: &[Module],
    videos: &[Video],
    mut note_loader: F,
) -> Result<String, ExportCourseNotesError>
where
    F: FnMut(&Video) -> Result<Option<Note>, ExportCourseNotesError>,
{
    let mut output = String::new();

    output.push_str("# Course Notes\n\n");
    output.push_str("## Course\n\n");
    output.push_str(&format!("- **Name:** {}\n", course_name));
    if let Some(desc) = course_description {
        let trimmed = desc.trim();
        if !trimmed.is_empty() {
            output.push_str(&format!("- **Description:** {}\n", trimmed));
        }
    }
    if !tags.is_empty() {
        let names = tags.iter().map(|t| t.name().to_string()).collect::<Vec<_>>().join(", ");
        output.push_str(&format!("- **Tags:** {}\n", names));
    }
    output.push_str("\n---\n\n");

    if modules.is_empty() {
        output.push_str("_No modules found._\n");
        return Ok(output);
    }

    let mut sorted_modules = modules.to_vec();
    sorted_modules.sort_by_key(|m| m.sort_order());

    for module in sorted_modules {
        output.push_str(&format!("## Module: {}\n\n", module.title()));

        let mut module_videos =
            videos.iter().filter(|v| v.module_id() == module.id()).cloned().collect::<Vec<_>>();

        module_videos.sort_by_key(|v| v.sort_order());

        if module_videos.is_empty() {
            output.push_str("_No videos in this module._\n\n");
            continue;
        }

        for video in module_videos {
            output.push_str(&format!("### {}\n\n", video.title()));

            match note_loader(&video)? {
                Some(note) => {
                    let content = note.content().trim();
                    if content.is_empty() {
                        output.push_str("_No notes for this video._\n\n");
                    } else {
                        output.push_str(content);
                        output.push_str("\n\n");
                    }
                },
                None => {
                    output.push_str("_No notes for this video._\n\n");
                },
            }
        }

        output.push_str("---\n\n");
    }

    Ok(output)
}
