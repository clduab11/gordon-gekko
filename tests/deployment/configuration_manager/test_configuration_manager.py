"""
Test suite for ConfigurationManager component using real data.

This module provides integration-style test coverage for the configuration manager,
focusing on real configuration scenarios without extensive mocking.
"""

import pytest
import json
import tempfile
from pathlib import Path
from typing import Dict, Any

from src.gordon_gekko.deployment.configuration_manager import (
    ConfigurationManager,
    ConfigurationError,
    ValidationError,
    SourceNotFoundError,
)


class TestConfigurationManagerInitialization:
    """Test ConfigurationManager initialization and basic setup."""

    def test_successful_initialization(self) -> None:
        """Test successful configuration manager initialization."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)

        assert manager.schema == schema
        assert manager.config == {}
        assert manager.cache == {}

    def test_initialization_with_invalid_schema(self) -> None:
        """Test initialization failure with invalid schema."""
        invalid_schema = "not_a_dict"

        with pytest.raises(ConfigurationError) as exc_info:
            ConfigurationManager(invalid_schema)

        assert "Invalid schema" in str(exc_info.value)

    def test_initialization_with_none_schema(self) -> None:
        """Test initialization with None schema."""
        manager = ConfigurationManager(None)

        assert manager.schema == {}
        assert manager.config == {}
        assert manager.cache == {}


class TestConfigurationManagerFileOperations:
    """Test file-based configuration operations."""

    def test_load_from_file_success(self) -> None:
        """Test successful configuration loading from file."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)

        config_data = {
            "database": {
                "host": "localhost",
                "port": 5432
            }
        }

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(config_data, f)
            temp_file_path = f.name

        try:
            result = manager.load_from_file(temp_file_path)

            assert result is True
            assert manager.get("database.host") == "localhost"
            assert manager.get("database.port") == 5432
        finally:
            Path(temp_file_path).unlink()

    def test_load_from_file_not_found(self) -> None:
        """Test file loading failure when file not found."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)

        with pytest.raises(ConfigurationError) as exc_info:
            manager.load_from_file("nonexistent_file.json")

        assert "not found" in str(exc_info.value).lower()

    def test_load_from_file_invalid_json(self) -> None:
        """Test file loading failure with invalid JSON."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)

        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            f.write("invalid json content")
            temp_file_path = f.name

        try:
            with pytest.raises(ConfigurationError) as exc_info:
                manager.load_from_file(temp_file_path)

            assert "json" in str(exc_info.value).lower()
        finally:
            Path(temp_file_path).unlink()


class TestConfigurationManagerConfigurationOperations:
    """Test configuration value operations."""

    def test_set_and_get_configuration_value(self) -> None:
        """Test setting and getting configuration values."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)

        # Test setting values
        manager.set("database.host", "production.db.com")
        manager.set("database.port", 5433)

        # Test getting values
        assert manager.get("database.host") == "production.db.com"
        assert manager.get("database.port") == 5433

    def test_get_configuration_value_with_default(self) -> None:
        """Test configuration value retrieval with default fallback."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)
        manager.set("database.host", "localhost")

        # Test getting existing value
        host = manager.get("database.host")
        assert host == "localhost"

        # Test getting with default for missing value
        port = manager.get("database.port", default=3000)
        assert port == 3000

    def test_get_configuration_value_missing_key(self) -> None:
        """Test configuration value retrieval for missing key."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.set("database.host", "localhost")

        with pytest.raises(ConfigurationError) as exc_info:
            manager.get("database.missing_key")

        assert "not found" in str(exc_info.value).lower()

    def test_configuration_caching(self) -> None:
        """Test configuration caching mechanism."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"database": {"host": "localhost"}}

        # First access should compute and cache
        host1 = manager.get("database.host")
        assert host1 == "localhost"

        # Second access should use cache
        host2 = manager.get("database.host")
        assert host2 == "localhost"

        # Verify cache was used
        assert "database.host" in manager.cache

    def test_cache_invalidation(self) -> None:
        """Test cache invalidation."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"database": {"host": "localhost"}}
        manager.cache = {"database.host": "cached_value"}

        manager.invalidate_cache("database.host")

        assert "database.host" not in manager.cache


class TestConfigurationManagerTypeSafeAccess:
    """Test type-safe configuration access methods."""

    def test_type_safe_access_integer(self) -> None:
        """Test type-safe integer access."""
        schema = {
            "database": {
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"database": {"port": 5432}}

        # Test integer access
        port = manager.get_int("database.port")
        assert port == 5432
        assert isinstance(port, int)

    def test_type_safe_access_float(self) -> None:
        """Test type-safe float access."""
        schema = {
            "trading": {
                "risk_tolerance": {"type": "float", "default": 0.02}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"trading": {"risk_tolerance": 0.02}}

        # Test float access
        risk = manager.get_float("trading.risk_tolerance")
        assert risk == 0.02
        assert isinstance(risk, float)

    def test_type_safe_access_string(self) -> None:
        """Test type-safe string access."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"database": {"host": "localhost"}}

        # Test string access
        host = manager.get_str("database.host")
        assert host == "localhost"
        assert isinstance(host, str)

    def test_type_safe_access_invalid_conversion(self) -> None:
        """Test type-safe access with invalid type conversion."""
        schema = {
            "database": {
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"database": {"port": "not_a_number"}}

        with pytest.raises(ValidationError) as exc_info:
            manager.get_int("database.port")

        assert "cannot convert" in str(exc_info.value).lower()


class TestConfigurationManagerValidation:
    """Test configuration validation functionality."""

    def test_configuration_validation_success(self) -> None:
        """Test successful configuration validation."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            },
            "trading": {
                "max_positions": {"type": "integer", "default": 10}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {
            "database": {"host": "localhost", "port": 5432},
            "trading": {"max_positions": 5}
        }

        result = manager.validate_configuration()

        assert result is True

    def test_configuration_validation_failure_missing_required(self) -> None:
        """Test configuration validation failure due to missing required fields."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {
            "database": {"port": 5432}  # Missing required 'host'
        }

        with pytest.raises(ValidationError) as exc_info:
            manager.validate_configuration()

        assert "required" in str(exc_info.value).lower()

    def test_configuration_validation_failure_invalid_type(self) -> None:
        """Test configuration validation failure due to invalid type."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {
            "database": {"host": "localhost", "port": "not_a_number"}
        }

        with pytest.raises(ValidationError) as exc_info:
            manager.validate_configuration()

        assert "type" in str(exc_info.value).lower()


class TestConfigurationManagerSensitiveData:
    """Test sensitive data handling."""

    def test_sensitive_data_handling(self) -> None:
        """Test sensitive data handling and masking."""
        schema = {
            "api_keys": {
                "coinbase": {"type": "string", "required": True, "sensitive": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {"api_keys": {"coinbase": "secret_key_123"}}

        # Test that sensitive data is masked
        masked = manager.get_masked("api_keys.coinbase")
        assert "secret" not in masked
        assert len(masked) == len("secret_key_123")

    def test_configuration_export_with_masking(self) -> None:
        """Test configuration export with sensitive data masking."""
        schema = {
            "api_keys": {
                "coinbase": {"type": "string", "required": True, "sensitive": True}
            },
            "database": {
                "host": {"type": "string", "required": True}
            }
        }

        manager = ConfigurationManager(schema)
        manager.config = {
            "api_keys": {"coinbase": "secret_key_123"},
            "database": {"host": "localhost"}
        }

        exported = manager.export_config(mask_sensitive=True)

        assert exported["api_keys"]["coinbase"] != "secret_key_123"
        assert exported["database"]["host"] == "localhost"


class TestConfigurationManagerRealWorldScenarios:
    """Test real-world configuration scenarios."""

    def test_realistic_trading_configuration(self) -> None:
        """Test configuration with realistic trading system values."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432},
                "password": {"type": "string", "required": True, "sensitive": True}
            },
            "trading": {
                "max_positions": {"type": "integer", "default": 10},
                "risk_tolerance": {"type": "float", "default": 0.02}
            },
            "api_keys": {
                "coinbase": {"type": "string", "required": True, "sensitive": True},
                "binance": {"type": "string", "required": False, "sensitive": True}
            }
        }

        manager = ConfigurationManager(schema)

        # Set realistic configuration
        manager.config = {
            "database": {
                "host": "prod-trading-cluster.db.example.com",
                "port": 5432,
                "password": "secure_prod_password_123!"
            },
            "trading": {
                "max_positions": 25,
                "risk_tolerance": 0.015
            },
            "api_keys": {
                "coinbase": "coinbase_prod_key_abcdef123456",
                "binance": "binance_prod_key_ghijkl789012"
            }
        }

        # Test all values are accessible
        assert manager.get("database.host") == "prod-trading-cluster.db.example.com"
        assert manager.get("trading.max_positions") == 25
        assert manager.get("trading.risk_tolerance") == 0.015
        assert manager.get("api_keys.coinbase") == "coinbase_prod_key_abcdef123456"

    def test_configuration_merge_priority(self) -> None:
        """Test configuration merging with proper priority handling."""
        schema = {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432}
            },
            "api_keys": {
                "coinbase": {"type": "string", "required": True, "sensitive": True}
            }
        }

        manager = ConfigurationManager(schema)

        file_config = {"database": {"host": "file_host", "port": 5432}}
        env_config = {"database": {"host": "env_host"}, "api_keys": {"coinbase": "test_key"}}

        # Environment variables should take priority
        merged = manager._merge_configs(file_config, env_config)

        assert merged["database"]["host"] == "env_host"  # env takes priority
        assert merged["database"]["port"] == 5432  # file value preserved
        assert merged["api_keys"]["coinbase"] == "test_key"


class TestConfigurationManagerIntegration:
    """Integration tests for ConfigurationManager."""

    @pytest.fixture
    def full_config_schema(self) -> Dict[str, Any]:
        """Full configuration schema for integration testing."""
        return {
            "database": {
                "host": {"type": "string", "required": True},
                "port": {"type": "integer", "default": 5432, "min": 1, "max": 65535},
                "name": {"type": "string", "default": "trading_db"},
                "password": {"type": "string", "required": True, "sensitive": True}
            },
            "trading": {
                "max_positions": {"type": "integer", "default": 10, "min": 1, "max": 100},
                "risk_tolerance": {"type": "float", "default": 0.02, "min": 0.001, "max": 0.1},
                "symbols": {"type": "list", "default": ["BTC/USD", "ETH/USD"]},
                "enabled": {"type": "boolean", "default": True}
            },
            "api_keys": {
                "coinbase": {"type": "string", "required": True, "sensitive": True},
                "binance": {"type": "string", "required": False, "sensitive": True},
                "oanda": {"type": "string", "required": False, "sensitive": True}
            },
            "monitoring": {
                "log_level": {"type": "string", "default": "INFO", "allowed": ["DEBUG", "INFO", "WARNING", "ERROR"]},
                "metrics_port": {"type": "integer", "default": 9090, "min": 1024, "max": 65535}
            }
        }

    async def test_full_configuration_lifecycle(
        self,
        full_config_schema: Dict[str, Any]
    ) -> None:
        """Test complete configuration lifecycle."""
        manager = ConfigurationManager(full_config_schema)

        # Test initialization
        assert manager.schema == full_config_schema
        assert manager.config == {}
        assert manager.cache == {}

        # Test setting values
        manager.set("database.host", "production.db.com")
        manager.set("database.port", 5433)
        manager.set("database.name", "trading_db")
        manager.set("api_keys.coinbase", "prod_key_123")
        manager.set("trading.enabled", True)

        # Test retrieval
        assert manager.get("database.host") == "production.db.com"
        assert manager.get("database.port") == 5433
        assert manager.get("database.name") == "trading_db"
        assert manager.get("api_keys.coinbase") == "prod_key_123"

        # Test defaults
        assert manager.get("trading.enabled") is True

    async def test_configuration_with_realistic_values(
        self,
        full_config_schema: Dict[str, Any]
    ) -> None:
        """Test configuration with realistic trading system values."""
        manager = ConfigurationManager(full_config_schema)

        # Set realistic configuration
        config = {
            "database": {
                "host": "prod-trading-cluster.db.example.com",
                "port": 5432,
                "name": "trading_production",
                "password": "secure_prod_password_123!"
            },
            "trading": {
                "max_positions": 25,
                "risk_tolerance": 0.015,
                "symbols": ["BTC/USD", "ETH/USD", "ADA/USD", "SOL/USD"],
                "enabled": True
            },
            "api_keys": {
                "coinbase": "coinbase_prod_key_abcdef123456",
                "binance": "binance_prod_key_ghijkl789012"
            },
            "monitoring": {
                "log_level": "INFO",
                "metrics_port": 9090
            }
        }

        manager.config = config

        # Test all values are accessible
        assert manager.get("database.host") == "prod-trading-cluster.db.example.com"
        assert manager.get("trading.max_positions") == 25
        assert manager.get("trading.risk_tolerance") == 0.015
        assert len(manager.get("trading.symbols")) == 4
        assert manager.get("api_keys.coinbase") == "coinbase_prod_key_abcdef123456"

        # Test sensitive data masking
        masked_key = manager.get_masked("api_keys.coinbase")
        assert masked_key == "co**************************56"

    async def test_error_handling_edge_cases(
        self,
        full_config_schema: Dict[str, Any]
    ) -> None:
        """Test error handling for edge cases."""
        manager = ConfigurationManager(full_config_schema)

        # Test accessing deeply nested non-existent keys
        with pytest.raises(ConfigurationError):
            manager.get("nonexistent.section.key")

        # Test type conversion errors
        manager.config = {"database": {"port": "not_a_number"}}

        with pytest.raises(ValidationError):
            manager.get_int("database.port")

        # Test cache invalidation
        manager.cache = {"database.host": "cached_value"}
        manager.invalidate_cache("database.host")
        assert "database.host" not in manager.cache