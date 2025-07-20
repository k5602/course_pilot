use super::{ExportFormat, ExportOptions, ExportResult, Exportable, utils};
use crate::types::Course;
use serde::{Serialize, Deserialize};
use serde_json;
use csv::Writer;
use printpdf::{PdfDocument, Mm};
use anyhow::Result;

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
                total_duration: self.structure.as_ref().map(|s| s.aggregate_total_duration()),
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
            writer.write_record(&["Type", "Title", "Module", "Section", "Duration", "Video_Index", "Created_At"])
                .map_err(|e| anyhow::anyhow!("Failed to write CSV header: {}", e))?;
            
            // Course header row
            writer.write_record(&[
                "Course",
                &self.name,
                "",
                "",
                "",
                "",
                &utils::format_timestamp(self.created_at)
            ]).map_err(|e| anyhow::anyhow!("Failed to write course header: {}", e))?;
            
            // If structured, export modules and sections
            if let Some(ref structure) = self.structure {
                for module in &structure.modules {
                    // Module row
                    writer.write_record(&[
                        "Module",
                        &module.title,
                        &module.title,
                        "",
                        &utils::format_duration(module.total_duration),
                        "",
                        ""
                    ]).map_err(|e| anyhow::anyhow!("Failed to write module row: {}", e))?;
                    
                    // Section rows
                    for section in &module.sections {
                        writer.write_record(&[
                            "Section",
                            &section.title,
                            &module.title,
                            &section.title,
                            &utils::format_duration(section.duration),
                            &section.video_index.to_string(),
                            ""
                        ]).map_err(|e| anyhow::anyhow!("Failed to write section row: {}", e))?;
                    }
                }
            } else {
                // If unstructured, export raw titles
                for (index, title) in self.raw_titles.iter().enumerate() {
                    writer.write_record(&[
                        "Video",
                        title,
                        "",
                        "",
                        "",
                        &index.to_string(),
                        ""
                    ]).map_err(|e| anyhow::anyhow!("Failed to write video row: {}", e))?;
                }
            }
            
            writer.flush().map_err(|e| anyhow::anyhow!("Failed to flush CSV writer: {}", e))?;
        }
        
        String::from_utf8(buffer)
            .map_err(|e| anyhow::anyhow!("Failed to convert CSV to UTF-8: {}", e))
    }
    
    fn export_pdf(&self) -> Result<Vec<u8>> {
        // Create a simple PDF document using the new API
        let mut doc = PdfDocument::new(&self.name);
        
        // Create content for the page
        let mut page_contents = Vec::new();
        
        // Add title text
        page_contents.push(printpdf::Op::SetTextCursor { 
            pos: printpdf::Point { 
                x: Mm(20.0).into(), 
                y: Mm(270.0).into() 
            } 
        });
        
        // For now, create a simple text-based PDF content
        let _content = self.generate_pdf_content()?;
        
        // Create a simple page with the content as a marker (placeholder)
        page_contents.push(printpdf::Op::Marker { 
            id: format!("course-export-{}", self.id) 
        });
        
        // Create the page
        let page = printpdf::PdfPage::new(Mm(210.0), Mm(297.0), page_contents);
        
        // Save the document
        let mut warnings = Vec::new();
        let pdf_bytes = doc
            .with_pages(vec![page])
            .save(&printpdf::PdfSaveOptions::default(), &mut warnings);
        
        Ok(pdf_bytes)
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
        let safe_name = self.name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "_");
        utils::generate_filename(&format!("course_{}", safe_name), format)
    }
}

impl Course {
    /// Generate formatted content for PDF export
    fn generate_pdf_content(&self) -> Result<String> {
        let mut content = String::new();
        
        // Title
        content.push_str(&format!("COURSE EXPORT: {}\n", self.name.to_uppercase()));
        content.push_str(&format!("Created: {}\n", utils::format_timestamp(self.created_at)));
        content.push_str(&format!("Total Videos: {}\n\n", self.video_count()));
        
        if let Some(ref structure) = self.structure {
            content.push_str("COURSE STRUCTURE\n");
            content.push_str("================\n\n");
            
            let total_duration = structure.aggregate_total_duration();
            content.push_str(&format!("Total Duration: {}\n", utils::format_duration(total_duration)));
            content.push_str(&format!("Total Modules: {}\n\n", structure.modules.len()));
            
            for (module_idx, module) in structure.modules.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", module_idx + 1, module.title));
                content.push_str(&format!("   Duration: {}\n", utils::format_duration(module.total_duration)));
                content.push_str(&format!("   Sections: {}\n\n", module.sections.len()));
                
                for (section_idx, section) in module.sections.iter().enumerate() {
                    content.push_str(&format!("   {}.{} {} ({})\n", 
                        module_idx + 1, 
                        section_idx + 1, 
                        section.title,
                        utils::format_duration(section.duration)
                    ));
                }
                content.push_str("\n");
            }
        } else {
            content.push_str("RAW VIDEO TITLES\n");
            content.push_str("================\n\n");
            
            for (idx, title) in self.raw_titles.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", idx + 1, title));
            }
        }
        
        Ok(content)
    }
}

/// Course-specific export utilities
pub mod course_utils {
    use super::*;
    
    /// Export multiple courses to a single file
    pub fn export_courses_batch(courses: &[Course], options: ExportOptions) -> Result<ExportResult> {
        if let Some(ref callback) = options.progress_callback {
            callback(0.0, "Starting batch course export...".to_string());
        }
        
        let data = match options.format {
            ExportFormat::Json => {
                let export_data: Vec<CourseExportData> = courses.iter().map(|course| {
                    CourseExportData {
                        course: course.clone(),
                        progress_info: None,
                        export_metadata: ExportMetadata {
                            exported_at: chrono::Utc::now(),
                            export_version: "1.0".to_string(),
                            total_videos: course.video_count(),
                            total_duration: course.structure.as_ref().map(|s| s.aggregate_total_duration()),
                        },
                    }
                }).collect();
                
                serde_json::to_string_pretty(&export_data)?.into_bytes()
            }
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                csv_data.push_str("Course_Name,Type,Title,Module,Section,Duration,Video_Index,Created_At\n");
                
                for course in courses {
                    let course_csv = course.export_csv()?;
                    // Skip header line and add course name to each row
                    for line in course_csv.lines().skip(1) {
                        csv_data.push_str(&format!("{},{}\n", 
                            utils::sanitize_csv_field(&course.name), 
                            line
                        ));
                    }
                }
                
                csv_data.into_bytes()
            }
            ExportFormat::Pdf => {
                let mut content = String::new();
                content.push_str("COURSE COLLECTION EXPORT\n");
                content.push_str("========================\n\n");
                content.push_str(&format!("Total Courses: {}\n", courses.len()));
                content.push_str(&format!("Exported: {}\n\n", utils::format_timestamp(chrono::Utc::now())));
                
                for course in courses {
                    content.push_str(&course.generate_pdf_content()?);
                    content.push_str("\n---\n\n");
                }
                
                let mut pdf_content = Vec::new();
                pdf_content.extend_from_slice(b"%PDF-1.4\n");
                pdf_content.extend_from_slice(content.as_bytes());
                pdf_content
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