# Alchemy API Reference — Atlas OS Backend

## Base URLs
- JSON-RPC: `https://{network}.g.alchemy.com/v2/{apiKey}`
- Data/Portfolio: `https://api.g.alchemy.com/data/v1/{apiKey}/...`

## Networks
- `eth-mainnet`, `eth-sepolia`, `base-mainnet`, `arb-mainnet`, `opt-mainnet`, `polygon-mainnet`, etc.

## Free Tier
- 30M compute units/month
- 500 CU/s throughput
- 5 apps, 5 webhooks

## Key APIs for Atlas OS

### 1. Token Balances (JSON-RPC)
```
POST https://{network}.g.alchemy.com/v2/{apiKey}
{
  "jsonrpc": "2.0",
  "method": "alchemy_getTokenBalances",
  "params": ["0xOwnerAddress", ["0xTokenAddress"]],
  "id": 1
}
→ { address, tokenBalances: [{ contractAddress, tokenBalance }] }
```

### 2. Token Metadata (JSON-RPC)
```
POST https://{network}.g.alchemy.com/v2/{apiKey}
{
  "jsonrpc": "2.0",
  "method": "alchemy_getTokenMetadata",
  "params": ["0xTokenAddress"],
  "id": 1
}
→ { name, symbol, decimals, logo }
```

### 3. Token Allowance (JSON-RPC)
```
POST https://{network}.g.alchemy.com/v2/{apiKey}
{
  "jsonrpc": "2.0",
  "method": "alchemy_getTokenAllowance",
  "params": [{ "contract": "0x...", "owner": "0x...", "spender": "0x..." }],
  "id": 1
}
→ "allowance_amount"
```

### 4. Portfolio — Tokens by Wallet (REST, multi-chain!)
```
POST https://api.g.alchemy.com/data/v1/{apiKey}/assets/tokens/by-address
{
  "addresses": [{ "address": "0x...", "networks": ["eth-mainnet", "base-mainnet"] }],
  "withMetadata": true,
  "withPrices": true,
  "includeNativeTokens": true,
  "includeErc20Tokens": true
}
→ { data: { tokens: [{ address, network, tokenAddress, tokenBalance, tokenMetadata, tokenPrices }] } }
```

### 5. Portfolio — Token Balances Only (REST, lighter)
```
POST https://api.g.alchemy.com/data/v1/{apiKey}/assets/tokens/balances/by-address
{
  "addresses": [{ "address": "0x...", "networks": ["eth-mainnet"] }],
  "includeNativeTokens": true,
  "includeErc20Tokens": true
}
→ { data: { tokens: [{ network, address, tokenAddress, tokenBalance }] } }
```

### 6. Transaction History (REST)
```
POST https://api.g.alchemy.com/data/v1/{apiKey}/transactions/history
{
  "addresses": [{ "address": "0x...", "networks": ["eth-mainnet"] }]
}
→ { transactions: [{ hash, timeStamp, blockNumber, fromAddress, toAddress, value, gasUsed, logs, internalTxns }] }
```

## Error Handling
- HTTP 429 = rate limited → retry with exponential backoff
- Always check `response.error` field even on 200
- Use `Retry-After` header when present

## Batch Requests
- Max 1000 requests per batch (HTTP)
- Max 20 per batch (WebSocket)
- Not supported for Token API, Transfers API, Trace API

## Rate Limiting Strategy
- Free: 500 CU/s, PAYG: 10,000 CU/s
- Implement: Redis-cached responses + exponential backoff
- Cache token metadata (rarely changes) for 24h
- Cache balances for 30s
- Cache prices for 10s
