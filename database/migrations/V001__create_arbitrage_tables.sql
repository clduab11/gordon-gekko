-- V001: Create arbitrage tables for Gordon Gekko trading system
-- This migration creates all necessary tables for arbitrage operations,
-- volatility tracking, fund transfers, and execution history.

-- Arbitrage opportunities table
CREATE TABLE IF NOT EXISTS arbitrage_opportunities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    buy_exchange VARCHAR(20) NOT NULL,
    sell_exchange VARCHAR(20) NOT NULL,
    buy_price DECIMAL(20, 8) NOT NULL,
    sell_price DECIMAL(20, 8) NOT NULL,
    price_difference DECIMAL(20, 8) NOT NULL,
    profit_percentage DECIMAL(5, 4) NOT NULL,
    estimated_profit DECIMAL(20, 2) NOT NULL,
    confidence_score DECIMAL(3, 2) NOT NULL CHECK (confidence_score BETWEEN 0 AND 1),
    max_quantity DECIMAL(20, 8) NOT NULL,
    time_sensitivity VARCHAR(10) NOT NULL CHECK (time_sensitivity IN ('Low', 'Medium', 'High', 'Critical')),
    risk_score DECIMAL(3, 2) NOT NULL CHECK (risk_score BETWEEN 0 AND 1),
    execution_complexity VARCHAR(10) NOT NULL CHECK (execution_complexity IN ('Simple', 'Moderate', 'Complex', 'Advanced')),
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'executed', 'expired', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Volatility scores table
CREATE TABLE IF NOT EXISTS volatility_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    exchange VARCHAR(20) NOT NULL,
    score DECIMAL(3, 2) NOT NULL CHECK (score BETWEEN 0 AND 1),
    price_change_1m DECIMAL(20, 8),
    price_change_5m DECIMAL(20, 8),
    price_change_15m DECIMAL(20, 8),
    volume_surge_factor DECIMAL(8, 4) NOT NULL DEFAULT 1.0,
    spread_tightness DECIMAL(3, 2) NOT NULL CHECK (spread_tightness BETWEEN 0 AND 1),
    momentum_indicator DECIMAL(3, 2) NOT NULL CHECK (momentum_indicator BETWEEN 0 AND 1),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cross-exchange transfers table
CREATE TABLE IF NOT EXISTS cross_exchange_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_exchange VARCHAR(20) NOT NULL,
    to_exchange VARCHAR(20) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL,
    urgency VARCHAR(10) NOT NULL CHECK (urgency IN ('Low', 'Normal', 'High', 'Critical')),
    priority VARCHAR(10) NOT NULL CHECK (priority IN ('Low', 'Normal', 'High', 'Critical', 'Emergency')),
    reason TEXT NOT NULL,
    requested_by VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    external_transfer_id VARCHAR(100),
    fees DECIMAL(20, 8) DEFAULT 0,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Arbitrage executions table
CREATE TABLE IF NOT EXISTS arbitrage_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    opportunity_id UUID NOT NULL REFERENCES arbitrage_opportunities(id),
    buy_order_id VARCHAR(100),
    sell_order_id VARCHAR(100),
    buy_exchange VARCHAR(20) NOT NULL,
    sell_exchange VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    buy_price DECIMAL(20, 8) NOT NULL,
    sell_price DECIMAL(20, 8) NOT NULL,
    buy_fee DECIMAL(20, 8) DEFAULT 0,
    sell_fee DECIMAL(20, 8) DEFAULT 0,
    net_profit DECIMAL(20, 2) NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'partial', 'completed', 'failed', 'cancelled')),
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Venue balances table for real-time balance tracking
CREATE TABLE IF NOT EXISTS venue_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange VARCHAR(20) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    available DECIMAL(20, 8) NOT NULL DEFAULT 0,
    total DECIMAL(20, 8) NOT NULL DEFAULT 0,
    reserved DECIMAL(20, 8) NOT NULL DEFAULT 0,
    usd_value DECIMAL(20, 2) NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(exchange, currency)
);

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_arbitrage_opportunities_symbol ON arbitrage_opportunities(symbol);
CREATE INDEX IF NOT EXISTS idx_arbitrage_opportunities_detected_at ON arbitrage_opportunities(detected_at);
CREATE INDEX IF NOT EXISTS idx_arbitrage_opportunities_status ON arbitrage_opportunities(status);
CREATE INDEX IF NOT EXISTS idx_arbitrage_opportunities_expires_at ON arbitrage_opportunities(expires_at);

CREATE INDEX IF NOT EXISTS idx_volatility_scores_symbol ON volatility_scores(symbol);
CREATE INDEX IF NOT EXISTS idx_volatility_scores_exchange ON volatility_scores(exchange);
CREATE INDEX IF NOT EXISTS idx_volatility_scores_timestamp ON volatility_scores(timestamp);
CREATE INDEX IF NOT EXISTS idx_volatility_scores_score ON volatility_scores(score DESC);

CREATE INDEX IF NOT EXISTS idx_cross_exchange_transfers_status ON cross_exchange_transfers(status);
CREATE INDEX IF NOT EXISTS idx_cross_exchange_transfers_requested_at ON cross_exchange_transfers(requested_at);
CREATE INDEX IF NOT EXISTS idx_cross_exchange_transfers_priority ON cross_exchange_transfers(priority);

CREATE INDEX IF NOT EXISTS idx_arbitrage_executions_opportunity_id ON arbitrage_executions(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_arbitrage_executions_started_at ON arbitrage_executions(started_at);
CREATE INDEX IF NOT EXISTS idx_arbitrage_executions_status ON arbitrage_executions(status);

CREATE INDEX IF NOT EXISTS idx_venue_balances_exchange ON venue_balances(exchange);
CREATE INDEX IF NOT EXISTS idx_venue_balances_currency ON venue_balances(currency);
CREATE INDEX IF NOT EXISTS idx_venue_balances_last_updated ON venue_balances(last_updated);

-- Update triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_arbitrage_opportunities_updated_at BEFORE UPDATE ON arbitrage_opportunities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_cross_exchange_transfers_updated_at BEFORE UPDATE ON cross_exchange_transfers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_arbitrage_executions_updated_at BEFORE UPDATE ON arbitrage_executions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();