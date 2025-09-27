-- V002: Create neural model tables for AI-powered arbitrage system
-- This migration creates tables for neural model management, predictions,
-- performance tracking, and training data for the Gordon Gekko AI system.

-- Neural models table
CREATE TABLE IF NOT EXISTS neural_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    model_type VARCHAR(20) NOT NULL CHECK (model_type IN ('MLP', 'LSTM', 'Transformer', 'NBeats', 'NHiTS')),
    version VARCHAR(20) NOT NULL,
    description TEXT,
    
    -- Model configuration
    input_features TEXT[] NOT NULL, -- Array of feature names
    output_features TEXT[] NOT NULL, -- Array of output names
    architecture JSONB NOT NULL, -- Model architecture configuration
    hyperparameters JSONB NOT NULL, -- Training hyperparameters
    
    -- Model metadata
    training_data_size INTEGER NOT NULL DEFAULT 0,
    validation_accuracy DECIMAL(5, 4), -- 0.0 to 1.0
    test_accuracy DECIMAL(5, 4), -- 0.0 to 1.0
    inference_time_ms DECIMAL(8, 2) NOT NULL DEFAULT 0,
    memory_usage_mb DECIMAL(8, 2) NOT NULL DEFAULT 0,
    
    -- File paths and deployment
    model_file_path TEXT NOT NULL,
    weights_file_path TEXT,
    config_file_path TEXT,
    is_active BOOLEAN NOT NULL DEFAULT false,
    is_production BOOLEAN NOT NULL DEFAULT false,
    
    -- Timestamps
    trained_at TIMESTAMPTZ,
    deployed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Model predictions table
CREATE TABLE IF NOT EXISTS model_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES neural_models(id),
    prediction_type VARCHAR(20) NOT NULL CHECK (prediction_type IN ('volatility', 'price', 'arbitrage', 'risk', 'sentiment')),
    
    -- Input data
    symbol VARCHAR(20) NOT NULL,
    exchange VARCHAR(20),
    input_data JSONB NOT NULL, -- Raw input features
    
    -- Prediction results
    prediction_value DECIMAL(20, 8) NOT NULL,
    confidence_score DECIMAL(3, 2) NOT NULL CHECK (confidence_score BETWEEN 0 AND 1),
    probability_distribution JSONB, -- For classification models
    
    -- Model performance
    inference_time_ms DECIMAL(6, 2) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    
    -- Validation (if ground truth available)
    actual_value DECIMAL(20, 8),
    prediction_error DECIMAL(20, 8),
    is_correct BOOLEAN,
    
    -- Timestamps
    predicted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    validated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Model performance tracking table
CREATE TABLE IF NOT EXISTS model_performance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES neural_models(id),
    
    -- Performance metrics
    date DATE NOT NULL,
    total_predictions INTEGER NOT NULL DEFAULT 0,
    correct_predictions INTEGER NOT NULL DEFAULT 0,
    accuracy DECIMAL(5, 4) NOT NULL DEFAULT 0, -- 0.0 to 1.0
    precision_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
    recall_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
    f1_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
    
    -- Regression metrics
    mean_absolute_error DECIMAL(10, 6),
    mean_squared_error DECIMAL(10, 6),
    root_mean_squared_error DECIMAL(10, 6),
    r_squared DECIMAL(5, 4), -- R² coefficient
    
    -- Performance statistics
    avg_inference_time_ms DECIMAL(6, 2) NOT NULL DEFAULT 0,
    max_inference_time_ms DECIMAL(6, 2) NOT NULL DEFAULT 0,
    min_inference_time_ms DECIMAL(6, 2) NOT NULL DEFAULT 0,
    total_inference_time_ms DECIMAL(12, 2) NOT NULL DEFAULT 0,
    
    -- Confidence statistics
    avg_confidence DECIMAL(3, 2) NOT NULL DEFAULT 0,
    high_confidence_predictions INTEGER NOT NULL DEFAULT 0, -- confidence > 0.8
    low_confidence_predictions INTEGER NOT NULL DEFAULT 0, -- confidence < 0.5
    
    -- Financial impact (for trading models)
    profitable_predictions INTEGER DEFAULT 0,
    total_profit_impact DECIMAL(20, 2) DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(model_id, date)
);

-- Training data table
CREATE TABLE IF NOT EXISTS training_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Data identification
    symbol VARCHAR(20) NOT NULL,
    exchange VARCHAR(20) NOT NULL,
    data_type VARCHAR(20) NOT NULL CHECK (data_type IN ('price', 'volume', 'volatility', 'sentiment', 'news', 'orderbook')),
    
    -- Feature data
    features JSONB NOT NULL, -- Input features as JSON
    target JSONB NOT NULL, -- Target values as JSON
    
    -- Data quality metrics
    data_quality_score DECIMAL(3, 2) CHECK (data_quality_score BETWEEN 0 AND 1),
    has_outliers BOOLEAN DEFAULT false,
    missing_values_pct DECIMAL(5, 2) DEFAULT 0,
    
    -- Temporal information
    timestamp TIMESTAMPTZ NOT NULL,
    time_window_minutes INTEGER NOT NULL DEFAULT 1,
    
    -- Data lineage
    source VARCHAR(100) NOT NULL, -- Data source (exchange API, websocket, etc.)
    collection_method VARCHAR(50) NOT NULL,
    
    -- Usage tracking
    used_for_training BOOLEAN DEFAULT false,
    used_for_validation BOOLEAN DEFAULT false,
    used_for_testing BOOLEAN DEFAULT false,
    model_ids UUID[], -- Array of model IDs that used this data
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_neural_models_name ON neural_models(name);
CREATE INDEX IF NOT EXISTS idx_neural_models_model_type ON neural_models(model_type);
CREATE INDEX IF NOT EXISTS idx_neural_models_is_active ON neural_models(is_active);
CREATE INDEX IF NOT EXISTS idx_neural_models_is_production ON neural_models(is_production);

CREATE INDEX IF NOT EXISTS idx_model_predictions_model_id ON model_predictions(model_id);
CREATE INDEX IF NOT EXISTS idx_model_predictions_symbol ON model_predictions(symbol);
CREATE INDEX IF NOT EXISTS idx_model_predictions_predicted_at ON model_predictions(predicted_at);
CREATE INDEX IF NOT EXISTS idx_model_predictions_prediction_type ON model_predictions(prediction_type);
CREATE INDEX IF NOT EXISTS idx_model_predictions_confidence ON model_predictions(confidence_score DESC);

CREATE INDEX IF NOT EXISTS idx_model_performance_model_id ON model_performance(model_id);
CREATE INDEX IF NOT EXISTS idx_model_performance_date ON model_performance(date);
CREATE INDEX IF NOT EXISTS idx_model_performance_accuracy ON model_performance(accuracy DESC);

CREATE INDEX IF NOT EXISTS idx_training_data_symbol ON training_data(symbol);
CREATE INDEX IF NOT EXISTS idx_training_data_exchange ON training_data(exchange);
CREATE INDEX IF NOT EXISTS idx_training_data_timestamp ON training_data(timestamp);
CREATE INDEX IF NOT EXISTS idx_training_data_data_type ON training_data(data_type);
CREATE INDEX IF NOT EXISTS idx_training_data_used_for_training ON training_data(used_for_training);

-- Update triggers for updated_at timestamps
CREATE TRIGGER update_neural_models_updated_at BEFORE UPDATE ON neural_models
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_performance_updated_at BEFORE UPDATE ON model_performance
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();