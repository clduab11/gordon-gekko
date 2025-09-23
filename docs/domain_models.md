# Gordon Gekko Domain Models

## Core Domain Entities

### 1. Trading System
Central orchestration entity that manages the overall trading system lifecycle.

**Attributes:**
- system_id: Unique identifier for the trading system instance
- status: Current operational status (initializing, running, stopped, error)
- version: System version for tracking updates
- config_hash: Hash of current configuration for change detection
- last_health_check: Timestamp of last successful health check
- error_count: Number of consecutive errors encountered
- mcp_connections: List of active MCP server connections

**Relationships:**
- Manages multiple Trading Accounts
- Orchestrates multiple Trading Strategies
- Monitors multiple Risk Profiles
- Integrates with multiple External Services

**Business Rules:**
- System must maintain at least one active MCP connection
- Error count must not exceed threshold (3 consecutive errors)
- Health checks must run every 30 seconds
- Configuration changes require system restart validation

### 2. Trading Account
Represents a trading account on any supported platform (Coinbase, Binance.US, OANDA).

**Attributes:**
- account_id: Unique identifier across all platforms
- platform: Trading platform (coinbase, binance_us, oanda)
- account_type: Type of account (spot, margin, futures)
- status: Account status (active, suspended, closed)
- balance: Current account balance in base currency
- available_balance: Funds available for trading
- reserved_balance: Funds reserved for open orders
- currency: Base currency of the account (USD, EUR, BTC, etc.)

**Relationships:**
- Belongs to one Trading System
- Has multiple Positions
- Executes multiple Orders
- Follows one Risk Profile
- Connected to one Platform Configuration

**Business Rules:**
- Balance must always be non-negative
- Available balance = total balance - reserved balance
- Account must be verified before trading
- Currency must be supported by the platform

### 3. Trading Strategy
Defines the algorithmic trading approach and parameters.

**Attributes:**
- strategy_id: Unique identifier for the strategy
- name: Human-readable strategy name
- type: Strategy type (momentum, mean_reversion, arbitrage, ml_based)
- status: Strategy status (active, paused, disabled)
- parameters: JSON configuration of strategy parameters
- performance_metrics: Current performance statistics
- risk_level: Risk tolerance level (low, medium, high)
- timeframe: Primary analysis timeframe (1m, 5m, 15m, 1h, 4h, 1d)
- entry_conditions: Conditions for entering positions
- exit_conditions: Conditions for exiting positions

**Relationships:**
- Applied to multiple Trading Accounts
- Generates multiple Trading Signals
- Evaluated by Strategy Performance
- Configured with Strategy Parameters

**Business Rules:**
- Must have at least one entry condition
- Must have at least one exit condition
- Risk level must be compatible with account risk profile
- Performance must be monitored continuously

### 4. Position
Represents an open trading position in the market.

**Attributes:**
- position_id: Unique identifier for the position
- account_id: Reference to the trading account
- symbol: Trading symbol (BTC-USD, EUR-USD, etc.)
- side: Position side (long, short)
- quantity: Position size/quantity
- entry_price: Average price at which position was opened
- current_price: Current market price
- unrealized_pnl: Current unrealized profit/loss
- realized_pnl: Realized profit/loss when partially closed
- status: Position status (open, partially_closed, closed)
- open_timestamp: When position was opened
- close_timestamp: When position was closed (if applicable)

**Relationships:**
- Belongs to one Trading Account
- Consists of multiple Fills
- Associated with one Risk Profile
- Generated from one Trading Signal

**Business Rules:**
- Quantity must be positive
- Unrealized PnL = (current_price - entry_price) * quantity * direction
- Position cannot exceed account position limits
- Must track partial closes separately

### 5. Order
Represents a trading order (buy/sell instruction).

**Attributes:**
- order_id: Unique identifier for the order
- account_id: Reference to the trading account
- symbol: Trading symbol
- side: Order side (buy, sell)
- type: Order type (market, limit, stop, stop_limit)
- quantity: Order quantity
- price: Limit/stop price (null for market orders)
- status: Order status (pending, filled, partially_filled, cancelled, rejected)
- filled_quantity: Quantity that has been filled
- remaining_quantity: Quantity remaining to be filled
- average_fill_price: Average price of all fills
- timestamp: When order was created
- fills: List of order fills (executions)

**Relationships:**
- Belongs to one Trading Account
- Can be part of multiple Positions
- Generated from one Trading Signal
- Executed through one Platform Connection

**Business Rules:**
- Quantity must be positive
- Price must be positive for limit/stop orders
- Filled quantity ≤ order quantity
- Order must comply with platform trading rules

### 6. Risk Profile
Defines risk management parameters and limits.

**Attributes:**
- profile_id: Unique identifier for the risk profile
- name: Human-readable profile name
- max_position_size: Maximum position size per trade
- max_portfolio_risk: Maximum risk as percentage of portfolio
- max_drawdown_limit: Maximum allowed drawdown
- stop_loss_trigger: Automatic stop-loss trigger level
- take_profit_trigger: Automatic take-profit trigger level
- max_open_positions: Maximum number of concurrent positions
- correlation_limit: Maximum correlation between positions
- var_limit: Value at Risk limit (95% confidence)
- risk_per_trade: Maximum risk per individual trade

**Relationships:**
- Applied to multiple Trading Accounts
- Enforced on multiple Positions
- Monitored by Risk Monitoring
- Configured by Risk Parameters

**Business Rules:**
- Max position size must be positive
- Risk percentages must be between 0 and 100
- VaR limit must be reasonable for the account size
- Stop loss must be more aggressive than take profit

### 7. Market Data
Represents real-time and historical market information.

**Attributes:**
- data_id: Unique identifier for the data point
- symbol: Trading symbol
- timestamp: When the data was recorded
- source: Data source (exchange, aggregator, etc.)
- price_open: Opening price for the period
- price_high: Highest price for the period
- price_low: Lowest price for the period
- price_close: Closing price for the period
- volume: Trading volume for the period
- bid_price: Current best bid price
- ask_price: Current best ask price
- bid_volume: Volume at best bid
- ask_volume: Volume at best ask

**Relationships:**
- Used by multiple Trading Strategies
- Analyzed by ML Models
- Cached in Market Data Cache
- Aggregated by Data Aggregator

**Business Rules:**
- Prices must be positive
- High ≥ low for any period
- Volume must be non-negative
- Timestamp must be sequential for the same symbol

### 8. ML Model
Represents machine learning models used for trading decisions.

**Attributes:**
- model_id: Unique identifier for the model
- name: Human-readable model name
- type: Model type (price_prediction, sentiment, anomaly_detection)
- version: Model version for tracking updates
- status: Model status (training, active, deprecated)
- accuracy: Model accuracy metrics
- parameters: Model hyperparameters and configuration
- training_data_range: Date range of training data
- last_trained: When model was last retrained
- prediction_horizon: Time horizon for predictions
- confidence_threshold: Minimum confidence for trading signals

**Relationships:**
- Generates Trading Signals
- Evaluated by Model Performance
- Uses Market Data
- Configured by ML Parameters

**Business Rules:**
- Accuracy must be validated before deployment
- Confidence threshold must be between 0 and 1
- Models must be retrained regularly
- Deprecated models should not generate new signals

## Domain Relationships

### System Architecture Relationships
```
Trading System (1) ─── manages ─── (N) Trading Account
Trading System (1) ─── orchestrates ─── (N) Trading Strategy
Trading System (1) ─── monitors ─── (N) Risk Profile
Trading System (1) ─── integrates ─── (N) External Service

Trading Account (1) ─── has ─── (N) Position
Trading Account (1) ─── executes ─── (N) Order
Trading Account (1) ─── follows ─── (1) Risk Profile

Trading Strategy (1) ─── generates ─── (N) Trading Signal
Trading Strategy (1) ─── applied_to ─── (N) Trading Account
Trading Strategy (1) ─── evaluated_by ─── (1) Strategy Performance

Position (1) ─── consists_of ─── (N) Fill
Position (1) ─── associated_with ─── (1) Risk Profile
Position (1) ─── generated_from ─── (1) Trading Signal

Order (1) ─── results_in ─── (N) Fill
Order (1) ─── generated_from ─── (1) Trading Signal
Order (1) ─── executed_through ─── (1) Platform Connection

ML Model (1) ─── generates ─── (N) Trading Signal
ML Model (1) ─── uses ─── (N) Market Data
ML Model (1) ─── evaluated_by ─── (1) Model Performance

Market Data (N) ─── analyzed_by ─── (1) ML Model
Market Data (N) ─── used_by ─── (N) Trading Strategy
```

### Data Flow Relationships
```
External Service ─── provides ─── Market Data
Market Data ─── feeds ─── Trading Strategy
Trading Strategy ─── generates ─── Trading Signal
Trading Signal ─── triggers ─── Order
Order ─── executed ─── Fill
Fill ─── updates ─── Position
Position ─── calculates ─── P&L
P&L ─── contributes ─── Portfolio Performance

Risk Profile ─── constrains ─── Order
Risk Profile ─── limits ─── Position
Risk Monitoring ─── validates ─── Trading Signal
Risk Monitoring ─── enforces ─── Risk Profile
```

## Data Structures

### Configuration Schema
```json
{
  "system": {
    "name": "string",
    "version": "string",
    "environment": "development|production",
    "log_level": "debug|info|warn|error"
  },
  "trading": {
    "platforms": ["coinbase", "binance_us", "oanda"],
    "default_currency": "USD",
    "max_positions_per_account": 10,
    "min_order_size": 0.001
  },
  "risk": {
    "max_portfolio_risk": 0.02,
    "max_position_risk": 0.005,
    "stop_loss_default": 0.02,
    "take_profit_default": 0.05
  },
  "ml": {
    "prediction_horizon": "1h",
    "confidence_threshold": 0.7,
    "retrain_frequency": "7d"
  }
}
```

### Performance Metrics Schema
```json
{
  "account_id": "string",
  "timestamp": "ISO8601",
  "total_return": "decimal",
  "daily_return": "decimal",
  "sharpe_ratio": "decimal",
  "max_drawdown": "decimal",
  "win_rate": "decimal",
  "avg_win": "decimal",
  "avg_loss": "decimal",
  "profit_factor": "decimal"
}
```

### Risk Metrics Schema
```json
{
  "account_id": "string",
  "timestamp": "ISO8601",
  "var_95": "decimal",
  "var_99": "decimal",
  "current_drawdown": "decimal",
  "max_drawdown": "decimal",
  "portfolio_heat": "decimal",
  "position_correlation": "decimal",
  "leverage_ratio": "decimal"
}
```

## State Transitions

### Trading Account States
```
uninitialized → initialized → active → suspended → closed
                          ↓         ↓
                      error ←→ maintenance
```

### Position States
```
pending → open → partially_closed → closed
     ↓        ↓
   error ←→ pending_close
```

### Order States
```
created → submitted → pending → filled
                    ↓         ↓
                cancelled ←→ rejected
                    ↓
                partially_filled → completed
```

### System States
```
initializing → ready → running → paused → stopped
             ↓       ↓         ↓
           error ←→ error ←→ error
```

## Business Rules and Invariants

### Portfolio Management
- Total position value ≤ account balance
- No conflicting positions (long + short same asset)
- Risk limits must be enforced before order submission
- Currency exposure must be within limits

### Risk Management
- Portfolio VaR ≤ configured limit
- Individual position risk ≤ max position risk
- Maximum drawdown protection must be active
- Correlation between positions ≤ threshold

### Trading Operations
- Orders must be validated before submission
- Price limits must be reasonable
- Quantity must meet minimum order requirements
- Platform-specific trading rules must be followed

### ML Operations
- Model confidence ≥ threshold for trading signals
- Predictions must be within reasonable bounds
- Model drift must be monitored and corrected
- Training data must be recent and representative

## Query Models

### Portfolio Queries
- Current positions by account
- P&L by time period
- Performance vs benchmark
- Risk metrics summary
- Position concentration analysis

### Trading Queries
- Active orders by status
- Execution history
- Strategy performance
- Market data by symbol/timeframe
- Trade reconciliation status

### Risk Queries
- Real-time risk exposure
- VaR calculations
- Drawdown monitoring
- Position limit utilization
- Correlation analysis

### System Health Queries
- Service status
- Connection health
- Performance metrics
- Error rates
- Resource utilization

## Event Models

### Trading Events
- OrderCreated
- OrderFilled
- PositionOpened
- PositionClosed
- TradeExecuted
- SignalGenerated

### Risk Events
- RiskLimitExceeded
- StopLossTriggered
- TakeProfitTriggered
- MarginCall
- PositionLimitReached

### System Events
- ServiceStarted
- ServiceStopped
- ConfigurationChanged
- ErrorOccurred
- HealthCheckFailed

### Market Events
- PriceAlert
- VolumeSpike
- MarketRegimeChange
- EconomicEvent
- NewsEvent