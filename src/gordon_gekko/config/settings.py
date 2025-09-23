"""
Gordon Gekko Configuration Settings

This module provides centralized configuration management for the Gordon Gekko
trading system using Pydantic settings with environment variable support.
"""

import os
from typing import Optional, List, Dict, Any
from pathlib import Path

from pydantic_settings import BaseSettings
from pydantic import Field, field_validator
from pydantic.types import SecretStr


class Settings(BaseSettings):
    """
    Application settings with environment variable support.

    All configuration is managed through environment variables with sensible
    defaults for development. Production deployments should override these
    via environment variables or configuration files.
    """

    # System Configuration
    system_name: str = Field(default="Gordon Gekko Trading System")
    debug: bool = Field(default=False)
    log_level: str = Field(default="INFO")

    # Database Configuration
    database_url: str = Field(
        default="postgresql://user:pass@localhost:5432/gordon_gekko",
        description="PostgreSQL connection URL"
    )
    database_pool_size: int = Field(default=10, ge=1, le=50)
    database_max_overflow: int = Field(default=20, ge=0)

    # Redis Configuration
    redis_url: str = Field(
        default="redis://localhost:6379/0",
        description="Redis connection URL"
    )
    redis_pool_size: int = Field(default=10, ge=1, le=100)

    # Message Broker Configuration
    message_broker_url: str = Field(
        default="redis://localhost:6379/1",
        description="Message broker URL for internal communication"
    )

    # Trading Platform API Keys (MCP Managed)
    coinbase_api_key: Optional[SecretStr] = Field(default=None)
    coinbase_api_secret: Optional[SecretStr] = Field(default=None)
    coinbase_base_url: str = Field(default="https://api.coinbase.com")

    binance_api_key: Optional[SecretStr] = Field(default=None)
    binance_api_secret: Optional[SecretStr] = Field(default=None)
    binance_base_url: str = Field(default="https://api.binance.us")

    oanda_api_key: Optional[SecretStr] = Field(default=None)
    oanda_account_id: Optional[str] = Field(default=None)
    oanda_base_url: str = Field(default="https://api-fxpractice.oanda.com")

    # AI/ML Services
    openrouter_api_key: Optional[SecretStr] = Field(default=None)
    openrouter_base_url: str = Field(default="https://openrouter.ai/api/v1")

    litellm_api_key: Optional[SecretStr] = Field(default=None)
    litellm_base_url: str = Field(default="https://api.litellm.ai")

    # GPU Configuration
    gpu_acceleration: bool = Field(default=True)
    gpu_memory_fraction: float = Field(default=0.8, ge=0.1, le=1.0)
    cuda_device: Optional[int] = Field(default=None)

    # Security Configuration
    jwt_secret_key: SecretStr = Field(
        default_factory=lambda: SecretStr("your-secret-key-change-in-production")
    )
    jwt_algorithm: str = Field(default="HS256")
    jwt_access_token_expire_minutes: int = Field(default=60)

    encryption_key: SecretStr = Field(
        default_factory=lambda: SecretStr("your-32-char-encryption-key!!!")
    )

    # Monitoring Configuration
    prometheus_enabled: bool = Field(default=True)
    grafana_enabled: bool = Field(default=False)
    grafana_url: str = Field(default="http://localhost:3000")

    metrics_port: int = Field(default=9090)
    metrics_address: str = Field(default="0.0.0.0")

    # Alerting Configuration
    alert_email_recipients: List[str] = Field(default_factory=list)
    slack_webhook_url: Optional[str] = Field(default=None)
    discord_webhook_url: Optional[str] = Field(default=None)

    # Rate Limiting
    max_requests_per_minute: int = Field(default=1000)
    max_trades_per_minute: int = Field(default=10)
    max_api_calls_per_second: int = Field(default=5)

    # Risk Management
    max_position_size: float = Field(default=0.05, ge=0.001, le=1.0)
    max_portfolio_var: float = Field(default=0.15, ge=0.01, le=0.5)
    max_drawdown_limit: float = Field(default=0.20, ge=0.05, le=0.95)

    # Performance Configuration
    api_timeout_seconds: int = Field(default=30, ge=5, le=300)
    connection_pool_timeout: int = Field(default=30, ge=5)
    max_concurrent_connections: int = Field(default=100, ge=10, le=1000)

    # File Paths
    config_dir: Path = Field(default_factory=lambda: Path.cwd() / "config")
    logs_dir: Path = Field(default_factory=lambda: Path.cwd() / "logs")
    data_dir: Path = Field(default_factory=lambda: Path.cwd() / "data")
    models_dir: Path = Field(default_factory=lambda: Path.cwd() / "models")

    # Feature Flags
    enable_ml_predictions: bool = Field(default=True)
    enable_risk_management: bool = Field(default=True)
    enable_portfolio_optimization: bool = Field(default=True)
    enable_real_time_monitoring: bool = Field(default=True)

    # Development Settings
    reload_on_changes: bool = Field(default=False)
    enable_api_docs: bool = Field(default=True)

    class Config:
        """Pydantic configuration."""
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False

        # Allow environment variables with prefixes
        env_prefix = "GORDON_GEKKE_"

    @field_validator("database_url")
    @classmethod
    def validate_database_url(cls, v: str) -> str:
        """Validate database URL format."""
        if not v.startswith(("postgresql://", "sqlite://")):
            raise ValueError("Database URL must start with postgresql:// or sqlite://")
        return v

    @field_validator("redis_url")
    @classmethod
    def validate_redis_url(cls, v: str) -> str:
        """Validate Redis URL format."""
        if not v.startswith(("redis://", "rediss://", "unix://")):
            raise ValueError("Redis URL must start with redis://, rediss://, or unix://")
        return v

    @field_validator("coinbase_base_url", "binance_base_url", "oanda_base_url")
    @classmethod
    def validate_api_base_urls(cls, v: str) -> str:
        """Validate API base URL formats."""
        if not v.startswith("https://"):
            raise ValueError("API base URL must start with https://")
        return v

    def get_database_config(self) -> Dict[str, Any]:
        """Get database-specific configuration."""
        return {
            "url": self.database_url,
            "pool_size": self.database_pool_size,
            "max_overflow": self.database_max_overflow,
        }

    def get_redis_config(self) -> Dict[str, Any]:
        """Get Redis-specific configuration."""
        return {
            "url": self.redis_url,
            "pool_size": self.redis_pool_size,
        }

    def get_trading_platform_configs(self) -> Dict[str, Dict[str, Any]]:
        """Get configuration for all trading platforms."""
        configs = {}

        if self.coinbase_api_key and self.coinbase_api_secret:
            configs["coinbase"] = {
                "api_key": self.coinbase_api_key.get_secret_value(),
                "api_secret": self.coinbase_api_secret.get_secret_value(),
                "base_url": self.coinbase_base_url,
                "sandbox": False,  # Set based on environment
            }

        if self.binance_api_key and self.binance_api_secret:
            configs["binance_us"] = {
                "api_key": self.binance_api_key.get_secret_value(),
                "api_secret": self.binance_api_secret.get_secret_value(),
                "base_url": self.binance_base_url,
                "testnet": False,
            }

        if self.oanda_api_key and self.oanda_account_id:
            configs["oanda"] = {
                "api_key": self.oanda_api_key.get_secret_value(),
                "account_id": self.oanda_account_id,
                "base_url": self.oanda_base_url,
                "environment": "live",
            }

        return configs

    def get_security_config(self) -> Dict[str, Any]:
        """Get security-related configuration."""
        return {
            "jwt_secret": self.jwt_secret_key.get_secret_value(),
            "jwt_algorithm": self.jwt_algorithm,
            "jwt_expire_minutes": self.jwt_access_token_expire_minutes,
            "encryption_key": self.encryption_key.get_secret_value(),
        }

    def get_monitoring_config(self) -> Dict[str, Any]:
        """Get monitoring configuration."""
        return {
            "prometheus_enabled": self.prometheus_enabled,
            "grafana_enabled": self.grafana_enabled,
            "grafana_url": self.grafana_url,
            "metrics_port": self.metrics_port,
            "metrics_address": self.metrics_address,
        }

    def is_production(self) -> bool:
        """Check if running in production environment."""
        return os.getenv("ENVIRONMENT", "development").lower() == "production"

    def get_log_config(self) -> Dict[str, Any]:
        """Get logging configuration."""
        return {
            "level": self.log_level,
            "format": "%(asctime)s - %(name)s - %(levelname)s - %(message)s" if not self.debug
                     else "%(asctime)s - %(name)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s",
            "structured": True,
        }

    def __repr__(self) -> str:
        """String representation of settings (without sensitive data)."""
        return f"Settings(system_name={self.system_name}, debug={self.debug}, environment=production)"


# Global settings instance
settings = Settings()