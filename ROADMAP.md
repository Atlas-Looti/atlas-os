# Atlas Perp — ROADMAP

> **Atlas Perp bukan sekedar CLI. Ini trading engine OS untuk Hyperliquid.**
>
> hypecli = thin CLI wrapper. Atlas Perp = platform.
> Risk management, USDC sizing, multi-mode trading, scripting engine,
> backtesting, data caching, DeFi integrations — semua dalam satu binary.

## Tech Stack
- **Language:** Rust (2024 edition)
- **SDK:** `hypersdk` (infinitefield) — `rust_decimal::Decimal`, `HttpClient`, `NonceHandler`, `PriceTick`
- **SDK Reference:** `workspace/hypersdk/` — source of truth
- **TUI:** ratatui + crossterm
- **Auth:** OS keyring (`keyring` crate) — private keys NEVER on disk
- **Revenue:** Builder fee injection on every order (mandatory, invisible to user)
- **Data:** SQLite (`rusqlite`) for local cache — `~/.atlas-perp/data/atlas.db`
- **Scripting:** YAML-based strategy files — `atlas run signal-btc.yaml`

## Architecture
```
crates/
├── types/    → Config, SizeInput, SizeMode, RiskConfig, WalletProfile, constants
├── utils/    → fmt, parse (parse_size), prompt, risk calculator
├── core/     → Engine (HttpClient + NonceHandler), AuthManager, workspace, Atlas-QL
└── cli/      → Commands, ratatui TUI (4 tabs), output formatters (table/json)
```

---

## Phase 0–3: Foundation ✅ DONE

### Phase 0: Scaffolding ✅
- Cargo workspace, dotfolder init, auth (keyring), engine wrapper, CLI routing, TUI skeleton

### Phase 1: Order Execution ✅
- Limit/market orders, close, cancel (OID/all) — all with builder fee

### Phase 2: Account Management ✅
- Leverage, isolated margin, USDC transfer

### Phase 3: Order Tracking ✅
- Open orders, fills/trade history

### Phase 3.5: Trading Modes & Risk ✅
- CFD/Futures modes, lot sizing, `atlas configure`
- Risk management (auto position sizing, SL/TP, exposure validation)
- USDC sizing (`$200`, `200u`, bare numbers) — `default_size_mode = usdc`
- Utils crate, 105 unit tests

---

## Phase 4: SDK Migration (hypersdk) 🔄

Swap `hyperliquid_rust_sdk` → `hypersdk`:
- Single `HttpClient` (was InfoClient + ExchangeClient)
- `rust_decimal::Decimal` everywhere (was f64)
- `NonceHandler` for thread-safe nonce generation
- `PriceTick` for auto tick/lot rounding on orders
- Asset index lookup (was string name)
- Builder fee via `client.place()` builder parameter
- **SDK ref:** `workspace/hypersdk/examples/hypercore/send_order.rs`

---

## Phase 5: Output Formatting 📋

Buat semua commands support `--json` dan `--table` output (seperti hypecli):

```bash
atlas status --json          # Raw JSON for piping/scripting
atlas orders --table         # Pretty table (default)
atlas fills --json | jq '.[] | select(.coin == "ETH")'
```

- Default: table (human-readable)
- `--json`: machine-readable JSON output
- Enables piping ke `jq`, scripts, dan integrasi lain

---

## Phase 6: Market Data 📋

Comprehensive market data commands:

```bash
# Prices
atlas price ETH                    # Current mid price
atlas price ETH BTC SOL            # Multiple assets
atlas price --all                  # All mids

# Markets
atlas markets                      # List all perp markets (name, leverage, index)
atlas markets --spot               # List spot markets
atlas markets --dex                # List available DEXes (HIP-3)

# Candles / K-Line
atlas candles ETH 15m              # 15-minute candles
atlas candles BTC 1h --limit 100   # Last 100 hourly candles
atlas candles ETH 1d --json        # Daily candles as JSON

# Funding
atlas funding ETH                  # Current funding rate
atlas funding --history ETH        # Funding rate history

# Order Book
atlas book ETH                     # L2 order book snapshot
atlas book ETH --depth 10          # Top 10 levels
```

---

## Phase 7: WebSocket Streaming 📋

Real-time data via `hypercore::mainnet_ws()`:

```bash
atlas stream trades BTC            # Live trade feed
atlas stream book ETH              # Live order book
atlas stream candles BTC 15m       # Live candles
atlas stream user                  # My fills, order updates, liquidations
atlas stream prices                # All mid prices, real-time
```

- `futures::Stream` based (native async)
- Auto-reconnect with subscription management
- Feed data into TUI for live dashboard
- Feed data into Atlas-QL for caching

---

## Phase 8: TUI Live Dashboard 📋

Upgrade TUI with real-time WebSocket data:

- **Dashboard tab:** Account value, PnL, margin, funding — live
- **Positions tab:** Live PnL with mark price updates
- **Orders tab:** Open orders with cancel (keybind)
- **Markets tab:** Live prices, 24h change, volume
- **Trade panel:** Place orders from within TUI
- **Order book widget:** Depth visualization
- **Chart:** ASCII candlestick chart (basic)

---

## Phase 9: Spot Trading 📋

Full spot market support via hypersdk:

```bash
atlas spot markets                 # List spot markets (PURR/USDC, etc.)
atlas spot tokens                  # List all tokens
atlas spot buy PURR 100            # Buy 100 PURR
atlas spot sell PURR 50 --price 0.05  # Limit sell
atlas spot balance                 # Spot balances
atlas spot transfer ETH 0.1 --to-perps   # Move to perps
atlas spot transfer ETH 0.1 --to-evm     # Move to EVM
```

- SDK: `client.spot()`, `client.spot_tokens()`, `client.user_balances()`
- Transfers: `transfer_to_spot()`, `transfer_to_perps()`, `transfer_to_evm()`

---

## Phase 10: Vault & Subaccounts 📋

```bash
# Vaults
atlas vault list                   # My vault equities
atlas vault details <vault_addr>   # Vault details (PnL, positions, followers)
atlas vault deposit <addr> 1000    # Deposit to vault

# Subaccounts
atlas sub list                     # List subaccounts
atlas sub create "sniper"          # Create subaccount
atlas sub switch "sniper"          # Switch active

# Agents
atlas agent approve <addr> "bot"   # Approve agent wallet
atlas agent list                   # List approved agents
```

- SDK: `vault_details()`, `user_vault_equities()`, `subaccounts()`, `approve_agent()`

---

## Phase 11: Atlas-QL (Data Caching) 📋

SQLite local database di `~/.atlas-perp/data/atlas.db`:

```bash
atlas history trades               # All trade history (from DB)
atlas history trades --coin ETH --from 2026-01-01
atlas history pnl                  # Daily PnL summary
atlas history pnl --weekly         # Weekly PnL
atlas history candles ETH 1h       # Cached candles
atlas history sync                 # Sync latest from Hyperliquid
```

### What Gets Cached
- **Trades:** All fills, with PnL and fees
- **Orders:** Historical orders, status changes
- **Candles:** K-line data per asset/interval
- **Funding:** Funding rate history
- **Snapshots:** Periodic account state snapshots
- **PnL:** Computed daily/weekly/monthly PnL

### Why
- Prevents rate-limiting from Hyperliquid API
- Enables complex queries without hammering the server
- Persists data across sessions
- Powers backtesting engine
- Agent/script queries use local DB first

---

## Phase 12: HyperEVM DeFi 📋

Integration with HyperEVM protocols via hypersdk:

```bash
# Morpho (Lending)
atlas defi morpho apy              # Top lending APYs
atlas defi morpho supply ETH       # Supply APY for ETH
atlas defi morpho borrow USDC      # Borrow APY
atlas defi morpho vaults           # Vault performance

# Uniswap V3
atlas defi uniswap pools           # Active pools
atlas defi uniswap price PURR      # Pool price

# EVM Transfers
atlas evm transfer ETH 0.1 --to-evm    # HyperCore → HyperEVM
atlas evm transfer ETH 0.1 --from-evm  # HyperEVM → HyperCore
```

- SDK: `hyperevm::morpho`, `hyperevm::uniswap`
- EVM bridging: `transfer_to_evm()`, `transfer_from_evm()`

---

## Phase 13: Script Engine (AtlasScript) 📋

**Ini yang bikin Atlas Perp beda dari semua CLI lain.**

YAML-based strategy execution:

```bash
atlas run signal-btc.yaml          # Execute a strategy file
atlas run dca-eth.yaml --dry-run   # Dry run (simulate only)
atlas run grid-sol.yaml --once     # Run once, don't loop
atlas scripts list                 # List saved scripts
atlas scripts validate my-bot.yaml # Validate syntax
```

### Script Format (YAML)
```yaml
# ~/.atlas-perp/agents/signal-btc.yaml
name: "BTC Signal Bot"
version: 1
trigger:
  type: interval
  every: 5m              # atau: on_price, on_fill, on_funding, cron

conditions:
  - price_above: { coin: BTC, value: 100000 }
  - funding_negative: { coin: BTC }

actions:
  - buy:
      coin: BTC
      size: $200
      leverage: 10
      stop_loss: -2%
      take_profit: 4%

  - notify: "BTC signal triggered at {{price}}"

risk:
  max_daily_loss: $500
  max_trades_per_day: 10
  cooldown: 30m
```

### Script Types

**1. Signal Bot** — Execute on conditions
```yaml
trigger: { type: on_price, coin: ETH, crosses_above: 4000 }
actions: [{ buy: { coin: ETH, size: $100 } }]
```

**2. DCA Bot** — Dollar cost averaging
```yaml
trigger: { type: cron, schedule: "0 9 * * *" }  # Daily 9am
actions: [{ buy: { coin: BTC, size: $50 } }]
```

**3. Grid Bot** — Grid trading
```yaml
trigger: { type: continuous }
strategy: grid
grid:
  coin: ETH
  lower: 3000
  upper: 4000
  levels: 10
  size_per_level: $100
```

**4. Data Script** — Query and analyze
```yaml
trigger: { type: manual }
actions:
  - query: { type: funding_rates, coins: [BTC, ETH, SOL] }
  - filter: { funding_below: -0.01 }
  - notify: "Negative funding: {{results}}"
```

**5. Arbitrage / Custom**
```yaml
trigger: { type: on_price }
conditions:
  - spread_above: { pair: [BTC, BTC-PERP], threshold: 0.5% }
actions:
  - buy: { coin: BTC, size: $500 }
  - sell: { coin: BTC-PERP, size: $500 }
```

### Script Runtime
- YAML parsed → validated → executed by Atlas runtime
- Access to all Engine methods (orders, queries, risk)
- Atlas-QL for data queries (local DB first, API fallback)
- Risk limits enforced per-script
- Logging to `~/.atlas-perp/logs/scripts/`
- `--dry-run` mode for testing
- Template variables: `{{price}}`, `{{account_value}}`, `{{pnl}}`

---

## Phase 14: Backtesting Engine 📋

Test strategies against historical data:

```bash
atlas backtest signal-btc.yaml --from 2025-01-01 --to 2026-01-01
atlas backtest dca-eth.yaml --period 6m
atlas backtest grid-sol.yaml --capital 10000 --leverage 5
```

### Output
```
╔══════════════════════════════════════════════════╗
║  BACKTEST RESULTS — signal-btc.yaml             ║
╠══════════════════════════════════════════════════╣
║  Period     : 2025-01-01 → 2026-01-01           ║
║  Trades     : 142                                ║
║  Win Rate   : 63.4%                              ║
║  Total PnL  : +$4,231.50                         ║
║  Max DD     : -$812.00 (8.1%)                    ║
║  Sharpe     : 1.82                               ║
║  R:R Avg    : 2.14                               ║
╚══════════════════════════════════════════════════╝
```

- Uses Atlas-QL cached candle/trade data
- Downloads missing historical data automatically
- Simulates order execution with slippage model
- Risk rules applied same as live trading
- Export results to JSON/CSV

---

## Phase 15: Multi-Sig & Advanced 📋

```bash
# Multi-signature
atlas multisig config              # Show multi-sig config
atlas multisig convert             # Convert to multi-sig wallet
atlas multisig order ...           # Place order with multi-sig

# Advanced
atlas doctor --full                # NTP sync, API latency, DB integrity
atlas export trades --csv          # Export trade history
atlas export pnl --monthly --csv  # Monthly PnL export
```

---

## Command Map (Complete Vision)

### Trading (Core)
| Command | Description |
|---|---|
| `atlas buy` | Market buy (USDC/units/lots) |
| `atlas sell` | Market sell/short |
| `atlas order` | Limit order |
| `atlas close` | Close position |
| `atlas cancel` | Cancel orders |
| `atlas orders` | List open orders |
| `atlas fills` | Trade history |

### Account
| Command | Description |
|---|---|
| `atlas status` | Account summary |
| `atlas leverage` | Set leverage |
| `atlas margin` | Update margin |
| `atlas transfer` | USDC transfer |
| `atlas auth` | Wallet management |

### Configuration
| Command | Description |
|---|---|
| `atlas configure` | Interactive config |
| `atlas configure size` | Default size mode (usdc/units/lots) |
| `atlas configure mode` | Trading mode (futures/cfd) |
| `atlas risk` | Risk calculator |

### Market Data
| Command | Description |
|---|---|
| `atlas price` | Current prices |
| `atlas markets` | List markets (perp/spot/dex) |
| `atlas candles` | K-line data |
| `atlas funding` | Funding rates |
| `atlas book` | Order book |

### Streaming
| Command | Description |
|---|---|
| `atlas stream trades` | Live trade feed |
| `atlas stream book` | Live order book |
| `atlas stream candles` | Live candles |
| `atlas stream user` | My events |

### Spot
| Command | Description |
|---|---|
| `atlas spot buy/sell` | Spot trading |
| `atlas spot balance` | Spot balances |
| `atlas spot transfer` | Move between perps/spot/evm |

### Vault & Agents
| Command | Description |
|---|---|
| `atlas vault` | Vault operations |
| `atlas sub` | Subaccounts |
| `atlas agent` | Agent approval |

### Data (Atlas-QL)
| Command | Description |
|---|---|
| `atlas history` | Cached trade/candle history |
| `atlas history pnl` | PnL reports |
| `atlas history sync` | Sync from API |
| `atlas export` | Export to CSV/JSON |

### DeFi (HyperEVM)
| Command | Description |
|---|---|
| `atlas defi morpho` | Lending APY/vaults |
| `atlas defi uniswap` | Pool prices/info |
| `atlas evm transfer` | EVM bridging |

### Scripting
| Command | Description |
|---|---|
| `atlas run` | Execute strategy YAML |
| `atlas scripts` | Manage scripts |
| `atlas backtest` | Test strategy on history |

### System
| Command | Description |
|---|---|
| `atlas doctor` | Health check |
| `atlas tui` | Interactive dashboard |
| `atlas multisig` | Multi-sig operations |

---

## Stats (2026-02-24)
- **4,841 lines** | **27 files** | **105 tests** | **zero warnings**
- 17 CLI commands (growing to 50+)
- 4 crates: types, utils, core, cli
- Builder fee on every order path
- Private keys never on disk
