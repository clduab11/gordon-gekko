"""
Comprehensive test suite for IntegrationTester component.
Tests component integration, dependency validation, and system health verification.
"""
import pytest
from unittest.mock import Mock, patch, MagicMock
from dataclasses import dataclass
from typing import Dict, List, Optional, Any
import asyncio
from datetime import datetime

from src.gordon_gekko.deployment.integration_tester import (
    IntegrationTester,
    IntegrationTestResult,
    ComponentDependency,
    IntegrationError,
    DependencyValidationError,
    HealthCheckError
)


class TestIntegrationTesterInitialization:
    """Test IntegrationTester initialization and setup."""

    def test_successful_initialization(self):
        """Test successful IntegrationTester initialization."""
        # Given
        config = {"test_timeout": 30, "retry_attempts": 3}
        dependencies = []

        # When
        tester = IntegrationTester(config, dependencies)

        # Then
        assert tester.config == config
        assert tester.dependencies == dependencies
        assert tester.test_timeout == 30
        assert tester.retry_attempts == 3

    def test_initialization_with_dependencies(self):
        """Test initialization with component dependencies."""
        # Given
        config = {"test_timeout": 60}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0"),
            ComponentDependency("api", "TradingAPI", "2.1.0")
        ]

        # When
        tester = IntegrationTester(config, dependencies)

        # Then
        assert len(tester.dependencies) == 2
        assert tester.dependencies[0].name == "database"
        assert tester.dependencies[0].component == "DatabaseManager"
        assert tester.dependencies[0].version == "1.0.0"

    def test_initialization_with_none_config(self):
        """Test initialization with None config raises error."""
        # Given
        config = None
        dependencies = []

        # When/Then
        with pytest.raises(IntegrationError):
            IntegrationTester(config, dependencies)

    def test_initialization_with_invalid_config_type(self):
        """Test initialization with invalid config type."""
        # Given
        config = "invalid_config"
        dependencies = []

        # When/Then
        with pytest.raises(IntegrationError):
            IntegrationTester(config, dependencies)


class TestComponentIntegration:
    """Test component integration functionality."""

    def test_validate_component_dependencies_success(self):
        """Test successful component dependency validation."""
        # Given
        config = {"test_timeout": 30}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0"),
            ComponentDependency("cache", "CacheManager", "1.2.0")
        ]
        tester = IntegrationTester(config, dependencies)

        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "healthy"}
        mock_cache_manager = Mock()
        mock_cache_manager.health_check.return_value = {"status": "healthy"}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager),
            'src.gordon_gekko.infrastructure.cache.manager': Mock(CacheManager=mock_cache_manager)
        }):
            result = tester.validate_component_dependencies()

        # Then
        assert result.success is True
        assert len(result.details) == 2
        mock_db_manager.health_check.assert_called_once()
        mock_cache_manager.health_check.assert_called_once()

    def test_validate_component_dependencies_missing_component(self):
        """Test dependency validation with missing component."""
        # Given
        config = {"test_timeout": 30}
        dependencies = [
            ComponentDependency("nonexistent", "MissingComponent", "1.0.0")
        ]
        tester = IntegrationTester(config, dependencies)

        # When
        result = tester.validate_component_dependencies()

        # Then
        assert result.success is False
        assert result.error_code == "COMPONENT_NOT_FOUND"
        assert "MissingComponent" in result.details[0].message

    def test_validate_component_dependencies_unhealthy_component(self):
        """Test dependency validation with unhealthy component."""
        # Given
        config = {"test_timeout": 30}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0")
        ]
        tester = IntegrationTester(config, dependencies)

        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "unhealthy"}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager)
        }):
            result = tester.validate_component_dependencies()

        # Then
        assert result.success is False
        assert result.error_code == "COMPONENT_UNHEALTHY"
        mock_db_manager.health_check.assert_called_once()

    def test_validate_component_dependencies_with_timeout(self):
        """Test dependency validation with timeout."""
        # Given
        config = {"test_timeout": 1}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0")
        ]
        tester = IntegrationTester(config, dependencies)

        mock_db_manager = Mock()
        mock_db_manager.health_check.side_effect = asyncio.TimeoutError()

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager)
        }):
            result = tester.validate_component_dependencies()

        # Then
        assert result.success is False
        assert result.error_code == "VALIDATION_TIMEOUT"
        assert "timeout" in result.details[0].message.lower()


class TestIntegrationTestResult:
    """Test IntegrationTestResult data class."""

    def test_integration_test_result_creation(self):
        """Test IntegrationTestResult creation."""
        # Given
        details = [{"component": "database", "status": "healthy"}]

        # When
        result = IntegrationTestResult(
            success=True,
            details=details,
            timestamp=datetime.now(),
            error_code=None,
            message="All tests passed"
        )

        # Then
        assert result.success is True
        assert result.details == details
        assert result.error_code is None
        assert result.message == "All tests passed"

    def test_integration_test_result_with_error(self):
        """Test IntegrationTestResult with error information."""
        # Given
        error_details = [{"component": "database", "error": "Connection failed"}]

        # When
        result = IntegrationTestResult(
            success=False,
            details=error_details,
            timestamp=datetime.now(),
            error_code="CONNECTION_ERROR",
            message="Database connection failed"
        )

        # Then
        assert result.success is False
        assert result.details == error_details
        assert result.error_code == "CONNECTION_ERROR"
        assert result.message == "Database connection failed"

    def test_integration_test_result_defaults(self):
        """Test IntegrationTestResult with default values."""
        # When
        result = IntegrationTestResult(success=True, details=[])

        # Then
        assert result.success is True
        assert result.details == []
        assert result.error_code is None
        assert result.message is None


class TestSystemHealthVerification:
    """Test system health verification functionality."""

    def test_full_system_health_check_success(self):
        """Test successful full system health check."""
        # Given
        config = {"test_timeout": 30, "enable_detailed_checks": True}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0"),
            ComponentDependency("cache", "CacheManager", "1.2.0"),
            ComponentDependency("api", "TradingAPI", "2.1.0")
        ]
        tester = IntegrationTester(config, dependencies)

        # Mock all components as healthy
        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "healthy", "latency": 10}

        mock_cache_manager = Mock()
        mock_cache_manager.health_check.return_value = {"status": "healthy", "hit_rate": 0.95}

        mock_api_manager = Mock()
        mock_api_manager.health_check.return_value = {"status": "healthy", "endpoints": 15}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager),
            'src.gordon_gekko.infrastructure.cache.manager': Mock(CacheManager=mock_cache_manager),
            'src.gordon_gekko.infrastructure.messaging.manager': Mock(TradingAPI=mock_api_manager)
        }):
            result = tester.perform_full_system_health_check()

        # Then
        assert result.success is True
        assert len(result.details) == 3
        assert result.details[0]["status"] == "healthy"

    def test_full_system_health_check_with_failures(self):
        """Test system health check with component failures."""
        # Given
        config = {"test_timeout": 30}
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0"),
            ComponentDependency("api", "TradingAPI", "2.1.0")
        ]
        tester = IntegrationTester(config, dependencies)

        # Mock mixed health status
        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "unhealthy", "error": "Connection timeout"}

        mock_api_manager = Mock()
        mock_api_manager.health_check.return_value = {"status": "healthy", "version": "2.1.0"}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager),
            'src.gordon_gekko.infrastructure.messaging.manager': Mock(TradingAPI=mock_api_manager)
        }):
            result = tester.perform_full_system_health_check()

        # Then
        assert result.success is False
        assert len(result.details) == 2
        assert result.details[0]["status"] == "unhealthy"
        assert result.details[1]["status"] == "healthy"

    def test_health_check_with_retries(self):
        """Test health check with retry mechanism."""
        # Given
        config = {"test_timeout": 30, "retry_attempts": 2, "retry_delay": 0.1}
        dependencies = [ComponentDependency("database", "DatabaseManager", "1.0.0")]
        tester = IntegrationTester(config, dependencies)

        mock_db_manager = Mock()
        # Fail first two times, succeed on third
        mock_db_manager.health_check.side_effect = [
            {"status": "unhealthy"},
            {"status": "unhealthy"},
            {"status": "healthy"}
        ]

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager)
        }):
            result = tester.perform_full_system_health_check()

        # Then
        assert result.success is True
        assert mock_db_manager.health_check.call_count == 3


class TestAPIIntegrationTesting:
    """Test API integration functionality."""

    def test_api_endpoint_connectivity_success(self):
        """Test successful API endpoint connectivity."""
        # Given
        config = {"test_timeout": 30, "api_endpoints": ["https://api.example.com/v1"]}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "ok"}

        # When
        with patch('requests.get') as mock_get:
            mock_get.return_value = mock_response
            result = tester.test_api_endpoint_connectivity("https://api.example.com/v1")

        # Then
        assert result.success is True
        assert result.details["status_code"] == 200
        mock_get.assert_called_once_with("https://api.example.com/v1", timeout=30)

    def test_api_endpoint_connectivity_failure(self):
        """Test failed API endpoint connectivity."""
        # Given
        config = {"test_timeout": 30}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        # When
        with patch('requests.get') as mock_get:
            mock_get.side_effect = Exception("Connection timeout")
            result = tester.test_api_endpoint_connectivity("https://api.example.com/v1")

        # Then
        assert result.success is False
        assert "Connection timeout" in result.details["error"]

    def test_api_endpoint_connectivity_with_auth(self):
        """Test API connectivity with authentication."""
        # Given
        config = {"test_timeout": 30, "api_key": "test_key_123"}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        mock_response = Mock()
        mock_response.status_code = 401  # Unauthorized

        # When
        with patch('requests.get') as mock_get:
            mock_get.return_value = mock_response
            result = tester.test_api_endpoint_connectivity(
                "https://api.example.com/v1",
                headers={"Authorization": "Bearer test_key_123"}
            )

        # Then
        assert result.success is False
        assert result.details["status_code"] == 401
        mock_get.assert_called_once_with(
            "https://api.example.com/v1",
            timeout=30,
            headers={"Authorization": "Bearer test_key_123"}
        )


class TestDatabaseConnectivity:
    """Test database connectivity functionality."""

    def test_database_connection_success(self):
        """Test successful database connection."""
        # Given
        config = {"test_timeout": 30}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        mock_connection = Mock()
        mock_connection.execute.return_value = "success"

        # When
        with patch('psycopg2.connect') as mock_connect:
            mock_connect.return_value = mock_connection
            result = tester.test_database_connectivity(
                "postgresql://user:pass@localhost:5432/testdb"
            )

        # Then
        assert result.success is True
        assert "connection_established" in result.details
        mock_connect.assert_called_once()

    def test_database_connection_failure(self):
        """Test failed database connection."""
        # Given
        config = {"test_timeout": 30}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        # When
        with patch('psycopg2.connect') as mock_connect:
            mock_connect.side_effect = Exception("Connection refused")
            result = tester.test_database_connectivity(
                "postgresql://user:pass@localhost:5432/testdb"
            )

        # Then
        assert result.success is False
        assert "Connection refused" in result.details["error"]

    def test_database_query_execution(self):
        """Test database query execution."""
        # Given
        config = {"test_timeout": 30}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        mock_connection = Mock()
        mock_cursor = Mock()
        mock_cursor.execute.return_value = None
        mock_cursor.fetchone.return_value = ("test_result",)
        mock_connection.cursor.return_value = mock_cursor

        # When
        with patch('psycopg2.connect') as mock_connect:
            mock_connect.return_value = mock_connection
            result = tester.test_database_query_execution(
                "postgresql://user:pass@localhost:5432/testdb",
                "SELECT 1 as test_column"
            )

        # Then
        assert result.success is True
        assert result.details["query_result"] == ("test_result",)
        mock_cursor.execute.assert_called_once_with("SELECT 1 as test_column")


class TestConfigurationValidation:
    """Test configuration validation functionality."""

    def test_validate_environment_config_success(self):
        """Test successful environment configuration validation."""
        # Given
        config = {
            "test_timeout": 30,
            "required_env_vars": ["DATABASE_URL", "API_KEY", "CACHE_HOST"]
        }
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        # When
        with patch.dict('os.environ', {
            'DATABASE_URL': 'postgresql://localhost:5432/test',
            'API_KEY': 'test_key_123',
            'CACHE_HOST': 'localhost:6379'
        }):
            result = tester.validate_environment_config()

        # Then
        assert result.success is True
        assert len(result.details) == 3
        assert all(detail["status"] == "present" for detail in result.details)

    def test_validate_environment_config_missing_vars(self):
        """Test environment config validation with missing variables."""
        # Given
        config = {
            "test_timeout": 30,
            "required_env_vars": ["DATABASE_URL", "API_KEY", "MISSING_VAR"]
        }
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        # When
        with patch.dict('os.environ', {
            'DATABASE_URL': 'postgresql://localhost:5432/test',
            'API_KEY': 'test_key_123'
            # MISSING_VAR is not set
        }):
            result = tester.validate_environment_config()

        # Then
        assert result.success is False
        assert len(result.details) == 3
        assert result.details[0]["status"] == "present"
        assert result.details[1]["status"] == "present"
        assert result.details[2]["status"] == "missing"


class TestErrorHandlingAndRecovery:
    """Test error handling and recovery functionality."""

    def test_retry_mechanism_success(self):
        """Test retry mechanism with eventual success."""
        # Given
        config = {"test_timeout": 30, "retry_attempts": 3, "retry_delay": 0.1}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        call_count = 0
        def side_effect():
            nonlocal call_count
            call_count += 1
            if call_count < 3:
                raise Exception(f"Attempt {call_count} failed")
            return "success"

        # When
        result = tester._execute_with_retry(side_effect, "test_operation")

        # Then
        assert result == "success"
        assert call_count == 3

    def test_retry_mechanism_exhaustion(self):
        """Test retry mechanism when all attempts fail."""
        # Given
        config = {"test_timeout": 30, "retry_attempts": 2, "retry_delay": 0.1}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        def side_effect():
            raise Exception("Always fails")

        # When/Then
        with pytest.raises(IntegrationError) as exc_info:
            tester._execute_with_retry(side_effect, "test_operation")

        assert "retry attempts exhausted" in str(exc_info.value).lower()
        assert "test_operation" in str(exc_info.value)

    def test_timeout_handling(self):
        """Test timeout handling in integration tests."""
        # Given
        config = {"test_timeout": 1}
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        async def long_running_operation():
            await asyncio.sleep(2)  # Longer than timeout
            return "completed"

        # When/Then
        with pytest.raises(asyncio.TimeoutError):
            asyncio.run(asyncio.wait_for(
                tester._execute_with_timeout(long_running_operation(), 1),
                timeout=1
            ))


class TestIntegrationScenarios:
    """Test complex integration scenarios."""

    def test_full_integration_test_success(self):
        """Test full integration test with all components healthy."""
        # Given
        config = {
            "test_timeout": 30,
            "retry_attempts": 2,
            "required_env_vars": ["DATABASE_URL", "API_KEY"],
            "api_endpoints": ["https://api.example.com/v1"],
            "enable_detailed_checks": True
        }
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0"),
            ComponentDependency("cache", "CacheManager", "1.2.0")
        ]
        tester = IntegrationTester(config, dependencies)

        # Mock all components and external services as healthy
        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "healthy"}

        mock_cache_manager = Mock()
        mock_cache_manager.health_check.return_value = {"status": "healthy"}

        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"status": "ok"}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager),
            'src.gordon_gekko.infrastructure.cache.manager': Mock(CacheManager=mock_cache_manager)
        }), patch.dict('os.environ', {
            'DATABASE_URL': 'postgresql://localhost:5432/test',
            'API_KEY': 'test_key_123'
        }), patch('requests.get') as mock_get:
            mock_get.return_value = mock_response
            result = tester.run_full_integration_test()

        # Then
        assert result.success is True
        assert len(result.details) > 0
        assert "database" in result.details
        assert "api" in result.details
        assert "environment" in result.details

    def test_full_integration_test_with_failures(self):
        """Test full integration test with multiple failures."""
        # Given
        config = {
            "test_timeout": 30,
            "required_env_vars": ["MISSING_VAR"],
            "api_endpoints": ["https://invalid-url"]
        }
        dependencies = [
            ComponentDependency("database", "DatabaseManager", "1.0.0")
        ]
        tester = IntegrationTester(config, dependencies)

        # Mock database as unhealthy
        mock_db_manager = Mock()
        mock_db_manager.health_check.return_value = {"status": "unhealthy"}

        # When
        with patch.dict('sys.modules', {
            'src.gordon_gekko.core.database_manager': Mock(DatabaseManager=mock_db_manager)
        }), patch('requests.get') as mock_get:
            mock_get.side_effect = Exception("Connection failed")
            result = tester.run_full_integration_test()

        # Then
        assert result.success is False
        assert "database" in result.details
        assert "api" in result.details
        assert "environment" in result.details

    def test_performance_monitoring_integration(self):
        """Test integration with performance monitoring."""
        # Given
        config = {
            "test_timeout": 30,
            "enable_performance_monitoring": True,
            "performance_thresholds": {
                "max_response_time": 100,
                "min_throughput": 1000
            }
        }
        dependencies = []
        tester = IntegrationTester(config, dependencies)

        # When
        result = tester.run_performance_integration_test()

        # Then
        assert result.success is True  # Should pass with default metrics
        assert "response_time" in result.details
        assert "throughput" in result.details
        assert "memory_usage" in result.details