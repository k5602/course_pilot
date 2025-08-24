//! Database maintenance and optimization utilities
//!
//! This module provides utilities for maintaining database performance,
//! including cleanup, optimization, and monitoring functions.

use crate::storage::{Database, DatabasePerformanceMetrics};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use log::info;
use rusqlite::params;

/// Database maintenance manager
pub struct DatabaseMaintenance {
    db: Database,
}

impl DatabaseMaintenance {
    /// Create a new database maintenance manager
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Run comprehensive database maintenance
    pub fn run_maintenance(&self) -> Result<MaintenanceReport> {
        info!("Starting database maintenance");
        let start_time = Utc::now();

        let mut report = MaintenanceReport {
            start_time,
            end_time: start_time,
            operations_performed: Vec::new(),
            errors: Vec::new(),
            performance_before: None,
            performance_after: None,
        };

        // Get performance metrics before maintenance
        match crate::storage::get_database_performance_metrics(&self.db) {
            Ok(metrics) => report.performance_before = Some(metrics),
            Err(e) => report
                .errors
                .push(format!("Failed to get initial metrics: {e}")),
        }

        // 1. Clean up old data
        if let Err(e) = self.cleanup_old_data() {
            report.errors.push(format!("Cleanup failed: {e}"));
        } else {
            report
                .operations_performed
                .push("Cleaned up old data".to_string());
        }

        // 2. Optimize database structure
        if let Err(e) = crate::storage::optimize_database(&self.db) {
            report.errors.push(format!("Optimization failed: {e}"));
        } else {
            report
                .operations_performed
                .push("Optimized database structure".to_string());
        }

        // 3. Rebuild indexes if needed
        if let Err(e) = self.rebuild_indexes_if_needed() {
            report.errors.push(format!("Index rebuild failed: {e}"));
        } else {
            report
                .operations_performed
                .push("Checked and rebuilt indexes".to_string());
        }

        // 4. Vacuum database if fragmented
        if let Err(e) = self.vacuum_if_needed() {
            report.errors.push(format!("Vacuum failed: {e}"));
        } else {
            report
                .operations_performed
                .push("Checked database fragmentation".to_string());
        }

        // 5. Update statistics
        if let Err(e) = self.update_statistics() {
            report.errors.push(format!("Statistics update failed: {e}"));
        } else {
            report
                .operations_performed
                .push("Updated query statistics".to_string());
        }

        // Get performance metrics after maintenance
        match crate::storage::get_database_performance_metrics(&self.db) {
            Ok(metrics) => report.performance_after = Some(metrics),
            Err(e) => report
                .errors
                .push(format!("Failed to get final metrics: {e}")),
        }

        report.end_time = Utc::now();
        info!(
            "Database maintenance completed in {:?}",
            report.end_time - report.start_time
        );

        Ok(report)
    }

    /// Clean up old data based on retention policies
    fn cleanup_old_data(&self) -> Result<()> {
        let conn = self.db.get_conn()?;
        let cutoff_date = Utc::now() - Duration::days(365); // Keep data for 1 year

        // Clean up old clustering feedback (keep last 1000 entries per course)
        conn.execute(
            r#"
            DELETE FROM clustering_feedback
            WHERE id NOT IN (
                SELECT id FROM clustering_feedback
                ORDER BY created_at DESC
                LIMIT 1000
            ) AND created_at < ?1
            "#,
            params![cutoff_date.to_rfc3339()],
        )?;

        // Clean up old A/B test results (keep last 6 months)
        let ab_test_cutoff = Utc::now() - Duration::days(180);
        conn.execute(
            "DELETE FROM ab_test_results WHERE timestamp < ?1",
            params![ab_test_cutoff.to_rfc3339()],
        )?;

        info!("Cleaned up old data");
        Ok(())
    }

    /// Rebuild indexes if database is large enough to benefit
    fn rebuild_indexes_if_needed(&self) -> Result<()> {
        let conn = self.db.get_conn()?;

        // Check database size
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;

        if page_count > 5000 {
            // Only rebuild for larger databases
            info!("Database is large ({page_count} pages), rebuilding indexes");
            conn.execute("REINDEX", [])?;
            info!("Indexes rebuilt successfully");
        } else {
            info!("Database is small ({page_count} pages), skipping index rebuild");
        }

        Ok(())
    }

    /// Vacuum database if fragmentation is high
    fn vacuum_if_needed(&self) -> Result<()> {
        let conn = self.db.get_conn()?;

        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;

        let fragmentation_ratio = if page_count > 0 {
            freelist_count as f64 / page_count as f64
        } else {
            0.0
        };

        if fragmentation_ratio > 0.1 {
            // More than 10% fragmentation
            info!(
                "High fragmentation detected ({:.1}%), running VACUUM",
                fragmentation_ratio * 100.0
            );
            conn.execute("VACUUM", [])?;
            info!("Database vacuumed successfully");
        } else {
            info!(
                "Low fragmentation ({:.1}%), skipping VACUUM",
                fragmentation_ratio * 100.0
            );
        }

        Ok(())
    }

    /// Update query optimizer statistics
    fn update_statistics(&self) -> Result<()> {
        let conn = self.db.get_conn()?;
        conn.execute("ANALYZE", [])?;
        info!("Query optimizer statistics updated");
        Ok(())
    }

    /// Check database health and return recommendations
    pub fn check_database_health(&self) -> Result<HealthReport> {
        let conn = self.db.get_conn()?;
        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();

        // Check database size
        let metrics = crate::storage::get_database_performance_metrics(&self.db)?;

        if metrics.total_size_bytes > 100_000_000 {
            // 100MB
            recommendations.push("Consider archiving old data to reduce database size".to_string());
        }

        if metrics.fragmentation_ratio > 0.15 {
            recommendations
                .push("High fragmentation detected, consider running VACUUM".to_string());
        }

        // Check connection pool usage
        let pool_utilization = if metrics.connection_pool_active > 0 {
            (metrics.connection_pool_active - metrics.connection_pool_idle) as f64
                / metrics.connection_pool_active as f64
        } else {
            0.0
        };

        if pool_utilization > 0.8 {
            warnings.push(
                "High connection pool utilization, consider increasing pool size".to_string(),
            );
        }

        // Check table sizes
        if metrics.notes_count > 10000 {
            recommendations
                .push("Large number of notes, consider implementing note archiving".to_string());
        }

        // Check for missing indexes (simplified check)
        let index_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )?;

        if index_count < 5 {
            warnings
                .push("Few custom indexes found, query performance may be suboptimal".to_string());
        }

        Ok(HealthReport {
            overall_health: if warnings.is_empty() && recommendations.len() < 2 {
                HealthStatus::Good
            } else if warnings.len() < 2 {
                HealthStatus::Fair
            } else {
                HealthStatus::Poor
            },
            metrics,
            recommendations,
            warnings,
            last_checked: Utc::now(),
        })
    }

    /// Get maintenance schedule recommendations
    pub fn get_maintenance_schedule(&self) -> Result<MaintenanceSchedule> {
        let metrics = crate::storage::get_database_performance_metrics(&self.db)?;

        // Base schedule on database size and activity
        let total_records = metrics.courses_count + metrics.plans_count + metrics.notes_count;

        let (frequency, operations) = if total_records < 1000 {
            // Small database - monthly maintenance
            (
                Duration::days(30),
                vec![
                    "Update statistics".to_string(),
                    "Check integrity".to_string(),
                ],
            )
        } else if total_records < 10000 {
            // Medium database - bi-weekly maintenance
            (
                Duration::days(14),
                vec![
                    "Update statistics".to_string(),
                    "Clean up old data".to_string(),
                    "Check fragmentation".to_string(),
                ],
            )
        } else {
            // Large database - weekly maintenance
            (
                Duration::days(7),
                vec![
                    "Update statistics".to_string(),
                    "Clean up old data".to_string(),
                    "Rebuild indexes".to_string(),
                    "Vacuum if needed".to_string(),
                ],
            )
        };

        Ok(MaintenanceSchedule {
            recommended_frequency: frequency,
            next_maintenance: Utc::now() + frequency,
            operations,
            estimated_duration_minutes: if total_records < 1000 {
                1
            } else if total_records < 10000 {
                5
            } else {
                15
            },
        })
    }
}

/// Maintenance operation report
#[derive(Debug, Clone)]
pub struct MaintenanceReport {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub operations_performed: Vec<String>,
    pub errors: Vec<String>,
    pub performance_before: Option<DatabasePerformanceMetrics>,
    pub performance_after: Option<DatabasePerformanceMetrics>,
}

/// Database health report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub overall_health: HealthStatus,
    pub metrics: DatabasePerformanceMetrics,
    pub recommendations: Vec<String>,
    pub warnings: Vec<String>,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Good,
    Fair,
    Poor,
}

/// Maintenance schedule recommendations
#[derive(Debug, Clone)]
pub struct MaintenanceSchedule {
    pub recommended_frequency: Duration,
    pub next_maintenance: DateTime<Utc>,
    pub operations: Vec<String>,
    pub estimated_duration_minutes: u32,
}

// Storage module tests removed - will be refactored later
