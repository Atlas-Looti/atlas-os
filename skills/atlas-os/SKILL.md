---
name: atlas-os
description: "Operate Atlas OS — a DeFi Operating System CLI for Hyperliquid perp/spot trading, 0x multi-chain swaps, real-time streaming, and market data. Use when: user asks about crypto trading, checking prices/funding/orderbook, managing positions, placing orders, streaming market data, technical analysis (RSI/MACD/VWAP/Bollinger), configuring Atlas OS, DEX token/pool info, risk calculation, portfolio status, or troubleshooting the atlas CLI. Triggers on: atlas commands, Hyperliquid, 0x swaps, DeFi operations, crypto prices, perp trading, spot trading, funding rates, orderbook, limit orders, market orders, leverage, margin, vault, subaccount, NDJSON streaming, TUI."
---

# Atlas OS

DeFi Operating System — agent-first, non-custodial, multi-protocol CLI. Single binary, JSON output, NDJSON streaming.

Binary: `atlas` · Config: `~/.atlas-os/atlas.json` · Keys: OS keyring only

## Agent Workflow

Atlas OS is designed for AI agents. Every session should follow this pattern:

```
1. atlas doctor --output json       → verify all systems green
2. atlas status --output json       → get account context (balances, positions, open orders)
3. atlas market hyperliquid price <SYMBOL> --output json  → read market
4. <execute action>                 → trade, stream, analyze
5. atlas status --output json       → confirm result
```

### JSON Contract

All commands support `--output json`. Response envelope:

```json
{"ok": true, "data": {...}}
{"ok": false, "error": {"code": "...", "category": "...", "recoverable": true, "hints": [...]}}
```

Exit codes: `0` success · `1` user error · `2` network · `3` system

Streaming uses NDJSON — one JSON object per line, no array wrapper.

## Onboarding (First Run)

```bash
# Generate wallet + configure
atlas profile generate main
atlas configure system api-key <key>           # from apps/frontend dashboard
atlas configure module set hyperliquid network mainnet
atlas doctor --output json                      # confirm all green
atlas status --output json                      # confirm balance visible
```

For testnet: `atlas configure module set hyperliquid network testnet`

## Full Command Reference

### System & Profile

| Command | Purpose |
|---|---|
| `atlas status [--output json]` | Account summary: balances, positions, open orders, account value |
| `atlas doctor [--output json]` | Health check with actionable `fix` hints per failing check |
| `atlas doctor --fix` | Auto-fix detected issues |
| `atlas tui` | Launch interactive Terminal UI (4 tabs: market, positions, orders, trade) |
| `atlas profile generate <name>` | Create new wallet (key stored in OS keyring) |
| `atlas profile import <name> --key <hex>` | Import existing private key |
| `atlas profile use <name>` | Switch active profile |
| `atlas profile list` | List all profiles with addresses |
| `atlas profile export <name>` | Export key (interactive confirmation) |

### Configuration

```bash
atlas configure show                                    # Full config dump
atlas configure system api-key <key>                    # Backend API key
atlas configure system profile <name>                   # Switch profile
atlas configure system verbose <true|false>              # Toggle verbose

atlas configure module list                             # List modules + status
atlas configure module enable <hl|zero_x>               # Enable module
atlas configure module disable <hl|zero_x>              # Disable module

# Hyperliquid settings
atlas configure module set hyperliquid network <mainnet|testnet>
atlas configure module set hyperliquid mode <futures|cfd>
atlas configure module set hyperliquid default-size-mode <usdc|units|lots>
atlas configure module set hyperliquid leverage <N>
atlas configure module set hyperliquid slippage <PCT>       # e.g. 0.05 = 5%
atlas configure module set hyperliquid lot <SYMBOL> <SIZE>  # e.g. lot BTC 0.001

# 0x settings
atlas configure module set zero_x default-chain <ethereum|arbitrum|base|...>
atlas configure module set zero_x default-slippage-bps <N>  # e.g. 100 = 1%
```

### Market Data — Hyperliquid

```bash
atlas market hyperliquid price <SYMBOL...>              # Mid prices (multi-symbol)
atlas market hyperliquid price --all                     # All listed assets
atlas market hyperliquid info <SYMBOL>                  # Price, spread, OI, volume
atlas market hyperliquid funding <SYMBOL>                # Funding rate history
atlas market hyperliquid orderbook <SYMBOL> [--depth 20] # Order book snapshot
atlas market hyperliquid candles <SYMBOL> [--timeframe 4h] [--limit 100]
atlas market hyperliquid list [--spot]                   # All listed assets
atlas market hyperliquid top [--sort gainers] [--limit 10]
atlas market hyperliquid spread <SYMBOL...>              # Bid-ask spreads
atlas market hyperliquid search <query>                  # Search by name
atlas market hyperliquid summary                         # Market overview
```

### Technical Analysis

```bash
atlas market hyperliquid rsi <SYMBOL> [--timeframe 1h] [--period 14]
atlas market hyperliquid macd <SYMBOL> [--timeframe 15m]
atlas market hyperliquid vwap <SYMBOL>
atlas market hyperliquid trend <SYMBOL>        # Multi-indicator composite signal
atlas market hyperliquid bbands <SYMBOL>        # Bollinger Bands
atlas market hyperliquid stoch <SYMBOL>         # Stochastic oscillator
atlas market hyperliquid adx <SYMBOL>           # ADX trend strength
atlas market hyperliquid atr <SYMBOL>           # Average True Range
atlas market hyperliquid ema <SYMBOL>           # Exponential MA
atlas market hyperliquid sma <SYMBOL>           # Simple MA
atlas market hyperliquid obv <SYMBOL>           # On-Balance Volume
atlas market hyperliquid cci <SYMBOL>           # Commodity Channel Index
atlas market hyperliquid willr <SYMBOL>         # Williams %R
atlas market hyperliquid sar <SYMBOL>           # Parabolic SAR
atlas market hyperliquid patterns <SYMBOL>      # Candlestick patterns
```

### Market Data — DEX / CoinGecko

```bash
atlas market trending                           # Trending coins
atlas market movers [--limit 20]                # Top gainers & losers
atlas market global                             # Global crypto market stats
atlas market defi                               # DeFi TVL & stats
atlas market coin <id>                          # Detailed coin info (e.g. bitcoin)

atlas market dex trending [--network base]      # Trending DEX pools
atlas market dex new                            # Newly listed pools
atlas market dex pools <network>                # Pools on network
atlas market dex pool <network> <address>       # Specific pool details
atlas market dex token <network> <address>      # Token info
atlas market dex networks                       # Supported networks
atlas market dex dexes <network>                # DEXes on network
atlas market dex search <query>                 # Search tokens/pools
```

### Streaming (NDJSON)

```bash
atlas stream prices <SYMBOL...>                 # Real-time price ticks
atlas stream trades <SYMBOL>                    # Trade-by-trade feed
atlas stream book <SYMBOL> [--depth 20]         # Order book updates
atlas stream candles <SYMBOL> <interval>        # 1m, 5m, 15m, 1h, 4h, 1d
atlas stream user                               # Personal fills + order events
```

Agent consumption: `atlas stream user --output json | while read line; do process "$line"; done`

### Hyperliquid Perp Trading

Alias: `atlas hl perp ...`

```bash
# Market orders
atlas hl perp buy <SYMBOL> <SIZE>               # Market long
atlas hl perp buy ETH 200                       # $200 USDC margin
atlas hl perp buy ETH $500 --leverage 10        # With leverage
atlas hl perp buy ETH 0.5eth                    # 0.5 ETH explicitly
atlas hl perp buy ETH 10lots                    # 10 × configured lot size
atlas hl perp sell <SYMBOL> <SIZE>               # Market short

# Position management
atlas hl perp close <SYMBOL>                    # Close entire position
atlas hl perp close <SYMBOL> --size 0.1          # Partial close
atlas hl perp close <SYMBOL> --slippage 0.01     # Custom slippage

# Limit orders
atlas hl perp order <SYMBOL> <SIDE> <SIZE> <PRICE>
atlas hl perp order ETH buy 200 3200             # Limit buy $200 at 3200
atlas hl perp order ETH sell $500 4000 --reduce-only

# Cancel
atlas hl perp cancel <SYMBOL>                   # Cancel all orders for symbol
atlas hl perp cancel <SYMBOL> --oid 12345        # Cancel specific order

# Query
atlas hl perp positions [--output json]          # Open positions
atlas hl perp orders [--output json]             # Open orders
atlas hl perp fills [--output json]              # Recent fills

# Position settings
atlas hl perp leverage <SYMBOL> <N>              # Set leverage
atlas hl perp leverage ETH 10 --cross            # Cross margin
atlas hl perp margin <SYMBOL> add|remove <AMT>   # Adjust isolated margin

# Transfer
atlas hl perp transfer deposit|withdraw <AMT>    # USDC to/from HL
atlas hl perp transfer <AMT> <ADDRESS>           # Send to address
```

### Size Input Modes

| Mode | Input | Meaning |
|---|---|---|
| USDC (default) | `200` or `$200` | $200 margin → size = (margin × leverage) / price |
| Units | `0.5eth` | 0.5 units of the asset |
| Lots | `10lots` | 10 × lot size from config table |

Explicit suffix always overrides `default_size_mode`.

### Hyperliquid Spot

```bash
atlas hl spot buy <TOKEN> <AMT>                 # Spot market buy
atlas hl spot sell <TOKEN> <AMT>                # Spot market sell
atlas hl spot balance                            # Spot token balances

atlas hl spot transfer <TOKEN> <AMT> spot-to-perp|perp-to-spot|to-evm
```

### Hyperliquid Vault / Sub / Agent / Risk

```bash
atlas hl vault details <ADDRESS>                # Vault info
atlas hl vault deposits [<ADDRESS>]             # Vault deposit history

atlas hl sub list                                # List subaccounts
atlas hl agent approve <ADDRESS> [--name "bot"] # Approve agent wallet

atlas hl sync [--full]                           # Sync trade history to local DB

atlas hl risk calc <COIN> <SIDE> <ENTRY> --stop <PRICE> [--leverage <N>]
atlas hl risk offline <COIN> <SIDE> <ENTRY> <ACCOUNT_SIZE> --stop <PRICE>
# Example: atlas hl risk calc ETH long 3200 --stop 3100 --leverage 5
```

### 0x Swaps (Multi-chain DEX Aggregator)

Alias: `atlas 0x ...` or `atlas swap ...`

```bash
atlas 0x quote <SELL_TOKEN> <BUY_TOKEN> <AMOUNT> [--chain ethereum] [--slippage <bps>]
atlas 0x swap <SELL_TOKEN> <BUY_TOKEN> <AMOUNT> [--chain ethereum] [--yes]
atlas 0x chains                                  # Supported chains
atlas 0x sources [--chain base]                  # Available DEX sources
```

Tokens are ERC20 contract addresses. Swap flow: price preview → confirm → firm quote → auto-approve (exact amount) → sign → broadcast → wait receipt.

### History & Export

```bash
atlas history trades [--protocol hl] [--coin ETH] [--limit 100] [--from 2025-01-01]
atlas history orders [--coin BTC] [--status filled]
atlas history pnl [--protocol hl] [--coin ETH]

atlas export trades --csv [--coin ETH]
atlas export trades --json
atlas export pnl --csv [--from 2025-01-01]
```

## Config Schema

Full schema at `~/.atlas-os/atlas.json`:

```json
{
  "system": {
    "active_profile": "main",
    "api_key": "atl_...",
    "verbose": false
  },
  "modules": {
    "hyperliquid": {
      "enabled": true, "network": "mainnet", "mode": "futures",
      "default_size_mode": "usdc", "default_leverage": 1, "default_slippage": 0.05,
      "lots": { "default_lot_size": 1.0, "assets": { "BTC": 0.001, "ETH": 0.01 } },
      "risk": { "max_risk_pct": 0.02, "max_positions": 10 }
    },
    "zero_x": {
      "enabled": false, "default_slippage_bps": 100, "default_chain": "ethereum"
    }
  }
}
```

No global trading config — all settings live per-module.

## Common Agent Workflows

For detailed JSON output schemas and NDJSON event formats, see `{baseDir}/references/json-schemas.md`.
For common trading workflows and decision trees, see `{baseDir}/references/workflows.md`.

## Troubleshooting

| Symptom | Diagnosis | Fix |
|---|---|---|
| `KEYRING_ERROR` | No key in OS keyring | `atlas profile generate <name>` or reimport |
| `NO_PROFILE` | No active profile | `atlas profile use <name>` |
| `API_KEY_MISSING` | Backend API key not set | `atlas configure system api-key <key>` |
| `BACKEND_UNREACHABLE` | Backend proxy down | `atlas doctor --output json` → check `backend` |
| `MODULE_DISABLED` | Module not enabled | `atlas configure module enable <hl\|zero_x>` |
| `INSUFFICIENT_MARGIN` | Not enough balance | Deposit more or reduce size |
| `SLIPPAGE_EXCEEDED` | Price moved too far | Increase `--slippage` or retry |
| `RATE_LIMITED` | Too many requests | Wait and retry |
| Wrong network | Trading on testnet/mainnet | `atlas configure module set hl network <net>` |

## Safety

- Private keys: OS keyring ONLY (service `atlas_os`) — never disk, log, env, or network
- Builder fee: 1 bps mandatory on every Hyperliquid perp order
- All prices/sizes: `rust_decimal::Decimal` — no floating point
- External API keys (0x, Alchemy): always via backend proxy, never in CLI
- Confirmation prompt before trades (skip with `--yes`)
- Exact-amount token approval for swaps (not unlimited)
