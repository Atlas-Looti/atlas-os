# 0x API v2 Reference (for Atlas OS)

## Base URL
- `https://api.0x.org`

## Auth Headers (REQUIRED on all requests)
- `0x-api-key: <key>` — from dashboard.0x.org
- `0x-version: v2`

## Endpoints

### AllowanceHolder (Recommended)
- `GET /swap/allowance-holder/price` — indicative price (taker optional)
- `GET /swap/allowance-holder/quote` — firm quote (taker required)

### Permit2 (Advanced)
- `GET /swap/permit2/price` — indicative price
- `GET /swap/permit2/quote` — firm quote (returns permit2 EIP-712 data)

### Utility
- `GET /swap/chains` — supported chains list
- `GET /sources?chainId=<id>` — liquidity sources per chain

## Required Query Params
- `chainId` (int) — e.g. 1 (Ethereum), 42161 (Arbitrum), 8453 (Base), etc.
- `buyToken` (address) — contract address, or `0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee` for native
- `sellToken` (address) — same
- `sellAmount` (string) — amount in base units (wei for ETH, etc.)

## Optional Query Params
- `taker` (address) — required for /quote, optional for /price
- `slippageBps` (int) — default 100 (1%)
- `swapFeeRecipient` + `swapFeeBps` — integrator fee (Atlas builder fee!)
- `swapFeeToken` — which token to take fee in
- `excludedSources` — comma-separated
- `recipient` — if different from taker
- `gasPrice` — target gas price in wei

## Response Shape (when liquidityAvailable=true)
```json
{
  "liquidityAvailable": true,
  "buyAmount": "99950000",
  "sellAmount": "100000000",
  "buyToken": "0x...",
  "sellToken": "0x...",
  "allowanceTarget": "0x0000000000001ff3684f28c67538d4d072c22734",
  "minBuyAmount": "98950500",
  "blockNumber": "21544530",
  "gasPrice": "7000000000",
  "route": {
    "fills": [{ "from": "0x...", "to": "0x...", "source": "Uniswap_V3", "proportionBps": "10000" }],
    "tokens": [{ "address": "0x...", "symbol": "USDC" }]
  },
  "fees": {
    "integratorFee": { "amount": "25000", "token": "0x...", "type": "volume" },
    "zeroExFee": null,
    "gasFee": null
  },
  "issues": {
    "allowance": { "actual": "0", "spender": "0x0000000000001ff3684f28c67538d4d072c22734" },
    "balance": null,
    "simulationIncomplete": false,
    "invalidSourcesPassed": []
  },
  "tokenMetadata": { "buyToken": {...}, "sellToken": {...} },
  "transaction": {  // ONLY in /quote responses
    "to": "0x...",   // Settler contract — send tx here
    "data": "0x...", // calldata
    "gas": "...",
    "gasPrice": "...",
    "value": "0"
  },
  "zid": "0x..."
}
```

## Key Architecture Notes
- **AllowanceHolder** = recommended. Simpler UX, lower gas, same security as Permit2.
- **Allowance target ≠ Entry point**: Set allowance on `issues.allowance.spender`, send tx to `transaction.to`
- **NEVER set allowance on Settler contract** — will lose funds
- **AllowanceHolder address (Cancun chains):** `0x0000000000001fF3684f28c67538d4D072C22734`
- **Permit2 address (all chains):** `0x000000000022D473030F116dDEE9F6B43aC78BA3`

## Chain IDs
- Ethereum: 1
- Arbitrum: 42161
- Base: 8453
- Optimism: 10
- Polygon: 137
- BSC: 56
- Avalanche: 43114
- Blast: 81457
- Scroll: 534352
- Linea: 59144
- Mantle: 5000
- (19+ total)

## Builder Fee Integration
Use `swapFeeRecipient` + `swapFeeBps` + `swapFeeToken` to inject Atlas builder fee on every swap.
Max default: 1000 bps (10%). Atlas uses 10 bps (0.01%).

## Trade Analytics API
- `GET /trade-analytics/swap` — completed swap trades (max 200/request)
  - `cursor` — pagination (null for first page, then `nextCursor`)
  - `startTimestamp` / `endTimestamp` — unix seconds filter
  - Returns: `trades[]` with `appName`, `buyToken`, `sellToken`, `buyAmount`, `sellAmount`, `volumeUsd`, `fees`, `taker`, `transactionHash`, `chainId`, `timestamp`
  - Useful for: revenue tracking (integrator fees), trade history, analytics dashboard
