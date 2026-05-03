use crate::models::*;
use rusqlite::{params, Connection};
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        db.seed_defaults()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                api_type TEXT NOT NULL DEFAULT 'openai',
                base_url TEXT NOT NULL,
                api_key TEXT NOT NULL DEFAULT '',
                models TEXT NOT NULL DEFAULT '[]',
                enabled INTEGER NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                weight INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                model TEXT NOT NULL,
                display_name TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                sort_index INTEGER NOT NULL DEFAULT 0,
                weight INTEGER NOT NULL DEFAULT 1,
                response_ms TEXT,
                cooldown_until INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (channel_id) REFERENCES channels(id)
            );
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                key TEXT UNIQUE NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 0,
                usage_limit INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                channel_id TEXT,
                channel_name TEXT,
                model TEXT,
                api_key_id TEXT,
                request_type TEXT NOT NULL DEFAULT 'chat',
                status_code INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL DEFAULT 0,
                prompt_tokens INTEGER NOT NULL DEFAULT 0,
                completion_tokens INTEGER NOT NULL DEFAULT 0,
                error TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                circuit_breaker_threshold INTEGER NOT NULL DEFAULT 5,
                circuit_breaker_reset_time INTEGER NOT NULL DEFAULT 300,
                retry_times INTEGER NOT NULL DEFAULT 3,
                timeout INTEGER NOT NULL DEFAULT 60000,
                auto_select_new_models INTEGER NOT NULL DEFAULT 1,
                max_tokens_per_month INTEGER NOT NULL DEFAULT 0,
                default_model TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS model_circuit_breaker (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                model TEXT NOT NULL,
                failures INTEGER NOT NULL DEFAULT 0,
                last_failure INTEGER,
                open INTEGER NOT NULL DEFAULT 0,
                cooldown_until INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (channel_id) REFERENCES channels(id),
                UNIQUE(channel_id, model)
            );
        ",
        )?;
        Ok(())
    }

    fn seed_defaults(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
        if count == 0 {
            let hash = crate::auth::hash_password("admin");
            conn.execute(
                "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![uuid::Uuid::new_v4().to_string(), "admin", hash, chrono::Utc::now().to_rfc3339()],
            )?;
        }
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO settings (id, circuit_breaker_threshold, circuit_breaker_reset_time, retry_times, timeout, auto_select_new_models, max_tokens_per_month, default_model) VALUES (1, 5, 300, 3, 60000, 1, 0, '')",
                [],
            )?;
        }
        Ok(())
    }

    pub fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<(String, String)>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let hash = crate::auth::hash_password(password);
        let result = conn.query_row(
            "SELECT id, username FROM users WHERE username = ?1 AND password_hash = ?2",
            params![username, hash],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        match result {
            Ok(pair) => Ok(Some(pair)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn change_password(
        &self,
        username: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let old_hash = crate::auth::hash_password(old_password);
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE username = ?1 AND password_hash = ?2",
            params![username, old_hash],
            |r| r.get(0),
        )?;
        if count == 0 {
            return Ok(false);
        }
        let new_hash = crate::auth::hash_password(new_password);
        conn.execute(
            "UPDATE users SET password_hash = ?1 WHERE username = ?2",
            params![new_hash, username],
        )?;
        Ok(true)
    }

    pub fn store_session(
        &self,
        token: &str,
        username: &str,
        expires_at: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sessions (token, username, expires_at) VALUES (?1, ?2, ?3)",
            params![token, username, expires_at],
        )?;
        Ok(())
    }

    pub fn verify_session(&self, token: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let result = conn.query_row(
            "SELECT username FROM sessions WHERE token = ?1 AND expires_at > ?2",
            params![token, now],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(u) => Ok(Some(u)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn delete_session(&self, token: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE token = ?1", params![token])?;
        Ok(())
    }

    pub fn list_entries(&self) -> Result<Vec<ApiEntry>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT e.id, e.channel_id, c.name, e.model, e.display_name, e.enabled, e.priority, e.sort_index, e.weight, e.response_ms, e.cooldown_until, e.created_at FROM entries e LEFT JOIN channels c ON e.channel_id = c.id ORDER BY e.priority DESC, e.sort_index ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ApiEntry {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                channel_name: row.get(2)?,
                model: row.get(3)?,
                display_name: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                priority: row.get(6)?,
                sort_index: row.get(7)?,
                weight: row.get(8)?,
                response_ms: row.get(9)?,
                cooldown_until: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        rows.collect()
    }

    pub fn create_entry(&self, req: &CreateEntry) -> Result<ApiEntry, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let priority = req.priority.unwrap_or(0);
        let max_index: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_index), -1) FROM entries",
            [],
            |r| r.get(0),
        )?;
        let sort_index = max_index + 1;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO entries (id, channel_id, model, display_name, enabled, priority, sort_index, weight, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, 1, ?7)",
            params![id, req.channel_id, req.model, req.display_name, priority, sort_index, now],
        )?;
        let channel_name: Option<String> = conn
            .query_row(
                "SELECT name FROM channels WHERE id = ?1",
                params![req.channel_id],
                |r| r.get(0),
            )
            .ok();
        Ok(ApiEntry {
            id,
            channel_id: req.channel_id.clone(),
            channel_name,
            model: req.model.clone(),
            display_name: req.display_name.clone(),
            enabled: true,
            priority,
            sort_index,
            weight: 1,
            response_ms: None,
            cooldown_until: None,
            created_at: now,
        })
    }

    pub fn toggle_entry(&self, id: &str, enabled: bool) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE entries SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_entry(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn update_entry(
        &self,
        id: &str,
        req: &UpdateEntry,
    ) -> Result<Option<ApiEntry>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let existing = conn.query_row(
            "SELECT id, channel_id, model, display_name, enabled, priority, sort_index, weight, response_ms, cooldown_until, created_at FROM entries WHERE id = ?1",
            params![id],
            |row| Ok(ApiEntry {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                channel_name: None,
                model: row.get(2)?,
                display_name: row.get(3)?,
                enabled: row.get::<_, i32>(4)? != 0,
                priority: row.get(5)?,
                sort_index: row.get(6)?,
                weight: row.get(7)?,
                response_ms: row.get(8)?,
                cooldown_until: row.get(9)?,
                created_at: row.get(10)?,
            }),
        );
        let mut entry = match existing {
            Ok(e) => e,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };
        if let Some(v) = &req.channel_id {
            entry.channel_id = v.clone();
        }
        if let Some(v) = &req.model {
            entry.model = v.clone();
        }
        if let Some(v) = &req.display_name {
            entry.display_name = Some(v.clone());
        }
        if let Some(v) = req.priority {
            entry.priority = v;
        }
        if let Some(v) = req.weight {
            entry.weight = v;
        }
        conn.execute(
            "UPDATE entries SET channel_id = ?1, model = ?2, display_name = ?3, priority = ?4, weight = ?5 WHERE id = ?6",
            params![entry.channel_id, entry.model, entry.display_name, entry.priority, entry.weight, id],
        )?;
        let channel_name: Option<String> = conn
            .query_row(
                "SELECT name FROM channels WHERE id = ?1",
                params![entry.channel_id],
                |r| r.get(0),
            )
            .ok();
        entry.channel_name = channel_name;
        Ok(Some(entry))
    }

    pub fn reorder_entries(&self, ordered_ids: &[String]) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        for (i, id) in ordered_ids.iter().enumerate() {
            conn.execute(
                "UPDATE entries SET sort_index = ?1 WHERE id = ?2",
                params![i as i32, id],
            )?;
        }
        Ok(())
    }

    pub fn list_channels(&self) -> Result<Vec<Channel>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, api_type, base_url, api_key, models, enabled, priority, weight, created_at, updated_at FROM channels ORDER BY priority DESC, weight DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Channel {
                id: row.get(0)?,
                name: row.get(1)?,
                api_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                priority: row.get(7)?,
                weight: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_channel(&self, id: &str) -> Result<Option<Channel>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, name, api_type, base_url, api_key, models, enabled, priority, weight, created_at, updated_at FROM channels WHERE id = ?1",
            params![id],
            |row| Ok(Channel {
                id: row.get(0)?,
                name: row.get(1)?,
                api_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                priority: row.get(7)?,
                weight: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            }),
        );
        match result {
            Ok(ch) => Ok(Some(ch)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn create_channel(&self, req: &CreateChannel) -> Result<Channel, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO channels (id, name, api_type, base_url, api_key, models, enabled, priority, weight, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9, ?10)",
            params![id, req.name, req.api_type, req.base_url, req.api_key, req.models, req.priority, req.weight, now, now],
        )?;
        Ok(Channel {
            id,
            name: req.name.clone(),
            api_type: req.api_type.clone(),
            base_url: req.base_url.clone(),
            api_key: req.api_key.clone(),
            models: req.models.clone(),
            enabled: true,
            priority: req.priority,
            weight: req.weight,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn update_channel(
        &self,
        id: &str,
        req: &UpdateChannel,
    ) -> Result<Option<Channel>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let existing = conn.query_row(
            "SELECT id, name, api_type, base_url, api_key, models, enabled, priority, weight, created_at, updated_at FROM channels WHERE id = ?1",
            params![id],
            |row| Ok(Channel {
                id: row.get(0)?,
                name: row.get(1)?,
                api_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                priority: row.get(7)?,
                weight: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            }),
        );
        let mut ch = match existing {
            Ok(c) => c,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };
        if let Some(v) = &req.name {
            ch.name = v.clone();
        }
        if let Some(v) = &req.api_type {
            ch.api_type = v.clone();
        }
        if let Some(v) = &req.base_url {
            ch.base_url = v.clone();
        }
        if let Some(v) = &req.api_key {
            ch.api_key = v.clone();
        }
        if let Some(v) = &req.models {
            ch.models = v.clone();
        }
        if let Some(v) = req.enabled {
            ch.enabled = v;
        }
        if let Some(v) = req.priority {
            ch.priority = v;
        }
        if let Some(v) = req.weight {
            ch.weight = v;
        }
        ch.updated_at = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE channels SET name = ?1, api_type = ?2, base_url = ?3, api_key = ?4, models = ?5, enabled = ?6, priority = ?7, weight = ?8, updated_at = ?9 WHERE id = ?10",
            params![ch.name, ch.api_type, ch.base_url, ch.api_key, ch.models, ch.enabled as i32, ch.priority, ch.weight, ch.updated_at, id],
        )?;
        Ok(Some(ch))
    }

    pub fn delete_channel(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM entries WHERE channel_id = ?1", params![id])?;
        let affected = conn.execute("DELETE FROM channels WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn toggle_channel(&self, id: &str, enabled: bool) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE channels SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i32, now, id],
        )?;
        Ok(affected > 0)
    }

    pub fn update_channel_models(
        &self,
        id: &str,
        models: &[String],
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let models_json = serde_json::to_string(models).unwrap_or_default();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE channels SET models = ?1, updated_at = ?2 WHERE id = ?3",
            params![models_json, now, id],
        )?;
        Ok(())
    }

    pub fn list_api_keys(&self) -> Result<Vec<ApiKey>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, key, usage_count, usage_limit, enabled, created_at FROM api_keys ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ApiKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key: row.get(2)?,
                usage_count: row.get(3)?,
                usage_limit: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn create_api_key(&self, req: &CreateApiKey) -> Result<ApiKey, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let key = format!("sk-{}", hex::encode(rand::random::<[u8; 16]>()));
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO api_keys (id, name, key, usage_count, usage_limit, enabled, created_at) VALUES (?1, ?2, ?3, 0, ?4, 1, ?5)",
            params![id, req.name, key, req.usage_limit, now],
        )?;
        Ok(ApiKey {
            id,
            name: req.name.clone(),
            key,
            usage_count: 0,
            usage_limit: req.usage_limit,
            enabled: true,
            created_at: now,
        })
    }

    pub fn delete_api_key(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM api_keys WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn toggle_api_key(&self, id: &str, enabled: bool) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE api_keys SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )?;
        Ok(affected > 0)
    }

    pub fn validate_api_key(&self, key: &str) -> Result<Option<ApiKey>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, name, key, usage_count, usage_limit, enabled, created_at FROM api_keys WHERE key = ?1 AND enabled = 1",
            params![key],
            |row| Ok(ApiKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key: row.get(2)?,
                usage_count: row.get(3)?,
                usage_limit: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            }),
        );
        match result {
            Ok(k) => Ok(Some(k)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn increment_key_usage(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE api_keys SET usage_count = usage_count + 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_logs(
        &self,
        limit: i64,
        offset: i64,
        channel_id: Option<&str>,
        model: Option<&str>,
        status: Option<i32>,
    ) -> Result<(Vec<RequestLog>, i64), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM logs", [], |r| r.get(0))?;
        let mut sql = "SELECT id, channel_id, channel_name, model, api_key_id, request_type, status_code, latency_ms, prompt_tokens, completion_tokens, error, created_at FROM logs".to_string();
        let mut conditions = Vec::new();
        if channel_id.is_some() {
            conditions.push("channel_id = ?1");
        }
        if model.is_some() {
            conditions.push("model LIKE ?2");
        }
        if status.is_some() {
            conditions.push("status_code >= ?3");
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?4 OFFSET ?5");
        let mut stmt = conn.prepare(&sql)?;
        let channel_id_val = channel_id.map(|s| s.to_string());
        let model_val = model.map(|s| format!("%{}%", s));
        let rows = stmt.query_map(
            params![
                channel_id_val,
                model_val,
                status.unwrap_or(0),
                limit,
                offset
            ],
            |row| {
                Ok(RequestLog {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    channel_name: row.get(2)?,
                    model: row.get(3)?,
                    api_key_id: row.get(4)?,
                    request_type: row.get(5)?,
                    status_code: row.get(6)?,
                    latency_ms: row.get(7)?,
                    prompt_tokens: row.get(8)?,
                    completion_tokens: row.get(9)?,
                    error: row.get(10)?,
                    created_at: row.get(11)?,
                })
            },
        )?;
        let logs = rows.collect::<Result<Vec<_>, _>>()?;
        Ok((logs, total))
    }

    pub fn get_log_stats(&self) -> Result<LogStats, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM logs", [], |r| r.get(0))?;
        let success: i64 = conn.query_row(
            "SELECT COUNT(*) FROM logs WHERE status_code >= 200 AND status_code < 300",
            [],
            |r| r.get(0),
        )?;
        let errors: i64 = conn.query_row(
            "SELECT COUNT(*) FROM logs WHERE status_code >= 400 OR status_code = 0",
            [],
            |r| r.get(0),
        )?;
        let today_start = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let today: i64 = conn.query_row(
            "SELECT COUNT(*) FROM logs WHERE created_at >= ?1",
            params![today_start],
            |r| r.get(0),
        )?;
        Ok(LogStats {
            total,
            success,
            errors,
            today,
        })
    }

    pub fn clear_logs(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM logs", [])?;
        Ok(())
    }

    pub fn create_log(&self, log: &RequestLog) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO logs (id, channel_id, channel_name, model, api_key_id, request_type, status_code, latency_ms, prompt_tokens, completion_tokens, error, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![log.id, log.channel_id, log.channel_name, log.model, log.api_key_id, log.request_type, log.status_code, log.latency_ms, log.prompt_tokens, log.completion_tokens, log.error, log.created_at],
        )?;
        Ok(())
    }

    pub fn get_settings(&self) -> Result<Settings, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT circuit_breaker_threshold, circuit_breaker_reset_time, retry_times, timeout, auto_select_new_models, max_tokens_per_month, default_model FROM settings WHERE id = 1",
            [],
            |row| Ok(Settings {
                circuit_breaker_threshold: row.get(0)?,
                circuit_breaker_reset_time: row.get(1)?,
                retry_times: row.get(2)?,
                timeout: row.get(3)?,
                auto_select_new_models: row.get::<_, i32>(4)? != 0,
                max_tokens_per_month: row.get(5)?,
                default_model: row.get(6)?,
            }),
        );
        match result {
            Ok(s) => Ok(s),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Settings::default()),
            Err(e) => Err(e),
        }
    }

    pub fn update_settings(&self, settings: &Settings) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE settings SET circuit_breaker_threshold = ?1, circuit_breaker_reset_time = ?2, retry_times = ?3, timeout = ?4, auto_select_new_models = ?5, max_tokens_per_month = ?6, default_model = ?7 WHERE id = 1",
            params![settings.circuit_breaker_threshold, settings.circuit_breaker_reset_time, settings.retry_times, settings.timeout, settings.auto_select_new_models as i32, settings.max_tokens_per_month, settings.default_model],
        )?;
        Ok(())
    }

    pub fn get_dashboard_stats(&self) -> Result<DashboardStats, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let total_requests: i64 = conn
            .query_row("SELECT COUNT(*) FROM logs", [], |r| r.get(0))
            .unwrap_or(0);
        let today_start = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let today_requests: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM logs WHERE created_at >= ?1",
                params![today_start],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let total_prompt_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(prompt_tokens), 0) FROM logs",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let total_completion_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(completion_tokens), 0) FROM logs",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let today_prompt_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(prompt_tokens), 0) FROM logs WHERE created_at >= ?1",
                params![today_start],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let today_completion_tokens: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(completion_tokens), 0) FROM logs WHERE created_at >= ?1",
                params![today_start],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(DashboardStats {
            total_requests,
            today_requests,
            total_prompt_tokens,
            total_completion_tokens,
            today_prompt_tokens,
            today_completion_tokens,
        })
    }

    pub fn get_model_ranking(&self, limit: i32) -> Result<Vec<ModelRanking>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT model, COUNT(*) as count, COALESCE(SUM(prompt_tokens + completion_tokens), 0) as tokens FROM logs WHERE model IS NOT NULL AND model != '' GROUP BY model ORDER BY count DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(ModelRanking {
                model: row.get(0)?,
                count: row.get(1)?,
                tokens: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_chart_data(
        &self,
        granularity: &str,
    ) -> Result<Vec<ChartDataPoint>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let date_format = match granularity {
            "hour" => "%Y-%m-%d %H:00",
            _ => "%Y-%m-%d",
        };
        let sql = format!(
            "SELECT strftime('{}', created_at) as time, model, COUNT(*) as value FROM logs WHERE model IS NOT NULL AND model != '' GROUP BY time, model ORDER BY time DESC LIMIT 100",
            date_format
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(ChartDataPoint {
                time: row.get(0)?,
                model: row.get(1)?,
                value: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_channel_tree_stats(&self) -> Result<Vec<ChannelTreeStats>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        // 获取所有渠道
        let channels: Vec<(String, String)> = {
            let mut stmt = conn.prepare("SELECT id, name FROM channels ORDER BY name")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let mut result = Vec::new();
        for (channel_id, channel_name) in channels {
            // 获取该渠道的模型统计
            let mut stmt = conn.prepare(
                "SELECT model, COUNT(*) as requests, COALESCE(SUM(prompt_tokens + completion_tokens), 0) as tokens,
                        SUM(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 ELSE 0 END) as success_count,
                        SUM(CASE WHEN status_code >= 400 OR status_code = 0 THEN 1 ELSE 0 END) as error_count
                 FROM logs WHERE channel_id = ?1 AND model IS NOT NULL AND model != ''
                 GROUP BY model ORDER BY requests DESC"
            )?;
            let model_rows = stmt.query_map(params![&channel_id], |row| {
                Ok(ModelTreeStats {
                    model: row.get(0)?,
                    requests: row.get(1)?,
                    tokens: row.get(2)?,
                    success_count: row.get(3)?,
                    error_count: row.get(4)?,
                })
            })?;
            let models: Vec<ModelTreeStats> = model_rows.collect::<Result<Vec<_>, _>>()?;

            let total_requests: i64 = models.iter().map(|m| m.requests).sum();
            let total_tokens: i64 = models.iter().map(|m| m.tokens).sum();

            if !models.is_empty() {
                result.push(ChannelTreeStats {
                    channel_id,
                    channel_name,
                    total_requests,
                    total_tokens,
                    models,
                });
            }
        }
        // 按总请求数排序
        result.sort_by_key(|b| std::cmp::Reverse(b.total_requests));
        Ok(result)
    }

    pub fn reset_model_breaker(
        &self,
        channel_id: &str,
        model: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE model_circuit_breaker SET failures = 0, open = 0, cooldown_until = NULL, updated_at = ?1 WHERE channel_id = ?2 AND model = ?3",
            params![chrono::Utc::now().to_rfc3339(), channel_id, model],
        )?;
        Ok(())
    }
}
