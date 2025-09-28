//! Coinbase Pro/Advanced Trade API Connector
//!
//! Implements the ExchangeConnector trait for Coinbase Pro and Advanced Trade APIs.
//! Supports:
//! - REST API for order management and account queries
//! - WebSocket streaming for real-time market data and order updates
//! - HMAC-SHA256 authentication
//! - Rate limiting and error handling

use crate::{
    utils::{hmac_sha256_signature, timestamp},
    Balance, ExchangeConnector, ExchangeError, ExchangeId, ExchangeOrder, ExchangeResult, Fill,
    MarketTick, OrderSide, OrderStatus, OrderType, RateLimiter, StreamMessage, TradingPair,
    TransferRequest, TransferStatus,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client, Method, RequestBuilder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;

/// Coinbase Pro API URLs
const COINBASE_PRO_API_URL: &str = "https://api.pro.coinbase.com";
const COINBASE_PRO_SANDBOX_URL: &str = "https://api-public.sandbox.pro.coinbase.com";
const COINBASE_PRO_WS_URL: &str = "wss://ws-feed.pro.coinbase.com";
const COINBASE_PRO_WS_SANDBOX_URL: &str = "wss://ws-feed-public.sandbox.pro.coinbase.com";

/// Coinbase Advanced Trade API URLs
const COINBASE_ADVANCED_API_URL: &str = "https://api.coinbase.com/api/v3/brokerage";
const COINBASE_ADVANCED_WS_URL: &str = "wss://advanced-trade-ws.coinbase.com";

#[derive(Debug, Clone)]
pub struct CoinbaseConfig {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
    pub sandbox: bool,
    pub use_advanced_trade: bool, // Use Advanced Trade API vs Pro API
}

/// Coinbase Pro/Advanced Trade connector
pub struct CoinbaseConnector {
    config: CoinbaseConfig,
    client: Client,
    rate_limiter: RateLimiter,
    base_url: String,
    ws_url: String,
    connected: bool,
}

impl CoinbaseConnector {
    pub fn new(config: CoinbaseConfig) -> Self {
        let base_url = if config.use_advanced_trade {
            COINBASE_ADVANCED_API_URL.to_string()
        } else if config.sandbox {
            COINBASE_PRO_SANDBOX_URL.to_string()
        } else {
            COINBASE_PRO_API_URL.to_string()
        };

        let ws_url = if config.use_advanced_trade {
            COINBASE_ADVANCED_WS_URL.to_string()
        } else if config.sandbox {
            COINBASE_PRO_WS_SANDBOX_URL.to_string()
        } else {
            COINBASE_PRO_WS_URL.to_string()
        };

        let client = Client::new();
        let rate_limiter = RateLimiter::new(10); // 10 requests per second limit

        Self {
            config,
            client,
            rate_limiter,
            base_url,
            ws_url,
            connected: false,
        }
    }

    /// Create authenticated request for Coinbase Pro API
    fn create_authenticated_request(
        &self,
        method: Method,
        path: &str,
        body: &str,
    ) -> RequestBuilder {
        let timestamp = timestamp();

        // Create message for signature: timestamp + method + path + body
        let message = format!("{}{}{}{}", timestamp, method.as_str(), path, body);
        let signature = hmac_sha256_signature(&self.config.api_secret, &message);

        let url = format!("{}{}", self.base_url, path);

        self.client
            .request(method, &url)
            .header("CB-ACCESS-KEY", &self.config.api_key)
            .header("CB-ACCESS-SIGN", signature)
            .header("CB-ACCESS-TIMESTAMP", timestamp)
            .header("CB-ACCESS-PASSPHRASE", &self.config.passphrase)
            .header("Content-Type", "application/json")
    }

    /// Handle API response and convert errors
    async fn handle_response<T>(&self, response: reqwest::Response) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        debug!("Coinbase API response: {} - {}", status, response_text);

        if status.is_success() {
            serde_json::from_str(&response_text)
                .map_err(|e| ExchangeError::InvalidRequest(format!("JSON parse error: {}", e)))
        } else {
            // Parse error response
            if let Ok(error_response) =
                serde_json::from_str::<CoinbaseErrorResponse>(&response_text)
            {
                Err(ExchangeError::Api {
                    code: status.as_u16().to_string(),
                    message: error_response.message,
                })
            } else {
                Err(ExchangeError::Api {
                    code: status.as_u16().to_string(),
                    message: response_text,
                })
            }
        }
    }
}

#[async_trait]
impl ExchangeConnector for CoinbaseConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Coinbase
    }

    async fn connect(&mut self) -> ExchangeResult<()> {
        info!(
            "Connecting to Coinbase {}...",
            if self.config.use_advanced_trade {
                "Advanced Trade"
            } else {
                "Pro"
            }
        );

        // Test connection by fetching account info
        self.rate_limiter.acquire().await?;

        let request = self.create_authenticated_request(Method::GET, "/accounts", "");
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        // Just check if we get a successful response
        if response.status().is_success() {
            self.connected = true;
            info!("Successfully connected to Coinbase");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("Failed to connect to Coinbase: {}", error_text);
            Err(ExchangeError::Authentication(error_text))
        }
    }

    async fn disconnect(&mut self) -> ExchangeResult<()> {
        self.connected = false;
        info!("Disconnected from Coinbase");
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn get_trading_pairs(&self) -> ExchangeResult<Vec<TradingPair>> {
        self.rate_limiter.acquire().await?;

        let request = self.client.get(&format!("{}/products", self.base_url));
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let products: Vec<CoinbaseProduct> = self.handle_response(response).await?;

        let trading_pairs = products
            .into_iter()
            .filter(|p| p.status == "online" && !p.trading_disabled)
            .map(|p| TradingPair {
                base: p.base_currency,
                quote: p.quote_currency,
                symbol: p.id,
            })
            .collect();

        Ok(trading_pairs)
    }

    async fn get_balances(&self) -> ExchangeResult<Vec<Balance>> {
        self.rate_limiter.acquire().await?;

        let request = self.create_authenticated_request(Method::GET, "/accounts", "");
        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let accounts: Vec<CoinbaseAccount> = self.handle_response(response).await?;

        let balances = accounts
            .into_iter()
            .map(|acc| Balance {
                currency: acc.currency,
                available: acc.available.parse().unwrap_or_default(),
                total: acc.balance.parse().unwrap_or_default(),
                hold: acc.hold.parse().unwrap_or_default(),
            })
            .collect();

        Ok(balances)
    }

    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> ExchangeResult<ExchangeOrder> {
        self.rate_limiter.acquire().await?;

        let coinbase_side = match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        let coinbase_type = match order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::Stop => "stop",
            OrderType::StopLimit => "stop_limit",
        };

        let mut order_request = CoinbaseOrderRequest {
            product_id: symbol.to_string(),
            side: coinbase_side.to_string(),
            order_type: coinbase_type.to_string(),
            size: Some(quantity.to_string()),
            price: price.map(|p| p.to_string()),
            ..Default::default()
        };

        // For market orders, use funds instead of size for buys
        if order_type == OrderType::Market && side == OrderSide::Buy {
            if let Some(p) = price {
                order_request.funds = Some((quantity * p).to_string());
                order_request.size = None;
            }
        }

        let body = serde_json::to_string(&order_request)
            .map_err(|e| ExchangeError::InvalidRequest(e.to_string()))?;

        let request = self
            .create_authenticated_request(Method::POST, "/orders", &body)
            .body(body);

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn cancel_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        self.rate_limiter.acquire().await?;

        let path = format!("/orders/{}", order_id);
        let request = self.create_authenticated_request(Method::DELETE, &path, "");

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn get_order(&self, order_id: &str) -> ExchangeResult<ExchangeOrder> {
        self.rate_limiter.acquire().await?;

        let path = format!("/orders/{}", order_id);
        let request = self.create_authenticated_request(Method::GET, &path, "");

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let coinbase_order: CoinbaseOrder = self.handle_response(response).await?;

        Ok(convert_coinbase_order(coinbase_order))
    }

    async fn get_market_data(&self, symbol: &str) -> ExchangeResult<MarketTick> {
        self.rate_limiter.acquire().await?;

        let url = format!("{}/products/{}/ticker", self.base_url, symbol);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        let ticker: CoinbaseTicker = self.handle_response(response).await?;

        Ok(MarketTick {
            symbol: symbol.to_string(),
            bid: ticker.bid.parse().unwrap_or_default(),
            ask: ticker.ask.parse().unwrap_or_default(),
            last: ticker.price.parse().unwrap_or_default(),
            volume_24h: ticker.volume.parse().unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn start_market_stream(
        &self,
        symbols: Vec<String>,
    ) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        if symbols.is_empty() {
            return Err(ExchangeError::InvalidRequest(
                "at least one product id must be supplied for Coinbase streams".into(),
            ));
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let ws_url = self.ws_url.clone();
        let products = symbols.clone();

        tokio::spawn(async move {
            if let Err(err) = run_coinbase_market_stream(ws_url, products, tx.clone()).await {
                error!(%err, "coinbase market stream terminated");
            }
        });

        Ok(rx)
    }

    async fn start_order_stream(&self) -> ExchangeResult<mpsc::UnboundedReceiver<StreamMessage>> {
        // WebSocket implementation would go here
        // For now, return a placeholder channel
        let (_tx, rx) = mpsc::unbounded_channel();
        warn!("Coinbase WebSocket order stream not yet implemented");
        Ok(rx)
    }

    async fn transfer_funds(&self, _request: TransferRequest) -> ExchangeResult<String> {
        // Coinbase doesn't support direct transfers to other exchanges via API
        Err(ExchangeError::InvalidRequest(
            "Direct fund transfers not supported by Coinbase API".to_string(),
        ))
    }

    async fn get_transfer_status(&self, _transfer_id: &str) -> ExchangeResult<TransferStatus> {
        Err(ExchangeError::InvalidRequest(
            "Transfer status not supported by Coinbase API".to_string(),
        ))
    }
}

async fn run_coinbase_market_stream(
    ws_url: String,
    products: Vec<String>,
    sender: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let url = Url::parse(&ws_url).map_err(|err| ExchangeError::Network(err.to_string()))?;
    let mut attempt: u32 = 0;

    loop {
        attempt = attempt.saturating_add(1);
        debug!(attempt, url = %url, "connecting to Coinbase market websocket");

        match connect_async(url.clone()).await {
            Ok((mut stream, _)) => {
                info!("coinbase websocket connected");
                attempt = 0;
                let subscribe = build_coinbase_subscription(&products);
                if let Err(err) = stream.send(Message::Text(subscribe)).await {
                    warn!(%err, "failed to send Coinbase subscription");
                    continue;
                }

                while let Some(message) = stream.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Err(err) = handle_coinbase_message(&text, &sender) {
                                warn!(%err, "failed to handle Coinbase message");
                            }
                        }
                        Ok(Message::Binary(bin)) => {
                            if let Ok(text) = String::from_utf8(bin) {
                                if let Err(err) = handle_coinbase_message(&text, &sender) {
                                    warn!(%err, "failed to handle Coinbase message");
                                }
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            if let Err(err) = stream.send(Message::Pong(payload)).await {
                                warn!(%err, "failed to pong Coinbase");
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            info!("coinbase websocket closed by peer");
                            break;
                        }
                        Err(err) => {
                            warn!(%err, "coinbase websocket error");
                            break;
                        }
                        _ => {}
                    }

                    if sender.is_closed() {
                        debug!("coinbase subscriber dropped channel; stopping stream");
                        return Ok(());
                    }
                }
            }
            Err(err) => {
                warn!(%err, "coinbase websocket connection failed");
            }
        }

        if sender.is_closed() {
            debug!("coinbase subscriber dropped channel; terminating reconnect loop");
            return Ok(());
        }

        let delay = websocket_backoff(attempt);
        warn!(?delay, attempt, "reconnecting to Coinbase websocket");
        sleep(delay).await;
    }
}

fn build_coinbase_subscription(products: &[String]) -> String {
    json!({
        "type": "subscribe",
        "product_ids": products,
        "channels": [
            { "name": "level2", "product_ids": products },
            { "name": "ticker", "product_ids": products },
            "matches"
        ]
    })
    .to_string()
}

fn handle_coinbase_message(
    payload: &str,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let value: serde_json::Value = serde_json::from_str(payload)
        .map_err(|err| ExchangeError::Network(format!("invalid coinbase payload: {err}")))?;

    let Some(message_type) = value.get("type").and_then(|v| v.as_str()) else {
        return Ok(());
    };

    match message_type {
        "ticker" => emit_coinbase_ticker(&value, sender)?,
        "l2update" => emit_coinbase_l2update(&value, sender)?,
        "snapshot" => emit_coinbase_snapshot(&value, sender)?,
        "error" => {
            let err_msg = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            warn!("coinbase stream error: {err_msg}");
        }
        "subscriptions" => {}
        _ => {}
    }

    Ok(())
}

fn emit_coinbase_ticker(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in ticker".into()))?;

    let bid = parse_decimal_opt(value.get("best_bid"))
        .or_else(|_| parse_decimal_opt(value.get("bid")))?;
    let ask = parse_decimal_opt(value.get("best_ask"))
        .or_else(|_| parse_decimal_opt(value.get("ask")))?;
    let last = parse_decimal_opt(value.get("price"))?;
    let volume = parse_decimal_opt(value.get("volume_24h")).unwrap_or_else(|_| Decimal::ZERO);
    let timestamp = parse_timestamp(value.get("time"));

    let tick = MarketTick {
        symbol: product_id.to_string(),
        bid,
        ask,
        last,
        volume_24h: volume,
        timestamp,
    };

    let _ = sender.send(StreamMessage::Tick(tick));
    Ok(())
}

fn emit_coinbase_l2update(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in l2update".into()))?;
    let timestamp = parse_timestamp(value.get("time"));

    if let Some(changes) = value.get("changes").and_then(|v| v.as_array()) {
        for change in changes {
            let Some(entries) = change.as_array() else {
                continue;
            };
            if entries.len() < 3 {
                continue;
            }
            let Some(side_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(price_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(2).and_then(|v| v.as_str()) else {
                continue;
            };

            let side = match side_str {
                "buy" => OrderSide::Buy,
                "sell" => OrderSide::Sell,
                _ => continue,
            };

            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in l2update: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in l2update: {err}"))
            })?;

            emit_coinbase_order_event(sender, product_id, side, price, quantity, timestamp);
        }
    }

    Ok(())
}

fn emit_coinbase_snapshot(
    value: &serde_json::Value,
    sender: &mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), ExchangeError> {
    let product_id = value
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Network("missing product_id in snapshot".into()))?;
    let timestamp = chrono::Utc::now();

    if let Some(bids) = value.get("bids").and_then(|v| v.as_array()) {
        for level in bids {
            let Some(entries) = level.as_array() else {
                continue;
            };
            if entries.len() < 2 {
                continue;
            }
            let Some(price_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in snapshot: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in snapshot: {err}"))
            })?;

            emit_coinbase_order_event(
                sender,
                product_id,
                OrderSide::Buy,
                price,
                quantity,
                timestamp,
            );
        }
    }

    if let Some(asks) = value.get("asks").and_then(|v| v.as_array()) {
        for level in asks {
            let Some(entries) = level.as_array() else {
                continue;
            };
            if entries.len() < 2 {
                continue;
            }
            let Some(price_str) = entries.get(0).and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(size_str) = entries.get(1).and_then(|v| v.as_str()) else {
                continue;
            };
            let price = Decimal::from_str(price_str).map_err(|err| {
                ExchangeError::Network(format!("invalid price in snapshot: {err}"))
            })?;
            let quantity = Decimal::from_str(size_str).map_err(|err| {
                ExchangeError::Network(format!("invalid size in snapshot: {err}"))
            })?;

            emit_coinbase_order_event(
                sender,
                product_id,
                OrderSide::Sell,
                price,
                quantity,
                timestamp,
            );
        }
    }

    Ok(())
}

fn emit_coinbase_order_event(
    sender: &mpsc::UnboundedSender<StreamMessage>,
    product_id: &str,
    side: OrderSide,
    price: Decimal,
    quantity: Decimal,
    timestamp: chrono::DateTime<chrono::Utc>,
) {
    let side_tag = match side {
        OrderSide::Buy => "bid",
        OrderSide::Sell => "ask",
    };
    let order_id = format!(
        "{}-{}-{}",
        product_id.replace(['-', '_'], ""),
        side_tag,
        timestamp.timestamp_nanos_opt().unwrap_or(0)
    );

    let fill = Fill {
        id: format!("{}-fill", order_id),
        order_id: order_id.clone(),
        price,
        quantity,
        fee: Decimal::ZERO,
        timestamp,
    };

    let order = ExchangeOrder {
        id: order_id,
        exchange_id: ExchangeId::Coinbase,
        symbol: product_id.to_string(),
        side,
        order_type: OrderType::Limit,
        quantity,
        price: Some(price),
        status: OrderStatus::Open,
        timestamp,
        fills: vec![fill],
    };

    let _ = sender.send(StreamMessage::OrderUpdate(order));
}

fn parse_decimal_opt(value: Option<&serde_json::Value>) -> Result<Decimal, ExchangeError> {
    let Some(raw) = value.and_then(|v| v.as_str()) else {
        return Ok(Decimal::ZERO);
    };
    Decimal::from_str(raw)
        .map_err(|err| ExchangeError::Network(format!("invalid decimal value '{raw}': {err}")))
}

fn parse_timestamp(value: Option<&serde_json::Value>) -> chrono::DateTime<chrono::Utc> {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now)
}

fn websocket_backoff(attempt: u32) -> Duration {
    let millis = (400.0 * 1.6_f64.powi(attempt.min(8) as i32)).min(10_000.0);
    Duration::from_millis(millis as u64)
}

// Coinbase API response structures
#[derive(Debug, Deserialize)]
struct CoinbaseErrorResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
struct CoinbaseProduct {
    id: String,
    base_currency: String,
    quote_currency: String,
    status: String,
    trading_disabled: bool,
}

#[derive(Debug, Deserialize)]
struct CoinbaseAccount {
    id: String,
    currency: String,
    balance: String,
    available: String,
    hold: String,
}

#[derive(Debug, Default, Serialize)]
struct CoinbaseOrderRequest {
    product_id: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    size: Option<String>,
    price: Option<String>,
    funds: Option<String>,
    time_in_force: Option<String>,
    cancel_after: Option<String>,
    post_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CoinbaseOrder {
    id: String,
    product_id: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    status: String,
    size: String,
    price: Option<String>,
    filled_size: String,
    executed_value: String,
    created_at: String,
    fill_fees: String,
}

#[derive(Debug, Deserialize)]
struct CoinbaseTicker {
    price: String,
    bid: String,
    ask: String,
    volume: String,
}

/// Convert Coinbase order to our ExchangeOrder format
fn convert_coinbase_order(coinbase_order: CoinbaseOrder) -> ExchangeOrder {
    let side = match coinbase_order.side.as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };

    let order_type = match coinbase_order.order_type.as_str() {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit,
        "stop" => OrderType::Stop,
        "stop_limit" => OrderType::StopLimit,
        _ => OrderType::Market,
    };

    let status = match coinbase_order.status.as_str() {
        "pending" => OrderStatus::Pending,
        "open" => OrderStatus::Open,
        "active" => OrderStatus::Open,
        "done" => {
            let filled_size: Decimal = coinbase_order.filled_size.parse().unwrap_or_default();
            let size: Decimal = coinbase_order.size.parse().unwrap_or_default();
            if filled_size >= size {
                OrderStatus::Filled
            } else if filled_size > Decimal::ZERO {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Cancelled
            }
        }
        "cancelled" => OrderStatus::Cancelled,
        "rejected" => OrderStatus::Rejected,
        _ => OrderStatus::Pending,
    };

    let timestamp = chrono::DateTime::parse_from_rfc3339(&coinbase_order.created_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    ExchangeOrder {
        id: coinbase_order.id,
        exchange_id: ExchangeId::Coinbase,
        symbol: coinbase_order.product_id,
        side,
        order_type,
        quantity: coinbase_order.size.parse().unwrap_or_default(),
        price: coinbase_order.price.as_ref().and_then(|p| p.parse().ok()),
        status,
        timestamp,
        fills: vec![], // Would need separate API call to get fills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_connector_creation() {
        let config = CoinbaseConfig {
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            passphrase: "test_passphrase".to_string(),
            sandbox: true,
            use_advanced_trade: false,
        };

        let connector = CoinbaseConnector::new(config);
        assert_eq!(connector.exchange_id(), ExchangeId::Coinbase);
        assert!(!connector.connected);
    }

    #[test]
    fn test_convert_coinbase_order() {
        let coinbase_order = CoinbaseOrder {
            id: "test-order-id".to_string(),
            product_id: "BTC-USD".to_string(),
            side: "buy".to_string(),
            order_type: "limit".to_string(),
            status: "open".to_string(),
            size: "1.0".to_string(),
            price: Some("50000.00".to_string()),
            filled_size: "0.0".to_string(),
            executed_value: "0.0".to_string(),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            fill_fees: "0.0".to_string(),
        };

        let exchange_order = convert_coinbase_order(coinbase_order);

        assert_eq!(exchange_order.id, "test-order-id");
        assert_eq!(exchange_order.symbol, "BTC-USD");
        assert_eq!(exchange_order.side, OrderSide::Buy);
        assert_eq!(exchange_order.order_type, OrderType::Limit);
        assert_eq!(exchange_order.status, OrderStatus::Open);
        assert_eq!(exchange_order.quantity, Decimal::new(1, 0));
        assert_eq!(exchange_order.price, Some(Decimal::new(50000, 0)));
    }
}
