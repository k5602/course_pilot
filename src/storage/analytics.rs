use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use rusqlite::params;
use std::collections::HashMap;
use uuid::Uuid;

use crate::storage::core::Database;
use crate::types::{
    ClusteringAlgorithm, ClusteringMetadata, ClusteringStrategy, Course, CourseStructure,
    VideoMetadata,
};

/// Clustering analytics for dashboard insights
#[derive(Debug, Clone, PartialEq)]
pub struct ClusteringAnalytics {
    pub total_courses: usize,
    pub clustered_courses: usize,
    pub average_quality_score: f32,
    pub algorithm_distribution: HashMap<ClusteringAlgorithm, usize>,
    pub strategy_distribution: HashMap<ClusteringStrategy, usize>,
    pub quality_distribution: QualityDistribution,
    pub processing_time_stats: ProcessingTimeStats,
}

/// Quality score distribution
#[derive(Debug, Clone, PartialEq)]
pub struct QualityDistribution {
    pub excellent: usize, // 0.8+
    pub good: usize,      // 0.6-0.8
    pub fair: usize,      // 0.4-0.6
    pub poor: usize,      // <0.4
}

/// Processing time statistics
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingTimeStats {
    pub average_ms: f64,
    pub median_ms: f64,
    pub min_ms: u64,
    pub max_ms: u64,
}

/// Clustering performance data point
#[derive(Debug, Clone)]
pub struct ClusteringPerformancePoint {
    pub timestamp: DateTime<Utc>,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub algorithm_used: ClusteringAlgorithm,
    pub strategy_used: ClusteringStrategy,
}

/// Get courses filtered by clustering quality
pub fn get_courses_by_clustering_quality(db: &Database, min_quality: f32) -> Result<Vec<Course>> {
    let conn = db.get_conn()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, raw_titles, structure
         FROM courses
         WHERE structure IS NOT NULL",
    )?;

    let course_iter = stmt.query_map([], |row| {
        let structure_json: String = row.get(4)?;
        let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 4)?;

        if let Some(clustering_metadata) = &structure.clustering_metadata {
            if clustering_metadata.quality_score >= min_quality {
                let raw_titles_json: String = row.get(3)?;
                let raw_titles: Vec<String> = parse_json_sqlite_at(&raw_titles_json, 3)?;

                let videos = raw_titles
                    .iter()
                    .map(|title| VideoMetadata::new_local(title.clone(), "".to_string()))
                    .collect();

                return Ok(Some(Course {
                    id: parse_uuid_sqlite(&row.get::<_, String>(0)?, 0)?,
                    name: row.get(1)?,
                    created_at: DateTime::from_timestamp(row.get::<_, i64>(2)?, 0)
                        .unwrap_or_else(Utc::now),
                    raw_titles,
                    videos,
                    structure: Some(structure),
                }));
            }
        }
        Ok(None)
    })?;

    let mut courses = Vec::new();
    for course_result in course_iter {
        if let Some(course) = course_result? {
            courses.push(course);
        }
    }

    Ok(courses)
}

/// Get comprehensive clustering analytics
pub fn get_clustering_analytics(db: &Database) -> Result<ClusteringAnalytics> {
    let conn = db.get_conn()?;

    // Get total course count
    let total_courses: usize = conn.query_row("SELECT COUNT(*) FROM courses", [], |row| {
        row.get::<_, i64>(0).map(|v| v as usize)
    })?;

    // Get courses with clustering data
    let mut stmt = conn.prepare("SELECT structure FROM courses WHERE structure IS NOT NULL")?;

    let structure_iter = stmt.query_map([], |row| {
        let structure_json: String = row.get(0)?;
        let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 0)?;
        Ok(structure)
    })?;

    let mut clustered_courses = 0;
    let mut quality_scores = Vec::new();
    let mut algorithm_counts: HashMap<ClusteringAlgorithm, usize> = HashMap::new();
    let mut strategy_counts: HashMap<ClusteringStrategy, usize> = HashMap::new();
    let mut processing_times = Vec::new();

    for structure_result in structure_iter {
        let structure = structure_result?;
        if let Some(clustering_metadata) = structure.clustering_metadata {
            clustered_courses += 1;
            quality_scores.push(clustering_metadata.quality_score);
            processing_times.push(clustering_metadata.processing_time_ms);

            *algorithm_counts.entry(clustering_metadata.algorithm_used).or_insert(0) += 1;
            *strategy_counts.entry(clustering_metadata.strategy_used).or_insert(0) += 1;
        }
    }

    // Calculate statistics
    let average_quality_score = if !quality_scores.is_empty() {
        quality_scores.iter().sum::<f32>() / quality_scores.len() as f32
    } else {
        0.0
    };

    let quality_distribution = calculate_quality_distribution(&quality_scores);
    let processing_time_stats = calculate_processing_time_stats(&processing_times);

    Ok(ClusteringAnalytics {
        total_courses,
        clustered_courses,
        average_quality_score,
        algorithm_distribution: algorithm_counts,
        strategy_distribution: strategy_counts,
        quality_distribution,
        processing_time_stats,
    })
}

/// Update clustering metadata for an existing course
pub fn update_clustering_metadata(
    db: &Database,
    course_id: Uuid,
    metadata: ClusteringMetadata,
) -> Result<()> {
    let conn = db.get_conn()?;

    // Get current course structure
    let current_structure: CourseStructure = conn.query_row(
        "SELECT structure FROM courses WHERE id = ?1",
        params![course_id.to_string()],
        |row| {
            let structure_json: String = row.get(0)?;
            let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 0)?;
            Ok(structure)
        },
    )?;

    // Update clustering metadata
    let updated_structure =
        CourseStructure { clustering_metadata: Some(metadata), ..current_structure };

    // Save updated structure
    let structure_json = serde_json::to_string(&updated_structure)?;
    conn.execute(
        "UPDATE courses SET structure = ?1 WHERE id = ?2",
        params![structure_json, course_id.to_string()],
    )?;

    Ok(())
}

/// Get courses with similar clustering characteristics
pub fn get_similar_courses_by_clustering(
    db: &Database,
    reference_course_id: Uuid,
    similarity_threshold: f32,
) -> Result<Vec<Course>> {
    let conn = db.get_conn()?;

    // Get reference course clustering metadata
    let reference_metadata: ClusteringMetadata = conn.query_row(
        "SELECT structure FROM courses WHERE id = ?1",
        params![reference_course_id.to_string()],
        |row| {
            let structure_json: String = row.get(0)?;
            let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 0)?;
            match structure.clustering_metadata {
                Some(meta) => Ok(meta),
                None => Err(rusqlite::Error::InvalidColumnType(
                    0,
                    "no clustering metadata".to_string(),
                    rusqlite::types::Type::Null,
                )),
            }
        },
    )?;

    // Find similar courses
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, raw_titles, structure
         FROM courses
         WHERE id != ?1 AND structure IS NOT NULL",
    )?;

    let course_iter = stmt.query_map(params![reference_course_id.to_string()], |row| {
        let structure_json: String = row.get(4)?;
        let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 4)?;

        if let Some(clustering_metadata) = &structure.clustering_metadata {
            // Calculate similarity based on algorithm, strategy, and quality
            let similarity =
                calculate_clustering_similarity(&reference_metadata, clustering_metadata);

            if similarity >= similarity_threshold {
                let raw_titles_json: String = row.get(3)?;
                let raw_titles: Vec<String> = parse_json_sqlite_at(&raw_titles_json, 3)?;
                let videos = raw_titles
                    .iter()
                    .map(|title| VideoMetadata::new_local(title.clone(), "".to_string()))
                    .collect();
                return Ok(Some(Course {
                    id: parse_uuid_sqlite(&row.get::<_, String>(0)?, 0)?,
                    name: row.get(1)?,
                    created_at: DateTime::from_timestamp(row.get::<_, i64>(2)?, 0)
                        .unwrap_or_else(Utc::now),
                    raw_titles,
                    videos,
                    structure: Some(structure),
                }));
            }
        }
        Ok(None)
    })?;

    let mut similar_courses = Vec::new();
    for course_result in course_iter {
        if let Some(course) = course_result? {
            similar_courses.push(course);
        }
    }

    Ok(similar_courses)
}

/// Get clustering performance history
pub fn get_clustering_performance_history(
    db: &Database,
    days: i64,
) -> Result<Vec<ClusteringPerformancePoint>> {
    let conn = db.get_conn()?;

    let cutoff_date = Utc::now() - Duration::days(days);

    let mut stmt = conn.prepare(
        "SELECT created_at, structure FROM courses
         WHERE structure IS NOT NULL AND created_at >= ?1
         ORDER BY created_at ASC",
    )?;

    let performance_iter = stmt.query_map(params![cutoff_date.timestamp()], |row| {
        let created_at =
            DateTime::from_timestamp(row.get::<_, i64>(0)?, 0).unwrap_or_else(Utc::now);

        let structure_json: String = row.get(1)?;
        let structure: CourseStructure = parse_json_sqlite_at(&structure_json, 1)?;

        if let Some(clustering_metadata) = structure.clustering_metadata {
            return Ok(Some(ClusteringPerformancePoint {
                timestamp: created_at,
                quality_score: clustering_metadata.quality_score,
                processing_time_ms: clustering_metadata.processing_time_ms,
                algorithm_used: clustering_metadata.algorithm_used,
                strategy_used: clustering_metadata.strategy_used,
            }));
        }
        Ok(None)
    })?;

    let mut performance_points = Vec::new();
    for point_result in performance_iter {
        if let Some(point) = point_result? {
            performance_points.push(point);
        }
    }

    Ok(performance_points)
}

// =======================
// Internal helper methods
// =======================

fn calculate_quality_distribution(quality_scores: &[f32]) -> QualityDistribution {
    let mut excellent = 0;
    let mut good = 0;
    let mut fair = 0;
    let mut poor = 0;

    for &score in quality_scores {
        match score {
            s if s >= 0.8 => excellent += 1,
            s if s >= 0.6 => good += 1,
            s if s >= 0.4 => fair += 1,
            _ => poor += 1,
        }
    }

    QualityDistribution { excellent, good, fair, poor }
}

fn calculate_processing_time_stats(processing_times: &[u64]) -> ProcessingTimeStats {
    if processing_times.is_empty() {
        return ProcessingTimeStats { average_ms: 0.0, median_ms: 0.0, min_ms: 0, max_ms: 0 };
    }

    let mut sorted_times = processing_times.to_vec();
    sorted_times.sort_unstable();

    let average_ms = processing_times.iter().sum::<u64>() as f64 / processing_times.len() as f64;
    let median_ms = if sorted_times.len() % 2 == 0 {
        let mid = sorted_times.len() / 2;
        (sorted_times[mid - 1] + sorted_times[mid]) as f64 / 2.0
    } else {
        sorted_times[sorted_times.len() / 2] as f64
    };

    ProcessingTimeStats {
        average_ms,
        median_ms,
        min_ms: *sorted_times.first().unwrap_or(&0),
        max_ms: *sorted_times.last().unwrap_or(&0),
    }
}

fn calculate_clustering_similarity(
    metadata1: &ClusteringMetadata,
    metadata2: &ClusteringMetadata,
) -> f32 {
    let mut similarity: f32 = 0.0;

    // Algorithm similarity (0.3 weight)
    if metadata1.algorithm_used == metadata2.algorithm_used {
        similarity += 0.3;
    }

    // Strategy similarity (0.2 weight)
    if metadata1.strategy_used == metadata2.strategy_used {
        similarity += 0.2;
    }

    // Quality similarity (0.3 weight)
    let quality_diff = (metadata1.quality_score - metadata2.quality_score).abs();
    let quality_similarity = 1.0 - quality_diff.min(1.0);
    similarity += quality_similarity * 0.3;

    // Cluster count similarity (0.2 weight)
    let cluster_diff =
        (metadata1.cluster_count as i32 - metadata2.cluster_count as i32).abs() as f32;
    let cluster_similarity = 1.0 - (cluster_diff / 10.0).min(1.0); // Normalize by 10
    similarity += cluster_similarity * 0.2;

    similarity.clamp(0.0, 1.0)
}

// =======================
// Local parsing helpers
// =======================

fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(s).map_err(|_| {
        rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), rusqlite::types::Type::Text)
    })
}

fn parse_json_sqlite_at<T: serde::de::DeserializeOwned>(
    s: &str,
    idx: usize,
) -> Result<T, rusqlite::Error> {
    serde_json::from_str(s).map_err(|e| {
        rusqlite::Error::InvalidColumnType(idx, format!("json: {e}"), rusqlite::types::Type::Text)
    })
}
