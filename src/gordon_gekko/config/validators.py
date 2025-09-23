"""
Configuration Validation Module

This module provides validation utilities for Gordon Gekko configuration settings.
"""

from typing import Any, Dict, Optional
from pydantic import BaseModel, ValidationError


class ConfigValidator:
    """Configuration validator for Gordon Gekko settings."""

    @staticmethod
    def validate_settings(settings: Dict[str, Any]) -> bool:
        """Validate configuration settings."""
        try:
            # Basic validation - check for required fields
            required_fields = [
                'database_url', 'redis_url', 'api_keys'
            ]

            for field in required_fields:
                if field not in settings:
                    return False

            return True
        except Exception:
            return False

    @staticmethod
    def validate_api_keys(api_keys: Dict[str, str]) -> bool:
        """Validate API keys configuration."""
        try:
            required_apis = ['coinbase', 'binance']
            return all(api in api_keys for api in required_apis)
        except Exception:
            return False


class ValidationResult(BaseModel):
    """Result of configuration validation."""
    is_valid: bool
    errors: Optional[list[str]] = None
    warnings: Optional[list[str]] = None