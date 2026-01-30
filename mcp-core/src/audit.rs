//! Audit logging for MCP operations

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

use crate::error::{McpError, McpResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub service: String,
    pub details: String,
    pub user_approved: bool,
    pub approval_token: Option<String>,
    pub result: String,
    pub snapshot_id: Option<String>,
}

pub struct AuditLogger {
    conn: Mutex<Connection>,
}

impl AuditLogger {
    pub fn new(db_path: &Path) -> McpResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| McpError::DatabaseError(e.to_string()))?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Create tables if not exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                service TEXT NOT NULL,
                details TEXT NOT NULL,
                user_approved INTEGER NOT NULL,
                approval_token TEXT,
                result TEXT NOT NULL,
                snapshot_id TEXT
            )",
            [],
        ).map_err(|e| McpError::DatabaseError(e.to_string()))?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)",
            [],
        ).map_err(|e| McpError::DatabaseError(e.to_string()))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Log an action
    pub fn log(&self, entry: AuditEntry) -> McpResult<String> {
        let conn = self.conn.lock()
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        conn.execute(
            "INSERT INTO audit_log (id, timestamp, action, service, details, user_approved, approval_token, result, snapshot_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id,
                entry.timestamp.to_rfc3339(),
                entry.action,
                entry.service,
                entry.details,
                entry.user_approved as i32,
                entry.approval_token,
                entry.result,
                entry.snapshot_id,
            ],
        ).map_err(|e| McpError::DatabaseError(e.to_string()))?;

        Ok(entry.id)
    }

    /// Create a new audit entry builder
    pub fn create_entry(service: &str, action: &str) -> AuditEntry {
        AuditEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            action: action.to_string(),
            service: service.to_string(),
            details: String::new(),
            user_approved: false,
            approval_token: None,
            result: "pending".to_string(),
            snapshot_id: None,
        }
    }

    /// Query audit logs with filters
    pub fn query(
        &self,
        service: Option<&str>,
        action: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: usize,
    ) -> McpResult<Vec<AuditEntry>> {
        let conn = self.conn.lock()
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        let mut sql = String::from("SELECT * FROM audit_log WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(s) = service {
            sql.push_str(" AND service = ?");
            params_vec.push(Box::new(s.to_string()));
        }

        if let Some(a) = action {
            sql.push_str(" AND action = ?");
            params_vec.push(Box::new(a.to_string()));
        }

        if let Some(f) = from {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(f.to_rfc3339()));
        }

        if let Some(t) = to {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(t.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        let entries = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap()
                    .with_timezone(&Utc),
                action: row.get(2)?,
                service: row.get(3)?,
                details: row.get(4)?,
                user_approved: row.get::<_, i32>(5)? != 0,
                approval_token: row.get(6)?,
                result: row.get(7)?,
                snapshot_id: row.get(8)?,
            })
        }).map_err(|e| McpError::DatabaseError(e.to_string()))?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry.map_err(|e| McpError::DatabaseError(e.to_string()))?);
        }

        Ok(result)
    }

    /// Get total count of audit entries
    pub fn count(&self) -> McpResult<usize> {
        let conn = self.conn.lock()
            .map_err(|e| McpError::DatabaseError(e.to_string()))?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_log",
            [],
            |row| row.get(0),
        ).map_err(|e| McpError::DatabaseError(e.to_string()))?;

        Ok(count as usize)
    }
}
