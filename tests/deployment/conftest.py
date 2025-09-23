"""
Comprehensive test configuration for Gordon Gekko deployment validation.

This module provides shared fixtures, configuration, and utilities for all
deployment-related tests, ensuring consistent testing environment and
production-ready validation.
"""

import asyncio
import os
import sys
from datetime import datetime, timedelta
from typing import AsyncGenerator, Dict, Any, List
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import pytest_asyncio
from pytest_benchmark.plugin import benchmark

# Add src to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'src'))

from gordon_gekko.config.settings import Settings


# Test configuration
def pytest_configure(config):
    """Configure pytest with deployment-specific settings."""
    config.addinivalue_line("markers", "deployment: Deployment orchestration tests")
    config.addinivalue_line("markers", "environment: Environment validation tests")
    config.addinivalue_line("markers", "service: Service management tests")
    config.addinivalue_line("markers", "api: API integration tests")
    config.addinivalue_line("markers", "gpu: GPU and hardware tests")
    config.addinivalue_line("markers", "security: Security validation tests")
    config.addinivalue_line("markers", "monitoring: Monitoring and metrics tests")
    config.addinivalue_line("markers", "performance: Performance benchmarking tests")
    config.addinivalue_line("markers", "integration: Multi-component integration tests")
    config.addinivalue_line("markers", "browser: Browser automation tests")
    config.addinivalue_line("markers", "browser_chromium: Chromium browser tests")
    config.addinivalue_line("markers", "browser_firefox: Firefox browser tests")
    config.addinivalue_line("markers", "browser_webkit: WebKit browser tests")
    config.addinivalue_line("markers", "browser_mobile: Mobile browser emulation tests")
    config.addinivalue_line("markers", "browser_accessibility: Accessibility validation tests")
    config.addinivalue_line("markers", "browser_performance: Browser performance tests")


def pytest_collection_modifyitems(config, items):
    """Add markers to tests based on their location."""
    for item in items:
        if "deployment_orchestrator" in str(item.fspath):
            item.add_marker(pytest.mark.deployment)
        elif "environment_validator" in str(item.fspath):
            item.add_marker(pytest.mark.environment)
        elif "service_manager" in str(item.fspath):
            item.add_marker(pytest.mark.service)
        elif "api_manager" in str(item.fspath):
            item.add_marker(pytest.mark.api)
        elif "gpu_manager" in str(item.fspath):
            item.add_marker(pytest.mark.gpu)
        elif "security_manager" in str(item.fspath):
            item.add_marker(pytest.mark.security)
        elif "monitoring_manager" in str(item.fspath):
            item.add_marker(pytest.mark.monitoring)
        elif "performance" in str(item.fspath):
            item.add_marker(pytest.mark.performance)
        elif "integration" in str(item.fspath):
            item.add_marker(pytest.mark.integration)


# Shared fixtures
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture(scope="session")
def mock_settings():
    """Provide mock settings for testing."""
    return Settings(
        database_url="postgresql://test:test@localhost:5432/test_db",
        redis_url="redis://localhost:6379/0",
        api_keys={
            "coinbase": "test_coinbase_key",
            "binance": "test_binance_key",
            "oanda": "test_oanda_key"
        },
        ml_models={
            "trading_model": "test_model_path",
            "risk_model": "test_risk_path"
        },
        security={
            "jwt_secret": "test_jwt_secret",
            "oauth_client_id": "test_oauth_client",
            "oauth_client_secret": "test_oauth_secret"
        }
    )


@pytest.fixture
def mock_environment_variables(monkeypatch):
    """Set up mock environment variables for testing."""
    test_env = {
        "DATABASE_URL": "postgresql://test:test@localhost:5432/test_db",
        "REDIS_URL": "redis://localhost:6379/0",
        "COINBASE_API_KEY": "test_coinbase_key",
        "BINANCE_API_KEY": "test_binance_key",
        "OANDA_API_KEY": "test_oanda_key",
        "JWT_SECRET_KEY": "test_jwt_secret",
        "OAUTH_CLIENT_ID": "test_oauth_client",
        "OAUTH_CLIENT_SECRET": "test_oauth_secret",
        "ENVIRONMENT": "testing"
    }

    for key, value in test_env.items():
        monkeypatch.setenv(key, value)


@pytest.fixture
async def mock_database_connection():
    """Mock database connection for testing."""
    connection = AsyncMock()
    connection.execute = AsyncMock()
    connection.fetch = AsyncMock(return_value=[])
    connection.fetchval = AsyncMock(return_value=None)
    connection.fetchrow = AsyncMock(return_value=None)
    connection.commit = AsyncMock()
    connection.rollback = AsyncMock()
    connection.close = AsyncMock()

    yield connection


@pytest.fixture
async def mock_redis_client():
    """Mock Redis client for testing."""
    redis_client = AsyncMock()
    redis_client.get = AsyncMock(return_value=None)
    redis_client.set = AsyncMock(return_value=True)
    redis_client.delete = AsyncMock(return_value=1)
    redis_client.exists = AsyncMock(return_value=False)
    redis_client.expire = AsyncMock(return_value=True)
    redis_client.ping = AsyncMock(return_value=True)

    yield redis_client


@pytest.fixture
def mock_docker_client():
    """Mock Docker client for testing."""
    docker_client = MagicMock()
    docker_client.containers = MagicMock()
    docker_client.containers.run = MagicMock()
    docker_client.containers.get = MagicMock()
    docker_client.containers.list = MagicMock(return_value=[])
    docker_client.images = MagicMock()
    docker_client.images.pull = MagicMock()
    docker_client.images.build = MagicMock()

    yield docker_client


@pytest.fixture
def mock_kubernetes_client():
    """Mock Kubernetes client for testing."""
    k8s_client = MagicMock()
    k8s_client.apps_v1 = MagicMock()
    k8s_client.apps_v1.deploy = MagicMock()
    k8s_client.core_v1 = MagicMock()
    k8s_client.core_v1.config_maps = MagicMock()
    k8s_client.core_v1.secrets = MagicMock()
    k8s_client.core_v1.services = MagicMock()

    yield k8s_client


@pytest.fixture
def mock_api_responses():
    """Mock API responses for different trading platforms."""
    return {
        "coinbase": {
            "status": "success",
            "data": {
                "id": "test-order-123",
                "status": "pending",
                "filled_size": "0.0",
                "executed_value": "0.0"
            }
        },
        "binance": {
            "status": "success",
            "data": {
                "orderId": 12345,
                "status": "NEW",
                "executedQty": "0.0",
                "cummulativeQuoteQty": "0.0"
            }
        },
        "oanda": {
            "status": "success",
            "data": {
                "order": {
                    "id": "123",
                    "state": "PENDING",
                    "filledUnits": 0,
                    "filledPrice": "0.0"
                }
            }
        }
    }


@pytest.fixture
def mock_gpu_devices():
    """Mock GPU devices for testing."""
    return [
        {
            "name": "Apple M1 Pro",
            "type": "mps",
            "memory": "16GB",
            "cores": 10,
            "is_available": True
        },
        {
            "name": "NVIDIA RTX 4090",
            "type": "cuda",
            "memory": "24GB",
            "cores": 16384,
            "is_available": True
        }
    ]


@pytest.fixture
def mock_jwt_tokens():
    """Mock JWT tokens for testing."""
    return {
        "valid_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJpYXQiOjE2NDEwMDAwMDAsImV4cCI6MTY0MTA4NjQwMH0.test-signature",
        "expired_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXVzZXIiLCJpYXQiOjE2NDAwMDAwMDAsImV4cCI6MTY0MDA4NjQwMH0.expired-signature",
        "invalid_token": "invalid.jwt.token"
    }


@pytest.fixture
def mock_prometheus_registry():
    """Mock Prometheus registry for testing."""
    registry = MagicMock()
    registry.collect = MagicMock(return_value=[])

    yield registry


@pytest.fixture
def mock_grafana_client():
    """Mock Grafana client for testing."""
    grafana_client = MagicMock()
    grafana_client.dashboards = MagicMock()
    grafana_client.alerts = MagicMock()
    grafana_client.users = MagicMock()

    yield grafana_client


@pytest.fixture
def performance_thresholds():
    """Performance thresholds for benchmarking tests."""
    return {
        "api_response_time": 0.050,  # 50ms
        "database_query_time": 0.010,  # 10ms
        "model_inference_time": 0.100,  # 100ms
        "deployment_time": 30.0,  # 30 seconds
        "health_check_time": 5.0  # 5 seconds
    }


@pytest.fixture
def security_test_data():
    """Security test data for penetration testing."""
    return {
        "sql_injection_payloads": [
            "'; DROP TABLE users; --",
            "' OR '1'='1",
            "1' UNION SELECT username, password FROM users --"
        ],
        "xss_payloads": [
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "javascript:alert('XSS')"
        ],
        "invalid_jwt_signatures": [
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid.signature",
            "invalid.jwt.token.format",
            ""
        ]
    }


# Async fixtures
@pytest_asyncio.fixture
async def async_test_context():
    """Async context for testing."""
    context = {
        "start_time": datetime.now(),
        "resources": [],
        "metrics": {}
    }

    yield context

    # Cleanup
    end_time = datetime.now()
    context["end_time"] = end_time
    context["duration"] = (end_time - context["start_time"]).total_seconds()


@pytest_asyncio.fixture
async def mock_async_database_pool():
    """Mock async database pool."""
    pool = AsyncMock()
    pool.acquire = AsyncMock()
    pool.release = AsyncMock()
    pool.close = AsyncMock()
    pool.size = 10

    yield pool


@pytest_asyncio.fixture
async def mock_async_http_client():
    """Mock async HTTP client."""
    client = AsyncMock()
    client.get = AsyncMock()
    client.post = AsyncMock()
    client.put = AsyncMock()
    client.delete = AsyncMock()
    client.close = AsyncMock()

    yield client


# Benchmarking utilities
def benchmark_with_threshold(benchmark_func, threshold: float):
    """Benchmark function with performance threshold validation."""
    def wrapper(*args, **kwargs):
        result = benchmark_func(*args, **kwargs)

        # Validate performance threshold
        if hasattr(result, 'stats') and hasattr(result.stats, 'mean'):
            mean_time = result.stats.mean
            if mean_time > threshold:
                pytest.fail(
                    f"Performance regression detected: {mean_time:.4f}s "
                    f"exceeds threshold of {threshold:.4f}s"
                )

        return result

    return wrapper


# Test utilities
class TestUtils:
    """Utility class for common test operations."""

    @staticmethod
    def mock_api_response(status_code: int = 200, data: Dict = None):
        """Create mock API response."""
        response = MagicMock()
        response.status_code = status_code
        response.json.return_value = data or {}
        response.raise_for_status = MagicMock()
        return response

    @staticmethod
    def mock_exception(exception_class, message: str = "Test exception"):
        """Create mock exception."""
        return exception_class(message)

    @staticmethod
    def assert_performance_metrics(metrics: Dict, thresholds: Dict):
        """Assert performance metrics meet thresholds."""
        for metric_name, threshold in thresholds.items():
            if metric_name in metrics:
                assert metrics[metric_name] <= threshold, (
                    f"Performance metric {metric_name}: {metrics[metric_name]} "
                    f"exceeds threshold {threshold}"
                )

    @staticmethod
    def validate_security_headers(headers: Dict):
        """Validate security headers."""
        required_headers = [
            'X-Content-Type-Options',
            'X-Frame-Options',
            'X-XSS-Protection',
            'Strict-Transport-Security'
        ]

        for header in required_headers:
            assert header in headers, f"Missing security header: {header}"


# Browser automation fixtures
@pytest.fixture(scope="session")
def browser_config():
    """Browser configuration for Playwright testing."""
    return {
        "headless": True,
        "slow_mo": 100,  # Slow down actions for debugging
        "timeout": 30000,  # 30 second timeout
        "viewport": {"width": 1280, "height": 720},
        "record_video": False,
        "record_screenshots": True,
        "base_url": "http://localhost:8000",
        "api_base_url": "http://localhost:8000/api"
    }


@pytest.fixture(scope="session")
def browser_context_args(browser_config):
    """Browser context arguments for Playwright."""
    return {
        "viewport": browser_config["viewport"],
        "record_video_dir": "test-results/videos/" if browser_config["record_video"] else None,
        "record_har_path": "test-results/network-logs/"
    }


@pytest.fixture
def mock_browser_page():
    """Mock browser page for testing."""
    mock_page = MagicMock()
    mock_page.goto = AsyncMock(return_value=None)
    mock_page.wait_for_load_state = AsyncMock(return_value=None)
    mock_page.click = AsyncMock(return_value=None)
    mock_page.type = AsyncMock(return_value=None)
    mock_page.fill = AsyncMock(return_value=None)
    mock_page.select_option = AsyncMock(return_value=None)
    mock_page.check = AsyncMock(return_value=None)
    mock_page.uncheck = AsyncMock(return_value=None)
    mock_page.wait_for_selector = AsyncMock(return_value=MagicMock())
    mock_page.screenshot = AsyncMock(return_value=b"mock_screenshot")
    mock_page.evaluate = AsyncMock(return_value=None)
    mock_page.reload = AsyncMock(return_value=None)
    mock_page.go_back = AsyncMock(return_value=None)
    mock_page.go_forward = AsyncMock(return_value=None)
    mock_page.close = AsyncMock(return_value=None)

    yield mock_page


@pytest.fixture
def mock_browser_context():
    """Mock browser context for testing."""
    mock_context = MagicMock()
    mock_context.new_page = AsyncMock(return_value=MagicMock())
    mock_context.pages = []
    mock_context.clear_cookies = AsyncMock(return_value=None)
    mock_context.add_cookies = AsyncMock(return_value=None)
    mock_context.close = AsyncMock(return_value=None)

    yield mock_context


@pytest.fixture
def mock_browser():
    """Mock browser instance for testing."""
    mock_browser = MagicMock()
    mock_browser.new_context = AsyncMock(return_value=MagicMock())
    mock_browser.new_page = AsyncMock(return_value=MagicMock())
    mock_browser.close = AsyncMock(return_value=None)
    mock_browser.version = "1.0.0"
    mock_browser.name = "MockBrowser"

    yield mock_browser


@pytest.fixture
def mock_playwright():
    """Mock Playwright instance for testing."""
    mock_playwright_instance = MagicMock()
    mock_playwright_instance.chromium = MagicMock()
    mock_playwright_instance.firefox = MagicMock()
    mock_playwright_instance.webkit = MagicMock()
    mock_playwright_instance.chromium.launch = AsyncMock(return_value=MagicMock())
    mock_playwright_instance.firefox.launch = AsyncMock(return_value=MagicMock())
    mock_playwright_instance.webkit.launch = AsyncMock(return_value=MagicMock())
    mock_playwright_instance.stop = AsyncMock(return_value=None)

    yield mock_playwright_instance


@pytest.fixture
def browser_test_data():
    """Test data for browser automation scenarios."""
    return {
        "trading_dashboard": {
            "url": "/dashboard",
            "expected_elements": [
                "portfolio-summary",
                "market-data",
                "active-orders",
                "risk-metrics"
            ],
            "user_interactions": [
                "view_portfolio",
                "check_positions",
                "review_risk"
            ]
        },
        "authentication": {
            "login_url": "/login",
            "username_field": "input[name='username']",
            "password_field": "input[name='password']",
            "submit_button": "button[type='submit']",
            "success_indicator": ".dashboard"
        },
        "api_validation": {
            "market_data_endpoint": "/api/market-data",
            "portfolio_endpoint": "/api/portfolio",
            "orders_endpoint": "/api/orders",
            "risk_endpoint": "/api/risk-analysis"
        }
    }


@pytest.fixture
def mobile_device_configs():
    """Mobile device configurations for browser testing."""
    return {
        "iPhone_12": {
            "viewport": {"width": 390, "height": 844},
            "user_agent": "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)",
            "device_scale_factor": 3
        },
        "Samsung_Galaxy_S21": {
            "viewport": {"width": 360, "height": 800},
            "user_agent": "Mozilla/5.0 (Linux; Android 11; SM-G996B)",
            "device_scale_factor": 2.625
        },
        "iPad_Pro": {
            "viewport": {"width": 1024, "height": 1366},
            "user_agent": "Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)",
            "device_scale_factor": 2
        }
    }


@pytest.fixture
def accessibility_test_config():
    """Accessibility testing configuration."""
    return {
        "enable_screen_reader": True,
        "check_color_contrast": True,
        "validate_keyboard_navigation": True,
        "check_aria_labels": True,
        "validate_form_labels": True,
        "check_heading_hierarchy": True,
        "validate_image_alt_text": True,
        "check_focus_management": True
    }


@pytest.fixture
def performance_test_config():
    """Performance testing configuration for browser tests."""
    return {
        "page_load_timeout": 5000,  # 5 seconds
        "interaction_timeout": 2000,  # 2 seconds
        "resource_timeout": 30000,  # 30 seconds
        "max_memory_usage_mb": 500,
        "max_network_requests": 50,
        "min_pass_rate": 0.95
    }


# Browser testing utilities
class BrowserTestUtils:
    """Utility class for browser automation testing."""

    @staticmethod
    def create_test_user_credentials():
        """Create test user credentials for browser testing."""
        return {
            "username": "test_trader@example.com",
            "password": "TestPassword123!",
            "display_name": "Test Trader"
        }

    @staticmethod
    def create_mock_trading_data():
        """Create mock trading data for browser validation."""
        return {
            "portfolio": {
                "total_value": 100000.0,
                "available_cash": 25000.0,
                "positions": [
                    {
                        "symbol": "AAPL",
                        "quantity": 100,
                        "current_price": 150.0,
                        "total_value": 15000.0
                    },
                    {
                        "symbol": "GOOGL",
                        "quantity": 50,
                        "current_price": 2800.0,
                        "total_value": 140000.0
                    }
                ]
            },
            "market_data": {
                "indices": {
                    "SP500": 4500.0,
                    "NASDAQ": 14000.0,
                    "DOW": 35000.0
                },
                "trending": ["TSLA", "NVDA", "META"]
            }
        }

    @staticmethod
    def validate_browser_capabilities(browser_capabilities):
        """Validate browser capabilities for testing."""
        required_capabilities = [
            "javascript_enabled",
            "network_interception",
            "screenshot_capture",
            "element_interaction",
            "form_filling",
            "navigation"
        ]

        for capability in required_capabilities:
            assert capability in browser_capabilities, f"Missing required capability: {capability}"

    @staticmethod
    def assert_performance_metrics(metrics, thresholds):
        """Assert browser performance metrics meet requirements."""
        for metric_name, threshold in thresholds.items():
            if metric_name in metrics:
                assert metrics[metric_name] <= threshold, (
                    f"Performance metric {metric_name}: {metrics[metric_name]} "
                    f"exceeds threshold {threshold}"
                )


# Export commonly used fixtures
__all__ = [
    'mock_settings',
    'mock_environment_variables',
    'mock_database_connection',
    'mock_redis_client',
    'mock_docker_client',
    'mock_kubernetes_client',
    'mock_api_responses',
    'mock_gpu_devices',
    'mock_jwt_tokens',
    'mock_prometheus_registry',
    'mock_grafana_client',
    'performance_thresholds',
    'security_test_data',
    'async_test_context',
    'mock_async_database_pool',
    'mock_async_http_client',
    'TestUtils',
    'browser_config',
    'browser_context_args',
    'mock_browser_page',
    'mock_browser_context',
    'mock_browser',
    'mock_playwright',
    'browser_test_data',
    'mobile_device_configs',
    'accessibility_test_config',
    'performance_test_config',
    'BrowserTestUtils'
]