"""
Comprehensive test suite for EnvironmentValidator component.

This module provides complete TDD test coverage for environment validation,
including hardware validation, software dependency checks, and network connectivity
testing following the London School TDD approach.
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from typing import Dict, Any, List, Optional

from src.gordon_gekko.deployment.environment_validator import (
    EnvironmentValidator,
    ValidationError,
    HardwareValidationError,
    SoftwareValidationError,
    NetworkValidationError,
)


class TestEnvironmentValidator:
    """Test suite for EnvironmentValidator class."""

    @pytest.fixture
    def mock_config(self) -> Dict[str, Any]:
        """Mock configuration for environment validator."""
        return {
            "hardware": {
                "min_cpu_cores": 4,
                "min_memory_gb": 16,
                "min_storage_gb": 100,
                "supported_architectures": ["x86_64", "arm64"],
                "gpu_required": True,
                "supported_gpu_types": ["nvidia", "apple_silicon"]
            },
            "software": {
                "python_version": "3.11+",
                "required_packages": [
                    "docker",
                    "redis-server",
                    "postgresql"
                ],
                "docker_version": "20.0+"
            },
            "network": {
                "required_endpoints": [
                    "https://api.coinbase.com",
                    "https://api.binance.us",
                    "https://api.oanda.com"
                ],
                "min_bandwidth_mbps": 10,
                "timeout_seconds": 5
            },
            "security": {
                "required_policies": [
                    "firewall_enabled",
                    "encryption_at_rest"
                ]
            }
        }

    @pytest.fixture
    def validator(self, mock_config: Dict[str, Any]) -> EnvironmentValidator:
        """Environment validator instance for testing."""
        return EnvironmentValidator(mock_config)

    @pytest.fixture
    def mock_system_info(self) -> AsyncMock:
        """Mock system information for testing."""
        system_info = AsyncMock()
        system_info.get_cpu_info.return_value = {
            "cores": 8,
            "architecture": "x86_64"
        }
        system_info.get_memory_info.return_value = {
            "total_gb": 32,
            "available_gb": 24
        }
        system_info.get_storage_info.return_value = {
            "total_gb": 500,
            "available_gb": 300
        }
        return system_info

    @pytest.fixture
    def mock_gpu_detector(self) -> AsyncMock:
        """Mock GPU detector for testing."""
        gpu_detector = AsyncMock()
        gpu_detector.detect_gpu.return_value = {
            "type": "nvidia",
            "memory_gb": 12,
            "driver_version": "535.0"
        }
        return gpu_detector

    async def test_initialization_success(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test successful environment validator initialization."""
        assert validator.config is not None
        assert validator.validation_results == []
        assert validator.overall_status == "initialized"

    async def test_initialization_with_invalid_config(self) -> None:
        """Test initialization failure with invalid configuration."""
        invalid_config = {"hardware": {}}

        with pytest.raises(ValidationError) as exc_info:
            EnvironmentValidator(invalid_config)

        assert "Invalid environment configuration" in str(exc_info.value)

    async def test_hardware_validation_success(
        self,
        validator: EnvironmentValidator,
        mock_system_info: AsyncMock,
        mock_gpu_detector: AsyncMock
    ) -> None:
        """Test successful hardware validation."""
        with patch.object(validator, 'system_info', mock_system_info), \
             patch.object(validator, 'gpu_detector', mock_gpu_detector):

            result = await validator._validate_hardware()

            assert result is True
            mock_system_info.get_cpu_info.assert_called_once()
            mock_system_info.get_memory_info.assert_called_once()
            mock_system_info.get_storage_info.assert_called_once()
            mock_gpu_detector.detect_gpu.assert_called_once()

    async def test_hardware_validation_failure_insufficient_cpu(
        self,
        validator: EnvironmentValidator,
        mock_system_info: AsyncMock
    ) -> None:
        """Test hardware validation failure due to insufficient CPU cores."""
        mock_system_info.get_cpu_info.return_value = {
            "cores": 2,
            "architecture": "x86_64"
        }

        with patch.object(validator, 'system_info', mock_system_info):
            with pytest.raises(HardwareValidationError) as exc_info:
                await validator._validate_hardware()

            assert "CPU cores" in str(exc_info.value)
            assert "insufficient" in str(exc_info.value).lower()

    async def test_hardware_validation_failure_unsupported_architecture(
        self,
        validator: EnvironmentValidator,
        mock_system_info: AsyncMock
    ) -> None:
        """Test hardware validation failure due to unsupported architecture."""
        mock_system_info.get_cpu_info.return_value = {
            "cores": 8,
            "architecture": "unsupported_arch"
        }

        with patch.object(validator, 'system_info', mock_system_info):
            with pytest.raises(HardwareValidationError) as exc_info:
                await validator._validate_hardware()

            assert "architecture" in str(exc_info.value)
            assert "unsupported" in str(exc_info.value).lower()

    async def test_hardware_validation_failure_insufficient_memory(
        self,
        validator: EnvironmentValidator,
        mock_system_info: AsyncMock
    ) -> None:
        """Test hardware validation failure due to insufficient memory."""
        mock_system_info.get_memory_info.return_value = {
            "total_gb": 8,
            "available_gb": 4
        }

        with patch.object(validator, 'system_info', mock_system_info):
            with pytest.raises(HardwareValidationError) as exc_info:
                await validator._validate_hardware()

            assert "memory" in str(exc_info.value)
            assert "insufficient" in str(exc_info.value).lower()

    async def test_software_validation_success(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test successful software validation."""
        with patch('sys.version_info', major=3, minor=12), \
             patch('subprocess.run') as mock_run, \
             patch('importlib.util.find_spec') as mock_find_spec:

            # Mock successful package detection
            mock_find_spec.return_value = MagicMock()
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = "Docker version 24.0.0"

            result = await validator._validate_software()

            assert result is True

    async def test_software_validation_failure_python_version(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test software validation failure due to Python version."""
        with patch('sys.version_info', major=3, minor=8):
            with pytest.raises(SoftwareValidationError) as exc_info:
                await validator._validate_software()

            assert "Python version" in str(exc_info.value)
            assert "3.8" in str(exc_info.value)

    async def test_software_validation_failure_missing_docker(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test software validation failure due to missing Docker."""
        with patch('sys.version_info', major=3, minor=12), \
             patch('subprocess.run') as mock_run, \
             patch('importlib.util.find_spec') as mock_find_spec:

            # Mock Docker command failure
            mock_find_spec.return_value = None
            mock_run.return_value.returncode = 1

            with pytest.raises(SoftwareValidationError) as exc_info:
                await validator._validate_software()

            assert "Docker" in str(exc_info.value)
            assert "not found" in str(exc_info.value).lower()

    async def test_network_validation_success(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test successful network validation."""
        with patch('aiohttp.ClientSession.get') as mock_get:
            # Mock successful HTTP responses
            mock_response = AsyncMock()
            mock_response.status = 200
            mock_response.json.return_value = {"status": "ok"}
            mock_get.return_value.__aenter__.return_value = mock_response

            result = await validator._validate_network()

            assert result is True
            assert mock_get.call_count >= 3  # At least 3 API endpoints

    async def test_network_validation_failure_unreachable_endpoint(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test network validation failure due to unreachable endpoint."""
        with patch('aiohttp.ClientSession.get') as mock_get:
            # Mock network failure
            mock_get.side_effect = Exception("Network unreachable")

            with pytest.raises(NetworkValidationError) as exc_info:
                await validator._validate_network()

            assert "network" in str(exc_info.value).lower()
            assert "unreachable" in str(exc_info.value).lower()

    async def test_network_validation_failure_timeout(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test network validation failure due to timeout."""
        with patch('aiohttp.ClientSession.get') as mock_get:
            # Mock timeout
            import asyncio
            mock_get.side_effect = asyncio.TimeoutError()

            with pytest.raises(NetworkValidationError) as exc_info:
                await validator._validate_network()

            assert "timeout" in str(exc_info.value).lower()

    async def test_security_validation_success(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test successful security validation."""
        with patch('os.path.exists') as mock_exists, \
             patch('subprocess.run') as mock_run:

            mock_exists.return_value = True
            mock_run.return_value.returncode = 0

            result = await validator._validate_security()

            assert result is True

    async def test_security_validation_failure_missing_firewall(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test security validation failure due to missing firewall."""
        with patch('os.path.exists') as mock_exists, \
             patch('subprocess.run') as mock_run:

            mock_exists.return_value = False  # Firewall config missing

            with pytest.raises(ValidationError) as exc_info:
                await validator._validate_security()

            assert "firewall" in str(exc_info.value).lower()

    async def test_full_validation_success(
        self,
        validator: EnvironmentValidator,
        mock_system_info: AsyncMock,
        mock_gpu_detector: AsyncMock
    ) -> None:
        """Test successful full environment validation."""
        with patch.object(validator, 'system_info', mock_system_info), \
             patch.object(validator, 'gpu_detector', mock_gpu_detector), \
             patch('sys.version_info', major=3, minor=12), \
             patch('subprocess.run') as mock_run, \
             patch('importlib.util.find_spec') as mock_find_spec, \
             patch('aiohttp.ClientSession.get') as mock_get, \
             patch('os.path.exists') as mock_exists:

            # Mock all successful validations
            mock_find_spec.return_value = MagicMock()
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = "Docker version 24.0.0"

            mock_response = AsyncMock()
            mock_response.status = 200
            mock_get.return_value.__aenter__.return_value = mock_response

            mock_exists.return_value = True

            result = await validator.validate_environment()

            assert result is True
            assert validator.overall_status == "valid"
            assert len(validator.validation_results) > 0

    async def test_full_validation_failure(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test full environment validation failure."""
        with patch('sys.version_info', major=3, minor=8):  # Old Python version
            with pytest.raises(SoftwareValidationError):
                await validator.validate_environment()

            assert validator.overall_status == "invalid"

    async def test_validation_report_generation(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test validation report generation."""
        validator.validation_results = [
            {"component": "CPU", "status": "pass", "details": "8 cores detected"},
            {"component": "Memory", "status": "pass", "details": "32GB available"},
            {"component": "Python", "status": "fail", "details": "Version 3.8 < 3.11"}
        ]

        report = validator.generate_report()

        assert "validation_results" in report
        assert "overall_status" in report
        assert "timestamp" in report
        assert len(report["validation_results"]) == 3

    async def test_remediation_steps_generation(
        self,
        validator: EnvironmentValidator
    ) -> None:
        """Test remediation steps generation."""
        validator.validation_results = [
            {"component": "Python", "status": "fail", "details": "Version 3.8 < 3.11"}
        ]

        remediation = validator.get_remediation_steps()

        assert "Python" in remediation
        assert "upgrade" in remediation["Python"].lower()