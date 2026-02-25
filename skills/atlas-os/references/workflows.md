# Atlas OS — Agent Trading Workflows

## Workflow 1: Check Portfolio Status

```bash
atlas doctor --output json       # Ensure everything is healthy
atlas status --output json       # Get full account snapshot
```

Decision tree:
- If doctor shows `fail` → read `fix` field and execute it
- If status shows positions → evaluate PnL, check for stop-loss needs
- If status shows no balance → need deposit first

## Workflow 2: Open a Perp Position (Hyperliquid)

```bash
# 1. Read market
atlas market hyperliquid price ETH --output json
atlas market hyperliquid funding ETH --output json    # Check funding direction
atlas market hyperliquid trend ETH --output json      # Multi-indicator signal

# 2. Check account
atlas status --output json                             # Verify available balance

# 3. Risk calculation
atlas hl risk calc ETH long 3200 --stop 3100 --leverage 5 --output json

# 4. Execute
atlas hl perp leverage ETH 5                           # Set leverage first
atlas hl perp buy ETH 200 --output json                # $200 USDC margin

# 5. Verify
atlas hl perp positions --output json                  # Confirm position opened
```

## Workflow 3: Close a Position

```bash
# 1. Check current position
atlas hl perp positions --output json

# 2. Close (full or partial)
atlas hl perp close ETH --output json                  # Full close
atlas hl perp close ETH --size 0.1 --output json       # Partial close

# 3. Verify
atlas hl perp positions --output json                  # Confirm closed
```

## Workflow 4: Place and Manage Limit Orders

```bash
# 1. Check orderbook for good price levels
atlas market hyperliquid orderbook ETH --depth 10 --output json

# 2. Place limit order
atlas hl perp order ETH buy 200 3200 --output json     # Limit buy at 3200

# 3. Monitor
atlas hl perp orders --output json                     # Check order status

# 4. Cancel if needed
atlas hl perp cancel ETH --oid 12345 --output json
```

## Workflow 5: Monitor with Streaming

```bash
# Real-time price monitoring
atlas stream prices BTC ETH SOL --output json
# Each line: {"event":"price","symbol":"BTC","price":"65400.5","protocol":"hyperliquid"}

# Watch for fills on your orders
atlas stream user --output json
# Each line: {"event":"fill","order_id":12345,"symbol":"ETH",...}

# Watch trade flow for signals
atlas stream trades ETH --output json
```

## Workflow 6: Technical Analysis Scan

```bash
# Quick multi-indicator check
atlas market hyperliquid trend ETH --output json

# Detailed breakdown
atlas market hyperliquid rsi ETH --output json
atlas market hyperliquid macd ETH --output json
atlas market hyperliquid bbands ETH --output json
atlas market hyperliquid vwap ETH --output json

# Decision: if RSI < 30 + MACD bullish crossover + price at lower BB → consider long
```

## Workflow 7: Cross-chain Swap (0x)

```bash
# 1. Get a price quote
atlas 0x quote 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
               0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 \
               1000 --chain ethereum --output json
# (USDC → WETH, 1000 units)

# 2. Execute the swap
atlas 0x swap 0xA0b86991... 0xC02aaA39... 1000 --chain ethereum --yes --output json

# 3. Wait for confirmation — output includes tx hash and receipt
```

## Workflow 8: Testnet Trading

```bash
# 1. Switch to testnet
atlas configure module set hyperliquid network testnet

# 2. Generate or use testnet profile
atlas profile generate testnet
atlas profile use testnet

# 3. Deposit testnet USDC via https://app.hyperliquid-testnet.xyz

# 4. Trade normally
atlas hl perp buy ETH 100 --output json

# 5. Switch back when done
atlas configure module set hyperliquid network mainnet
atlas profile use main
```

## Workflow 9: Risk Management Check

```bash
# Before entering any trade:
atlas hl risk calc ETH long 3200 --stop 3100 --leverage 5 --output json

# Offline calculation (without live data):
atlas hl risk offline BTC long 100000 50000 --stop 98000 --output json

# Check existing exposure
atlas status --output json
# Look at: margin_used, account_value, positions count vs max_positions config
```

## Workflow 10: Full Market Scan

```bash
# 1. What's trending?
atlas market trending --output json
atlas market movers --limit 10 --output json

# 2. HL-specific movers
atlas market hyperliquid top --sort gainers --limit 10 --output json

# 3. DeFi overview
atlas market defi --output json
atlas market global --output json

# 4. DEX activity
atlas market dex trending --network base --output json
```

## Error Recovery Patterns

### Auth errors
```bash
# KEYRING_ERROR → reimport key
atlas profile generate <name>
# or
atlas profile import <name> --key <hex>

# API_KEY_MISSING
atlas configure system api-key <key>
```

### Execution errors
```bash
# SLIPPAGE_EXCEEDED → increase tolerance or retry
atlas hl perp buy ETH 200 --slippage 0.1 --output json

# INSUFFICIENT_MARGIN → check balance, reduce size, or deposit
atlas status --output json
atlas hl perp transfer deposit 500 --output json
```

### Network errors
```bash
# BACKEND_UNREACHABLE → check connectivity
atlas doctor --output json
# If backend is down, market data commands may still work (direct HL API)
# Trading commands require backend for API key validation
```
