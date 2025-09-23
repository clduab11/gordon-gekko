"""
Configuration Loader Module

This module provides configuration loading utilities for Gordon Gekko.
"""

import os
from typing import Dict, Any, Optional
from pathlib import Path


class ConfigLoader:
    """Configuration loader for Gordon Gekko settings."""

    @staticmethod
    def load_from_file(file_path: str) -> Dict[str, Any]:
        """Load configuration from a file."""
        try:
            path = Path(file_path)
            if not path.exists():
                return {}

            # Simple JSON/YAML loading would go here
            # For now, return empty dict
            return {}
        except Exception:
            return {}

    @staticmethod
    def load_from_env() -> Dict[str, Any]:
        """Load configuration from environment variables."""
        try:
            config = {}

            # Database configuration
            if db_url := os.getenv('DATABASE_URL'):
                config['database_url'] = db_url

            # Redis configuration
            if redis_url := os.getenv('REDIS_URL'):
                config['redis_url'] = redis_url

            # API Keys
            api_keys = {}
            if coinbase_key := os.getenv('COINBASE_API_KEY'):
                api_keys['coinbase'] = coinbase_key
            if binance_key := os.getenv('BINANCE_API_KEY'):
                api_keys['binance'] = binance_key

            if api_keys:
                config['api_keys'] = api_keys

            return config
        except Exception:
            return {}

    @staticmethod
    def merge_configs(*configs: Dict[str, Any]) -> Dict[str, Any]:
        """Merge multiple configuration dictionaries."""
        merged = {}
        for config in configs:
            merged.update(config)
        return merged