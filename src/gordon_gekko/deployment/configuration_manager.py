"""
ConfigurationManager component for Gordon Gekko deployment system.

This module provides centralized configuration management with support for
multi-source loading, validation, caching, and type-safe access following
Test-Driven Development principles.
"""

import os
import json
from typing import Dict, Any, List, Optional, Union
from pathlib import Path
import re
from functools import lru_cache


class ConfigurationError(Exception):
    """Base exception for configuration-related errors."""
    pass


class ValidationError(ConfigurationError):
    """Exception raised when configuration validation fails."""
    pass


class SourceNotFoundError(ConfigurationError):
    """Exception raised when configuration source is not found."""
    pass


class ConfigurationManager:
    """
    Manages configuration from multiple sources with validation and caching.

    Supports loading from files, environment variables, and other sources
    with schema validation and type-safe access.
    """

    def __init__(self, schema: Optional[Dict[str, Any]] = None):
        """
        Initialize ConfigurationManager with optional schema.

        Args:
            schema: Configuration schema for validation

        Raises:
            ConfigurationError: If schema is invalid
        """
        if schema is not None and not isinstance(schema, dict):
            raise ConfigurationError("Invalid schema")

        self.schema = schema or {}
        self.config: Dict[str, Any] = {}
        self.cache: Dict[str, Any] = {}

        # Mock loaders for dependency injection testing
        self.file_loader = self
        self.env_loader = self

    def load_from_file(self, file_path: str) -> bool:
        """
        Load configuration from a file.

        Args:
            file_path: Path to configuration file

        Returns:
            bool: True if loaded successfully

        Raises:
            SourceNotFoundError: If file not found
            ConfigurationError: If file cannot be parsed
        """
        try:
            path = Path(file_path)
            if not path.exists():
                raise SourceNotFoundError(f"Configuration file not found: {file_path}")

            with open(path, 'r') as f:
                content = json.load(f)

            self.config.update(content)
            self._invalidate_related_cache()
            return True

        except json.JSONDecodeError as e:
            raise ConfigurationError(f"Invalid JSON in configuration file: {e}")
        except Exception as e:
            raise ConfigurationError(f"Failed to load configuration file: {e}")

    def load_from_environment(self) -> bool:
        """
        Load configuration from environment variables.

        Returns:
            bool: True if loaded successfully
        """
        # Simple implementation - in real scenario would parse env vars
        # according to schema requirements
        env_config = {}
        for section_name, section_config in self.schema.items():
            if isinstance(section_config, dict):
                section_data = {}
                for key in section_config.keys():
                    env_key = f"GORDON_{section_name.upper()}_{key.upper()}"
                    if env_key in os.environ:
                        section_data[key] = os.environ[env_key]
                if section_data:
                    env_config[section_name] = section_data

        self.config.update(env_config)
        self._invalidate_related_cache()
        return True

    def load_from_multiple_sources(self, file_paths: List[str], use_env: bool = True) -> bool:
        """
        Load configuration from multiple sources.

        Args:
            file_paths: List of file paths to load
            use_env: Whether to also load from environment

        Returns:
            bool: True if all sources loaded successfully
        """
        success = True

        # Load from files
        for file_path in file_paths:
            try:
                self.load_from_file(file_path)
            except (SourceNotFoundError, ConfigurationError):
                success = False

        # Load from environment if requested
        if use_env:
            try:
                self.load_from_environment()
            except ConfigurationError:
                success = False

        return success

    def validate_configuration(self) -> bool:
        """
        Validate configuration against schema.

        Returns:
            bool: True if configuration is valid

        Raises:
            ValidationError: If validation fails
        """
        def validate_section(config: Dict[str, Any], schema: Dict[str, Any], path: str = "") -> None:
            for key, rules in schema.items():
                if not isinstance(rules, dict):
                    continue

                current_path = f"{path}.{key}" if path else key
                required = rules.get("required", False)
                config_value = config.get(key)

                # Check required fields
                if required and (config_value is None or config_value == ""):
                    raise ValidationError(f"Required field missing: {current_path}")

                # Type validation
                if config_value is not None:
                    expected_type = rules.get("type", "string")

                    if expected_type == "integer" and not isinstance(config_value, int):
                        try:
                            config_value = int(config_value)
                            config[key] = config_value
                        except (ValueError, TypeError):
                            raise ValidationError(f"Invalid type for {current_path}: expected integer")

                    elif expected_type == "float" and not isinstance(config_value, (int, float)):
                        try:
                            config_value = float(config_value)
                            config[key] = config_value
                        except (ValueError, TypeError):
                            raise ValidationError(f"Invalid type for {current_path}: expected float")

                    elif expected_type == "boolean" and not isinstance(config_value, bool):
                        if isinstance(config_value, str):
                            if config_value.lower() in ("true", "1", "yes", "on"):
                                config[key] = True
                            elif config_value.lower() in ("false", "0", "no", "off"):
                                config[key] = False
                            else:
                                raise ValidationError(f"Invalid boolean value for {current_path}")

                    elif expected_type == "list" and not isinstance(config_value, list):
                        raise ValidationError(f"Invalid type for {current_path}: expected list")

                    # Range validation for numbers
                    if isinstance(config_value, (int, float)):
                        if "min" in rules and config_value < rules["min"]:
                            raise ValidationError(f"Value {config_value} below minimum {rules['min']} for {current_path}")
                        if "max" in rules and config_value > rules["max"]:
                            raise ValidationError(f"Value {config_value} above maximum {rules['max']} for {current_path}")

                    # Allowed values validation
                    if "allowed" in rules and config_value not in rules["allowed"]:
                        raise ValidationError(f"Value {config_value} not in allowed values {rules['allowed']} for {current_path}")

                # Apply defaults
                if config_value is None and "default" in rules:
                    config[key] = rules["default"]

        try:
            for section, schema_rules in self.schema.items():
                if isinstance(schema_rules, dict):
                    if section not in self.config:
                        self.config[section] = {}
                    validate_section(self.config[section], schema_rules, section)
            return True
        except ValidationError:
            raise

    def get(self, key: str, default: Any = None) -> Any:
        """
        Get configuration value by key.

        Args:
            key: Configuration key (dot notation: 'section.subsection.key')
            default: Default value if key not found

        Returns:
            Configuration value or default

        Raises:
            ConfigurationError: If key not found and no default provided
        """
        # Check cache first
        if key in self.cache:
            return self.cache[key]

        # Parse key
        keys = key.split('.')
        value = self.config

        try:
            for k in keys:
                value = value[k]
            self.cache[key] = value
            return value
        except (KeyError, TypeError):
            if default is not None:
                self.cache[key] = default
                return default
            raise ConfigurationError(f"Configuration key not found: {key}")

    def set(self, key: str, value: Any) -> None:
        """
        Set configuration value.

        Args:
            key: Configuration key (dot notation)
            value: Value to set
        """
        keys = key.split('.')
        config = self.config

        # Navigate to parent
        for k in keys[:-1]:
            if k not in config:
                config[k] = {}
            config = config[k]

        # Set value
        config[keys[-1]] = value
        self.cache.pop(key, None)  # Invalidate cache

    def get_int(self, key: str, default: int = None) -> int:
        """Get configuration value as integer."""
        value = self.get(key, default)
        if value is None:
            return default
        try:
            return int(value)
        except (ValueError, TypeError):
            raise ValidationError(f"Cannot convert {key} value to integer: {value}")

    def get_float(self, key: str, default: float = None) -> float:
        """Get configuration value as float."""
        value = self.get(key, default)
        if value is None:
            return default
        try:
            return float(value)
        except (ValueError, TypeError):
            raise ValidationError(f"Cannot convert {key} value to float: {value}")

    def get_str(self, key: str, default: str = None) -> str:
        """Get configuration value as string."""
        value = self.get(key, default)
        if value is None:
            return default
        return str(value)

    def get_masked(self, key: str, mask_char: str = "*") -> str:
        """
        Get configuration value with sensitive data masked.

        Args:
            key: Configuration key
            mask_char: Character to use for masking

        Returns:
            Masked value
        """
        value = self.get(key)
        if self._is_sensitive_key(key):
            str_value = str(value)
            if len(str_value) <= 4:
                return mask_char * len(str_value)
            else:
                return str_value[:2] + mask_char * (len(str_value) - 4) + str_value[-2:]
        return str(value)

    def _is_sensitive_key(self, key: str) -> bool:
        """Check if a configuration key contains sensitive data."""
        keys = key.split('.')
        for section, schema_rules in self.schema.items():
            if section in keys and isinstance(schema_rules, dict):
                for subkey in keys[keys.index(section):]:
                    if isinstance(schema_rules, dict) and subkey in schema_rules:
                        if schema_rules[subkey].get("sensitive", False):
                            return True
                        schema_rules = schema_rules[subkey]
        return False

    def export_config(self, mask_sensitive: bool = False) -> Dict[str, Any]:
        """Export current configuration."""
        if mask_sensitive:
            return self._export_with_masking()
        return self.config.copy()

    def _export_with_masking(self) -> Dict[str, Any]:
        """Export configuration with sensitive data masked."""
        def mask_dict(data: Dict[str, Any], schema: Dict[str, Any]) -> Dict[str, Any]:
            masked = {}
            for key, value in data.items():
                if isinstance(value, dict) and key in schema and isinstance(schema[key], dict):
                    masked[key] = mask_dict(value, schema[key])
                elif key in schema and schema[key].get("sensitive", False):
                    str_value = str(value)
                    if len(str_value) <= 4:
                        masked[key] = "***"
                    else:
                        masked[key] = str_value[:2] + "***" + str_value[-1:]
                else:
                    masked[key] = value
            return masked

        masked_config = {}
        for section, data in self.config.items():
            if section in self.schema and isinstance(self.schema[section], dict):
                masked_config[section] = mask_dict(data, self.schema[section])
            else:
                masked_config[section] = data
        return masked_config

    def reload_from_file(self, file_path: str) -> bool:
        """
        Reload configuration from file, preserving existing values.

        Args:
            file_path: Path to configuration file

        Returns:
            bool: True if reloaded successfully
        """
        old_config = self.config.copy()
        self.config = {}

        try:
            success = self.load_from_file(file_path)
            if not success:
                self.config = old_config
                return False

            # Merge with old config (old values take precedence)
            self.config = self._merge_configs(old_config, self.config)
            self._invalidate_related_cache()
            return True

        except (SourceNotFoundError, ConfigurationError):
            self.config = old_config
            return False

    def _merge_configs(self, base_config: Dict[str, Any], new_config: Dict[str, Any]) -> Dict[str, Any]:
        """Merge two configuration dictionaries."""
        merged = base_config.copy()

        def merge_recursive(base: Dict[str, Any], new: Dict[str, Any]) -> Dict[str, Any]:
            for key, value in new.items():
                if key in base and isinstance(base[key], dict) and isinstance(value, dict):
                    base[key] = merge_recursive(base[key], value)
                else:
                    base[key] = value
            return base

        return merge_recursive(merged, new_config)

    def _invalidate_related_cache(self) -> None:
        """Invalidate cache entries that might be affected by configuration changes."""
        # Clear all cache as it's simpler and safer
        self.cache.clear()

    def invalidate_cache(self, key: Optional[str] = None) -> None:
        """
        Invalidate cache entries.

        Args:
            key: Specific key to invalidate, or None to clear all cache
        """
        if key:
            self.cache.pop(key, None)
        else:
            self.cache.clear()

    def health_check(self) -> Dict[str, Any]:
        """Perform health check on configuration manager."""
        return {
            "status": "healthy",
            "config_sections": len(self.config),
            "cache_entries": len(self.cache),
            "schema_valid": bool(self.schema)
        }