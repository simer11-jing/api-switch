use crate::db::Database;
use crate::models::{ModelBreakerStatus, ResetModelBreakerRequest};
use rusqlite::params;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

struct ChannelBreakerState {
    failures: u32,
    last_failure: Option<Instant>,
    open: bool,
}

pub struct CircuitBreakerManager {
    channel_states: Arc<RwLock<HashMap<String, ChannelBreakerState>>>,
    threshold: u32,
    reset_duration: Duration,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            channel_states: Arc::new(RwLock::new(HashMap::new())),
            threshold: 5,
            reset_duration: Duration::from_secs(300),
        }
    }

    pub async fn is_channel_available(&self, channel_id: &str) -> bool {
        let states = self.channel_states.read().await;
        if let Some(state) = states.get(channel_id) {
            if state.open {
                if let Some(last) = state.last_failure {
                    if last.elapsed() > self.reset_duration {
                        return true;
                    }
                }
                return false;
            }
        }
        true
    }

    pub async fn record_channel_success(&self, channel_id: &str) {
        let mut states = self.channel_states.write().await;
        if let Some(state) = states.get_mut(channel_id) {
            state.failures = 0;
            state.open = false;
            state.last_failure = None;
        }
    }

    pub async fn record_channel_failure(&self, channel_id: &str) {
        let mut states = self.channel_states.write().await;
        let state = states.entry(channel_id.to_string()).or_insert(ChannelBreakerState {
            failures: 0,
            last_failure: None,
            open: false,
        });
        state.failures += 1;
        state.last_failure = Some(Instant::now());
        if state.failures >= self.threshold {
            state.open = true;
            tracing::warn!("🔴 Channel circuit breaker OPEN for {}", channel_id);
        }
    }

    pub async fn is_model_available(&self, db: &Database, channel_id: &str, model: &str) -> bool {
        if !self.is_channel_available(channel_id).await {
            return false;
        }
        let conn = db.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let result: Result<(i32, i32, Option<i64>), _> = conn.query_row(
            "SELECT failures, open, cooldown_until FROM model_circuit_breaker WHERE channel_id = ?1 AND model = ?2",
            params![channel_id, model],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );
        match result {
            Ok((failures, open, cooldown_until)) => {
                if open == 1 {
                    if let Some(cooldown) = cooldown_until {
                        if now < cooldown {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if failures >= 3 {
                    return false;
                }
            }
            Err(_) => {}
        }
        true
    }

    pub async fn record_model_failure(&self, db: &Database, channel_id: &str, model: &str) {
        let now = chrono::Utc::now();
        let now_ts = now.timestamp();
        let cooldown_ts = now_ts + 300;

        {
            let conn = db.conn.lock().unwrap();
            let existing: Result<i32, _> = conn.query_row(
                "SELECT failures FROM model_circuit_breaker WHERE channel_id = ?1 AND model = ?2",
                params![channel_id, model],
                |row| row.get(0),
            );

            if existing.is_ok() {
                let failures: i32 = conn.query_row(
                    "SELECT failures FROM model_circuit_breaker WHERE channel_id = ?1 AND model = ?2",
                    params![channel_id, model],
                    |row| row.get(0),
                ).unwrap_or(0);

                if failures + 1 >= 3 {
                    conn.execute(
                        "UPDATE model_circuit_breaker SET failures = failures + 1, open = 1, cooldown_until = ?1, updated_at = ?2 WHERE channel_id = ?3 AND model = ?4",
                        params![cooldown_ts, now.to_rfc3339(), channel_id, model],
                    ).ok();
                    tracing::warn!("🔴 Model circuit breaker OPEN for {}/{}", channel_id, model);
                } else {
                    conn.execute(
                        "UPDATE model_circuit_breaker SET failures = failures + 1, last_failure = ?1, updated_at = ?2 WHERE channel_id = ?3 AND model = ?4",
                        params![now_ts, now.to_rfc3339(), channel_id, model],
                    ).ok();
                }
            } else {
                conn.execute(
                    "INSERT INTO model_circuit_breaker (id, channel_id, model, failures, last_failure, open, cooldown_until, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, 0, NULL, ?5, ?6)",
                    params![uuid::Uuid::new_v4().to_string(), channel_id, model, now_ts, now.to_rfc3339(), now.to_rfc3339()],
                ).ok();
            }
        }
    }

    pub async fn record_model_success(&self, db: &Database, channel_id: &str, model: &str) {
        let conn = db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE model_circuit_breaker SET failures = 0, open = 0, cooldown_until = NULL, updated_at = ?1 WHERE channel_id = ?2 AND model = ?3",
            params![now, channel_id, model],
        ).ok();
    }

    pub async fn reset_model_breaker(&self, db: &Database, channel_id: &str, model: &str) {
        let conn = db.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE model_circuit_breaker SET failures = 0, open = 0, cooldown_until = NULL, updated_at = ?1 WHERE channel_id = ?2 AND model = ?3",
            params![now, channel_id, model],
        ).ok();
    }

    pub async fn get_model_breaker_statuses(&self, db: &Database) -> Vec<ModelBreakerStatus> {
        let conn = db.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT channel_id, model, failures, open, cooldown_until FROM model_circuit_breaker"
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt.query_map([], |row| {
            Ok(ModelBreakerStatus {
                channel_id: row.get(0)?,
                model: row.get(1)?,
                failures: row.get(2)?,
                open: row.get::<_, i32>(3)? != 0,
                cooldown_until: row.get(4)?,
            })
        });
        match rows {
            Ok(r) => r.filter_map(|x| x.ok()).collect(),
            Err(_) => vec![],
        }
    }
}
