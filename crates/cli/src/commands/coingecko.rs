//! `atlas market` CoinGecko-powered commands.
//!
//! These call the Atlas backend API which proxies to CoinGecko.
//! Requires `atlas-server` to be running.

use anyhow::Result;
use atlas_core::BackendClient;
use atlas_core::output::OutputFormat;

/// Helper: ensure backend is reachable, return client.
async fn backend() -> Result<BackendClient> {
    let client = BackendClient::from_config()?;
    if !client.health().await? {
        anyhow::bail!(
            "Atlas backend not reachable. Start it with: atlas-server\n\
             Or set api_key: atlas configure system api-key <key>"
        );
    }
    Ok(client)
}

/// `atlas market global` — global crypto market stats (CoinGecko).
pub async fn global(fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/global", &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            if let Some(d) = data.get("data") {
                let mcap = d.get("total_market_cap").and_then(|m| m.get("usd"))
                    .and_then(|v| v.as_f64()).unwrap_or(0.0);
                let vol = d.get("total_volume").and_then(|m| m.get("usd"))
                    .and_then(|v| v.as_f64()).unwrap_or(0.0);
                let btc_dom = d.get("market_cap_percentage").and_then(|m| m.get("btc"))
                    .and_then(|v| v.as_f64()).unwrap_or(0.0);
                let eth_dom = d.get("market_cap_percentage").and_then(|m| m.get("eth"))
                    .and_then(|v| v.as_f64()).unwrap_or(0.0);
                let coins = d.get("active_cryptocurrencies")
                    .and_then(|v| v.as_u64()).unwrap_or(0);
                let mcap_chg = d.get("market_cap_change_percentage_24h_usd")
                    .and_then(|v| v.as_f64()).unwrap_or(0.0);

                println!("┌─────────────────────────────────────────────────┐");
                println!("│  🌍 GLOBAL CRYPTO MARKET                        │");
                println!("├─────────────────────────────────────────────────┤");
                println!("│  Total MCap    : ${:<28.2}B │", mcap / 1e9);
                println!("│  24h Volume    : ${:<28.2}B │", vol / 1e9);
                println!("│  MCap 24h Chg  : {:>+28.2}% │", mcap_chg);
                println!("│  BTC Dominance : {:>28.1}% │", btc_dom);
                println!("│  ETH Dominance : {:>28.1}% │", eth_dom);
                println!("│  Active Coins  : {:<29} │", coins);
                println!("└─────────────────────────────────────────────────┘");
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
    }

    Ok(())
}

/// `atlas market trending` — trending coins (CoinGecko).
pub async fn trending(fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/trending", &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🔥 Trending Coins\n");
            if let Some(coins) = data.get("coins").and_then(|c| c.as_array()) {
                println!("{:<6} {:<20} {:<8} {:>12} {:>10}", "#", "NAME", "SYMBOL", "PRICE", "24h CHG");
                println!("{}", "─".repeat(60));
                for (i, coin) in coins.iter().enumerate() {
                    if let Some(item) = coin.get("item") {
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let symbol = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
                        let price = item.get("data").and_then(|d| d.get("price"))
                            .and_then(|v| v.as_f64());
                        let chg = item.get("data")
                            .and_then(|d| d.get("price_change_percentage_24h"))
                            .and_then(|p| p.get("usd"))
                            .and_then(|v| v.as_f64());

                        let price_str = price.map(|p| {
                            if p < 0.01 { format!("${:.6}", p) }
                            else if p < 1.0 { format!("${:.4}", p) }
                            else { format!("${:.2}", p) }
                        }).unwrap_or("—".into());

                        let chg_str = chg.map(|c| format!("{:+.2}%", c)).unwrap_or("—".into());

                        println!("{:<6} {:<20} {:<8} {:>12} {:>10}",
                            i + 1,
                            &name[..name.len().min(19)],
                            symbol.to_uppercase(),
                            price_str,
                            chg_str);
                    }
                }
            }
        }
    }

    Ok(())
}

/// `atlas market coin <id>` — detailed coin info (CoinGecko).
pub async fn coin(id: &str, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = format!("/api/coingecko/coins/{}", id.to_lowercase());
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let symbol = data.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
            let md = data.get("market_data");

            let price = md.and_then(|d| d.get("current_price"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());
            let mcap = md.and_then(|d| d.get("market_cap"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());
            let vol = md.and_then(|d| d.get("total_volume"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());
            let chg_24h = md.and_then(|d| d.get("price_change_percentage_24h"))
                .and_then(|v| v.as_f64());
            let chg_7d = md.and_then(|d| d.get("price_change_percentage_7d"))
                .and_then(|v| v.as_f64());
            let chg_30d = md.and_then(|d| d.get("price_change_percentage_30d"))
                .and_then(|v| v.as_f64());
            let ath = md.and_then(|d| d.get("ath"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());
            let ath_chg = md.and_then(|d| d.get("ath_change_percentage"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());
            let circ = md.and_then(|d| d.get("circulating_supply"))
                .and_then(|v| v.as_f64());
            let total = md.and_then(|d| d.get("total_supply"))
                .and_then(|v| v.as_f64());
            let rank = data.get("market_cap_rank").and_then(|v| v.as_u64());
            let fdv = md.and_then(|d| d.get("fully_diluted_valuation"))
                .and_then(|p| p.get("usd")).and_then(|v| v.as_f64());

            println!("┌─────────────────────────────────────────────────┐");
            println!("│  {} ({})  #{:<36}│", name, symbol.to_uppercase(),
                rank.map(|r| r.to_string()).unwrap_or("—".into()));
            println!("├─────────────────────────────────────────────────┤");
            println!("│  Price         : ${:<29} │",
                price.map(|p| format!("{:.2}", p)).unwrap_or("—".into()));
            println!("│  Market Cap    : ${:<29} │",
                mcap.map(|m| format!("{:.0}", m / 1e6).to_string() + "M").unwrap_or("—".into()));
            println!("│  FDV           : ${:<29} │",
                fdv.map(|f| format!("{:.0}M", f / 1e6)).unwrap_or("—".into()));
            println!("│  24h Volume    : ${:<29} │",
                vol.map(|v| format!("{:.0}M", v / 1e6)).unwrap_or("—".into()));
            println!("├─────────────────────────────────────────────────┤");
            println!("│  24h Change    : {:>+29.2}% │", chg_24h.unwrap_or(0.0));
            println!("│  7d Change     : {:>+29.2}% │", chg_7d.unwrap_or(0.0));
            println!("│  30d Change    : {:>+29.2}% │", chg_30d.unwrap_or(0.0));
            println!("├─────────────────────────────────────────────────┤");
            println!("│  ATH           : ${:<29} │",
                ath.map(|a| format!("{:.2}", a)).unwrap_or("—".into()));
            println!("│  From ATH      : {:>+29.2}% │", ath_chg.unwrap_or(0.0));
            println!("│  Circ. Supply  : {:<30} │",
                circ.map(|c| format!("{:.0}", c)).unwrap_or("—".into()));
            println!("│  Total Supply  : {:<30} │",
                total.map(|t| format!("{:.0}", t)).unwrap_or("—".into()));
            println!("└─────────────────────────────────────────────────┘");
        }
    }

    Ok(())
}

/// `atlas market movers [--limit 10]` — top gainers & losers (CoinGecko).
pub async fn movers(limit: usize, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/top-movers", &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("📊 Top Movers (CoinGecko)\n");

            if let Some(gainers) = data.get("top_gainers").and_then(|g| g.as_array()) {
                println!("🟢 TOP GAINERS");
                println!("{:<20} {:<8} {:>12} {:>10}", "NAME", "SYMBOL", "PRICE", "24h CHG");
                println!("{}", "─".repeat(53));
                for coin in gainers.iter().take(limit) {
                    let name = coin.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let sym = coin.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
                    let price = coin.get("usd").and_then(|v| v.as_f64());
                    let chg = coin.get("usd_24h_change").and_then(|v| v.as_f64());
                    println!("{:<20} {:<8} {:>12} {:>+10.2}%",
                        &name[..name.len().min(19)],
                        sym.to_uppercase(),
                        price.map(|p| format!("${:.4}", p)).unwrap_or("—".into()),
                        chg.unwrap_or(0.0));
                }
            }

            println!();

            if let Some(losers) = data.get("top_losers").and_then(|g| g.as_array()) {
                println!("🔴 TOP LOSERS");
                println!("{:<20} {:<8} {:>12} {:>10}", "NAME", "SYMBOL", "PRICE", "24h CHG");
                println!("{}", "─".repeat(53));
                for coin in losers.iter().take(limit) {
                    let name = coin.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let sym = coin.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
                    let price = coin.get("usd").and_then(|v| v.as_f64());
                    let chg = coin.get("usd_24h_change").and_then(|v| v.as_f64());
                    println!("{:<20} {:<8} {:>12} {:>+10.2}%",
                        &name[..name.len().min(19)],
                        sym.to_uppercase(),
                        price.map(|p| format!("${:.4}", p)).unwrap_or("—".into()),
                        chg.unwrap_or(0.0));
                }
            }
        }
    }

    Ok(())
}

/// `atlas market dex trending` — trending onchain pools.
pub async fn dex_trending(network: Option<&str>, limit: usize, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = match network {
        Some(net) => format!("/api/coingecko/onchain/trending-pools/{}", net),
        None => "/api/coingecko/onchain/trending-pools".to_string(),
    };
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🔥 Trending Pools{}\n",
                network.map(|n| format!(" ({})", n)).unwrap_or_default());
            print_pools_table(data.get("data"), limit);
        }
    }
    Ok(())
}

/// `atlas market dex new` — newly created pools.
pub async fn dex_new(network: Option<&str>, limit: usize, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = match network {
        Some(net) => format!("/api/coingecko/onchain/new-pools/{}", net),
        None => "/api/coingecko/onchain/new-pools".to_string(),
    };
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🆕 New Pools{}\n",
                network.map(|n| format!(" ({})", n)).unwrap_or_default());
            print_pools_table(data.get("data"), limit);
        }
    }
    Ok(())
}

/// `atlas market dex pools <network>` — top pools on a network.
pub async fn dex_top_pools(network: &str, limit: usize, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = format!("/api/coingecko/onchain/pools/{}", network);
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("📊 Top Pools — {}\n", network);
            print_pools_table(data.get("data"), limit);
        }
    }
    Ok(())
}

/// `atlas market dex pool <network> <address>` — pool details.
pub async fn dex_pool_detail(network: &str, address: &str, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = format!("/api/coingecko/onchain/pools/{}/{}", network, address);
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            if let Some(pool) = data.get("data").and_then(|d| d.get("attributes")) {
                let name = pool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let vol = pool.get("volume_usd").and_then(|v| v.get("h24"))
                    .and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
                let liq = pool.get("reserve_in_usd")
                    .and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
                let price = pool.get("base_token_price_usd")
                    .and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok());
                let chg_5m = pool.get("price_change_percentage")
                    .and_then(|p| p.get("m5")).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let chg_1h = pool.get("price_change_percentage")
                    .and_then(|p| p.get("h1")).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let chg_24h = pool.get("price_change_percentage")
                    .and_then(|p| p.get("h24")).and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let buys = pool.get("transactions")
                    .and_then(|t| t.get("h24")).and_then(|h| h.get("buys"))
                    .and_then(|v| v.as_u64());
                let sells = pool.get("transactions")
                    .and_then(|t| t.get("h24")).and_then(|h| h.get("sells"))
                    .and_then(|v| v.as_u64());

                println!("┌─────────────────────────────────────────────────┐");
                println!("│  🏊 {}  {}/{:<31}│", name, network, address.get(..8).unwrap_or(address));
                println!("├─────────────────────────────────────────────────┤");
                println!("│  Token Price   : ${:<29} │",
                    price.map(|p| format!("{:.6}", p)).unwrap_or("—".into()));
                println!("│  24h Volume    : ${:<29} │",
                    vol.map(|v| format!("{:.0}K", v / 1e3)).unwrap_or("—".into()));
                println!("│  Liquidity     : ${:<29} │",
                    liq.map(|l| format!("{:.0}K", l / 1e3)).unwrap_or("—".into()));
                println!("├─────────────────────────────────────────────────┤");
                println!("│  5m Change     : {:>+29.2}% │", chg_5m.unwrap_or(0.0));
                println!("│  1h Change     : {:>+29.2}% │", chg_1h.unwrap_or(0.0));
                println!("│  24h Change    : {:>+29.2}% │", chg_24h.unwrap_or(0.0));
                println!("├─────────────────────────────────────────────────┤");
                println!("│  24h Buys      : {:<30} │", buys.unwrap_or(0));
                println!("│  24h Sells     : {:<30} │", sells.unwrap_or(0));
                println!("└─────────────────────────────────────────────────┘");
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
    }
    Ok(())
}

/// `atlas market dex token <network> <address>` — token info.
pub async fn dex_token_info(network: &str, address: &str, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = format!("/api/coingecko/onchain/tokens/{}/{}/info", network, address);
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            if let Some(attrs) = data.get("data").and_then(|d| d.get("attributes")) {
                let name = attrs.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let symbol = attrs.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
                let desc = attrs.get("description").and_then(|v| v.as_str()).unwrap_or("");
                let website = attrs.get("websites").and_then(|w| w.as_array())
                    .and_then(|a| a.first()).and_then(|v| v.as_str());

                println!("┌─────────────────────────────────────────────────┐");
                println!("│  🪙 {} ({})  on {:<22}│",
                    name, symbol.to_uppercase(), network);
                println!("│  {:<47} │", address);
                println!("├─────────────────────────────────────────────────┤");
                if !desc.is_empty() {
                    // Wrap description to fit box
                    for line in desc.chars().collect::<Vec<_>>().chunks(45) {
                        let s: String = line.iter().collect();
                        println!("│  {:<47} │", s);
                    }
                }
                if let Some(url) = website {
                    println!("│  🔗 {:<44} │", url);
                }
                println!("└─────────────────────────────────────────────────┘");
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }
    }
    Ok(())
}

/// `atlas market dex networks` — list supported networks.
pub async fn dex_networks(fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/onchain/networks", &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🌐 Supported Networks\n");
            if let Some(networks) = data.get("data").and_then(|d| d.as_array()) {
                println!("{:<25} {:<30}", "ID", "NAME");
                println!("{}", "─".repeat(55));
                for net in networks {
                    let id = net.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let name = net.get("attributes")
                        .and_then(|a| a.get("name"))
                        .and_then(|v| v.as_str()).unwrap_or("?");
                    println!("{:<25} {:<30}", id, name);
                }
                println!("\n({} networks)", networks.len());
            }
        }
    }
    Ok(())
}

/// `atlas market dex dexes <network>` — list DEXes on a network.
pub async fn dex_dexes(network: &str, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let path = format!("/api/coingecko/onchain/dexes/{}", network);
    let data = client.get(&path, &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🏪 DEXes on {}\n", network);
            if let Some(dexes) = data.get("data").and_then(|d| d.as_array()) {
                println!("{:<30} {:<30}", "ID", "NAME");
                println!("{}", "─".repeat(60));
                for dex in dexes {
                    let id = dex.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let name = dex.get("attributes")
                        .and_then(|a| a.get("name"))
                        .and_then(|v| v.as_str()).unwrap_or("?");
                    println!("{:<30} {:<30}", id, name);
                }
                println!("\n({} DEXes)", dexes.len());
            }
        }
    }
    Ok(())
}

/// `atlas market dex search <query>` — search onchain tokens/pools.
pub async fn dex_search(query: &str, fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/onchain/search", &[("query", query)]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            println!("🔍 Onchain search: '{}'\n", query);

            // Pools
            if let Some(pools) = data.get("data")
                .and_then(|d| d.get("attributes"))
                .and_then(|a| a.get("pools"))
                .and_then(|p| p.as_array())
            {
                if !pools.is_empty() {
                    println!("POOLS:");
                    println!("{:<30} {:<12} {:<15} {:>12}", "NAME", "NETWORK", "DEX", "PRICE");
                    println!("{}", "─".repeat(72));
                    for pool in pools.iter().take(10) {
                        let name = pool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let net = pool.get("network").and_then(|n| n.get("identifier"))
                            .and_then(|v| v.as_str()).unwrap_or("?");
                        let dex = pool.get("dex").and_then(|d| d.get("name"))
                            .and_then(|v| v.as_str()).unwrap_or("?");
                        let price = pool.get("price_in_usd")
                            .and_then(|v| v.as_str());
                        println!("{:<30} {:<12} {:<15} {:>12}",
                            &name[..name.len().min(29)],
                            &net[..net.len().min(11)],
                            &dex[..dex.len().min(14)],
                            price.map(|p| format!("${}", p)).unwrap_or("—".into()));
                    }
                    println!();
                }
            }

            // Tokens (if present — CoinGecko search may also return tokens)
            if let Some(tokens) = data.get("data")
                .and_then(|d| d.get("attributes"))
                .and_then(|a| a.get("tokens"))
                .and_then(|t| t.as_array())
            {
                if !tokens.is_empty() {
                    println!("TOKENS:");
                    println!("{:<20} {:<8} {:<15} {:>12}", "NAME", "SYMBOL", "NETWORK", "PRICE");
                    println!("{}", "─".repeat(58));
                    for token in tokens.iter().take(10) {
                        let name = token.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let sym = token.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
                        let net = token.get("network").and_then(|n| n.get("identifier"))
                            .and_then(|v| v.as_str()).unwrap_or("?");
                        let price = token.get("price_in_usd")
                            .and_then(|v| v.as_str());
                        println!("{:<20} {:<8} {:<15} {:>12}",
                            &name[..name.len().min(19)],
                            sym.to_uppercase(),
                            net,
                            price.map(|p| format!("${}", p)).unwrap_or("—".into()));
                    }
                }
            }
        }
    }
    Ok(())
}

// ── Helper: render pool table ──────────────────────────────────────────

fn print_pools_table(data: Option<&serde_json::Value>, limit: usize) {
    if let Some(pools) = data.and_then(|d| d.as_array()) {
        println!("{:<30} {:<10} {:>14} {:>12} {:>10}",
            "POOL", "DEX", "VOLUME 24h", "LIQUIDITY", "24h CHG");
        println!("{}", "─".repeat(80));
        for pool in pools.iter().take(limit) {
            let name = pool.get("attributes")
                .and_then(|a| a.get("name"))
                .and_then(|v| v.as_str()).unwrap_or("?");
            let dex = pool.get("relationships")
                .and_then(|r| r.get("dex"))
                .and_then(|d| d.get("data"))
                .and_then(|d| d.get("id"))
                .and_then(|v| v.as_str()).unwrap_or("?");
            let vol = pool.get("attributes")
                .and_then(|a| a.get("volume_usd"))
                .and_then(|v| v.get("h24"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());
            let liq = pool.get("attributes")
                .and_then(|a| a.get("reserve_in_usd"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());
            let chg = pool.get("attributes")
                .and_then(|a| a.get("price_change_percentage"))
                .and_then(|p| p.get("h24"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            println!("{:<30} {:<10} {:>14} {:>12} {:>+10.2}%",
                &name[..name.len().min(29)],
                &dex[..dex.len().min(9)],
                vol.map(|v| format!("${:.0}K", v / 1e3)).unwrap_or("—".into()),
                liq.map(|l| format!("${:.0}K", l / 1e3)).unwrap_or("—".into()),
                chg.unwrap_or(0.0));
        }
    } else {
        println!("No pools found.");
    }
}

/// `atlas market defi` — global DeFi stats (CoinGecko).
pub async fn defi(fmt: OutputFormat) -> Result<()> {
    let client = backend().await?;
    let data = client.get("/api/coingecko/global/defi", &[]).await?;

    match fmt {
        OutputFormat::Json => println!("{}", serde_json::to_string(&data)?),
        OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&data)?),
        OutputFormat::Table => {
            if let Some(d) = data.get("data") {
                let tvl = d.get("defi_market_cap").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let vol = d.get("trading_volume_24h").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let dom = d.get("defi_dominance").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let top_name = d.get("top_coin_name").and_then(|v| v.as_str());
                let top_dom = d.get("top_coin_defi_dominance")
                    .and_then(|v| v.as_f64());

                println!("┌─────────────────────────────────────────────────┐");
                println!("│  🏦 GLOBAL DeFi STATS                           │");
                println!("├─────────────────────────────────────────────────┤");
                println!("│  DeFi MCap     : ${:<28.2}B │", tvl.unwrap_or(0.0) / 1e9);
                println!("│  24h Volume    : ${:<28.2}B │", vol.unwrap_or(0.0) / 1e9);
                println!("│  DeFi Dom.     : {:>28.2}% │", dom.unwrap_or(0.0));
                println!("│  Top Protocol  : {:<30} │",
                    top_name.unwrap_or("—"));
                println!("│  Top Dom.      : {:>28.2}% │", top_dom.unwrap_or(0.0));
                println!("└─────────────────────────────────────────────────┘");
            }
        }
    }

    Ok(())
}
