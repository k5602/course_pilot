use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Export format enumeration supporting JSON, CSV, and PDF formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Pdf,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "JSON"),
            ExportFormat::Csv => write!(f, "CSV"),
            ExportFormat::Pdf => write!(f, "PDF"),
        }
    }
}

/// Progress callback function type for export operations
pub type ProgressCallback = Box<dyn Fn(f32, String) + Send + Sync>;

/// Export options for customizing export behavior
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_progress: bool,
    pub include_timestamps: bool,
    pub progress_callback: Option<ProgressCallback>,
}

impl std::fmt::Debug for ExportOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportOptions")
            .field("format", &self.format)
            .field("include_metadata", &self.include_metadata)
            .field("include_progress", &self.include_progress)
            .field("include_timestamps", &self.include_timestamps)
            .field("progress_callback", &self.progress_callback.is_some())
            .finish()
    }
}

impl Clone for ExportOptions {
    fn clone(&self) -> Self {
        Self {
            format: self.format,
            include_metadata: self.include_metadata,
            include_progress: self.include_progress,
            include_timestamps: self.include_timestamps,
            progress_callback: None, // Cannot clone function pointers
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_progress: true,
            include_timestamps: true,
            progress_callback: None,
        }
    }
}

/// Export result containing the exported data and metadata
#[derive(Debug, Clone)]
pub struct ExportResult {
    pub data: Vec<u8>,
    pub format: ExportFormat,
    pub filename: String,
    pub size_bytes: usize,
}

/// Trait for types that can be exported to different formats
pub trait Exportable {
    /// Export to JSON format with complete data structure
    fn export_json(&self) -> Result<String>;

    /// Export to CSV format with tabular representation
    fn export_csv(&self) -> Result<String>;

    /// Export to PDF format with formatted document
    fn export_pdf(&self) -> Result<Vec<u8>>;

    /// Export with custom options and progress tracking
    fn export_with_options(&self, options: ExportOptions) -> Result<ExportResult>;

    /// Get suggested filename for export
    fn get_export_filename(&self, format: ExportFormat) -> String;
}

/// Export utility functions for format conversion and file generation
pub mod utils {
    use super::*;
    use chrono::{DateTime, Utc};

    /// Format duration for human-readable display
    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{hours}h {minutes}m {seconds}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds}s")
        } else {
            format!("{seconds}s")
        }
    }

    /// Format timestamp for export
    pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
        timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Sanitize string for CSV export (escape quotes and commas)
    pub fn sanitize_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Generate unique filename with timestamp
    pub fn generate_filename(base_name: &str, format: ExportFormat) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let extension = match format {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Pdf => "pdf",
        };
        format!("{base_name}_{timestamp}.{extension}")
    }

    /// Validate export data for corruption
    pub fn validate_export_data(data: &[u8], format: ExportFormat) -> Result<()> {
        match format {
            ExportFormat::Json => {
                // Validate JSON structure
                serde_json::from_slice::<serde_json::Value>(data)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON export data: {}", e))?;
            }
            ExportFormat::Csv => {
                // Basic CSV validation - check for valid UTF-8
                std::str::from_utf8(data)
                    .map_err(|e| anyhow::anyhow!("Invalid CSV export data: {}", e))?;
            }
            ExportFormat::Pdf => {
                // Basic PDF validation - check for PDF header
                if !data.starts_with(b"%PDF-") {
                    return Err(anyhow::anyhow!(
                        "Invalid PDF export data: missing PDF header"
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Error types specific to export operations
#[derive(thiserror::Error, Debug)]
pub enum ExportError {
    #[error("Export format not supported: {format}")]
    UnsupportedFormat { format: String },

    #[error("Export data validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Export operation was cancelled")]
    Cancelled,

    #[error("Export failed due to insufficient data: {details}")]
    InsufficientData { details: String },

    #[error("PDF generation failed: {reason}")]
    PdfGenerationFailed { reason: String },

    #[error("CSV generation failed: {reason}")]
    CsvGenerationFailed { reason: String },
}

pub mod course;
pub mod notes;
pub mod plan;
pub mod io;
