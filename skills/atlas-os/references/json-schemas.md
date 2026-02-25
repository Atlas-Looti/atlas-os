# Atlas OS — JSON Output Schemas

## Envelope

Every `--output json` response:
```json
{"ok": true, "data": {...}}
{"ok": false, "error": {"code": "...", "category": "...", "message": "...", "recoverable": true, "hints": [...]}}
```

---

## Status
```json
{"ok": true, "data": {
  "profile": "main", "address": "0xc0a1...", "network": "Mainnet",
  "modules": ["hyperliquid"],
  "balances": [{"asset": "USDC", "total": "5000.00", "available": "4800.00", "protocol": "hyperliquid"}],
  "positions": [
    {"symbol": "ETH", "side": "long", "size": "0.5", "entry_price": "3200.00",
     "mark_price": "3350.00", "unrealized_pnl": "75.00", "leverage": 5,
     "liquidation_price": "2800.00", "margin_mode": "cross", "protocol": "hyperliquid"}
  ],
  "account_value": "5075.00", "margin_used": "320.00",
  "net_position": "1675.00", "withdrawable": "4755.00", "open_orders": 2
}}
```

## Doctor
```json
{"ok": true, "data": {"checks": [
  {"name": "profile", "status": "ok", "value": "main"},
  {"name": "keyring", "status": "ok"},
  {"name": "api_key", "status": "ok"},
  {"name": "backend", "status": "ok", "value": "295ms", "latency_ms": 295},
  {"name": "hyperliquid", "status": "ok", "value": "mainnet", "network": "mainnet"}
]}}
```

Failed check includes `fix`:
```json
{"name": "api_key", "status": "fail", "fix": "Run: atlas configure system api-key <key>"}
```

## Prices
```json
{"ok": true, "data": {"prices": [
  {"symbol": "BTC", "price": "65400.5", "protocol": "hyperliquid"},
  {"symbol": "ETH", "price": "3500.20", "protocol": "hyperliquid"}
]}}
```

## Positions
```json
{"ok": true, "data": {"positions": [
  {"symbol": "ETH", "side": "long", "size": "0.5", "entry_price": "3200.00",
   "mark_price": "3350.00", "unrealized_pnl": "75.00", "leverage": 5,
   "liquidation_price": "2800.00", "margin_mode": "cross", "protocol": "hyperliquid"}
]}}
```

## Order Result (buy/sell/close/order)
```json
{"ok": true, "data": {
  "order_id": "12345678", "symbol": "ETH", "side": "buy", "size": "0.0571",
  "price": "3500.20", "filled": "0.0571", "status": "filled",
  "fee": "0.12", "builder_fee_bps": 1, "protocol": "hyperliquid",
  "timestamp": 1708828200
}}
```

## Orders (open)
```json
{"ok": true, "data": {"orders": [
  {"order_id": "12345", "symbol": "ETH", "side": "buy", "size": "0.5",
   "price": "3200.00", "filled": "0.0", "status": "open", "protocol": "hyperliquid"}
]}}
```

## Fills
```json
{"ok": true, "data": {"fills": [
  {"order_id": "12345", "symbol": "ETH", "side": "buy", "size": "0.05",
   "price": "3500.00", "fee": "0.02", "timestamp": 1708828205}
]}}
```

## Funding Rates
```json
{"ok": true, "data": {"coin": "BTC", "rates": [
  {"coin": "BTC", "rate": "0.0000125", "premium": "-0.0003", "time": "2026-02-25 01:00:00"}
]}}
```

## Orderbook
```json
{"ok": true, "data": {
  "symbol": "ETH", "bids": [["3499.50", "12.5"], ["3499.00", "8.2"]],
  "asks": [["3500.00", "5.1"], ["3500.50", "10.3"]]
}}
```

## Candles
```json
{"ok": true, "data": {"candles": [
  {"time": "2026-02-25T00:00:00Z", "open": "3480", "high": "3520",
   "low": "3470", "close": "3500", "volume": "1234.5"}
]}}
```

## 0x Quote
```json
{"ok": true, "data": {
  "sell_token": "0xA0b8...", "buy_token": "0xC02a...",
  "sell_amount": "1000000000", "buy_amount": "285714285714",
  "price": "0.000285", "gas_estimate": "150000",
  "allowance_required": true, "allowance_spender": "0x0000..."
}}
```

## Profile List
```json
{"ok": true, "data": {"profiles": [
  {"name": "main", "address": "0xc0a1...", "active": true},
  {"name": "testnet", "address": "0xe4cb...", "active": false}
]}}
```

## Configure Show
```json
{"ok": true, "data": {
  "system": {"active_profile": "main", "api_key": "atl_...", "verbose": false},
  "modules": {
    "hyperliquid": {"enabled": true, "network": "mainnet", "mode": "futures", "...": "..."},
    "zero_x": {"enabled": false, "default_chain": "ethereum", "...": "..."}
  }
}}
```

---

## Error Codes

| Code | Category | Recoverable | Typical Hint |
|---|---|---|---|
| `NO_PROFILE` | config | yes | `atlas profile generate <name>` |
| `KEYRING_ERROR` | auth | no | Check OS keyring service |
| `API_KEY_MISSING` | config | yes | `atlas configure system api-key <key>` |
| `MODULE_DISABLED` | config | yes | `atlas configure module enable <module>` |
| `INVALID_SYMBOL` | validation | yes | Check symbol with `atlas market hyperliquid list` |
| `INVALID_SIZE` | validation | yes | Size must be positive number |
| `UNSUPPORTED_CHAIN` | validation | yes | Check `atlas 0x chains` |
| `INSUFFICIENT_MARGIN` | execution | yes | Reduce size or deposit more |
| `SLIPPAGE_EXCEEDED` | execution | yes | Increase `--slippage` |
| `ORDER_REJECTED` | execution | yes | Check HL order requirements |
| `POSITION_NOT_FOUND` | execution | yes | No open position for symbol |
| `RATE_LIMITED` | network | yes | Wait and retry |
| `BACKEND_UNREACHABLE` | network | yes | Check internet / backend status |
| `PROTOCOL_TIMEOUT` | network | yes | Retry |
| `NETWORK_MISMATCH` | config | yes | Switch network |
| `INTERNAL_ERROR` | system | no | Report bug |

---

## NDJSON Stream Events

### stream prices
```json
{"event": "price", "symbol": "BTC", "price": "65400.5", "protocol": "hyperliquid"}
```

### stream trades
```json
{"event": "trade", "symbol": "ETH", "price": "3502.50", "size": "0.12", "side": "buy", "timestamp": 1708828200}
```

### stream book
```json
{"event": "book", "symbol": "ETH", "bids": [["3499.50", "12.5"]], "asks": [["3500.00", "5.1"]], "timestamp": 1708828200}
```

### stream candles
```json
{"event": "candle", "symbol": "ETH", "interval": "1h", "open": "3480", "high": "3520", "low": "3470", "close": "3500", "volume": "1234.5", "timestamp": 1708828200}
```

### stream user — fill
```json
{"event": "fill", "order_id": 12345, "symbol": "ETH", "side": "buy", "size": "0.05", "price": "3500.00", "timestamp": 1708828205}
```

### stream user — order update
```json
{"event": "order_cancelled", "order_id": 12346, "symbol": "BTC", "reason": "user_request", "timestamp": 1708828210}
```
