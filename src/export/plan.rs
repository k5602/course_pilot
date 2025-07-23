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

            let video_indices_str = item
                .video_indices
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(";");

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
        let content = self.generate_pdf_content()?;

        // Simple PDF structure (placeholder implementation)
        let mut pdf_content = Vec::new();
        pdf_content.extend_from_slice(b"%PDF-1.4\n");
        pdf_content.extend_from_slice(content.as_bytes());

        Ok(pdf_content)
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
            }
            ExportFormat::Csv => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating CSV schedule...".to_string());
                }
                self.export_csv()?.into_bytes()
            }
            ExportFormat::Pdf => {
                if let Some(ref callback) = options.progress_callback {
                    callback(25.0, "Generating PDF schedule...".to_string());
                }
                self.export_pdf()?
            }
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

    /// Generate formatted content for PDF export
    fn generate_pdf_content(&self) -> Result<String> {
        let mut content = String::new();
        let progress_summary = self.calculate_progress_summary();

        // Header
        content.push_str("STUDY PLAN EXPORT\n");
        content.push_str("=================\n\n");

        // Plan metadata
        content.push_str(&format!("Plan ID: {}\n", self.id));
        content.push_str(&format!("Course ID: {}\n", self.course_id));
        content.push_str(&format!(
            "Created: {}\n",
            utils::format_timestamp(self.created_at)
        ));
        content.push_str(&format!(
            "Exported: {}\n\n",
            utils::format_timestamp(chrono::Utc::now())
        ));

        // Settings
        content.push_str("PLAN SETTINGS\n");
        content.push_str("-------------\n");
        content.push_str(&format!(
            "Start Date: {}\n",
            self.settings.start_date.format("%Y-%m-%d")
        ));
        content.push_str(&format!(
            "Sessions per Week: {}\n",
            self.settings.sessions_per_week
        ));
        content.push_str(&format!(
            "Session Length: {} minutes\n",
            self.settings.session_length_minutes
        ));
        content.push_str(&format!(
            "Include Weekends: {}\n\n",
            if self.settings.include_weekends {
                "Yes"
            } else {
                "No"
            }
        ));

        // Progress summary
        content.push_str("PROGRESS SUMMARY\n");
        content.push_str("----------------\n");
        content.push_str(&format!(
            "Total Sessions: {}\n",
            progress_summary.total_sessions
        ));
        content.push_str(&format!(
            "Completed Sessions: {}\n",
            progress_summary.completed_sessions
        ));
        content.push_str(&format!(
            "Progress: {:.1}%\n",
            progress_summary.progress_percentage
        ));

        if let Some(completion_date) = progress_summary.estimated_completion_date {
            content.push_str(&format!(
                "Estimated Completion: {}\n",
                completion_date.format("%Y-%m-%d")
            ));
        }
        content.push('\n');

        // Schedule
        content.push_str("STUDY SCHEDULE\n");
        content.push_str("==============\n\n");

        let mut current_week = 0;
        let start_date = if let Some(first_item) = self.items.first() {
            first_item.date
        } else {
            self.created_at
        };

        for (index, item) in self.items.iter().enumerate() {
            let days_from_start = (item.date - start_date).num_days();
            let week_number = (days_from_start / 7) + 1;

            if week_number != current_week {
                current_week = week_number;
                content.push_str(&format!("Week {week_number}\n"));
                content.push_str("------\n");
            }

            let status_icon = if item.completed { "✓" } else { "○" };
            let video_indices = item
                .video_indices
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            content.push_str(&format!(
                "{} Session {} - {} ({})\n",
                status_icon,
                index + 1,
                item.date.format("%Y-%m-%d %a"),
                item.date.format("%H:%M")
            ));
            content.push_str(&format!("   Module: {}\n", item.module_title));
            content.push_str(&format!("   Section: {}\n", item.section_title));
            if !video_indices.is_empty() {
                content.push_str(&format!("   Videos: {video_indices}\n"));
            }
            content.push('\n');
        }

        Ok(content)
    }
}

/// Plan-specific export utilities
pub mod plan_utils {
    use super::*;

    /// Export plan with calendar format for external calendar applications
    pub fn export_plan_calendar(plan: &Plan) -> Result<String> {
        let mut ical_content = String::new();

        // iCalendar header
        ical_content.push_str("BEGIN:VCALENDAR\n");
        ical_content.push_str("VERSION:2.0\n");
        ical_content.push_str("PRODID:-//Course Pilot//Study Plan//EN\n");
        ical_content.push_str("CALSCALE:GREGORIAN\n");

        // Add events for each plan item
        for (index, item) in plan.items.iter().enumerate() {
            let event_id = format!("{}_{}", plan.id, index);
            let start_time = item.date.format("%Y%m%dT%H%M%SZ");
            let end_time = (item.date
                + chrono::Duration::minutes(plan.settings.session_length_minutes as i64))
            .format("%Y%m%dT%H%M%SZ");

            ical_content.push_str("BEGIN:VEVENT\n");
            ical_content.push_str(&format!("UID:{event_id}\n"));
            ical_content.push_str(&format!("DTSTART:{start_time}\n"));
            ical_content.push_str(&format!("DTEND:{end_time}\n"));
            ical_content.push_str(&format!("SUMMARY:Study Session: {}\n", item.section_title));
            ical_content.push_str(&format!(
                "DESCRIPTION:Module: {}\\nSection: {}\n",
                item.module_title, item.section_title
            ));
            ical_content.push_str("STATUS:CONFIRMED\n");
            ical_content.push_str("END:VEVENT\n");
        }

        ical_content.push_str("END:VCALENDAR\n");

        Ok(ical_content)
    }

    /// Generate study statistics for plan analysis
    pub fn generate_plan_statistics(plan: &Plan) -> Result<String> {
        let progress_summary = plan.calculate_progress_summary();
        let mut stats = String::new();

        stats.push_str("STUDY PLAN STATISTICS\n");
        stats.push_str("=====================\n\n");

        // Basic statistics
        stats.push_str(&format!(
            "Total Sessions Planned: {}\n",
            progress_summary.total_sessions
        ));
        stats.push_str(&format!(
            "Sessions Completed: {}\n",
            progress_summary.completed_sessions
        ));
        stats.push_str(&format!(
            "Completion Rate: {:.1}%\n",
            progress_summary.progress_percentage
        ));

        // Time analysis
        let total_planned_minutes =
            progress_summary.total_sessions as u32 * progress_summary.average_session_length;
        let completed_minutes =
            progress_summary.completed_sessions as u32 * progress_summary.average_session_length;

        stats.push_str(&format!(
            "Total Planned Time: {} hours\n",
            total_planned_minutes / 60
        ));
        stats.push_str(&format!(
            "Time Completed: {} hours\n",
            completed_minutes / 60
        ));
        stats.push_str(&format!(
            "Time Remaining: {} hours\n",
            (total_planned_minutes - completed_minutes) / 60
        ));

        // Weekly analysis
        stats.push_str(&format!(
            "Sessions per Week: {}\n",
            progress_summary.sessions_per_week
        ));
        stats.push_str(&format!(
            "Weekly Time Commitment: {} hours\n",
            (progress_summary.sessions_per_week as u32 * progress_summary.average_session_length)
                / 60
        ));

        if let Some(completion_date) = progress_summary.estimated_completion_date {
            stats.push_str(&format!(
                "Estimated Completion: {}\n",
                completion_date.format("%Y-%m-%d")
            ));
        }

        Ok(stats)
    }
}
