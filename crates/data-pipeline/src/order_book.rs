use ahash::AHashMap;
use event_bus::{MarketPayload, OrderBookLevel};
use exchange_connectors::{OrderSide, TradingPair};
use rust_decimal::Decimal;

/// Represents a single side of the order book (bids or asks).
#[derive(Debug, Default, Clone)]
pub struct OrderBookSide(AHashMap<Decimal, Decimal>);

impl OrderBookSide {
    fn apply_level(&mut self, price: Decimal, quantity: Decimal) {
        if quantity.is_zero() {
            self.0.remove(&price);
        } else {
            self.0.insert(price, quantity);
        }
    }

    pub fn best(&self, descending: bool) -> Option<(Decimal, Decimal)> {
        self.0
            .iter()
            .max_by(|(lp, _), (rp, _)| {
                if descending {
                    lp.partial_cmp(rp).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    rp.partial_cmp(lp).unwrap_or(std::cmp::Ordering::Equal)
                }
            })
            .map(|(price, qty)| (*price, *qty))
    }
}

/// Level 2 order book maintenance with delta compression.
#[derive(Debug, Default, Clone)]
pub struct LevelTwoBook {
    instrument: Option<TradingPair>,
    bids: OrderBookSide,
    asks: OrderBookSide,
    depth: usize,
}

impl LevelTwoBook {
    pub fn instrument(&self) -> Option<TradingPair> {
        self.instrument.clone()
    }

    pub fn apply(&mut self, update: OrderBookUpdate) -> MarketPayload {
        self.instrument = Some(update.pair.clone());
        self.depth = self.depth.max(update.depth_hint);

        match update.side {
            OrderSide::Buy => self.bids.apply_level(update.price, update.quantity),
            OrderSide::Sell => self.asks.apply_level(update.price, update.quantity),
        }

        MarketPayload::OrderBookDelta {
            pair: update.pair,
            bid_updates: if matches!(update.side, OrderSide::Buy) {
                vec![order_level(update.price, update.quantity)]
            } else {
                Vec::new()
            },
            ask_updates: if matches!(update.side, OrderSide::Sell) {
                vec![order_level(update.price, update.quantity)]
            } else {
                Vec::new()
            },
            sequence: update.sequence,
        }
    }
}

fn order_level(price: Decimal, quantity: Decimal) -> OrderBookLevel {
    OrderBookLevel {
        price,
        size: quantity,
    }
}

/// Domain-specific order book update derived from exchange order events.
#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub pair: TradingPair,
    pub side: OrderSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub sequence: u64,
    pub depth_hint: usize,
}

impl OrderBookUpdate {
    pub fn new(
        pair: TradingPair,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        sequence: u64,
    ) -> Self {
        Self {
            pair,
            side,
            price,
            quantity,
            sequence,
            depth_hint: 64,
        }
    }
}
