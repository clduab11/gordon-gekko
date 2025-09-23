"""
Deployment orchestrator for Gordon Gekko trading system.

This module provides comprehensive deployment orchestration capabilities including
health checks, rollback mechanisms, and coordination logic for production deployments.
"""

import uuid
import asyncio
from datetime import datetime
from typing import Dict, Any, Optional
from contextlib import asynccontextmanager


class DeploymentError(Exception):
    """Exception raised for deployment-related errors."""
    pass


class RollbackError(Exception):
    """Exception raised for rollback-related errors."""
    pass


class HealthCheckError(Exception):
    """Exception raised for health check-related errors."""
    pass


class DeploymentOrchestrator:
    """
    Orchestrates deployment operations with health checks and rollback capabilities.

    This class manages the complete deployment lifecycle including pre-deployment
    validation, service coordination, health monitoring, and rollback procedures.
    """

    def __init__(self, config: Dict[str, Any]) -> None:
        """
        Initialize deployment orchestrator with configuration.

        Args:
            config: Configuration dictionary containing deployment parameters

        Raises:
            DeploymentError: If configuration is invalid
        """
        if not config or not config.get("deployment"):
            raise DeploymentError("Invalid deployment configuration")

        self.config = config
        self.deployment_id: Optional[str] = None
        self.status = "initialized"
        self.rollback_enabled = config.get("rollback", {}).get("enabled", True)
        self.current_attempt = 0

        # Initialize dependencies (will be set by dependency injection)
        self.service_manager = None
        self.environment_validator = None
        self.monitoring_manager = None

    def _update_status(self, new_status: str) -> None:
        """
        Update deployment status and track progression.

        Args:
            new_status: New status to set
        """
        self.status = new_status

    async def _cleanup_deployment_resources(self) -> None:
        """
        Clean up deployment resources on failure.
        """
        if self.service_manager and self.deployment_id:
            await self.service_manager.cleanup_deployment(self.deployment_id)

    @asynccontextmanager
    async def _timeout_context(self, timeout_seconds: int):
        """
        Async context manager for timeout handling.

        Args:
            timeout_seconds: Maximum time to wait before raising timeout error
        """
        try:
            # Create a task that will be cancelled after timeout
            yield
        except asyncio.TimeoutError:
            raise DeploymentError(f"Deployment timeout after {timeout_seconds} seconds")
        except Exception as e:
            # Re-raise any other exceptions as-is
            raise e

    async def _deploy_with_timeout(self, timeout_seconds: int):
        """
        Deploy all services with timeout handling.

        Args:
            timeout_seconds: Maximum time to wait for deployment

        Raises:
            DeploymentError: If deployment times out or fails
        """
        async with self._timeout_context(timeout_seconds):
            services = self.config.get("services", {})
            deployment_task = asyncio.create_task(self._deploy_all_services(services))
            monitoring_task = asyncio.create_task(self._monitor_deployment_progress())

            try:
                await asyncio.wait_for(deployment_task, timeout=timeout_seconds)
            except asyncio.TimeoutError:
                monitoring_task.cancel()
                raise DeploymentError(f"Deployment timeout after {timeout_seconds} seconds")
            finally:
                monitoring_task.cancel()

    async def _deploy_all_services(self, services: dict) -> None:
        """Deploy all services sequentially."""
        for service_name in services:
            await self.service_manager.deploy_service(service_name)

    async def _monitor_deployment_progress(self) -> None:
        """Monitor deployment progress and provide status updates."""
        # Simple monitoring - in real implementation would track detailed progress
        await asyncio.sleep(0.1)  # Small delay to simulate monitoring

    def _generate_deployment_id(self) -> str:
        """
        Generate a unique deployment identifier.

        Returns:
            str: Unique deployment ID with timestamp
        """
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        unique_id = str(uuid.uuid4())[:8]
        self.deployment_id = f"deploy_{timestamp}_{unique_id}"
        return self.deployment_id

    async def _validate_pre_deployment(self) -> bool:
        """
        Validate system requirements before deployment.

        Returns:
            bool: True if validation passes

        Raises:
            DeploymentError: If validation fails
        """
        if not self.environment_validator:
            raise DeploymentError("Environment validator not configured")

        # Validate requirements
        if not await self.environment_validator.validate_requirements():
            raise DeploymentError("Pre-deployment validation failed: requirements")

        # Check compatibility
        if not await self.environment_validator.check_compatibility():
            raise DeploymentError("Pre-deployment validation failed: compatibility")

        return True

    async def _coordinate_deployment(self) -> bool:
        """
        Coordinate deployment across all services with retry logic.

        Returns:
            bool: True if deployment succeeds

        Raises:
            DeploymentError: If deployment fails after all retry attempts
        """
        if not self.service_manager:
            raise DeploymentError("Service manager not configured")

        # Get configuration
        timeout = self.config.get("deployment", {}).get("timeout", 300)
        max_retries = self.config.get("deployment", {}).get("max_retries", 3)

        # Attempt deployment with retries
        for attempt in range(max_retries):
            self.current_attempt = attempt + 1

            try:
                # Update status to in progress
                self._update_status("in_progress")

                # Deploy all services with timeout
                await self._deploy_with_timeout(timeout)

                # Update status to completed
                self._update_status("completed")

                # Record metrics
                if self.monitoring_manager:
                    await self.monitoring_manager.record_deployment_metrics()

                return True

            except Exception as e:
                # Update status to failed
                self._update_status("failed")

                # Clean up resources on failure
                await self._cleanup_deployment_resources()

                # If this is the last attempt, raise the error
                if attempt == max_retries - 1:
                    raise DeploymentError("Maximum deployment attempts exceeded") from e
                else:
                    # Continue to next attempt
                    continue

        # This should never be reached, but just in case
        raise DeploymentError("Maximum deployment attempts exceeded")

    async def deploy(self) -> bool:
        """
        Public method to initiate deployment with retry logic.

        Returns:
            bool: True if deployment succeeds

        Raises:
            DeploymentError: If deployment fails after all retry attempts
        """
        try:
            # Reset attempt counter for new deployment
            self.current_attempt = 0
            self.deployment_id = self._generate_deployment_id()

            # Validate pre-deployment requirements
            await self._validate_pre_deployment()

            # Attempt deployment with retries
            while self.current_attempt < self.config.get("deployment", {}).get("max_retries", 3):
                try:
                    return await self._coordinate_deployment()
                except Exception as retry_exception:
                    self.current_attempt += 1
                    if self.current_attempt >= self.config.get("deployment", {}).get("max_retries", 3):
                        raise DeploymentError("Maximum deployment attempts exceeded") from retry_exception
                    # Continue to next attempt

            # This should never be reached, but just in case
            raise DeploymentError("Maximum deployment attempts exceeded")

        except DeploymentError:
            # Re-raise deployment errors as-is
            raise
        except Exception as e:
            # Wrap other exceptions
            raise DeploymentError(f"Deployment failed: {str(e)}") from e

    async def _orchestrate_health_checks(self) -> bool:
        """
        Orchestrate health checks across all deployed services.

        Returns:
            bool: True if all health checks pass

        Raises:
            HealthCheckError: If health checks fail
        """
        if not self.service_manager:
            raise HealthCheckError("Service manager not configured")

        services = self.config.get("services", {})
        for service_name in services:
            status = await self.service_manager.get_service_status(service_name)
            if status.get("status") != "healthy":
                raise HealthCheckError("Health checks failed")

        return True

    async def _execute_rollback(self) -> bool:
        """
        Execute rollback procedure for failed deployment.

        Returns:
            bool: True if rollback succeeds

        Raises:
            RollbackError: If rollback fails
        """
        if not self.rollback_enabled:
            raise RollbackError("Rollback not enabled")

        if not self.service_manager:
            raise RollbackError("Service manager not configured")

        if not self.deployment_id:
            raise RollbackError("No deployment ID available for rollback")

        # Execute rollback for all services
        services = self.config.get("services", {})
        try:
            for service_name in services:
                await self.service_manager.rollback_service(service_name)
        except Exception as e:
            # Send alert on rollback failure
            if self.monitoring_manager:
                await self.monitoring_manager.send_alert()
            raise RollbackError(f"Rollback failed: {str(e)}")

        # Send alert on successful rollback
        if self.monitoring_manager:
            await self.monitoring_manager.send_alert()

        return True