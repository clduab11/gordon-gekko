# Gordon Gekko Domain Model Specification

## Overview

This document defines the core domain entities, relationships, and business rules for the Gordon Gekko autonomous trading system converted to Rust with neural network integration.

## Core Domain Entities

### 1. Trading System
**Primary Entity**: Central orchestrator managing the entire trading ecosystem

**Attributes**:
- `system_id`: Unique identifier (UUID)
- `status`: System operational status (Initializing, Running, Stopping, Error, Maintenance)
- `health_score`: Overall system health metric (0.0-100.0)
- `version`: System version string
- `uptime_seconds`: Total operational time
- `last_health_check`: Timestamp of last health verification
- `active_connections`: Count of external system connections
- `error_count`: Total error occurrences since startup
- `performance_metrics`: Real-time performance indicators

**Validation Rules**:
- `health_score` must be between 0.0 and 100.0
- `status` must be valid enum value
- `version` must follow semantic versioning pattern
- `error_count` cannot decrease (audit trail requirement)

**Invariants**:
- System cannot have negative uptime
- Health score reflects actual service availability
- Error count only increases during operation

### 2. Trading Account
**Primary Entity**: Financial account representation across trading platforms

**Attributes**:
- `account_id`: Unique platform-specific identifier
- `platform`: Trading platform enum (Coinbase, BinanceUS, OANDA, etc.)
- `account_type`: Account classification (Live, Paper, Sandbox)
- `base_currency`: Primary account currency
- `available_balance`: Funds available for trading
- `reserved_balance`: Funds held in open orders
- `total_balance`: Sum of available and reserved balances
- `leverage_ratio`: Account leverage multiplier
- `margin_utilization`: Current margin usage percentage
- `status`: Account operational status (Active, Suspended, Maintenance)
- `last_sync_timestamp`: Last successful balance synchronization

**Validation Rules**:
- `available_balance` >= 0
- `reserved_balance` >= 0
- `total_balance` = `available_balance` + `reserved_balance`
- `margin_utilization` <= 100.0
- `leverage_ratio` > 0
- `base_currency` must be valid ISO currency code

**Business Rules**:
- Cannot place orders exceeding available balance
- Margin utilization alerts at configurable thresholds
- Automatic position liquidation at critical margin levels

### 3. Trading Position
**Primary Entity**: Open position tracking across all assets

**Attributes**:
- `position_id`: Unique position identifier
- `account_id`: Reference to trading account
- `symbol`: Asset symbol (BTC/USD, AAPL, etc.)
- `position_type`: Long or Short
- `entry_price`: Average price of position entry
- `current_price`: Latest market price
- `quantity`: Total position size
- `unrealized_pnl`: Current profit/loss amount
- `unrealized_pnl_percentage`: Current profit/loss percentage
- `entry_timestamp`: Position opening time
- `last_update_timestamp`: Last price update time
- `stop_loss_price`: Configured stop loss level
- `take_profit_price`: Configured take profit level
- `status`: Position status (Open, PartiallyClosed, Closed)

**Validation Rules**:
- `quantity` > 0 for open positions
- `entry_price` > 0
- `current_price` > 0
- `unrealized_pnl_percentage` calculation must be accurate
- Stop loss and take profit prices must be valid for position type

**Business Rules**:
- Automatic stop loss execution at configured levels
- Take profit execution at target levels
- Position sizing based on account risk parameters

### 4. Market Data
**Primary Entity**: Real-time and historical market information

**Attributes**:
- `symbol`: Asset symbol
- `exchange`: Data source exchange
- `timestamp`: Data point timestamp
- `price_open`: Opening price for period
- `price_high`: Highest price for period
- `price_low`: Lowest price for period
- `price_close`: Closing price for period
- `volume`: Trading volume
- `data_type`: Data granularity (Tick, 1m, 5m, 1h, 1d)
- `source_reliability`: Data source confidence score

**Validation Rules**:
- Price fields must be positive and logically consistent
- `price_high` >= `price_open` and `price_high` >= `price_close`
- `price_low` <= `price_open` and `price_low` <= `price_close`
- Volume >= 0
- Timestamp chronological ordering

**Quality Rules**:
- Data staleness detection
- Price gap validation
- Volume spike detection
- Source reliability scoring

### 5. Neural Network Model
**Primary Entity**: Machine learning model for trading predictions

**Attributes**:
- `model_id`: Unique model identifier
- `model_type`: Neural network architecture (NHITS, NBEATSx, LSTM, Transformer)
- `version`: Model version string
- `parameters`: Model configuration parameters
- `training_data_start`: Start date of training data
- `training_data_end`: End date of training data
- `features`: Input feature set
- `target_variable`: Prediction target (price, volatility, volume)
- `performance_metrics`: Model accuracy and performance indicators
- `last_training_timestamp`: Last model training completion
- `prediction_horizon`: Forecast time horizon
- `confidence_threshold`: Minimum confidence for predictions
- `status`: Model operational status

**Validation Rules**:
- `performance_metrics` must include accuracy, precision, recall
- `prediction_horizon` > 0
- `confidence_threshold` between 0.0 and 1.0
- Model parameters must be valid for architecture type

**Performance Rules**:
- Minimum accuracy thresholds for deployment
- Regular retraining schedules
- Performance degradation detection

### 6. Trading Strategy
**Primary Entity**: Algorithmic trading strategy configuration

**Attributes**:
- `strategy_id`: Unique strategy identifier
- `name`: Human-readable strategy name
- `description`: Strategy description
- `parameters`: Strategy configuration parameters
- `risk_parameters`: Risk management settings
- `entry_conditions`: Market entry criteria
- `exit_conditions`: Market exit criteria
- `position_sizing_rules`: Position size calculation rules
- `timeframe`: Strategy execution timeframe
- `enabled`: Strategy activation status
- `performance_history`: Historical performance data

**Validation Rules**:
- `risk_parameters` must include max loss, max drawdown
- `entry_conditions` must be logically valid
- `exit_conditions` must include stop loss criteria
- `position_sizing_rules` must prevent excessive exposure

**Business Rules**:
- Strategy backtesting before deployment
- Risk parameter validation against account limits
- Performance monitoring and automatic disabling

### 7. Order
**Primary Entity**: Trading order representation

**Attributes**:
- `order_id`: Unique order identifier
- `account_id`: Reference to trading account
- `symbol`: Asset symbol
- `order_type`: Market, Limit, Stop, StopLimit
- `side`: Buy or Sell
- `quantity`: Order quantity
- `price`: Order price (None for market orders)
- `status`: Order status (Pending, Filled, PartiallyFilled, Cancelled, Rejected)
- `timestamp`: Order creation time
- `filled_quantity`: Quantity already executed
- `average_fill_price`: Average execution price
- `fees`: Total fees paid
- `time_in_force`: Order duration (GTC, IOC, FOK)

**Validation Rules**:
- `quantity` > 0
- `price` > 0 for limit orders
- Status transitions must follow valid order lifecycle
- Filled quantity <= order quantity

**Execution Rules**:
- Price limit validation for limit orders
- Immediate execution for market orders
- Partial fill handling
- Fee calculation accuracy

### 8. Risk Management Rule
**Primary Entity**: Risk control and monitoring rules

**Attributes**:
- `rule_id`: Unique rule identifier
- `rule_type`: Risk type (Exposure, Drawdown, Volatility, Concentration)
- `threshold`: Risk threshold value
- `operator`: Comparison operator (>, <, >=, <=, ==)
- `action`: Action to take (Alert, Reduce, Stop, EmergencyClose)
- `cooldown_period`: Minimum time between rule triggers
- `enabled`: Rule activation status
- `last_triggered`: Last rule activation timestamp

**Validation Rules**:
- `threshold` must be appropriate for rule type
- `cooldown_period` >= 0
- `action` must be valid for rule type
- Rule logic must be consistent

**Trigger Rules**:
- Real-time risk monitoring
- Immediate action execution
- Cooldown period enforcement
- Audit trail maintenance

## Entity Relationships

### 1. Trading System → Trading Account (1:N)
- One system manages multiple trading accounts
- System aggregates account-level risk metrics
- Account failures can affect system health

### 2. Trading Account → Trading Position (1:N)
- One account contains multiple positions
- Account balance reflects position P&L
- Position liquidation affects account balance

### 3. Trading Account → Order (1:N)
- One account generates multiple orders
- Order execution affects account balance
- Failed orders impact account status

### 4. Market Data → Neural Network Model (N:1)
- Multiple data streams feed one model
- Model predictions based on data patterns
- Data quality affects model performance

### 5. Neural Network Model → Trading Strategy (N:M)
- Multiple models can inform multiple strategies
- Strategy performance depends on model accuracy
- Model updates require strategy revalidation

### 6. Trading Strategy → Order (1:N)
- One strategy generates multiple orders
- Strategy parameters determine order characteristics
- Order execution validates strategy effectiveness

### 7. Risk Management Rule → Trading Account (N:M)
- Multiple rules apply to multiple accounts
- Rule violations trigger account actions
- Account risk profile determines applicable rules

## Data Structures and Types

### 1. Price and Quantity Types
```rust
// High-precision decimal types for financial calculations
type Price = Decimal;        // 8 decimal places precision
type Quantity = Decimal;     // 8 decimal places precision
type Balance = Decimal;      // 8 decimal places precision
type Percentage = f64;       // 0.0 to 100.0 range
type Timestamp = i64;        // Unix timestamp in nanoseconds
```

### 2. System Status Enum
```rust
enum SystemStatus {
    Initializing,
    Running,
    Stopping,
    Error,
    Maintenance,
    Degraded
}
```

### 3. Order Status Enum
```rust
enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Expired
}
```

### 4. Position Type Enum
```rust
enum PositionType {
    Long,
    Short
}
```

### 5. Timeframe Enum
```rust
enum Timeframe {
    Tick,
    M1, M5, M15, M30,
    H1, H4, H8, H12,
    D1, W1, MN1
}
```

## State Transitions

### 1. Trading System Lifecycle
```
Initializing → Running (successful startup)
Initializing → Error (startup failure)
Running → Stopping (graceful shutdown)
Running → Error (runtime failure)
Running → Maintenance (scheduled maintenance)
Stopping → Stopped (successful shutdown)
Error → Running (recovery successful)
Maintenance → Running (maintenance complete)
```

### 2. Order Execution Lifecycle
```
Pending → Filled (immediate execution)
Pending → PartiallyFilled → Filled (partial execution)
Pending → Cancelled (user cancellation)
Pending → Rejected (validation failure)
Pending → Expired (time expiration)
```

### 3. Position Management Lifecycle
```
Open → PartiallyClosed (partial position close)
Open → Closed (full position close)
PartiallyClosed → Closed (remaining position close)
```

### 4. Neural Model Lifecycle
```
Training → Validating (cross-validation)
Validating → Deployed (performance acceptable)
Deployed → Retraining (performance degradation)
Deployed → Retired (model obsolescence)
```

## Business Process Flows

### 1. Market Data Processing
```
Data Reception → Quality Validation → Normalization →
Feature Engineering → Model Input → Prediction Generation →
Strategy Evaluation → Signal Generation → Order Creation
```

### 2. Risk Management Process
```
Position Monitoring → Risk Rule Evaluation → Threshold Check →
Violation Detection → Action Trigger → Position Adjustment →
Balance Reconciliation → Performance Update
```

### 3. Trade Execution Process
```
Strategy Signal → Order Generation → Pre-trade Validation →
Platform Submission → Execution Confirmation → Position Update →
Risk Recalculation → Performance Tracking
```

### 4. Neural Network Retraining
```
Performance Monitoring → Degradation Detection → Data Collection →
Model Retraining → Validation Testing → Deployment Update →
Performance Verification → Rollback (if needed)
```

## Validation Rules

### 1. Pre-trade Validation
- Account has sufficient balance
- Order size meets minimum requirements
- Price limits are within market bounds
- Risk parameters are not violated
- Market is open for trading

### 2. Post-trade Validation
- Execution price is within slippage tolerance
- Fees are correctly calculated
- Position is accurately updated
- Balance reconciliation is correct
- Risk metrics are recalculated

### 3. System Health Validation
- All critical services are operational
- Database connections are healthy
- Cache performance is acceptable
- External API connectivity is stable
- Memory and CPU usage within limits

## Aggregate Boundaries

### 1. Trading Session Aggregate
- Groups all activities within a trading session
- Maintains session-level statistics
- Handles session start/end logic
- Manages session data persistence

### 2. Account Aggregate
- Manages all positions for an account
- Calculates account-level risk metrics
- Handles balance and margin calculations
- Enforces account-specific rules

### 3. Strategy Aggregate
- Groups related orders by strategy
- Calculates strategy performance
- Manages strategy lifecycle
- Handles strategy-specific configurations

### 4. Market Data Aggregate
- Groups data points by symbol and timeframe
- Maintains data consistency
- Handles data quality validation
- Manages historical data retention

## Event Flows

### 1. Market Data Events
- PriceUpdate (real-time price changes)
- VolumeSpike (unusual volume activity)
- GapDetected (price gaps identified)
- DataStale (data staleness detected)

### 2. Trading Events
- OrderCreated (new order generated)
- OrderFilled (order execution confirmed)
- PositionOpened (new position established)
- PositionClosed (position liquidation)

### 3. Risk Events
- RiskThresholdExceeded (risk limit violation)
- MarginCall (margin requirement not met)
- PositionLimitReached (maximum positions hit)
- DrawdownLimitHit (maximum drawdown exceeded)

### 4. System Events
- ServiceStarted (component startup)
- ServiceStopped (component shutdown)
- HealthCheckFailed (health validation failure)
- ConfigurationUpdated (settings changed)

## Query Models

### 1. Position Queries
- GetPositionsByAccount(account_id)
- GetPositionsBySymbol(symbol)
- GetOpenPositions()
- GetPositionsByStrategy(strategy_id)

### 2. Performance Queries
- GetAccountPerformance(account_id, timeframe)
- GetStrategyPerformance(strategy_id, timeframe)
- GetSystemPerformance(timeframe)
- GetModelPerformance(model_id, timeframe)

### 3. Risk Queries
- GetRiskMetrics(account_id)
- GetMarginUtilization(account_id)
- GetExposureByAsset(account_id)
- GetRiskViolations(account_id, timeframe)

### 4. Market Data Queries
- GetPriceHistory(symbol, timeframe, start_date, end_date)
- GetMarketDepth(symbol, depth)
- GetVolumeProfile(symbol, timeframe)
- GetVolatilityMetrics(symbol, timeframe)

## Domain Events

### 1. Trading Domain Events
- TradeExecuted (trade completion)
- PositionValueChanged (P&L update)
- AccountBalanceUpdated (balance change)
- StrategySignalGenerated (trading signal)

### 2. Risk Domain Events
- RiskLimitBreached (risk threshold exceeded)
- MarginCallIssued (margin requirement)
- PositionForceClosed (forced liquidation)
- AccountSuspended (account frozen)

### 3. Market Domain Events
- PriceBreakthrough (key level breach)
- VolumeAnomaly (unusual volume)
- TrendReversal (trend change detected)
- LiquidityChange (market depth change)

### 4. System Domain Events
- ServiceHealthChanged (service status change)
- ConfigurationChanged (settings updated)
- ModelRetrained (model update)
- DataQualityIssue (data problem detected)

## Glossary

- **NHITS**: Neural Hierarchical Interpolation for Time Series
- **NBEATSx**: Neural Basis Expansion Analysis with Exogenous variables
- **P&L**: Profit and Loss
- **Slippage**: Difference between expected and actual execution price
- **Drawdown**: Peak-to-trough decline in account value
- **Margin Call**: Demand for additional funds to cover losses
- **Time in Force**: Order execution duration constraints
- **Market Depth**: Order book liquidity at different price levels
- **Volatility**: Price fluctuation measure
- **Backtesting**: Strategy performance evaluation on historical data

This domain model provides a comprehensive foundation for the Rust conversion, ensuring all business logic, relationships, and constraints are properly captured and validated.