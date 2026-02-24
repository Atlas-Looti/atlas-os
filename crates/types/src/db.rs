//! Database filter types for querying cached data.

/// Filter for querying cached fills from the local database.
#[derive(Debug, Clone, Default)]
pub struct FillFilter {
    /// Filter by coin symbol (e.g. "ETH").
    pub coin: Option<String>,
    /// Start time (inclusive) in milliseconds since epoch.
    pub from_ms: Option<i64>,
    /// End time (inclusive) in milliseconds since epoch.
    pub to_ms: Option<i64>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

/// Filter for querying cached orders from the local database.
#[derive(Debug, Clone, Default)]
pub struct OrderFilter {
    /// Filter by coin symbol (e.g. "ETH").
    pub coin: Option<String>,
    /// Filter by order status (e.g. "open", "filled", "canceled").
    pub status: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_filter_default() {
        let f = FillFilter::default();
        assert!(f.coin.is_none());
        assert!(f.from_ms.is_none());
        assert!(f.to_ms.is_none());
        assert!(f.limit.is_none());
    }

    #[test]
    fn test_order_filter_default() {
        let f = OrderFilter::default();
        assert!(f.coin.is_none());
        assert!(f.status.is_none());
        assert!(f.limit.is_none());
    }

    #[test]
    fn test_fill_filter_with_values() {
        let f = FillFilter {
            coin: Some("ETH".to_string()),
            from_ms: Some(1000),
            to_ms: Some(2000),
            limit: Some(50),
        };
        assert_eq!(f.coin.as_deref(), Some("ETH"));
        assert_eq!(f.from_ms, Some(1000));
        assert_eq!(f.to_ms, Some(2000));
        assert_eq!(f.limit, Some(50));
    }

    #[test]
    fn test_order_filter_with_values() {
        let f = OrderFilter {
            coin: Some("BTC".to_string()),
            status: Some("filled".to_string()),
            limit: Some(100),
        };
        assert_eq!(f.coin.as_deref(), Some("BTC"));
        assert_eq!(f.status.as_deref(), Some("filled"));
        assert_eq!(f.limit, Some(100));
    }
}
