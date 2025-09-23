"""
Resource provisioner for Gordon Gekko deployment system.

This module provides comprehensive resource provisioning capabilities including
cloud resource management, API integration, and cost optimization for the
Gordon Gekko autonomous trading system deployment.
"""

from typing import Dict, Any, Optional
import uuid


class ProvisioningError(Exception):
    """Exception raised for provisioning-related errors."""
    pass


class ResourceQuotaExceededError(ProvisioningError):
    """Exception raised when resource quotas are exceeded."""
    pass


class APIConnectionError(ProvisioningError):
    """Exception raised for API connection errors."""
    pass


class ResourceProvisioner:
    """
    Resource provisioner for managing cloud resources and API connections.

    This class handles resource provisioning, cost tracking, and lifecycle
    management for the Gordon Gekko trading system deployment.
    """

    def __init__(self, config: Dict[str, Any]) -> None:
        """
        Initialize resource provisioner with configuration.

        Args:
            config: Configuration dictionary containing provisioning parameters

        Raises:
            ProvisioningError: If configuration is invalid
        """
        if not config or not isinstance(config, dict):
            raise ProvisioningError("Invalid configuration")

        self.config = config
        self.resource_id: Optional[str] = None
        self.status = "initialized"
        self.cost_tracker = 0.0

        # Dependencies will be injected later
        self.cloud_client = None
        self.api_manager = None

    async def provision_resources(self) -> bool:
        """
        Provision cloud resources and establish API connections.

        Returns:
            bool: True if provisioning succeeds

        Raises:
            ProvisioningError: If provisioning fails
        """
        if not self.cloud_client or not self.api_manager:
            raise ProvisioningError("Dependencies not configured")

        # Establish API connections first
        await self._establish_api_connections()

        # Provision cloud resources
        self.resource_id = str(uuid.uuid4())
        await self.cloud_client.create_instance()

        # Track costs
        await self._track_cost(10.0)

        # Enforce cost limits
        await self._enforce_cost_limits()

        self.status = "provisioned"
        return True

    async def _establish_api_connections(self) -> bool:
        """
        Establish connections to all required APIs.

        Returns:
            bool: True if all connections succeed

        Raises:
            APIConnectionError: If any API connection fails
        """
        if not self.api_manager:
            raise APIConnectionError("API manager not configured")

        api_endpoints = self.config.get("api_endpoints", {})
        for api_name in api_endpoints:
            if not await self.api_manager.test_connection():
                raise APIConnectionError(f"API connection failed for {api_name}")

        return True

    async def _track_cost(self, cost: float) -> None:
        """
        Track resource costs.

        Args:
            cost: Cost amount to add to tracker
        """
        self.cost_tracker += cost

    async def _enforce_cost_limits(self) -> None:
        """
        Enforce cost limits to prevent budget overruns.

        Raises:
            ProvisioningError: If cost limits are exceeded
        """
        cost_limits = self.config.get("cost_limits", {})
        daily_limit = cost_limits.get("daily", float('inf'))

        if self.cost_tracker > daily_limit:
            raise ProvisioningError("Cost limit exceeded")

    async def deallocate_resources(self) -> bool:
        """
        Deallocate provisioned resources.

        Returns:
            bool: True if deallocation succeeds

        Raises:
            ProvisioningError: If deallocation fails
        """
        if not self.resource_id:
            raise ProvisioningError("No resources to deallocate")

        if self.cloud_client:
            await self.cloud_client.delete_instance()

        self.resource_id = None
        self.status = "deallocated"
        return True

    async def get_resource_status(self) -> str:
        """
        Get current resource status.

        Returns:
            str: Current status of resources
        """
        if not self.resource_id or not self.cloud_client:
            return "not_provisioned"

        return await self.cloud_client.get_instance_status(self.resource_id)

    async def health_check(self) -> bool:
        """
        Perform comprehensive health check.

        Returns:
            bool: True if all components are healthy
        """
        if not self.resource_id:
            return False

        # Check cloud resource status
        cloud_status = await self.cloud_client.get_instance_status()
        if cloud_status != "running":
            return False

        # Check API connections
        if self.api_manager and not await self.api_manager.test_connection():
            return False

        return True