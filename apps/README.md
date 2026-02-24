# Atlas OS — Apps

Atlas OS adalah platform DeFi yang terdiri dari dua layanan utama: **Backend** (API gateway ke protokol DeFi) dan **Frontend** (dashboard manajemen akun pengguna).

---

## User Flow

```
User (browser)
  → login via Clerk
  → buka Dashboard (Frontend)
  → generate Atlas API Key
  → pakai key tersebut di CLI / SDK / aplikasi lain

CLI / SDK / Aplikasi
  → kirim request ke Backend dengan X-API-Key: atl_xxx
  → Backend verifikasi key, proxy ke DeFi protocols
  → return data ke client
```

---

## Backend

**Dipakai oleh:** CLI, SDK, aplikasi third-party yang sudah punya Atlas API Key.

**Fungsi utama:**
- Proxy ke DeFi protocols (Alchemy RPC, CoinGecko, 0x Swap) tanpa expose API key ke client
- Autentikasi via Atlas API Key (`atl_xxx`) untuk machine-to-machine
- Inject platform fee (0.1%) otomatis ke semua 0x swap request
- Track compute usage per user/key

**Endpoints:**

| Endpoint | Fungsi |
|---|---|
| `GET /atlas-os/me` | Profile user + info API key yang dipakai |
| `GET /atlas-os/rpc/:chain/...` | EVM RPC proxy — balance, block, gas, tx, contract |
| `GET /atlas-os/dex/...` | DEX market data — tokens, pools, trending |
| `GET /atlas-os/0x/swap/...` | 0x swap price & quote (AllowanceHolder + Permit2) |
| `POST /atlas-os/compute/usage` | Record compute event |

---

## Frontend

**Dipakai oleh:** User yang login via Clerk di browser.

**Fungsi utama:**
- Manajemen Atlas API Keys (create, list, revoke)
- Monitoring compute usage (chart history per key)

**Pages:**

| Page | Fungsi |
|---|---|
| `/` | Landing / login |
| `/dashboard` | API key management + compute usage chart |

---

## Environment Variables

**Backend** (`.env`):

| Variable | Keterangan |
|---|---|
| `DATABASE_URL` | PostgreSQL |
| `REDIS_URL` | Redis cache |
| `CLERK_SECRET_KEY` | Clerk server SDK |
| `ALCHEMY_API_KEY` | EVM RPC |
| `COINGECKO_API_KEY` | Market data |
| `ZERO_EX_API_KEY` | 0x swap |
| `ZERO_EX_FEE_RECIPIENT` | Wallet penerima platform fee |
| `ZERO_EX_FEE_BPS` | Fee amount (default: `10` = 0.1%) |

**Frontend** (`.env.local`):

| Variable | Keterangan |
|---|---|
| `NEXT_PUBLIC_API_URL` | URL backend (di-bake saat build) |
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | Clerk public key |
| `CLERK_SECRET_KEY` | Clerk server key |
