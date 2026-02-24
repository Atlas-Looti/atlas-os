//! Local SQLite database for caching trades, orders, and sync state.
//!
//! All Decimal values are stored as TEXT and parsed back with `rust_decimal` on read.
//! Uses WAL mode for concurrent read safety.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use atlas_types::db::{FillFilter, OrderFilter};

/// A cached fill row read from the database.
#[derive(Debug, Clone)]
pub struct DbFill {
    pub coin: String,
    pub px: String,
    pub sz: String,
    pub side: String,
    pub time_ms: i64,
    pub fee: String,
    pub hash: String,
    pub oid: i64,
    pub closed_pnl: String,
}

/// A cached order row read from the database.
#[derive(Debug, Clone)]
pub struct DbOrder {
    pub coin: String,
    pub side: String,
    pub limit_px: String,
    pub sz: String,
    pub oid: i64,
    pub timestamp_ms: i64,
    pub status: String,
    pub order_type: String,
}

/// Local SQLite database handle.
pub struct AtlasDb {
    conn: Connection,
}

impl AtlasDb {
    /// Open (or create) the database at `~/.atlas-os/data/atlas.db`.
    /// Enables WAL mode and creates tables if they don't exist.
    pub fn open() -> Result<Self> {
        let db_path = crate::workspace::root_dir()?.join("data/atlas.db");

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create DB directory: {}", parent.display()))?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

        // Enable WAL mode for concurrent access
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Create all tables and indices if they don't exist.
    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS fills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                coin TEXT NOT NULL,
                px TEXT NOT NULL,
                sz TEXT NOT NULL,
                side TEXT NOT NULL,
                time_ms INTEGER NOT NULL,
                fee TEXT NOT NULL,
                hash TEXT UNIQUE NOT NULL,
                oid INTEGER NOT NULL,
                closed_pnl TEXT NOT NULL DEFAULT '0'
            );
            CREATE INDEX IF NOT EXISTS idx_fills_coin ON fills(coin);
            CREATE INDEX IF NOT EXISTS idx_fills_time ON fills(time_ms);

            CREATE TABLE IF NOT EXISTS orders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                coin TEXT NOT NULL,
                side TEXT NOT NULL,
                limit_px TEXT NOT NULL,
                sz TEXT NOT NULL,
                oid INTEGER UNIQUE NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                status TEXT NOT NULL,
                order_type TEXT NOT NULL DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS idx_orders_coin ON orders(coin);
            CREATE INDEX IF NOT EXISTS idx_orders_time ON orders(timestamp_ms);

            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_ms INTEGER NOT NULL
            );
            "
        ).context("Failed to initialize database tables")?;

        Ok(())
    }

    // ─── Fills ──────────────────────────────────────────────────────

    /// Insert fills into the database (upsert by hash, skips duplicates).
    /// Returns the number of newly inserted rows.
    pub fn insert_fills(&self, fills: &[DbFill]) -> Result<usize> {
        let mut inserted = 0usize;
        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO fills (coin, px, sz, side, time_ms, fee, hash, oid, closed_pnl)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            )?;

            for fill in fills {
                let rows = stmt.execute(params![
                    fill.coin,
                    fill.px,
                    fill.sz,
                    fill.side,
                    fill.time_ms,
                    fill.fee,
                    fill.hash,
                    fill.oid,
                    fill.closed_pnl,
                ])?;
                inserted += rows;
            }
        }

        tx.commit()?;
        Ok(inserted)
    }

    /// Query fills with optional filters.
    pub fn query_fills(&self, filter: &FillFilter) -> Result<Vec<DbFill>> {
        let mut sql = String::from(
            "SELECT coin, px, sz, side, time_ms, fee, hash, oid, closed_pnl FROM fills WHERE 1=1"
        );
        let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref coin) = filter.coin {
            sql.push_str(" AND coin = ?");
            bind_values.push(Box::new(coin.clone()));
        }
        if let Some(from) = filter.from_ms {
            sql.push_str(" AND time_ms >= ?");
            bind_values.push(Box::new(from));
        }
        if let Some(to) = filter.to_ms {
            sql.push_str(" AND time_ms <= ?");
            bind_values.push(Box::new(to));
        }

        sql.push_str(" ORDER BY time_ms DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            bind_values.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DbFill {
                coin: row.get(0)?,
                px: row.get(1)?,
                sz: row.get(2)?,
                side: row.get(3)?,
                time_ms: row.get(4)?,
                fee: row.get(5)?,
                hash: row.get(6)?,
                oid: row.get(7)?,
                closed_pnl: row.get(8)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    /// Get the most recent fill timestamp in the database.
    pub fn last_fill_time(&self) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT MAX(time_ms) FROM fills"
        )?;
        let result: Option<i64> = stmt.query_row([], |row| row.get(0))?;
        Ok(result)
    }

    // ─── Orders ─────────────────────────────────────────────────────

    /// Insert orders into the database (upsert by oid).
    /// Returns the number of newly inserted rows.
    pub fn insert_orders(&self, orders: &[DbOrder]) -> Result<usize> {
        let mut inserted = 0usize;
        let tx = self.conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO orders (coin, side, limit_px, sz, oid, timestamp_ms, status, order_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            )?;

            for order in orders {
                let rows = stmt.execute(params![
                    order.coin,
                    order.side,
                    order.limit_px,
                    order.sz,
                    order.oid,
                    order.timestamp_ms,
                    order.status,
                    order.order_type,
                ])?;
                inserted += rows;
            }
        }

        tx.commit()?;
        Ok(inserted)
    }

    /// Query orders with optional filters.
    pub fn query_orders(&self, filter: &OrderFilter) -> Result<Vec<DbOrder>> {
        let mut sql = String::from(
            "SELECT coin, side, limit_px, sz, oid, timestamp_ms, status, order_type FROM orders WHERE 1=1"
        );
        let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref coin) = filter.coin {
            sql.push_str(" AND coin = ?");
            bind_values.push(Box::new(coin.clone()));
        }
        if let Some(ref status) = filter.status {
            sql.push_str(" AND status = ?");
            bind_values.push(Box::new(status.clone()));
        }

        sql.push_str(" ORDER BY timestamp_ms DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            bind_values.iter().map(|b| b.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(DbOrder {
                coin: row.get(0)?,
                side: row.get(1)?,
                limit_px: row.get(2)?,
                sz: row.get(3)?,
                oid: row.get(4)?,
                timestamp_ms: row.get(5)?,
                status: row.get(6)?,
                order_type: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ─── Sync State ─────────────────────────────────────────────────

    /// Get a sync state value by key.
    pub fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT value FROM sync_state WHERE key = ?1"
        )?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set a sync state value.
    pub fn set_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value, updated_ms) VALUES (?1, ?2, ?3)",
            params![key, value, now_ms],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = AtlasDb::open_in_memory().unwrap();
        assert!(db.last_fill_time().unwrap().is_none());
    }

    #[test]
    fn test_insert_and_query_fills() {
        let db = AtlasDb::open_in_memory().unwrap();

        let fills = vec![
            DbFill {
                coin: "ETH".into(),
                px: "3500.00".into(),
                sz: "0.5".into(),
                side: "Buy".into(),
                time_ms: 1700000000000,
                fee: "1.75".into(),
                hash: "0xabc123".into(),
                oid: 100,
                closed_pnl: "0".into(),
            },
            DbFill {
                coin: "BTC".into(),
                px: "105000.00".into(),
                sz: "0.01".into(),
                side: "Sell".into(),
                time_ms: 1700000001000,
                fee: "5.25".into(),
                hash: "0xdef456".into(),
                oid: 101,
                closed_pnl: "50.00".into(),
            },
        ];

        let inserted = db.insert_fills(&fills).unwrap();
        assert_eq!(inserted, 2);

        // Query all
        let all = db.query_fills(&FillFilter::default()).unwrap();
        assert_eq!(all.len(), 2);

        // Query by coin
        let eth_only = db.query_fills(&FillFilter {
            coin: Some("ETH".into()),
            ..Default::default()
        }).unwrap();
        assert_eq!(eth_only.len(), 1);
        assert_eq!(eth_only[0].coin, "ETH");

        // Query by time range
        let time_filter = db.query_fills(&FillFilter {
            from_ms: Some(1700000000500),
            ..Default::default()
        }).unwrap();
        assert_eq!(time_filter.len(), 1);
        assert_eq!(time_filter[0].coin, "BTC");

        // Query with limit
        let limited = db.query_fills(&FillFilter {
            limit: Some(1),
            ..Default::default()
        }).unwrap();
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn test_fill_dedup_by_hash() {
        let db = AtlasDb::open_in_memory().unwrap();

        let fill = DbFill {
            coin: "ETH".into(),
            px: "3500.00".into(),
            sz: "0.5".into(),
            side: "Buy".into(),
            time_ms: 1700000000000,
            fee: "1.75".into(),
            hash: "0xabc123".into(),
            oid: 100,
            closed_pnl: "0".into(),
        };

        let inserted1 = db.insert_fills(&[fill.clone()]).unwrap();
        assert_eq!(inserted1, 1);

        // Insert same hash again — should be ignored
        let inserted2 = db.insert_fills(&[fill]).unwrap();
        assert_eq!(inserted2, 0);

        let all = db.query_fills(&FillFilter::default()).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_last_fill_time() {
        let db = AtlasDb::open_in_memory().unwrap();

        assert!(db.last_fill_time().unwrap().is_none());

        let fills = vec![
            DbFill {
                coin: "ETH".into(),
                px: "3500".into(),
                sz: "0.5".into(),
                side: "Buy".into(),
                time_ms: 1000,
                fee: "1".into(),
                hash: "h1".into(),
                oid: 1,
                closed_pnl: "0".into(),
            },
            DbFill {
                coin: "BTC".into(),
                px: "50000".into(),
                sz: "0.01".into(),
                side: "Sell".into(),
                time_ms: 2000,
                fee: "2".into(),
                hash: "h2".into(),
                oid: 2,
                closed_pnl: "0".into(),
            },
        ];

        db.insert_fills(&fills).unwrap();
        assert_eq!(db.last_fill_time().unwrap(), Some(2000));
    }

    #[test]
    fn test_insert_and_query_orders() {
        let db = AtlasDb::open_in_memory().unwrap();

        let orders = vec![
            DbOrder {
                coin: "ETH".into(),
                side: "Buy".into(),
                limit_px: "3500.00".into(),
                sz: "0.5".into(),
                oid: 200,
                timestamp_ms: 1700000000000,
                status: "filled".into(),
                order_type: "Limit".into(),
            },
            DbOrder {
                coin: "BTC".into(),
                side: "Sell".into(),
                limit_px: "105000.00".into(),
                sz: "0.01".into(),
                oid: 201,
                timestamp_ms: 1700000001000,
                status: "open".into(),
                order_type: "Limit".into(),
            },
        ];

        let inserted = db.insert_orders(&orders).unwrap();
        assert_eq!(inserted, 2);

        // Query all
        let all = db.query_orders(&OrderFilter::default()).unwrap();
        assert_eq!(all.len(), 2);

        // Query by coin
        let btc_only = db.query_orders(&OrderFilter {
            coin: Some("BTC".into()),
            ..Default::default()
        }).unwrap();
        assert_eq!(btc_only.len(), 1);

        // Query by status
        let filled = db.query_orders(&OrderFilter {
            status: Some("filled".into()),
            ..Default::default()
        }).unwrap();
        assert_eq!(filled.len(), 1);
        assert_eq!(filled[0].coin, "ETH");
    }

    #[test]
    fn test_order_upsert_by_oid() {
        let db = AtlasDb::open_in_memory().unwrap();

        let order = DbOrder {
            coin: "ETH".into(),
            side: "Buy".into(),
            limit_px: "3500.00".into(),
            sz: "0.5".into(),
            oid: 200,
            timestamp_ms: 1700000000000,
            status: "open".into(),
            order_type: "Limit".into(),
        };

        db.insert_orders(&[order]).unwrap();

        // Update status to filled
        let updated = DbOrder {
            coin: "ETH".into(),
            side: "Buy".into(),
            limit_px: "3500.00".into(),
            sz: "0.5".into(),
            oid: 200,
            timestamp_ms: 1700000000000,
            status: "filled".into(),
            order_type: "Limit".into(),
        };

        db.insert_orders(&[updated]).unwrap();

        let all = db.query_orders(&OrderFilter::default()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].status, "filled");
    }

    #[test]
    fn test_sync_state() {
        let db = AtlasDb::open_in_memory().unwrap();

        assert!(db.get_sync_state("last_fill_sync").unwrap().is_none());

        db.set_sync_state("last_fill_sync", "1700000000000").unwrap();
        let val = db.get_sync_state("last_fill_sync").unwrap();
        assert_eq!(val.as_deref(), Some("1700000000000"));

        // Update
        db.set_sync_state("last_fill_sync", "1700000001000").unwrap();
        let val = db.get_sync_state("last_fill_sync").unwrap();
        assert_eq!(val.as_deref(), Some("1700000001000"));
    }
}
