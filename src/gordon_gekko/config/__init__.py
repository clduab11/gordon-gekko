"""
Configuration Management Module

This module provides centralized configuration management for the Gordon Gekko
trading system, supporting multiple environments, validation, and secure
credential management through environment variables.

Features:
- Environment-based configuration (dev, staging, prod)
- Configuration validation and type safety
- Secure credential management
- Runtime configuration updates
- Configuration inheritance and defaults

The configuration system follows a hierarchical approach:
1. Default values (hardcoded fallbacks)
2. Configuration files (YAML/JSON)
3. Environment variables
4. Runtime overrides
"""

from .settings import Settings
from .validators import ConfigValidator
from .loader import ConfigLoader

__all__ = [
    "Settings",
    "ConfigValidator",
    "ConfigLoader"
]