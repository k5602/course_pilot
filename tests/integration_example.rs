//! Complete integration example demonstrating the full clustering pipeline
//!
//! This example shows how the enhanced clustering system integrates with
//! ingest, storage, and planning components for a complete workflow.

use crate::ingest::{ImportProgress, ImportStage, import_and_structure_youtube};
use crate::planner::generate_plan;
use crate::storage::database::Database;
use crate::storage::{get_clustering_analytics, save_course, save_plan};
use crate::types::{Course, Plan, PlanSettings};
use chrono::Utc;
use std::path::Path;

/// Complete integration example: YouTube → Clustering → Planning → Storage
pub async fn complete_integration_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize database
    let db_path = Path::new("example_course_pilot.db");
    let db = Database::new(db_path)?;

    println!("🚀 Starting complete integration example...");

    // 2. Import and structure course with clustering
    let youtube_url = "https://www.youtube.com/playlist?list=EXAMPLE";
    let api_key = "your_youtube_api_key";

    let progress_callback = |progress: ImportProgress| match progress.stage {
        ImportStage::Starting => println!("📥 Starting import..."),
        ImportStage::Importing => println!("📥 Importing: {}", progress.message),
        ImportStage::Structuring => println!("🧠 Structuring: {}", progress.message),
        ImportStage::Clustering => {
            if let Some(stage) = progress.clustering_stage {
                println!("🔄 Clustering stage {}/4: {}", stage + 1, progress.message);
            }
        }
        ImportStage::Optimizing => println!("⚡ Optimizing: {}", progress.message),
        ImportStage::Saving => println!("💾 Saving: {}", progress.message),
        ImportStage::Complete => println!("✅ Complete: {}", progress.message),
        ImportStage::Failed => println!("❌ Failed: {}", progress.message),
    };

    // This would normally use real YouTube data
    let course = simulate_import_and_structure(&db, progress_callback).await?;

    // 3. Generate clustering-aware study plan
    println!("\n📅 Generating clustering-aware study plan...");

    let plan_settings = PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    };

    let plan = generate_plan(&course, &plan_settings)?;
    save_plan(&db, &plan)?;

    println!("✅ Generated plan with {} sessions", plan.items.len());

    // 4. Display clustering insights
    println!("\n📊 Clustering Analytics:");
    let analytics = get_clustering_analytics(&db)?;

    println!("  • Total courses: {}", analytics.total_courses);
    println!("  • Clustered courses: {}", analytics.clustered_courses);
    println!(
        "  • Average quality: {:.2}",
        analytics.average_quality_score
    );
    println!("  • Quality distribution:");
    println!(
        "    - Excellent (80%+): {}",
        analytics.quality_distribution.excellent
    );
    println!(
        "    - Good (60-80%): {}",
        analytics.quality_distribution.good
    );
    println!(
        "    - Fair (40-60%): {}",
        analytics.quality_distribution.fair
    );
    println!("    - Poor (<40%): {}", analytics.quality_distribution.poor);

    // 5. Show plan details with clustering context
    println!("\n📋 Study Plan Details:");
    if let Some(clustering_metadata) = &course.structure.as_ref().unwrap().clustering_metadata {
        println!("  • Algorithm: {:?}", clustering_metadata.algorithm_used);
        println!("  • Strategy: {:?}", clustering_metadata.strategy_used);
        println!(
            "  • Quality Score: {:.2}",
            clustering_metadata.quality_score
        );
        println!(
            "  • Processing Time: {}ms",
            clustering_metadata.processing_time_ms
        );
        println!("  • Clusters: {}", clustering_metadata.cluster_count);

        if !clustering_metadata.content_topics.is_empty() {
            println!("  • Topics:");
            for topic in &clustering_metadata.content_topics {
                println!(
                    "    - {} (relevance: {:.2})",
                    topic.keyword, topic.relevance_score
                );
            }
        }
    }

    println!("\n🎯 Sessions:");
    for (i, item) in plan.items.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {} ({} videos)",
            i + 1,
            item.date.format("%Y-%m-%d"),
            item.module_title,
            item.video_indices.len()
        );
    }

    if plan.items.len() > 5 {
        println!("  ... and {} more sessions", plan.items.len() - 5);
    }

    println!("\n🎉 Integration example completed successfully!");

    Ok(())
}

/// Simulate the import and structure process for demonstration
async fn simulate_import_and_structure(
    db: &Database,
    progress_callback: impl Fn(ImportProgress),
) -> Result<Course, Box<dyn std::error::Error>> {
    // Simulate import progress
    progress_callback(ImportProgress {
        stage: ImportStage::Starting,
        progress: 0.0,
        message: "Initializing import...".to_string(),
        clustering_stage: None,
    });

    // Simulate importing video titles
    let sample_titles = vec![
        "Introduction to Machine Learning".to_string(),
        "Linear Regression Fundamentals".to_string(),
        "Classification Algorithms".to_string(),
        "Decision Trees and Random Forests".to_string(),
        "Neural Networks Basics".to_string(),
        "Deep Learning Introduction".to_string(),
        "Convolutional Neural Networks".to_string(),
        "Natural Language Processing".to_string(),
        "Advanced NLP Techniques".to_string(),
        "Model Evaluation and Validation".to_string(),
        "Hyperparameter Tuning".to_string(),
        "Deployment and Production".to_string(),
    ];

    progress_callback(ImportProgress {
        stage: ImportStage::Importing,
        progress: 0.3,
        message: format!("Imported {} videos", sample_titles.len()),
        clustering_stage: None,
    });

    // Create course and structure it
    let mut course = Course::new(
        "Machine Learning Masterclass".to_string(),
        sample_titles.clone(),
    );

    // Simulate clustering progress
    progress_callback(ImportProgress {
        stage: ImportStage::Clustering,
        progress: 0.5,
        message: "Analyzing content...".to_string(),
        clustering_stage: Some(0),
    });

    std::thread::sleep(std::time::Duration::from_millis(200));
    progress_callback(ImportProgress {
        stage: ImportStage::Clustering,
        progress: 0.6,
        message: "Performing TF-IDF analysis...".to_string(),
        clustering_stage: Some(1),
    });

    std::thread::sleep(std::time::Duration::from_millis(300));
    progress_callback(ImportProgress {
        stage: ImportStage::Clustering,
        progress: 0.7,
        message: "Clustering content...".to_string(),
        clustering_stage: Some(2),
    });

    std::thread::sleep(std::time::Duration::from_millis(200));
    progress_callback(ImportProgress {
        stage: ImportStage::Clustering,
        progress: 0.8,
        message: "Optimizing clusters...".to_string(),
        clustering_stage: Some(3),
    });

    std::thread::sleep(std::time::Duration::from_millis(100));
    progress_callback(ImportProgress {
        stage: ImportStage::Clustering,
        progress: 0.85,
        message: "Finalizing structure...".to_string(),
        clustering_stage: Some(4),
    });

    // Structure the course using the NLP processor
    let structure = crate::nlp::structure_course(sample_titles)?;
    course.structure = Some(structure);

    progress_callback(ImportProgress {
        stage: ImportStage::Saving,
        progress: 0.9,
        message: "Saving course...".to_string(),
        clustering_stage: None,
    });

    // Save to database
    save_course(db, &course)?;

    progress_callback(ImportProgress {
        stage: ImportStage::Complete,
        progress: 1.0,
        message: "Course imported and structured successfully!".to_string(),
        clustering_stage: None,
    });

    Ok(course)
}

/// Demonstrate clustering analytics and insights
pub fn demonstrate_clustering_insights(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Clustering Analytics Dashboard");
    println!("================================");

    let analytics = get_clustering_analytics(db)?;

    // Overall statistics
    println!("\n📈 Overall Statistics:");
    println!("  Total Courses: {}", analytics.total_courses);
    println!("  Clustered Courses: {}", analytics.clustered_courses);
    println!(
        "  Clustering Coverage: {:.1}%",
        (analytics.clustered_courses as f32 / analytics.total_courses as f32) * 100.0
    );
    println!(
        "  Average Quality Score: {:.2}",
        analytics.average_quality_score
    );

    // Algorithm distribution
    println!("\n🔧 Algorithm Usage:");
    for (algorithm, count) in &analytics.algorithm_distribution {
        println!("  {:?}: {} courses", algorithm, count);
    }

    // Strategy distribution
    println!("\n📋 Strategy Usage:");
    for (strategy, count) in &analytics.strategy_distribution {
        println!("  {:?}: {} courses", strategy, count);
    }

    // Quality distribution
    println!("\n⭐ Quality Distribution:");
    let total_clustered = analytics.clustered_courses as f32;
    if total_clustered > 0.0 {
        println!(
            "  Excellent (80%+): {} ({:.1}%)",
            analytics.quality_distribution.excellent,
            (analytics.quality_distribution.excellent as f32 / total_clustered) * 100.0
        );
        println!(
            "  Good (60-80%): {} ({:.1}%)",
            analytics.quality_distribution.good,
            (analytics.quality_distribution.good as f32 / total_clustered) * 100.0
        );
        println!(
            "  Fair (40-60%): {} ({:.1}%)",
            analytics.quality_distribution.fair,
            (analytics.quality_distribution.fair as f32 / total_clustered) * 100.0
        );
        println!(
            "  Poor (<40%): {} ({:.1}%)",
            analytics.quality_distribution.poor,
            (analytics.quality_distribution.poor as f32 / total_clustered) * 100.0
        );
    }

    // Performance statistics
    println!("\n⚡ Performance Statistics:");
    println!(
        "  Average Processing Time: {:.1}ms",
        analytics.processing_time_stats.average_ms
    );
    println!(
        "  Median Processing Time: {:.1}ms",
        analytics.processing_time_stats.median_ms
    );
    println!(
        "  Fastest Processing: {}ms",
        analytics.processing_time_stats.min_ms
    );
    println!(
        "  Slowest Processing: {}ms",
        analytics.processing_time_stats.max_ms
    );

    Ok(())
}

/// Example of finding similar courses using clustering
pub fn demonstrate_course_similarity(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    use crate::storage::get_similar_courses_by_clustering;

    println!("🔍 Course Similarity Analysis");
    println!("============================");

    // Get all courses
    let courses = crate::storage::load_courses(db)?;

    if courses.len() < 2 {
        println!("Need at least 2 courses to demonstrate similarity analysis");
        return Ok(());
    }

    // Find similar courses to the first one
    let reference_course = &courses[0];
    println!(
        "\n📚 Finding courses similar to: '{}'",
        reference_course.name
    );

    let similar_courses = get_similar_courses_by_clustering(db, reference_course.id, 0.6)?;

    if similar_courses.is_empty() {
        println!("  No similar courses found (similarity threshold: 60%)");
    } else {
        println!("  Found {} similar courses:", similar_courses.len());
        for course in &similar_courses {
            if let Some(structure) = &course.structure {
                if let Some(clustering_metadata) = &structure.clustering_metadata {
                    println!(
                        "    • {} (quality: {:.2}, algorithm: {:?})",
                        course.name,
                        clustering_metadata.quality_score,
                        clustering_metadata.algorithm_used
                    );
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_integration_pipeline() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();

        // Test the simulation (since we can't use real YouTube API in tests)
        let progress_callback = |_progress: ImportProgress| {
            // Silent for tests
        };

        let course = simulate_import_and_structure(&db, progress_callback)
            .await
            .unwrap();

        // Verify course was structured
        assert!(course.structure.is_some());

        // Verify clustering metadata exists
        let structure = course.structure.as_ref().unwrap();
        assert!(structure.clustering_metadata.is_some());

        // Test plan generation
        let plan_settings = PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
            advanced_settings: None,
        };

        let plan = generate_plan(&course, &plan_settings).unwrap();
        assert!(!plan.items.is_empty());

        // Test analytics
        let analytics = get_clustering_analytics(&db).unwrap();
        assert_eq!(analytics.total_courses, 1);
        assert_eq!(analytics.clustered_courses, 1);
    }
}
