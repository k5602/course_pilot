use super::{ExportFormat, ExportOptions, ExportResult, Exportable, utils};
use crate::types::Course;
use anyhow::Result;
use csv::Writer;
use serde::{Deserialize, Serialize};
use serde_json;

/// Extended course data for export with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseExportData {
    pub course: Course,
    pub progress_info: Option<ProgressInfo>,
    pub export_metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub completed_count: usize,
    pub total_count: usize,
    pub percentage: f32,
    pub estimated_time_remaining: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub export_version: String,
    pub total_videos: usize,
    pub total_duration: Option<std::time::Duration>,
}

impl Exportable for Course {
    fn export_json(&self) -> Result<String> {
        let export_data = CourseExportData {
            course: self.clone(),
            progress_info: None, // Will be populated by backend when available
            export_metadata: ExportMetadata {
                exported_at: chrono::Utc::now(),
                export_version: "1.0".to_string(),
                total_videos: self.video_count(),
                total_duration: self
                    .structure
                    .as_ref()
                    .map(|s| s.aggregate_total_duration()),
            },
        };

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize course to JSON: {}", e))
    }

    fn export_csv(&self) -> Result<String> {
        let mut buffer = Vec::new();
        {
            let mut writer = Writer::from_writer(&mut buffer);

            // Write CSV header
            writer
                .write_record([
                    "Type",
                    "Title",
                    "Module",
                    "Section",
                    "Duration",
                    "Video_Index",
                    "Created_At",
                ])
                .map_err(|e| anyhow::anyhow!("Failed to write CSV header: {}", e))?;

            // Course header row
            writer
                .write_record([
                    "Course",
                    &self.name,
                    "",
                    "",
                    "",
                    "",
                    &utils::format_timestamp(self.created_at),
                ])
                .map_err(|e| anyhow::anyhow!("Failed to write course header: {}", e))?;

            // If structured, export modules and sections
            if let Some(ref structure) = self.structure {
                for module in &structure.modules {
                    // Module row
                    writer
                        .write_record([
                            "Module",
                            &module.title,
                            &module.title,
                            "",
                            &utils::format_duration(module.total_duration),
                            "",
                            "",
                        ])
                        .map_err(|e| anyhow::anyhow!("Failed to write module row: {}", e))?;

                    // Section rows
                    for section in &module.sections {
                        writer
                            .write_record([
                                "Section",
                                &section.title,
                                &module.title,
                                &section.title,
                                &utils::format_duration(section.duration),
                                &section.video_index.to_string(),
                                "",
                            ])
                            .map_err(|e| anyhow::anyhow!("Failed to write section row: {}", e))?;
                    }
                }
            } else {
                // If unstructured, export raw titles
                for (index, title) in self.raw_titles.iter().enumerate() {
                    writer
                        .write_record(["Video", title, "", "", "", &index.to_string(), ""])
                        .map_err(|e| anyhow::anyhow!("Failed to write video row: {}", e))?;
                }
            }

            writer
                .flush()
                .map_err(|e| anyhow::anyhow!("Failed to flush CSV writer: {}", e))?;
        }

        String::from_utf8(buffer)
            .map_err(|e| anyhow::anyhow!("Failed to convert CSV to UTF-8: {}", e))
    }

    fn export_pdf(&self) -> Result<Vec<u8>> {
        // Build a simple PDF using genpdf without requiring external font files
        let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
            .map_err(|e| anyhow::anyhow!("Failed to load font family: {}", e))?;
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title(format!("Course Export: {}", self.name));

        // Page decorator with margins
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        // Header
        doc.push(genpdf::elements::Paragraph::new(format!(
            "Course: {}",
            self.name
        )));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "Created At: {}",
            crate::export::utils::format_timestamp(self.created_at)
        )));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "Total Videos: {}",
            self.video_count()
        )));
        doc.push(genpdf::elements::Paragraph::new(" "));

        // Content
        if let Some(structure) = &self.structure {
            doc.push(genpdf::elements::Paragraph::new("Structure:"));
            for module in &structure.modules {
                doc.push(genpdf::elements::Paragraph::new(format!(
                    "• Module: {} (Duration: {})",
                    module.title,
                    crate::export::utils::format_duration(module.total_duration)
                )));
                for section in &module.sections {
                    doc.push(genpdf::elements::Paragraph::new(format!(
                        "    - {} (Video #{}) [{}]",
                        section.title,
                        section.video_index,
                        crate::export::utils::format_duration(section.duration)
                    )));
                }
            }
        } else {
            doc.push(genpdf::elements::Paragraph::new(
                "Raw Video Titles (Unstructured):",
            ));
            for (idx, title) in self.raw_titles.iter().enumerate() {
                doc.push(genpdf::elements::Paragraph::new(format!(
                    "• {:>3}. {}",
                    idx, title
                )));
            }
        }

        // Render to bytes
        let mut bytes = Vec::new();
        doc.render(&mut bytes)
            .map_err(|e| anyhow::anyhow!("Failed to render PDF: {}", e))?;
        Ok(bytes)
    }

    fn export_with_options(&self, options: ExportOptions) -> Result<ExportResult> {
        if let Some(ref callback) = options.progress_callback {
            callback(0.0, "Starting course export...".to_string());
        }

        let data = match options.format {
            ExportFormat::Json => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating JSON data...".to_string());
                }
                self.export_json()?.into_bytes()
            }
            ExportFormat::Csv => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating CSV data...".to_string());
                }
                self.export_csv()?.into_bytes()
            }
            ExportFormat::Pdf => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating PDF document...".to_string());
                }
                self.export_pdf()?
            }
        };

        if let Some(ref callback) = options.progress_callback {
            callback(75.0, "Validating export data...".to_string());
        }

        // Validate the exported data
        utils::validate_export_data(&data, options.format)?;

        if let Some(ref callback) = options.progress_callback {
            callback(100.0, "Export completed successfully".to_string());
        }

        Ok(ExportResult {
            filename: self.get_export_filename(options.format),
            size_bytes: data.len(),
            format: options.format,
            data,
        })
    }

    fn get_export_filename(&self, format: ExportFormat) -> String {
        let safe_name = self
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .replace(' ', "_");
        utils::generate_filename(&format!("course_{safe_name}"), format)
    }
}

impl Course {}

/// Course-specific export utilities
pub mod course_utils {
    use super::*;

    /// Export multiple courses to a single file
    pub fn export_courses_batch(
        courses: &[Course],
        options: ExportOptions,
    ) -> Result<ExportResult> {
        if let Some(ref callback) = options.progress_callback {
            callback(0.0, "Starting batch course export...".to_string());
        }

        let data = match options.format {
            ExportFormat::Json => {
                let export_data: Vec<CourseExportData> = courses
                    .iter()
                    .map(|course| CourseExportData {
                        course: course.clone(),
                        progress_info: None,
                        export_metadata: ExportMetadata {
                            exported_at: chrono::Utc::now(),
                            export_version: "1.0".to_string(),
                            total_videos: course.video_count(),
                            total_duration: course
                                .structure
                                .as_ref()
                                .map(|s| s.aggregate_total_duration()),
                        },
                    })
                    .collect();

                serde_json::to_string_pretty(&export_data)?.into_bytes()
            }
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                csv_data.push_str(
                    "Course_Name,Type,Title,Module,Section,Duration,Video_Index,Created_At\n",
                );

                for course in courses {
                    let course_csv = course.export_csv()?;
                    // Skip header line and add course name to each row
                    for line in course_csv.lines().skip(1) {
                        csv_data.push_str(&format!(
                            "{},{}\n",
                            utils::sanitize_csv_field(&course.name),
                            line
                        ));
                    }
                }

                csv_data.into_bytes()
            }
            ExportFormat::Pdf => {
                // Build a batch PDF containing all courses
                let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
                    .map_err(|e| anyhow::anyhow!("Failed to load font family: {}", e))?;
                let mut doc = genpdf::Document::new(font_family);
                doc.set_title("Courses Batch Export".to_string());

                let mut decorator = genpdf::SimplePageDecorator::new();
                decorator.set_margins(10);
                doc.set_page_decorator(decorator);

                // Title page
                doc.push(genpdf::elements::Paragraph::new("Courses Batch Export"));
                doc.push(genpdf::elements::Paragraph::new(format!(
                    "Generated: {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                )));
                doc.push(genpdf::elements::Paragraph::new(" "));

                for (i, course) in courses.iter().enumerate() {
                    doc.push(genpdf::elements::Paragraph::new(format!(
                        "==================== Course {} ====================",
                        i + 1
                    )));
                    doc.push(genpdf::elements::Paragraph::new(format!(
                        "Name: {}",
                        course.name
                    )));
                    doc.push(genpdf::elements::Paragraph::new(format!(
                        "Created At: {}",
                        crate::export::utils::format_timestamp(course.created_at)
                    )));
                    doc.push(genpdf::elements::Paragraph::new(format!(
                        "Total Videos: {}",
                        course.video_count()
                    )));
                    doc.push(genpdf::elements::Paragraph::new(" "));

                    if let Some(structure) = &course.structure {
                        doc.push(genpdf::elements::Paragraph::new("Structure:"));
                        for module in &structure.modules {
                            doc.push(genpdf::elements::Paragraph::new(format!(
                                "• Module: {} (Duration: {})",
                                module.title,
                                crate::export::utils::format_duration(module.total_duration)
                            )));
                            for section in &module.sections {
                                doc.push(genpdf::elements::Paragraph::new(format!(
                                    "    - {} (Video #{}) [{}]",
                                    section.title,
                                    section.video_index,
                                    crate::export::utils::format_duration(section.duration)
                                )));
                            }
                        }
                    } else {
                        doc.push(genpdf::elements::Paragraph::new(
                            "Raw Video Titles (Unstructured):",
                        ));
                        for (idx, title) in course.raw_titles.iter().enumerate() {
                            doc.push(genpdf::elements::Paragraph::new(format!(
                                "• {:>3}. {}",
                                idx, title
                            )));
                        }
                    }

                    doc.push(genpdf::elements::Paragraph::new(" "));
                }

                let mut data = Vec::new();
                doc.render(&mut data)
                    .map_err(|e| anyhow::anyhow!("Failed to render PDF: {}", e))?;
                data
            }
        };

        if let Some(ref callback) = options.progress_callback {
            callback(100.0, "Batch export completed successfully".to_string());
        }

        Ok(ExportResult {
            filename: utils::generate_filename("courses_batch", options.format),
            size_bytes: data.len(),
            format: options.format,
            data,
        })
    }
}
