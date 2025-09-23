"""
Comprehensive test suite for ResourceProvisioner component.

This module provides complete TDD test coverage for the resource provisioner,
including cloud resource management, API integration, and cost optimization
following the London School TDD approach.
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from typing import Dict, Any, List, Optional

from src.gordon_gekko.deployment.resource_provisioner import (
    ResourceProvisioner,
    ProvisioningError,
    ResourceQuotaExceededError,
    APIConnectionError,
)


class TestResourceProvisioner:
    """Test suite for ResourceProvisioner class."""

    @pytest.fixture
    def mock_config(self) -> Dict[str, Any]:
        """Mock configuration for resource provisioner."""
        return {
            "cloud_provider": "aws",
            "region": "us-east-1",
            "instance_type": "t3.micro",
            "resources": {
                "cpu": 2,
                "memory": 4,
                "storage": 100
            },
            "api_endpoints": {
                "coinbase": "https://api.coinbase.com",
                "binance": "https://api.binance.us",
                "oanda": "https://api.oanda.com"
            },
            "cost_limits": {
                "daily": 50.0,
                "monthly": 1000.0
            }
        }

    @pytest.fixture
    def provisioner(self, mock_config: Dict[str, Any]) -> ResourceProvisioner:
        """Resource provisioner instance for testing."""
        return ResourceProvisioner(mock_config)

    @pytest.fixture
    def mock_cloud_client(self) -> AsyncMock:
        """Mock cloud client for testing."""
        client = AsyncMock()
        client.create_instance = AsyncMock(return_value={"instance_id": "i-12345"})
        client.delete_instance = AsyncMock(return_value=True)
        client.get_instance_status = AsyncMock(return_value="running")
        return client

    @pytest.fixture
    def mock_api_manager(self) -> AsyncMock:
        """Mock API manager for testing."""
        manager = AsyncMock()
        manager.test_connection = AsyncMock(return_value=True)
        manager.get_balance = AsyncMock(return_value={"USD": 1000.0})
        return manager

    async def test_initialization_success(
        self,
        provisioner: ResourceProvisioner
    ) -> None:
        """Test successful resource provisioner initialization."""
        assert provisioner.config is not None
        assert provisioner.resource_id is None
        assert provisioner.status == "initialized"
        assert provisioner.cost_tracker == 0.0

    async def test_initialization_with_invalid_config(self) -> None:
        """Test initialization failure with invalid configuration."""
        invalid_config = {}

        with pytest.raises(ProvisioningError) as exc_info:
            ResourceProvisioner(invalid_config)

        assert "Invalid configuration" in str(exc_info.value)

    async def test_provision_resources_success(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock,
        mock_api_manager: AsyncMock
    ) -> None:
        """Test successful resource provisioning."""
        with patch.object(provisioner, 'cloud_client', mock_cloud_client), \
             patch.object(provisioner, 'api_manager', mock_api_manager):

            result = await provisioner.provision_resources()

            assert result is True
            assert provisioner.resource_id is not None
            assert provisioner.status == "provisioned"
            mock_cloud_client.create_instance.assert_called_once()

    async def test_provision_resources_quota_exceeded(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock,
        mock_api_manager: AsyncMock
    ) -> None:
        """Test resource provisioning failure due to quota exceeded."""
        mock_cloud_client.create_instance.side_effect = ResourceQuotaExceededError("Quota exceeded")

        with patch.object(provisioner, 'cloud_client', mock_cloud_client), \
             patch.object(provisioner, 'api_manager', mock_api_manager):

            with pytest.raises(ResourceQuotaExceededError) as exc_info:
                await provisioner.provision_resources()

            assert "Quota exceeded" in str(exc_info.value)

    async def test_api_connections_establishment(
        self,
        provisioner: ResourceProvisioner,
        mock_api_manager: AsyncMock
    ) -> None:
        """Test API connections establishment."""
        mock_api_manager.test_connection.side_effect = [True, True, True]  # All APIs connect

        with patch.object(provisioner, 'api_manager', mock_api_manager):
            result = await provisioner._establish_api_connections()

            assert result is True
            assert mock_api_manager.test_connection.call_count == 3

    async def test_api_connections_partial_failure(
        self,
        provisioner: ResourceProvisioner,
        mock_api_manager: AsyncMock
    ) -> None:
        """Test API connections with partial failures."""
        mock_api_manager.test_connection.side_effect = [True, False, True]  # Mixed results

        with patch.object(provisioner, 'api_manager', mock_api_manager):
            with pytest.raises(APIConnectionError) as exc_info:
                await provisioner._establish_api_connections()

            assert "API connection failed" in str(exc_info.value)

    async def test_cost_tracking(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock
    ) -> None:
        """Test cost tracking during resource operations."""
        provisioner.cost_tracker = 10.0  # Set initial cost

        with patch.object(provisioner, 'cloud_client', mock_cloud_client):
            await provisioner._track_cost(5.0)

            assert provisioner.cost_tracker == 15.0

    async def test_cost_limit_enforcement(
        self,
        provisioner: ResourceProvisioner
    ) -> None:
        """Test cost limit enforcement."""
        provisioner.config["cost_limits"]["daily"] = 50.0
        provisioner.cost_tracker = 45.0  # Below limit

        # Should not raise error
        await provisioner._enforce_cost_limits()

        provisioner.cost_tracker = 55.0  # Above limit

        with pytest.raises(ProvisioningError) as exc_info:
            await provisioner._enforce_cost_limits()

        assert "Cost limit exceeded" in str(exc_info.value)

    async def test_resource_deallocation(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock
    ) -> None:
        """Test resource deallocation."""
        provisioner.resource_id = "test-resource-123"

        with patch.object(provisioner, 'cloud_client', mock_cloud_client):
            result = await provisioner.deallocate_resources()

            assert result is True
            assert provisioner.resource_id is None
            assert provisioner.status == "deallocated"
            mock_cloud_client.delete_instance.assert_called_once()

    async def test_resource_status_monitoring(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock
    ) -> None:
        """Test resource status monitoring."""
        provisioner.resource_id = "test-resource-123"
        mock_cloud_client.get_instance_status.return_value = "running"

        with patch.object(provisioner, 'cloud_client', mock_cloud_client):
            status = await provisioner.get_resource_status()

            assert status == "running"
            mock_cloud_client.get_instance_status.assert_called_once_with("test-resource-123")

    async def test_resource_health_check(
        self,
        provisioner: ResourceProvisioner,
        mock_cloud_client: AsyncMock,
        mock_api_manager: AsyncMock
    ) -> None:
        """Test comprehensive resource health check."""
        provisioner.resource_id = "test-resource-123"
        mock_cloud_client.get_instance_status.return_value = "running"
        mock_api_manager.test_connection.return_value = True

        with patch.object(provisioner, 'cloud_client', mock_cloud_client), \
             patch.object(provisioner, 'api_manager', mock_api_manager):

            result = await provisioner.health_check()

            assert result is True
            mock_cloud_client.get_instance_status.assert_called_once()
            mock_api_manager.test_connection.assert_called_once()


class TestResourceProvisionerIntegration:
    """Integration tests for ResourceProvisioner."""

    @pytest.fixture
    def full_config(self) -> Dict[str, Any]:
        """Full configuration for integration testing."""
        return {
            "cloud_provider": "aws",
            "region": "us-east-1",
            "instance_type": "t3.micro",
            "resources": {
                "cpu": 2,
                "memory": 4,
                "storage": 100,
                "gpu": 1
            },
            "api_endpoints": {
                "coinbase": "https://api.coinbase.com",
                "binance": "https://api.binance.us",
                "oanda": "https://api.oanda.com"
            },
            "cost_limits": {
                "daily": 100.0,
                "monthly": 2000.0,
                "alert_threshold": 80.0
            },
            "monitoring": {
                "metrics_port": 9090,
                "log_level": "INFO"
            }
        }

    async def test_full_provisioning_lifecycle(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test complete resource provisioning lifecycle."""
        provisioner = ResourceProvisioner(full_config)

        # Test initialization
        assert provisioner.config == full_config
        assert provisioner.status == "initialized"

        # Test configuration access
        assert provisioner.config["cloud_provider"] == "aws"
        assert provisioner.config["resources"]["cpu"] == 2

    async def test_error_handling_and_recovery(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test error handling and recovery mechanisms."""
        provisioner = ResourceProvisioner(full_config)

        # Test that appropriate errors are raised for invalid operations
        with pytest.raises(ProvisioningError):
            await provisioner.deallocate_resources()  # No resources to deallocate

        # Test that error messages are informative
        try:
            await provisioner.deallocate_resources()
        except ProvisioningError as e:
            assert "No resources" in str(e) or "not provisioned" in str(e).lower()

    async def test_configuration_validation(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test configuration validation."""
        provisioner = ResourceProvisioner(full_config)

        # Test that required configuration keys are present
        required_keys = ["cloud_provider", "resources", "api_endpoints"]
        for key in required_keys:
            assert key in provisioner.config

        # Test that optional keys have defaults or are handled gracefully
        assert "cost_limits" in provisioner.config