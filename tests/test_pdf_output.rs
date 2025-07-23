use course_pilot::export::Exportable;
use course_pilot::types::{Plan, PlanSettings};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs;

fn main() -> anyhow::Result<()> {
    // Create a test plan
    let course_id = Uuid::new_v4();
    let settings = PlanSettings {
        start_date: Utc::now().date_naive(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
    };
    
    let mut plan = Plan::new(course_id, settings);
    
    // Add some test items
    plan.items.push(course_pilot::types::PlanItem {
        date: Utc::now(),
        module_title: "Introduction to Programming".to_string(),
        section_title: "Variables and Data Types".to_string(),
        video_indices: vec![0, 1, 2],
        completed: false,
    });
    
    plan.items.push(course_pilot::types::PlanItem {
        date: Utc::now() + chrono::Duration::days(2),
        module_title: "Introduction to Programming".to_string(),
        section_title: "Control Structures".to_string(),
        video_indices: vec![3, 4],
        completed: true,
    });
    
    // Generate PDF content to see what it looks like
    let content = plan.generate_pdf_content()?;
    println!("Generated PDF content:\n{}", content);
    
    // Generate actual PDF
    let pdf_data = plan.export_pdf()?;
    println!("\nPDF data size: {} bytes", pdf_data.len());
    println!("PDF header: {:?}", &pdf_data[..20]);
    
    // Save to file for inspection
    fs::write("test_output.pdf", &pdf_data)?;
    println!("PDF saved to test_output.pdf");
    
    Ok(())
}