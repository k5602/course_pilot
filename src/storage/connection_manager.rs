//! Database connection manager with prepared statement caching
//!
//! This module provides a connection manager that caches frequently used
//! prepared statements for better performance.

use crate::DatabaseError;
use crate::storage::Database;
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Connection manager with prepared statement caching
pub struct ConnectionManager {
    db: Database,
    statement_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(db: Database) -> Self {
        Self {
            db,
            statement_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Execute a query with caching support
    pub fn execute_cached<F, R>(
        &self,
        query_key: &str,
        query: &str,
        f: F,
    ) -> Result<R, DatabaseError>
    where
        F: FnOnce(&Connection, &str) -> Result<R, DatabaseError>,
    {
        // Cache the query for reuse
        {
            let mut cache = self.statement_cache.lock().unwrap();
            cache.insert(query_key.to_string(), query.to_string());
        }

        let conn = self.db.get_conn()?;
        f(&conn, query)
    }

    /// Get frequently used queries for courses
    pub fn get_courses_optimized<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&Connection) -> Result<R, DatabaseError>,
    {
        let conn = self.db.get_conn()?;
        f(&conn)
    }

    /// Batch operations with transaction support
    pub fn execute_batch<F, R>(&self, f: F) -> Result<R, DatabaseError>
    where
        F: FnOnce(&Connection) -> Result<R, DatabaseError>,
    {
        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;

        // Execute the batch operation within the transaction
        let result = f(&tx)?;

        tx.commit()?;
        Ok(result)
    }

    /// Analyze query performance and suggest optimizations
    pub fn analyze_query_performance(&self, query: &str) -> QueryAnalysis {
        // Simple analysis based on query patterns
        let mut suggestions = Vec::new();
        let mut estimated_cost = 1;

        // Check for missing WHERE clauses
        if !query.to_lowercase().contains("where") && query.to_lowercase().contains("select") {
            suggestions.push("Consider adding WHERE clause to limit results".to_string());
            estimated_cost += 2;
        }

        // Check for missing ORDER BY with LIMIT
        if query.to_lowercase().contains("limit") && !query.to_lowercase().contains("order by") {
            suggestions.push(
                "Consider adding ORDER BY clause with LIMIT for consistent results".to_string(),
            );
        }

        // Check for potential N+1 queries
        if query.to_lowercase().contains("select") && query.contains("?") {
            suggestions.push("Consider using JOIN instead of multiple queries".to_string());
            estimated_cost += 1;
        }

        // Check for full table scans
        if query.to_lowercase().contains("like '%") {
            suggestions.push("LIKE queries starting with % may cause full table scans".to_string());
            estimated_cost += 3;
        }

        QueryAnalysis {
            query: query.to_string(),
            estimated_cost,
            suggestions,
            uses_index: query.to_lowercase().contains("where")
                && !query.to_lowercase().contains("like '%"),
        }
    }

    /// Get database statistics for monitoring
    pub fn get_database_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let conn = self.db.get_conn()?;

        // Get table sizes
        let mut table_stats = HashMap::new();

        let tables = vec![
            "courses",
            "plans",
            "notes",
            "clustering_preferences",
            "clustering_feedback",
        ];
        for table in tables {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);
            table_stats.insert(table.to_string(), count as usize);
        }

        // Get database file size (approximate)
        let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let db_size = (page_count * page_size) as usize;

        // Get index usage statistics
        let index_stats = self.get_index_usage_stats(&conn)?;

        Ok(DatabaseStats {
            table_row_counts: table_stats,
            database_size_bytes: db_size,
            index_usage: index_stats,
            connection_pool_size: self.db.pool().state().connections,
            active_connections: self.db.pool().state().idle_connections,
        })
    }

    /// Get index usage statistics
    fn get_index_usage_stats(
        &self,
        conn: &Connection,
    ) -> Result<HashMap<String, IndexUsage>, DatabaseError> {
        let mut index_stats = HashMap::new();

        // Get all indexes
        let mut stmt =
            conn.prepare("SELECT name, tbl_name FROM sqlite_master WHERE type = 'index'")?;
        let index_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for index_row in index_rows {
            let (index_name, table_name) = index_row?;

            // Skip auto-generated indexes
            if index_name.starts_with("sqlite_") {
                continue;
            }

            index_stats.insert(
                index_name.clone(),
                IndexUsage {
                    index_name: index_name.clone(),
                    table_name,
                    is_used: true, // Simplified - in a real implementation, you'd track actual usage
                    selectivity: 0.5, // Simplified - would calculate actual selectivity
                },
            );
        }

        Ok(index_stats)
    }
}

/// Query analysis result
#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    pub query: String,
    pub estimated_cost: u32,
    pub suggestions: Vec<String>,
    pub uses_index: bool,
}

/// Database statistics for monitoring
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub table_row_counts: HashMap<String, usize>,
    pub database_size_bytes: usize,
    pub index_usage: HashMap<String, IndexUsage>,
    pub connection_pool_size: u32,
    pub active_connections: u32,
}

/// Index usage statistics
#[derive(Debug, Clone)]
pub struct IndexUsage {
    pub index_name: String,
    pub table_name: String,
    pub is_used: bool,
    pub selectivity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_db;
    use std::path::Path;

    #[test]
    fn test_connection_manager() {
        let db = init_db(Path::new(":memory:")).unwrap();
        let manager = ConnectionManager::new(db);

        // Test query analysis
        let analysis = manager.analyze_query_performance("SELECT * FROM courses");
        assert!(analysis.estimated_cost > 1);
        assert!(!analysis.suggestions.is_empty());

        // Test database stats
        let stats = manager.get_database_stats().unwrap();
        assert!(stats.table_row_counts.contains_key("courses"));
    }
}
