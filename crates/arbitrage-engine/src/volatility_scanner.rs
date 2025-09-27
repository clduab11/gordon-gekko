//! Volatility Scanner - Identifies High-Volatility Trading Opportunities
//!
//! This module implements aggressive volatility scanning across multiple exchanges
//! to identify the most volatile assets for arbitrage opportunities.

use crate::{ArbitrageError, ArbitrageResult, VolatilityScore};
use exchange_connectors::{ExchangeConnector, ExchangeId, MarketTick};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Time windows for volatility calculation
const VOLATILITY_WINDOWS: &[u64] = &[60, 300, 900]; // 1min, 5min, 15min in seconds

/// Volatility scanner that monitors all exchanges for high-volatility instruments
pub struct VolatilityScanner {
    exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>,
    historical_prices: Arc<RwLock<HashMap<String, PriceHistory>>>,
    volatility_scores: Arc<RwLock<HashMap<String, VolatilityScore>>>,
    trading_pairs: Arc<RwLock<HashMap<ExchangeId, Vec<String>>>>,
}

/// Price history tracking for volatility calculations
#[derive(Debug, Clone)]
struct PriceHistory {
    prices: Vec<PricePoint>,
    volumes: Vec<VolumePoint>,
    max_history_size: usize,
}

#[derive(Debug, Clone)]
struct PricePoint {
    price: Decimal,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct VolumePoint {
    volume: Decimal,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl VolatilityScanner {
    /// Create a new volatility scanner
    pub fn new(exchanges: HashMap<ExchangeId, Arc<dyn ExchangeConnector>>) -> Self {
        Self {
            exchanges,
            historical_prices: Arc::new(RwLock::new(HashMap::new())),
            volatility_scores: Arc::new(RwLock::new(HashMap::new())),
            trading_pairs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize scanner by fetching trading pairs from all exchanges
    pub async fn initialize(&self) -> ArbitrageResult<()> {
        info!("🔍 Initializing volatility scanner across {} exchanges", self.exchanges.len());

        let mut trading_pairs = self.trading_pairs.write().await;
        
        for (exchange_id, connector) in &self.exchanges {
            match connector.get_trading_pairs().await {
                Ok(pairs) => {
                    let symbols: Vec<String> = pairs.into_iter().map(|p| p.symbol).collect();
                    info!("📊 Loaded {} trading pairs from {:?}", symbols.len(), exchange_id);
                    trading_pairs.insert(*exchange_id, symbols);
                }
                Err(e) => {
                    warn!("Failed to fetch trading pairs from {:?}: {}", exchange_id, e);
                    return Err(ArbitrageError::Exchange(format!(
                        "Failed to initialize exchange {:?}: {}", exchange_id, e
                    )));
                }
            }
        }

        info!("✅ Volatility scanner initialized successfully");
        Ok(())
    }

    /// Scan volatility across all exchanges and instruments
    pub async fn scan_volatility(&self) -> ArbitrageResult<Vec<VolatilityScore>> {
        debug!("🎯 Starting volatility scan across all exchanges");

        let mut all_scores = Vec::new();
        let trading_pairs = self.trading_pairs.read().await;

        for (exchange_id, symbols) in trading_pairs.iter() {
            if let Some(connector) = self.exchanges.get(exchange_id) {
                for symbol in symbols {
                    match self.calculate_volatility_score(exchange_id, symbol, connector).await {
                        Ok(score) => {
                            all_scores.push(score.clone());
                            
                            // Update internal volatility scores
                            let mut scores = self.volatility_scores.write().await;
                            let key = format!("{:?}:{}", exchange_id, symbol);
                            scores.insert(key, score);
                        }
                        Err(e) => {
                            debug!("Failed to calculate volatility for {}:{:?}: {}", symbol, exchange_id, e);
                        }
                    }
                }
            }
        }

        // Sort by volatility score descending (most volatile first)
        all_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        info!("📈 Volatility scan complete: {} instruments analyzed, top score: {:.2}", 
              all_scores.len(), 
              all_scores.first().map(|s| s.score).unwrap_or(0.0));

        Ok(all_scores)
    }

    /// Get top volatile instruments for targeting
    pub async fn get_top_volatile_instruments(&self, limit: usize) -> Vec<VolatilityScore> {
        let scores = self.volatility_scores.read().await;
        let mut all_scores: Vec<VolatilityScore> = scores.values().cloned().collect();
        
        // Sort by score descending
        all_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        all_scores.into_iter().take(limit).collect()
    }

    /// Calculate volatility score for a specific instrument
    async fn calculate_volatility_score(
        &self,
        exchange_id: &ExchangeId,
        symbol: &str,
        connector: &Arc<dyn ExchangeConnector>,
    ) -> ArbitrageResult<VolatilityScore> {
        // Get current market data
        let market_data = connector.get_market_data(symbol).await
            .map_err(|e| ArbitrageError::Exchange(e.to_string()))?;

        // Update price history
        self.update_price_history(exchange_id, symbol, &market_data).await;

        // Calculate volatility components
        let price_changes = self.calculate_price_changes(exchange_id, symbol).await;
        let volume_surge = self.calculate_volume_surge(exchange_id, symbol).await;
        let spread_tightness = self.calculate_spread_tightness(&market_data);
        let momentum = self.calculate_momentum(exchange_id, symbol).await;

        // Combine components into final volatility score
        let volatility_score = self.combine_volatility_factors(
            &price_changes,
            volume_surge,
            spread_tightness,
            momentum,
        );

        Ok(VolatilityScore {
            symbol: symbol.to_string(),
            exchange: *exchange_id,
            score: volatility_score,
            price_change_1m: price_changes.get(&60).cloned().unwrap_or_default(),
            price_change_5m: price_changes.get(&300).cloned().unwrap_or_default(),
            price_change_15m: price_changes.get(&900).cloned().unwrap_or_default(),
            volume_surge_factor: volume_surge,
            spread_tightness,
            momentum_indicator: momentum,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Update price history for an instrument
    async fn update_price_history(&self, exchange_id: &ExchangeId, symbol: &str, market_data: &MarketTick) {
        let mut history = self.historical_prices.write().await;
        let key = format!("{:?}:{}", exchange_id, symbol);
        
        let price_history = history.entry(key).or_insert_with(|| PriceHistory {
            prices: Vec::new(),
            volumes: Vec::new(),
            max_history_size: 1000, // Keep last 1000 data points
        });

        // Add new price point
        price_history.prices.push(PricePoint {
            price: market_data.last,
            timestamp: market_data.timestamp,
        });

        // Add new volume point
        price_history.volumes.push(VolumePoint {
            volume: market_data.volume_24h,
            timestamp: market_data.timestamp,
        });

        // Trim history if too large
        if price_history.prices.len() > price_history.max_history_size {
            price_history.prices.remove(0);
        }
        if price_history.volumes.len() > price_history.max_history_size {
            price_history.volumes.remove(0);
        }
    }

    /// Calculate price changes over different time windows
    async fn calculate_price_changes(&self, exchange_id: &ExchangeId, symbol: &str) -> HashMap<u64, Decimal> {
        let history = self.historical_prices.read().await;
        let key = format!("{:?}:{}", exchange_id, symbol);
        
        let mut changes = HashMap::new();
        
        if let Some(price_history) = history.get(&key) {
            if price_history.prices.len() < 2 {
                return changes;
            }

            let current_price = price_history.prices.last().unwrap().price;
            let now = chrono::Utc::now();

            for &window_seconds in VOLATILITY_WINDOWS {
                let window_start = now - chrono::Duration::seconds(window_seconds as i64);
                
                // Find the price closest to window_start
                if let Some(historical_price) = price_history.prices.iter()
                    .filter(|p| p.timestamp >= window_start)
                    .min_by_key(|p| (p.timestamp - window_start).num_seconds().abs()) {
                    
                    let price_change = current_price - historical_price.price;
                    changes.insert(window_seconds, price_change);
                }
            }
        }
        
        changes
    }

    /// Calculate volume surge factor
    async fn calculate_volume_surge(&self, exchange_id: &ExchangeId, symbol: &str) -> f64 {
        let history = self.historical_prices.read().await;
        let key = format!("{:?}:{}", exchange_id, symbol);
        
        if let Some(price_history) = history.get(&key) {
            if price_history.volumes.len() < 10 {
                return 1.0; // No surge if insufficient data
            }

            let recent_volume = price_history.volumes.last().unwrap().volume;
            let average_volume: Decimal = price_history.volumes.iter()
                .rev()
                .take(20) // Last 20 data points
                .map(|v| v.volume)
                .sum::<Decimal>() / Decimal::new(20, 0);

            if average_volume > Decimal::ZERO {
                let surge_factor = recent_volume / average_volume;
                surge_factor.to_string().parse().unwrap_or(1.0)
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    /// Calculate spread tightness (tighter spreads = higher score)
    fn calculate_spread_tightness(&self, market_data: &MarketTick) -> f64 {
        let spread = market_data.ask - market_data.bid;
        let mid_price = (market_data.ask + market_data.bid) / Decimal::new(2, 0);
        
        if mid_price > Decimal::ZERO {
            let spread_percentage: f64 = (spread / mid_price).to_string().parse().unwrap_or(1.0);
            // Invert so tighter spreads get higher scores (max 1.0)
            (1.0 - spread_percentage.min(1.0)).max(0.0)
        } else {
            0.0
        }
    }

    /// Calculate price momentum indicator
    async fn calculate_momentum(&self, exchange_id: &ExchangeId, symbol: &str) -> f64 {
        let history = self.historical_prices.read().await;
        let key = format!("{:?}:{}", exchange_id, symbol);
        
        if let Some(price_history) = history.get(&key) {
            if price_history.prices.len() < 20 {
                return 0.5; // Neutral momentum if insufficient data
            }

            // Calculate simple momentum as price direction consistency
            let recent_prices: Vec<Decimal> = price_history.prices.iter()
                .rev()
                .take(20)
                .map(|p| p.price)
                .collect();

            let mut up_moves = 0;
            let mut total_moves = 0;

            for i in 1..recent_prices.len() {
                if recent_prices[i-1] != recent_prices[i] {
                    if recent_prices[i] > recent_prices[i-1] {
                        up_moves += 1;
                    }
                    total_moves += 1;
                }
            }

            if total_moves > 0 {
                up_moves as f64 / total_moves as f64
            } else {
                0.5
            }
        } else {
            0.5
        }
    }

    /// Combine volatility factors into final score
    fn combine_volatility_factors(
        &self,
        price_changes: &HashMap<u64, Decimal>,
        volume_surge: f64,
        spread_tightness: f64,
        momentum: f64,
    ) -> f64 {
        // Weight different factors for Gordon Gekko style aggressive scoring
        let price_volatility_weight = 0.4;
        let volume_surge_weight = 0.3;
        let spread_weight = 0.2;
        let momentum_weight = 0.1;

        // Calculate price volatility score from price changes
        let mut price_volatility = 0.0;
        if !price_changes.is_empty() {
            let total_change: f64 = price_changes.values()
                .map(|change| change.abs().to_string().parse().unwrap_or(0.0))
                .sum();
            price_volatility = (total_change / price_changes.len() as f64).min(1.0);
        }

        // Normalize volume surge (cap at 5x)
        let normalized_volume_surge = ((volume_surge - 1.0) / 4.0).min(1.0).max(0.0);

        // Momentum contribution (deviation from 0.5 indicates strong direction)
        let momentum_contribution = (momentum - 0.5).abs() * 2.0;

        // Final weighted score
        let final_score = price_volatility * price_volatility_weight
            + normalized_volume_surge * volume_surge_weight
            + spread_tightness * spread_weight
            + momentum_contribution * momentum_weight;

        final_score.min(1.0).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exchange_connectors::MarketTick;

    #[test]
    fn test_spread_tightness_calculation() {
        let scanner = VolatilityScanner::new(HashMap::new());
        
        let market_data = MarketTick {
            symbol: "BTC-USD".to_string(),
            bid: Decimal::new(49950, 0), // $499.50
            ask: Decimal::new(50050, 0), // $500.50
            last: Decimal::new(50000, 0),
            volume_24h: Decimal::new(100000, 0),
            timestamp: chrono::Utc::now(),
        };

        let tightness = scanner.calculate_spread_tightness(&market_data);
        assert!(tightness > 0.0 && tightness <= 1.0);
    }

    #[test]
    fn test_volatility_factors_combination() {
        let scanner = VolatilityScanner::new(HashMap::new());
        
        let mut price_changes = HashMap::new();
        price_changes.insert(60, Decimal::new(100, 0)); // $1.00 change in 1 minute
        
        let score = scanner.combine_volatility_factors(
            &price_changes,
            2.5, // 2.5x volume surge
            0.8, // 80% spread tightness
            0.7, // 70% upward momentum
        );

        assert!(score >= 0.0 && score <= 1.0);
    }
}