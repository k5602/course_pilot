use course_pilot::export::Exportable;
use course_pilot::types::{Plan, PlanSettings, PlanItem};
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;

#[tokio::test]
async fn test_pdf_content_generation() -> Result<()> {
    // Create a test plan with realistic data
    let course_id = Uuid::new_v4();
    let settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };
    
    let mut plan = Plan::new(course_id, settings);
    
    // Add some test items
    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Introduction to Programming".to_string(),
        section_title: "Variables and Data Types".to_string(),
        video_indices: vec![0, 1, 2],
        completed: false,
    });
    
    plan.items.push(PlanItem {
        date: Utc::now() + chrono::Duration::days(2),
        module_title: "Introduction to Programming".to_string(),
        section_title: "Control Structures".to_string(),
        video_indices: vec![3, 4],
        completed: true,
    });
    
    plan.items.push(PlanItem {
        date: Utc::now() + chrono::Duration::days(4),
        module_title: "Object-Oriented Programming".to_string(),
        section_title: "Classes and Objects".to_string(),
        video_indices: vec![5, 6, 7, 8],
        completed: false,
    });
    
    // Generate PDF content to see what it looks like
    let content = plan.generate_pdf_content()?;
    println!("Generated PDF content:\n{}", content);
    
    // Verify content structure
    assert!(content.contains("STUDY PLAN EXPORT"));
    assert!(content.contains("PLAN SETTINGS"));
    assert!(content.contains("PROGRESS SUMMARY"));
    assert!(content.contains("STUDY SCHEDULE"));
    assert!(content.contains("Introduction to Programming"));
    assert!(content.contains("Object-Oriented Programming"));
    
    // Generate actual PDF
    let pdf_data = plan.export_pdf()?;
    println!("\nPDF data size: {} bytes", pdf_data.len());
    println!("PDF header: {:?}", std::str::from_utf8(&pdf_data[..20]).unwrap_or("Invalid UTF-8"));
    
    // Verify PDF structure
    assert!(!pdf_data.is_empty());
    assert!(pdf_data.starts_with(b"%PDF-"));
    
    // Save to file for inspection
    std::fs::write("test_output.pdf", &pdf_data)?;
    println!("PDF saved to test_output.pdf");
    
    Ok(())
}