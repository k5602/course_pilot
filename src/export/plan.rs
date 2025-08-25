use super::*;
use crate::types::Plan;
use serde_json;

/// Extended plan data for export with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExportData {
    pub plan: Plan,
    pub progress_summary: PlanProgressSummary,
    pub export_metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanProgressSummary {
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub progress_percentage: f32,
    pub estimated_completion_date: Option<chrono::DateTime<chrono::Utc>>,
    pub sessions_per_week: u8,
    pub average_session_length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub export_version: String,
    pub plan_type: String,
}

impl Exportable for Plan {
    fn export_json(&self) -> Result<String> {
        let progress_summary = self.calculate_progress_summary();

        let export_data = PlanExportData {
            plan: self.clone(),
            progress_summary,
            export_metadata: ExportMetadata {
                exported_at: chrono::Utc::now(),
                export_version: "1.0".to_string(),
                plan_type: "Study Plan".to_string(),
            },
        };

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize plan to JSON: {}", e))
    }

    fn export_csv(&self) -> Result<String> {
        let mut csv_data = String::new();

        // CSV Header
        csv_data
            .push_str("Date,Module,Section,Video_Indices,Completed,Session_Number,Week_Number\n");

        // Calculate week numbers for better organization
        let start_date = if let Some(first_item) = self.items.first() {
            first_item.date
        } else {
            self.created_at
        };

        for (index, item) in self.items.iter().enumerate() {
            let days_from_start = (item.date - start_date).num_days();
            let week_number = (days_from_start / 7) + 1;
            let session_number = index + 1;

            let video_indices_str =
                item.video_indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");

            csv_data.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                item.date.format("%Y-%m-%d"),
                utils::sanitize_csv_field(&item.module_title),
                utils::sanitize_csv_field(&item.section_title),
                utils::sanitize_csv_field(&video_indices_str),
                if item.completed { "Yes" } else { "No" },
                session_number,
                week_number
            ));
        }

        Ok(csv_data)
    }

    fn export_pdf(&self) -> Result<Vec<u8>> {
        // Build a simple PDF using genpdf without requiring external font files
        let font_family = genpdf::fonts::from_files("./fonts", "LiberationSans", None)
            .map_err(|e| anyhow::anyhow!("Failed to load font family: {}", e))?;
        let mut doc = genpdf::Document::new(font_family);
        doc.set_title(format!("Study Plan Export: {}", self.id));

        // Page decorator with margins
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        doc.set_page_decorator(decorator);

        // Header
        doc.push(genpdf::elements::Paragraph::new("Study Plan"));
        doc.push(genpdf::elements::Paragraph::new(format!("Plan ID: {}", self.id)));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "Created At: {}",
            crate::export::utils::format_timestamp(self.created_at)
        )));
        doc.push(genpdf::elements::Paragraph::new(" "));

        // Progress summary
        let summary = self.calculate_progress_summary();
        doc.push(genpdf::elements::Paragraph::new("Progress Summary:"));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "• Total Sessions: {}",
            summary.total_sessions
        )));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "• Completed Sessions: {}",
            summary.completed_sessions
        )));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "• Progress: {:.1}%",
            summary.progress_percentage
        )));
        if let Some(date) = summary.estimated_completion_date {
            doc.push(genpdf::elements::Paragraph::new(format!(
                "• Estimated Completion: {}",
                crate::export::utils::format_timestamp(date)
            )));
        }
        doc.push(genpdf::elements::Paragraph::new(format!(
            "• Sessions/Week: {}",
            summary.sessions_per_week
        )));
        doc.push(genpdf::elements::Paragraph::new(format!(
            "• Avg Session Length: {} min",
            summary.average_session_length
        )));
        doc.push(genpdf::elements::Paragraph::new(" "));

        // Sessions detail
        doc.push(genpdf::elements::Paragraph::new("Sessions:"));
        for (index, item) in self.items.iter().enumerate() {
            let indices_str =
                item.video_indices.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",");
            doc.push(genpdf::elements::Paragraph::new(format!(
                "• {:>3}. {} | {} | Videos [{}] | Date: {} | {}",
                index + 1,
                item.module_title,
                item.section_title,
                indices_str,
                item.date.format("%Y-%m-%d"),
                if item.completed { "Completed" } else { "Pending" }
            )));
        }

        // Render to bytes
        let mut bytes = Vec::new();
        doc.render(&mut bytes).map_err(|e| anyhow::anyhow!("Failed to render PDF: {}", e))?;
        Ok(bytes)
    }

    fn export_with_options(&self, options: ExportOptions) -> Result<ExportResult> {
        if let Some(ref callback) = options.progress_callback {
            callback(0.0, "Starting plan export...".to_string());
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
                    callback(25.0, "Generating CSV schedule...".to_string());
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
            callback(100.0, "Plan export completed successfully".to_string());
        }

        Ok(ExportResult {
            filename: self.get_export_filename(options.format),
            size_bytes: data.len(),
            format: options.format,
            data,
        })
    }

    fn get_export_filename(&self, format: ExportFormat) -> String {
        let plan_name = format!("plan_{}", self.id);
        utils::generate_filename(&plan_name, format)
    }
}

impl Plan {
    /// Calculate comprehensive progress summary for export
    fn calculate_progress_summary(&self) -> PlanProgressSummary {
        let total_sessions = self.items.len();
        let completed_sessions = self.items.iter().filter(|item| item.completed).count();
        let progress_percentage = if total_sessions > 0 {
            (completed_sessions as f32 / total_sessions as f32) * 100.0
        } else {
            0.0
        };

        // Estimate completion date based on current progress and settings
        let estimated_completion_date = if completed_sessions < total_sessions {
            let remaining_sessions = total_sessions - completed_sessions;
            let sessions_per_week = self.settings.sessions_per_week as f32;
            let weeks_remaining = (remaining_sessions as f32 / sessions_per_week).ceil();
            let days_remaining = (weeks_remaining * 7.0) as i64;

            Some(chrono::Utc::now() + chrono::Duration::days(days_remaining))
        } else {
            None
        };

        PlanProgressSummary {
            total_sessions,
            completed_sessions,
            progress_percentage,
            estimated_completion_date,
            sessions_per_week: self.settings.sessions_per_week,
            average_session_length: self.settings.session_length_minutes,
        }
    }
}
