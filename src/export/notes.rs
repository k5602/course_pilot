use super::*;
use crate::types::Note;
use serde_json;
use std::collections::HashMap;

/// Extended notes data for export with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesExportData {
    pub notes: Vec<Note>,
    pub export_metadata: ExportMetadata,
    pub statistics: NotesStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesStatistics {
    pub total_notes: usize,
    pub course_level_notes: usize,
    pub video_level_notes: usize,
    pub notes_with_timestamps: usize,
    pub unique_tags: Vec<String>,
    pub notes_by_course: HashMap<String, usize>,
    pub average_note_length: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub export_version: String,
    pub export_type: String,
}

impl Exportable for Vec<Note> {
    fn export_json(&self) -> Result<String> {
        let statistics = self.calculate_statistics();

        let export_data = NotesExportData {
            notes: self.clone(),
            statistics,
            export_metadata: ExportMetadata {
                exported_at: chrono::Utc::now(),
                export_version: "1.0".to_string(),
                export_type: "Notes Collection".to_string(),
            },
        };

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize notes to JSON: {}", e))
    }

    fn export_csv(&self) -> Result<String> {
        let mut csv_data = String::new();

        // CSV Header
        csv_data.push_str(
            "Note_ID,Course_ID,Video_ID,Content,Tags,Timestamp_Seconds,Created_At,Updated_At\n",
        );

        for note in self {
            let video_id_str =
                note.video_id.map(|id| id.to_string()).unwrap_or_else(|| "".to_string());

            let tags_str = note.tags.join(";");

            let timestamp_str =
                note.timestamp.map(|ts| ts.to_string()).unwrap_or_else(|| "".to_string());

            csv_data.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                note.id,
                note.course_id,
                utils::sanitize_csv_field(&video_id_str),
                utils::sanitize_csv_field(&note.content),
                utils::sanitize_csv_field(&tags_str),
                utils::sanitize_csv_field(&timestamp_str),
                utils::format_timestamp(note.created_at),
                utils::format_timestamp(note.updated_at)
            ));
        }

        Ok(csv_data)
    }

    fn export_pdf(&self) -> Result<Vec<u8>> {
        // Build a simple PDF using genpdf, loading a font family from the local fonts directory
        let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
            .map_err(|e| anyhow::anyhow!("Failed to load font family: {}", e))?;
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title("Notes Export".to_string());

        // Page decorator with margins
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        // Header
        doc.push(genpdf::elements::Paragraph::new("Notes Export"));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "Generated: {}",
            crate::export::utils::format_timestamp(chrono::Utc::now())
        )));
        doc.push(genpdf::elements::Paragraph::new(" "));

        // Content generated from notes
        let content = self.generate_pdf_content()?;
        doc.push(genpdf::elements::Paragraph::new(content));

        // Render to bytes
        let mut bytes = Vec::new();
        doc.render(&mut bytes).map_err(|e| anyhow::anyhow!("Failed to render PDF: {}", e))?;
        Ok(bytes)
    }

    fn export_with_options(&self, options: ExportOptions) -> Result<ExportResult> {
        if let Some(ref callback) = options.progress_callback {
            callback(0.0, "Starting notes export...".to_string());
        }

        let data = match options.format {
            ExportFormat::Json => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating JSON data...".to_string());
                }
                self.export_json()?.into_bytes()
            },
            ExportFormat::Csv => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating CSV data...".to_string());
                }
                self.export_csv()?.into_bytes()
            },
            ExportFormat::Pdf => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating PDF document...".to_string());
                }
                self.export_pdf()?
            },
        };

        if let Some(ref callback) = options.progress_callback {
            callback(75.0, "Validating export data...".to_string());
        }

        utils::validate_export_data(&data, options.format)?;

        if let Some(ref callback) = options.progress_callback {
            callback(100.0, "Notes export completed successfully".to_string());
        }

        Ok(ExportResult {
            filename: self.get_export_filename(options.format),
            size_bytes: data.len(),
            format: options.format,
            data,
        })
    }

    fn get_export_filename(&self, format: ExportFormat) -> String {
        utils::generate_filename("notes", format)
    }
}

/// Extension trait for notes-specific export functionality
pub trait NotesExportExtensions {
    fn calculate_statistics(&self) -> NotesStatistics;
    fn generate_pdf_content(&self) -> Result<String>;
    fn export_markdown(&self) -> Result<String>;
}

impl NotesExportExtensions for Vec<Note> {
    fn calculate_statistics(&self) -> NotesStatistics {
        self.as_slice().calculate_statistics()
    }

    fn generate_pdf_content(&self) -> Result<String> {
        self.as_slice().generate_pdf_content()
    }

    fn export_markdown(&self) -> Result<String> {
        self.as_slice().export_markdown()
    }
}

impl NotesExportExtensions for &[Note] {
    fn calculate_statistics(&self) -> NotesStatistics {
        let total_notes = self.len();
        let course_level_notes = self.iter().filter(|note| note.video_id.is_none()).count();
        let video_level_notes = self.iter().filter(|note| note.video_id.is_some()).count();
        let notes_with_timestamps = self.iter().filter(|note| note.timestamp.is_some()).count();

        // Collect unique tags
        let mut all_tags = std::collections::HashSet::new();
        for note in *self {
            for tag in &note.tags {
                all_tags.insert(tag.clone());
            }
        }
        let mut unique_tags: Vec<String> = all_tags.into_iter().collect();
        unique_tags.sort();

        // Count notes by course
        let mut notes_by_course = HashMap::new();
        for note in *self {
            let course_id_str = note.course_id.to_string();
            *notes_by_course.entry(course_id_str).or_insert(0) += 1;
        }

        // Calculate average note length
        let total_length: usize = self.iter().map(|note| note.content.len()).sum();
        let average_note_length =
            if total_notes > 0 { total_length as f32 / total_notes as f32 } else { 0.0 };

        NotesStatistics {
            total_notes,
            course_level_notes,
            video_level_notes,
            notes_with_timestamps,
            unique_tags,
            notes_by_course,
            average_note_length,
        }
    }

    fn generate_pdf_content(&self) -> Result<String> {
        let mut content = String::new();
        let statistics = self.calculate_statistics();

        // Header
        content.push_str("NOTES EXPORT\n");
        content.push_str("============\n\n");

        // Statistics
        content.push_str("NOTES STATISTICS\n");
        content.push_str("----------------\n");
        content.push_str(&format!("Total Notes: {}\n", statistics.total_notes));
        content.push_str(&format!("Course-level Notes: {}\n", statistics.course_level_notes));
        content.push_str(&format!("Video-level Notes: {}\n", statistics.video_level_notes));
        content.push_str(&format!("Notes with Timestamps: {}\n", statistics.notes_with_timestamps));
        content.push_str(&format!(
            "Average Note Length: {:.1} characters\n",
            statistics.average_note_length
        ));
        content.push_str(&format!("Unique Tags: {}\n", statistics.unique_tags.len()));

        if !statistics.unique_tags.is_empty() {
            content.push_str(&format!("Tags: {}\n", statistics.unique_tags.join(", ")));
        }
        content.push('\n');

        // Group notes by course
        let mut notes_by_course: HashMap<uuid::Uuid, Vec<&Note>> = HashMap::new();
        for note in *self {
            notes_by_course.entry(note.course_id).or_default().push(note);
        }

        // Export notes grouped by course
        for (course_id, course_notes) in notes_by_course {
            content.push_str(&format!("COURSE: {course_id}\n"));
            content.push_str(&format!("{}\n", "=".repeat(50)));

            // Sort notes by creation date
            let mut sorted_notes = course_notes;
            sorted_notes.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            for note in sorted_notes {
                content.push_str(&format!("Note ID: {}\n", note.id));
                content
                    .push_str(&format!("Created: {}\n", utils::format_timestamp(note.created_at)));
                content
                    .push_str(&format!("Updated: {}\n", utils::format_timestamp(note.updated_at)));

                if let Some(video_id) = note.video_id {
                    content.push_str(&format!("Video ID: {video_id}\n"));
                }

                if let Some(timestamp) = note.timestamp {
                    let duration = std::time::Duration::from_secs(timestamp as u64);
                    content.push_str(&format!(
                        "Timestamp: {} ({})\n",
                        timestamp,
                        utils::format_duration(duration)
                    ));
                }

                if !note.tags.is_empty() {
                    content.push_str(&format!("Tags: {}\n", note.tags.join(", ")));
                }

                content.push_str("\nContent:\n");
                content.push_str(&format!("{}\n", note.content));
                content.push_str(&format!("{}\n\n", "-".repeat(40)));
            }

            content.push('\n');
        }

        Ok(content)
    }

    fn export_markdown(&self) -> Result<String> {
        let mut markdown = String::new();
        let statistics = self.calculate_statistics();

        // Header
        markdown.push_str("# Notes Export\n\n");

        // Statistics
        markdown.push_str("## Statistics\n\n");
        markdown.push_str(&format!("- **Total Notes:** {}\n", statistics.total_notes));
        markdown
            .push_str(&format!("- **Course-level Notes:** {}\n", statistics.course_level_notes));
        markdown.push_str(&format!("- **Video-level Notes:** {}\n", statistics.video_level_notes));
        markdown.push_str(&format!(
            "- **Notes with Timestamps:** {}\n",
            statistics.notes_with_timestamps
        ));
        markdown.push_str(&format!(
            "- **Average Note Length:** {:.1} characters\n",
            statistics.average_note_length
        ));

        if !statistics.unique_tags.is_empty() {
            markdown.push_str(&format!("- **Tags:** {}\n", statistics.unique_tags.join(", ")));
        }
        markdown.push('\n');

        // Group notes by course
        let mut notes_by_course: HashMap<uuid::Uuid, Vec<&Note>> = HashMap::new();
        for note in *self {
            notes_by_course.entry(note.course_id).or_default().push(note);
        }

        // Export notes grouped by course
        for (course_id, course_notes) in notes_by_course {
            markdown.push_str(&format!("## Course: {course_id}\n\n"));

            // Sort notes by creation date
            let mut sorted_notes = course_notes;
            sorted_notes.sort_by(|a, b| a.created_at.cmp(&b.created_at));

            for note in sorted_notes {
                markdown.push_str(&format!("### Note: {}\n\n", note.id));

                // Metadata table
                markdown.push_str("| Field | Value |\n");
                markdown.push_str("|-------|-------|\n");
                markdown.push_str(&format!(
                    "| Created | {} |\n",
                    utils::format_timestamp(note.created_at)
                ));
                markdown.push_str(&format!(
                    "| Updated | {} |\n",
                    utils::format_timestamp(note.updated_at)
                ));

                if let Some(video_id) = note.video_id {
                    markdown.push_str(&format!("| Video ID | {video_id} |\n"));
                }

                if let Some(timestamp) = note.timestamp {
                    let duration = std::time::Duration::from_secs(timestamp as u64);
                    markdown.push_str(&format!(
                        "| Timestamp | {} ({}) |\n",
                        timestamp,
                        utils::format_duration(duration)
                    ));
                }

                if !note.tags.is_empty() {
                    markdown.push_str(&format!("| Tags | {} |\n", note.tags.join(", ")));
                }

                markdown.push('\n');

                // Content
                markdown.push_str("**Content:**\n\n");
                markdown.push_str(&format!("{}\n\n", note.content));
                markdown.push_str("---\n\n");
            }
        }

        Ok(markdown)
    }
}

/// Notes-specific export utilities
pub mod notes_utils {
    use super::NotesExportExtensions;
    use super::*;

    /// Export notes filtered by tags
    pub fn export_notes_by_tags(
        notes: &[Note],
        tags: &[String],
        options: ExportOptions,
    ) -> Result<ExportResult> {
        let filtered_notes: Vec<Note> = notes
            .iter()
            .filter(|note| tags.iter().any(|tag| note.tags.contains(tag)))
            .cloned()
            .collect();

        if filtered_notes.is_empty() {
            return Err(anyhow::anyhow!(
                "No notes found with the specified tags: {}",
                tags.join(", ")
            ));
        }

        filtered_notes.export_with_options(options)
    }

    /// Export notes within a date range
    pub fn export_notes_by_date_range(
        notes: &[Note],
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
        options: ExportOptions,
    ) -> Result<ExportResult> {
        let filtered_notes: Vec<Note> = notes
            .iter()
            .filter(|note| note.created_at >= start_date && note.created_at <= end_date)
            .cloned()
            .collect();

        if filtered_notes.is_empty() {
            return Err(anyhow::anyhow!("No notes found in the specified date range"));
        }

        filtered_notes.export_with_options(options)
    }

    /// Export notes with timestamps for video synchronization
    pub fn export_timestamped_notes(
        notes: &[Note],
        options: ExportOptions,
    ) -> Result<ExportResult> {
        let timestamped_notes: Vec<Note> =
            notes.iter().filter(|note| note.timestamp.is_some()).cloned().collect();

        if timestamped_notes.is_empty() {
            return Err(anyhow::anyhow!("No timestamped notes found"));
        }

        timestamped_notes.export_with_options(options)
    }

    /// Generate notes summary report
    pub fn generate_notes_summary(notes: &[Note]) -> Result<String> {
        let statistics = notes.calculate_statistics();
        let mut summary = String::new();

        summary.push_str("NOTES SUMMARY REPORT\n");
        summary.push_str("====================\n\n");

        summary
            .push_str(&format!("Generated: {}\n\n", utils::format_timestamp(chrono::Utc::now())));

        // Overview
        summary.push_str("OVERVIEW\n");
        summary.push_str("--------\n");
        summary.push_str(&format!("Total Notes: {}\n", statistics.total_notes));
        summary.push_str(&format!("Course-level Notes: {}\n", statistics.course_level_notes));
        summary.push_str(&format!("Video-level Notes: {}\n", statistics.video_level_notes));
        summary.push_str(&format!("Notes with Timestamps: {}\n", statistics.notes_with_timestamps));
        summary.push_str(&format!(
            "Average Note Length: {:.1} characters\n\n",
            statistics.average_note_length
        ));

        // Tags analysis
        if !statistics.unique_tags.is_empty() {
            summary.push_str("TAGS ANALYSIS\n");
            summary.push_str("-------------\n");
            summary.push_str(&format!("Total Unique Tags: {}\n", statistics.unique_tags.len()));
            summary.push_str("Tags: ");
            summary.push_str(&statistics.unique_tags.join(", "));
            summary.push_str("\n\n");
        }

        // Course distribution
        if !statistics.notes_by_course.is_empty() {
            summary.push_str("COURSE DISTRIBUTION\n");
            summary.push_str("-------------------\n");
            for (course_id, count) in &statistics.notes_by_course {
                summary.push_str(&format!("{course_id}: {count} notes\n"));
            }
            summary.push('\n');
        }

        Ok(summary)
    }
}
