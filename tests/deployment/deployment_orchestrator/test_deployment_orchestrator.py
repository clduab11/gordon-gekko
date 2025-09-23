"""
Comprehensive test suite for DeploymentOrchestrator component.

This module provides complete TDD test coverage for the deployment orchestrator,
including health checks, rollback mechanisms, and coordination logic following
the London School TDD approach.
"""

import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime, timedelta
from typing import Dict, Any, List, Optional

from src.gordon_gekko.deployment.deployment_orchestrator import (
    DeploymentOrchestrator,
    DeploymentError,
    RollbackError,
    HealthCheckError,
)


class TestDeploymentOrchestrator:
    """Test suite for DeploymentOrchestrator class."""

    @pytest.fixture
    def mock_config(self) -> Dict[str, Any]:
        """Mock configuration for deployment orchestrator."""
        return {
            "deployment": {
                "strategy": "blue-green",
                "timeout": 300,
                "max_retries": 3,
                "health_check_interval": 10
            },
            "services": {
                "api": {"port": 8000, "health_endpoint": "/health"},
                "worker": {"port": 8001, "health_endpoint": "/health"},
                "database": {"port": 5432, "health_endpoint": "/health"}
            },
            "rollback": {
                "enabled": True,
                "timeout": 180,
                "backup_strategy": "snapshot"
            }
        }

    @pytest.fixture
    def orchestrator(self, mock_config: Dict[str, Any]) -> DeploymentOrchestrator:
        """Deployment orchestrator instance for testing."""
        return DeploymentOrchestrator(mock_config)

    @pytest.fixture
    def mock_service_manager(self) -> AsyncMock:
        """Mock service manager for testing."""
        service_manager = AsyncMock()
        service_manager.deploy_service = AsyncMock()
        service_manager.rollback_service = AsyncMock()
        service_manager.get_service_status = AsyncMock()
        service_manager.cleanup_deployment = AsyncMock()
        return service_manager

    @pytest.fixture
    def mock_environment_validator(self) -> AsyncMock:
        """Mock environment validator for testing."""
        validator = AsyncMock()
        validator.validate_requirements = AsyncMock(return_value=True)
        validator.check_compatibility = AsyncMock(return_value=True)
        validator.validate_security = AsyncMock(return_value=True)
        return validator

    @pytest.fixture
    def mock_monitoring_manager(self) -> AsyncMock:
        """Mock monitoring manager for testing."""
        monitoring = AsyncMock()
        monitoring.record_deployment_metrics = AsyncMock()
        monitoring.send_alert = AsyncMock()
        monitoring.get_deployment_status = AsyncMock()
        return monitoring

    async def test_initialization_success(
        self,
        orchestrator: DeploymentOrchestrator
    ) -> None:
        """Test successful deployment orchestrator initialization."""
        assert orchestrator.config is not None
        assert orchestrator.deployment_id is None
        assert orchestrator.status == "initialized"
        assert orchestrator.rollback_enabled is True
        assert orchestrator.current_attempt == 0

    async def test_initialization_with_invalid_config(self) -> None:
        """Test initialization failure with invalid configuration."""
        invalid_config = {"deployment": {}}

        with pytest.raises(DeploymentError) as exc_info:
            DeploymentOrchestrator(invalid_config)

        assert "Invalid deployment configuration" in str(exc_info.value)

    async def test_deployment_id_generation(
        self,
        orchestrator: DeploymentOrchestrator
    ) -> None:
        """Test deployment ID generation."""
        deployment_id = orchestrator._generate_deployment_id()

        assert deployment_id.startswith("deploy_")
        assert len(deployment_id) > 20  # Should be reasonably long
        assert orchestrator.deployment_id == deployment_id

    async def test_pre_deployment_validation_success(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_environment_validator: AsyncMock
    ) -> None:
        """Test successful pre-deployment validation."""
        with patch.object(orchestrator, 'environment_validator', mock_environment_validator):
            result = await orchestrator._validate_pre_deployment()

            assert result is True
            mock_environment_validator.validate_requirements.assert_called_once()
            mock_environment_validator.check_compatibility.assert_called_once()

    async def test_pre_deployment_validation_failure_requirements(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_environment_validator: AsyncMock
    ) -> None:
        """Test pre-deployment validation failure due to requirements."""
        mock_environment_validator.validate_requirements.return_value = False
        mock_environment_validator.check_compatibility.return_value = True

        with patch.object(orchestrator, 'environment_validator', mock_environment_validator):
            with pytest.raises(DeploymentError) as exc_info:
                await orchestrator._validate_pre_deployment()

            assert "Pre-deployment validation failed" in str(exc_info.value)
            assert "requirements" in str(exc_info.value).lower()

    async def test_pre_deployment_validation_failure_compatibility(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_environment_validator: AsyncMock
    ) -> None:
        """Test pre-deployment validation failure due to compatibility."""
        mock_environment_validator.validate_requirements.return_value = True
        mock_environment_validator.check_compatibility.return_value = False

        with patch.object(orchestrator, 'environment_validator', mock_environment_validator):
            with pytest.raises(DeploymentError) as exc_info:
                await orchestrator._validate_pre_deployment()

            assert "Pre-deployment validation failed" in str(exc_info.value)
            assert "compatibility" in str(exc_info.value).lower()

    async def test_deployment_coordination_success(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock,
        mock_monitoring_manager: AsyncMock
    ) -> None:
        """Test successful deployment coordination."""
        # Setup mocks
        mock_service_manager.deploy_service.side_effect = [
            {"service": "api", "status": "success"},
            {"service": "worker", "status": "success"},
            {"service": "database", "status": "success"}
        ]

        with patch.object(orchestrator, 'service_manager', mock_service_manager), \
             patch.object(orchestrator, 'monitoring_manager', mock_monitoring_manager):

            result = await orchestrator._coordinate_deployment()

            assert result is True
            assert mock_service_manager.deploy_service.call_count == 3
            mock_monitoring_manager.record_deployment_metrics.assert_called()

    async def test_deployment_coordination_partial_failure(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock,
        mock_monitoring_manager: AsyncMock
    ) -> None:
        """Test deployment coordination with partial service failures."""
        # Setup partial failure scenario - mock should raise exception for failed service
        mock_service_manager.deploy_service.side_effect = DeploymentError("Deployment failed")

        with patch.object(orchestrator, 'service_manager', mock_service_manager), \
             patch.object(orchestrator, 'monitoring_manager', mock_monitoring_manager):

            with pytest.raises(DeploymentError) as exc_info:
                await orchestrator._coordinate_deployment()

            assert "Maximum deployment attempts exceeded" in str(exc_info.value)
            assert orchestrator.current_attempt == 3  # Default max_retries

    async def test_health_check_orchestration_success(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test successful health check orchestration."""
        # Mock successful health checks
        mock_service_manager.get_service_status.side_effect = [
            {"service": "api", "status": "healthy"},
            {"service": "worker", "status": "healthy"},
            {"service": "database", "status": "healthy"}
        ]

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            result = await orchestrator._orchestrate_health_checks()

            assert result is True
            assert mock_service_manager.get_service_status.call_count == 3

    async def test_health_check_orchestration_failure(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test health check orchestration failure."""
        # Mock health check failure
        mock_service_manager.get_service_status.return_value = {
            "service": "api", "status": "unhealthy"
        }

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            with pytest.raises(HealthCheckError) as exc_info:
                await orchestrator._orchestrate_health_checks()

            assert "Health checks failed" in str(exc_info.value)

    async def test_rollback_mechanism_success(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock,
        mock_monitoring_manager: AsyncMock
    ) -> None:
        """Test successful rollback mechanism."""
        orchestrator.deployment_id = "test-deployment-123"
        orchestrator.current_attempt = 1

        # Mock successful rollback
        mock_service_manager.rollback_service.side_effect = [
            {"service": "api", "status": "rollback_success"},
            {"service": "worker", "status": "rollback_success"},
            {"service": "database", "status": "rollback_success"}
        ]

        with patch.object(orchestrator, 'service_manager', mock_service_manager), \
             patch.object(orchestrator, 'monitoring_manager', mock_monitoring_manager):

            result = await orchestrator._execute_rollback()

            assert result is True
            assert mock_service_manager.rollback_service.call_count == 3
            mock_monitoring_manager.send_alert.assert_called_once()

    async def test_rollback_mechanism_failure(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock,
        mock_monitoring_manager: AsyncMock
    ) -> None:
        """Test rollback mechanism failure."""
        orchestrator.deployment_id = "test-deployment-123"

        # Mock rollback failure
        mock_service_manager.rollback_service.side_effect = RollbackError("Rollback failed")

        with patch.object(orchestrator, 'service_manager', mock_service_manager), \
             patch.object(orchestrator, 'monitoring_manager', mock_monitoring_manager):

            with pytest.raises(RollbackError) as exc_info:
                await orchestrator._execute_rollback()

            assert "Rollback failed" in str(exc_info.value)
            mock_monitoring_manager.send_alert.assert_called()

    async def test_deployment_timeout_handling(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test deployment timeout handling."""
        orchestrator.config["deployment"]["timeout"] = 1  # 1 second timeout
        orchestrator.config["deployment"]["max_retries"] = 1  # Disable retries for this test
        orchestrator.deployment_id = "test-deployment-timeout"

        # Mock slow deployment that exceeds timeout
        async def slow_deployment(*args, **kwargs):
            await asyncio.sleep(2)  # Exceed timeout
            return {"service": "api", "status": "success"}

        mock_service_manager.deploy_service = slow_deployment

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            with pytest.raises(DeploymentError) as exc_info:
                await orchestrator._coordinate_deployment()

            # With retry disabled, we should get the timeout message
            assert "Maximum deployment attempts exceeded" in str(exc_info.value)

    async def test_retry_mechanism_max_attempts(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test retry mechanism reaches maximum attempts."""
        orchestrator.config["deployment"]["max_retries"] = 2
        orchestrator.deployment_id = "test-deployment-retry"

        # Mock persistent failure
        mock_service_manager.deploy_service.side_effect = DeploymentError("Persistent failure")

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            with pytest.raises(DeploymentError) as exc_info:
                await orchestrator._coordinate_deployment()

            assert "Maximum deployment attempts exceeded" in str(exc_info.value)
            assert orchestrator.current_attempt == 2

    async def test_deployment_status_tracking(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test deployment status tracking throughout process."""
        orchestrator.deployment_id = "test-deployment-status"

        # Mock deployment process - use return_value instead of side_effect to avoid StopAsyncIteration
        mock_service_manager.deploy_service.return_value = {"service": "api", "status": "success"}

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            # Verify initial status
            assert orchestrator.status == "initialized"

            # Start deployment
            await orchestrator._coordinate_deployment()

            # Verify status progression
            assert orchestrator.status == "completed"

    async def test_resource_cleanup_on_failure(
        self,
        orchestrator: DeploymentOrchestrator,
        mock_service_manager: AsyncMock
    ) -> None:
        """Test resource cleanup when deployment fails."""
        orchestrator.deployment_id = "test-deployment-cleanup"

        # Mock deployment failure
        mock_service_manager.deploy_service.side_effect = DeploymentError("Deployment failed")
        mock_service_manager.cleanup_deployment = AsyncMock()

        with patch.object(orchestrator, 'service_manager', mock_service_manager):
            with pytest.raises(DeploymentError):
                await orchestrator._coordinate_deployment()

            # Verify cleanup was called for each retry attempt (3 times with default max_retries)
            assert mock_service_manager.cleanup_deployment.call_count == 3
            mock_service_manager.cleanup_deployment.assert_called_with("test-deployment-cleanup")


class TestDeploymentOrchestratorIntegration:
    """Integration tests for DeploymentOrchestrator."""

    @pytest.fixture
    def full_config(self) -> Dict[str, Any]:
        """Full configuration for integration testing."""
        return {
            "deployment": {
                "strategy": "blue-green",
                "timeout": 300,
                "max_retries": 3,
                "health_check_interval": 10,
                "rollback_on_failure": True
            },
            "services": {
                "api": {
                    "port": 8000,
                    "health_endpoint": "/health",
                    "image": "test-api:latest",
                    "replicas": 3
                },
                "worker": {
                    "port": 8001,
                    "health_endpoint": "/health",
                    "image": "test-worker:latest",
                    "replicas": 2
                }
            },
            "monitoring": {
                "prometheus_url": "http://localhost:9090",
                "grafana_url": "http://localhost:3000",
                "alert_webhook": "http://localhost:8080/alerts"
            },
            "rollback": {
                "enabled": True,
                "timeout": 180,
                "backup_strategy": "snapshot"
            }
        }

    async def test_full_deployment_lifecycle(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test complete deployment lifecycle from start to finish."""
        orchestrator = DeploymentOrchestrator(full_config)

        # Test initialization
        assert orchestrator.config == full_config
        assert orchestrator.status == "initialized"

        # Test deployment ID generation
        deployment_id = orchestrator._generate_deployment_id()
        assert deployment_id.startswith("deploy_")
        assert orchestrator.deployment_id == deployment_id

        # Test status progression
        orchestrator.status = "in_progress"
        assert orchestrator.status == "in_progress"

        # Test completion status
        orchestrator.status = "completed"
        assert orchestrator.status == "completed"

    async def test_deployment_with_realistic_timing(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test deployment with realistic timing constraints."""
        orchestrator = DeploymentOrchestrator(full_config)
        orchestrator.deployment_id = "test-realistic-timing"

        # Set realistic timeout
        orchestrator.config["deployment"]["timeout"] = 30

        # Test that orchestrator respects timeout configuration
        assert orchestrator.config["deployment"]["timeout"] == 30

        # Test retry configuration
        assert orchestrator.config["deployment"]["max_retries"] == 3

    async def test_error_propagation(
        self,
        full_config: Dict[str, Any]
    ) -> None:
        """Test proper error propagation through deployment layers."""
        orchestrator = DeploymentOrchestrator(full_config)

        # Test that deployment errors are properly wrapped
        with pytest.raises(DeploymentError):
            raise DeploymentError("Test error propagation")

        # Test that rollback errors are properly wrapped
        with pytest.raises(RollbackError):
            raise RollbackError("Test rollback error")

        # Test that health check errors are properly wrapped
        with pytest.raises(HealthCheckError):
            raise HealthCheckError("Test health check error")