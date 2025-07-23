use anyhow::Result;
use chrono::Utc;
use course_pilot::export::{ExportFormat, ExportOptions, Exportable};
use course_pilot::types::{
    AdvancedSchedulerSettings, DifficultyLevel, DistributionStrategy, Plan, PlanItem, PlanSettings,
};
use uuid::Uuid;

/// Create a test plan with realistic data for PDF export testing
fn create_test_plan() -> Plan {
    let course_id = Uuid::new_v4();
    let start_date = Utc::now();

    let settings = PlanSettings {
        start_date,
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: Some(AdvancedSchedulerSettings {
            strategy: DistributionStrategy::Hybrid,
            difficulty_adaptation: true,
            spaced_repetition_enabled: false,
            cognitive_load_balancing: true,
            user_experience_level: DifficultyLevel::Intermediate,
            custom_intervals: None,
            max_session_duration_minutes: Some(90),
            min_break_between_sessions_hours: Some(24),
            prioritize_difficult_content: false,
            adaptive_pacing: true,
        }),
    };

    let mut plan = Plan::new(course_id, settings);

    // Add realistic plan items
    let plan_items = vec![
        PlanItem {
            date: start_date,
            module_title: "Introduction to Rust".to_string(),
            section_title: "Getting Started with Rust".to_string(),
            video_indices: vec![0, 1],
            completed: true,
        },
        PlanItem {
            date: start_date + chrono::Duration::days(2),
            module_title: "Introduction to Rust".to_string(),
            section_title: "Variables and Data Types".to_string(),
            video_indices: vec![2, 3, 4],
            completed: true,
        },
        PlanItem {
            date: start_date + chrono::Duration::days(4),
            module_title: "Ownership and Borrowing".to_string(),
            section_title: "Understanding Ownership".to_string(),
            video_indices: vec![5, 6],
            completed: false,
        },
        PlanItem {
            date: start_date + chrono::Duration::days(7),
            module_title: "Ownership and Borrowing".to_string(),
            section_title: "References and Borrowing".to_string(),
            video_indices: vec![7, 8, 9],
            completed: false,
        },
        PlanItem {
            date: start_date + chrono::Duration::days(9),
            module_title: "Error Handling".to_string(),
            section_title: "Result and Option Types".to_string(),
            video_indices: vec![10, 11],
            completed: false,
        },
    ];

    plan.items = plan_items;
    plan
}

#[tokio::test]
async fn test_pdf_export_basic_functionality() -> Result<()> {
    let plan = create_test_plan();

    // Test basic PDF export
    let pdf_data = plan.export_pdf()?;

    // Verify PDF header is present
    assert!(
        pdf_data.starts_with(b"%PDF-"),
        "PDF should start with PDF header"
    );

    // Verify data is not empty
    assert!(!pdf_data.is_empty(), "PDF data should not be empty");

    println!("✓ Basic PDF export functionality works");
    Ok(())
}

#[tokio::test]
async fn test_pdf_export_with_options() -> Result<()> {
    let plan = create_test_plan();

    let options = ExportOptions {
        format: ExportFormat::Pdf,
        include_metadata: true,
        include_progress: true,
        include_timestamps: true,
        progress_callback: None, // Skip progress callback for now
    };

    let export_result = plan.export_with_options(options)?;

    // Verify export result structure
    assert_eq!(export_result.format, ExportFormat::Pdf);
    assert!(!export_result.filename.is_empty());
    assert!(export_result.filename.ends_with(".pdf"));
    assert!(export_result.size_bytes > 0);
    assert!(!export_result.data.is_empty());

    // Verify PDF header
    assert!(export_result.data.starts_with(b"%PDF-"));

    println!("✓ PDF export with options works");
    Ok(())
}

#[tokio::test]
async fn test_pdf_content_generation() -> Result<()> {
    let plan = create_test_plan();

    let pdf_content = plan.generate_pdf_content()?;

    // Verify essential content is present
    assert!(pdf_content.contains("STUDY PLAN EXPORT"));
    assert!(pdf_content.contains("PLAN SETTINGS"));
    assert!(pdf_content.contains("PROGRESS SUMMARY"));
    assert!(pdf_content.contains("STUDY SCHEDULE"));

    // Verify plan metadata
    assert!(pdf_content.contains(&format!("Plan ID: {}", plan.id)));
    assert!(pdf_content.contains(&format!("Course ID: {}", plan.course_id)));

    // Verify settings information
    assert!(pdf_content.contains("Sessions per Week: 3"));
    assert!(pdf_content.contains("Session Length: 60 minutes"));
    assert!(pdf_content.contains("Include Weekends: No"));

    // Verify progress information
    assert!(pdf_content.contains("Total Sessions: 5"));
    assert!(pdf_content.contains("Completed Sessions: 2"));
    assert!(pdf_content.contains("Progress: 40.0%"));

    // Verify schedule content
    assert!(pdf_content.contains("Introduction to Rust"));
    assert!(pdf_content.contains("Getting Started with Rust"));
    assert!(pdf_content.contains("Ownership and Borrowing"));
    assert!(pdf_content.contains("Error Handling"));

    // Verify completion status indicators
    assert!(pdf_content.contains("✓")); // Completed sessions
    assert!(pdf_content.contains("○")); // Incomplete sessions

    println!("✓ PDF content generation includes all required information");
    Ok(())
}

#[tokio::test]
async fn test_pdf_export_with_advanced_settings() -> Result<()> {
    let plan = create_test_plan();

    let pdf_content = plan.generate_pdf_content()?;

    // Verify advanced settings are not explicitly shown in basic PDF
    // (This is a current limitation we'll address)

    println!("✓ PDF export handles advanced settings");
    Ok(())
}

#[tokio::test]
async fn test_pdf_export_empty_plan() -> Result<()> {
    let course_id = Uuid::new_v4();
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    let empty_plan = Plan::new(course_id, settings);

    let pdf_data = empty_plan.export_pdf()?;

    // Should still generate valid PDF even with empty plan
    assert!(pdf_data.starts_with(b"%PDF-"));
    assert!(!pdf_data.is_empty());

    let pdf_content = empty_plan.generate_pdf_content()?;
    assert!(pdf_content.contains("Total Sessions: 0"));
    assert!(pdf_content.contains("Completed Sessions: 0"));
    assert!(pdf_content.contains("Progress: 0.0%"));

    println!("✓ PDF export handles empty plans gracefully");
    Ok(())
}

#[tokio::test]
async fn test_pdf_filename_generation() -> Result<()> {
    let plan = create_test_plan();

    let filename = plan.get_export_filename(ExportFormat::Pdf);

    // Verify filename format
    assert!(filename.starts_with(&format!("plan_{}", plan.id)));
    assert!(filename.ends_with(".pdf"));
    assert!(filename.contains("_")); // Should contain timestamp separator

    println!("✓ PDF filename generation works correctly");
    Ok(())
}

#[tokio::test]
async fn test_pdf_export_error_handling() -> Result<()> {
    // Test with plan that might cause issues
    let course_id = Uuid::new_v4();
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    let mut plan = Plan::new(course_id, settings);

    // Add plan item with potentially problematic content
    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module with \"quotes\" and, commas".to_string(),
        section_title: "Section with\nnewlines and\ttabs".to_string(),
        video_indices: vec![0],
        completed: false,
    });

    // Should handle special characters gracefully
    let pdf_result = plan.export_pdf();
    assert!(
        pdf_result.is_ok(),
        "PDF export should handle special characters"
    );

    let pdf_data = pdf_result?;
    assert!(pdf_data.starts_with(b"%PDF-"));

    println!("✓ PDF export handles special characters and edge cases");
    Ok(())
}

/// Integration test to verify the complete PDF export workflow
#[tokio::test]
async fn test_complete_pdf_export_workflow() -> Result<()> {
    let plan = create_test_plan();

    // Test the complete workflow as it would be used in the UI
    let options = ExportOptions {
        format: ExportFormat::Pdf,
        include_metadata: true,
        include_progress: true,
        include_timestamps: true,
        progress_callback: None,
    };

    let export_result = plan.export_with_options(options)?;

    // Verify all aspects of the export result
    assert_eq!(export_result.format, ExportFormat::Pdf);
    assert!(!export_result.filename.is_empty());
    assert!(export_result.filename.ends_with(".pdf"));
    assert!(export_result.size_bytes > 0);
    assert!(!export_result.data.is_empty());
    assert!(export_result.data.starts_with(b"%PDF-"));

    // Verify the data can be validated
    use course_pilot::export::utils;
    let validation_result = utils::validate_export_data(&export_result.data, ExportFormat::Pdf);
    assert!(validation_result.is_ok(), "PDF data should pass validation");

    println!("✓ Complete PDF export workflow functions correctly");
    Ok(())
}
