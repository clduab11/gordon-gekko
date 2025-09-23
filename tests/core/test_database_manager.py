"""
Tests for DatabaseManager

This module contains comprehensive tests for the DatabaseManager class,
covering connection management, query execution, error handling, and
health checks.
"""

import pytest
from unittest.mock import AsyncMock, patch
from typing import Dict, Any

from src.gordon_gekko.core.database_manager import (
    DatabaseManager,
    ConnectionPoolError,
    QueryExecutionError,
)


class TestDatabaseManager:
    """Test cases for DatabaseManager class."""

    def mock_settings(self) -> Dict[str, Any]:
        """Mock settings for testing."""
        return {
            "url": "postgresql://test:test@localhost:5432/test_db",
            "pool_size": 10,
            "max_overflow": 20
        }

    def db_manager(self) -> DatabaseManager:
        """Database manager instance for testing."""
        return DatabaseManager(self.mock_settings())

    async def test_initialization(self) -> None:
        """Test database manager initialization."""
        db_manager = self.db_manager()
        assert db_manager.pool is None
        assert db_manager.config is not None
        assert "url" in db_manager.config

    async def test_initialize_success(self) -> None:
        """Test successful database pool initialization."""
        db_manager = self.db_manager()

        with patch('asyncpg.create_pool') as mock_create_pool:
            mock_pool = AsyncMock()
            mock_create_pool.return_value = mock_pool

            await db_manager.initialize()

            mock_create_pool.assert_called_once()
            assert db_manager.pool == mock_pool

    async def test_initialize_failure(self) -> None:
        """Test database initialization failure."""
        db_manager = self.db_manager()

        with patch('asyncpg.create_pool') as mock_create_pool:
            mock_create_pool.side_effect = Exception("Connection failed")

            with pytest.raises(ConnectionPoolError) as exc_info:
                await db_manager.initialize()

            assert (
                "Failed to initialize database connection pool"
                in str(exc_info.value)
            )

    async def test_close_pool(self) -> None:
        """Test closing database pool."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        db_manager.pool = mock_pool

        await db_manager.close()

        mock_pool.close.assert_called_once()
        assert db_manager.pool is None

    async def test_health_check_healthy(self) -> None:
        """Test health check when database is healthy."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        db_manager.pool = mock_pool

        mock_connection = AsyncMock()
        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.fetchval.return_value = 1

        result = await db_manager.health_check()

        assert result["status"] == "healthy"
        assert "pool_size" in result
        assert "free_connections" in result
        assert "used_connections" in result

    async def test_health_check_unhealthy_pool_not_initialized(
        self
    ) -> None:
        """Test health check when pool is not initialized."""
        db_manager = self.db_manager()
        db_manager.pool = None

        result = await db_manager.health_check()

        assert result["status"] == "unhealthy"
        assert "Database pool not initialized" in result["error"]

    async def test_health_check_unhealthy_connection_error(
        self
    ) -> None:
        """Test health check when connection fails."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.side_effect = Exception("Connection timeout")

        result = await db_manager.health_check()

        assert result["status"] == "unhealthy"
        assert "Connection timeout" in result["error"]

    async def test_get_connection_success(self) -> None:
        """Test successful connection acquisition."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        async with db_manager.get_connection() as connection:
            assert connection == mock_connection

        mock_pool.acquire.assert_called_once()

    async def test_get_connection_pool_not_initialized(self) -> None:
        """Test connection acquisition when pool is not initialized."""
        db_manager = self.db_manager()
        db_manager.pool = None

        with pytest.raises(ConnectionPoolError) as exc_info:
            async with db_manager.get_connection():
                pass

        assert "Database pool not initialized" in str(exc_info.value)

    async def test_get_connection_acquisition_error(self) -> None:
        """Test connection acquisition failure."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.side_effect = Exception("Pool exhausted")

        with pytest.raises(ConnectionPoolError) as exc_info:
            async with db_manager.get_connection():
                pass

        assert "Failed to acquire database connection" in str(exc_info.value)

    async def test_execute_query_success(self) -> None:
        """Test successful query execution."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.execute.return_value = "Query executed"

        result = await db_manager.execute_query("SELECT * FROM test")

        assert result == "Query executed"
        mock_connection.execute.assert_called_once_with("SELECT * FROM test")

    async def test_execute_query_with_retry(self) -> None:
        """Test query execution with retry logic."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        # Simulate connection failure then success
        mock_pool.acquire.side_effect = [
            Exception("Connection failed"),  # First attempt fails
            mock_connection  # Second attempt succeeds
        ]

        mock_connection.execute.return_value = "Success"

        result = await db_manager.execute_query("SELECT * FROM test")

        assert result == "Success"
        assert mock_pool.acquire.call_count == 2

    async def test_execute_query_max_retries(self) -> None:
        """Test query execution with max retries exceeded."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.side_effect = Exception(
            "Persistent connection failure"
        )

        with pytest.raises(QueryExecutionError) as exc_info:
            await db_manager.execute_query("SELECT * FROM test")

        assert "Database query failed after 3 attempts" in str(exc_info.value)

    async def test_execute_query_fetch_single(self) -> None:
        """Test query execution with single result fetch."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.fetchval.return_value = "single_result"

        result = await db_manager.execute_query(
            "SELECT COUNT(*) FROM test",
            fetch_single=True
        )

        assert result == "single_result"
        mock_connection.fetchval.assert_called_once_with(
            "SELECT COUNT(*) FROM test"
        )

    async def test_execute_query_fetch_multiple(self) -> None:
        """Test query execution with multiple results fetch."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.fetch.return_value = [{"id": 1}, {"id": 2}]

        result = await db_manager.execute_query(
            "SELECT * FROM test",
            fetch=True
        )

        assert result == [{"id": 1}, {"id": 2}]
        mock_connection.fetch.assert_called_once_with("SELECT * FROM test")

    async def test_execute_query_with_transaction(self) -> None:
        """Test query execution with transaction commit."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        await db_manager.execute_query(
            "INSERT INTO test VALUES (1)",
            commit=True
        )

        mock_connection.commit.assert_called_once()

    async def test_execute_transaction_success(self) -> None:
        """Test successful transaction execution."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        queries = [
            ("INSERT INTO test VALUES (1)", ()),
            ("INSERT INTO test VALUES (2)", ())
        ]

        await db_manager.execute_transaction(queries)

        assert mock_connection.execute.call_count == 2
        mock_connection.commit.assert_called_once()

    async def test_execute_transaction_failure(self) -> None:
        """Test transaction execution failure."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.execute.side_effect = Exception("Transaction failed")

        queries = [("INSERT INTO test VALUES (1)", ())]

        with pytest.raises(QueryExecutionError) as exc_info:
            await db_manager.execute_transaction(queries)

        assert "Transaction failed" in str(exc_info.value)
        mock_connection.rollback.assert_called_once()

    async def test_create_tables_success(self) -> None:
        """Test successful table creation."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        tables = {
            "test_table": (
                "CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY)"
            )
        }

        await db_manager.create_tables(tables)

        mock_connection.execute.assert_called_once_with(
            "CREATE TABLE IF NOT EXISTS test_table (id SERIAL PRIMARY KEY)"
        )

    async def test_create_tables_failure(self) -> None:
        """Test table creation failure."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.execute.side_effect = Exception(
            "Table creation failed"
        )

        tables = {
            "test_table": "CREATE TABLE test_table (id SERIAL PRIMARY KEY)"
        }

        with pytest.raises(QueryExecutionError) as exc_info:
            await db_manager.create_tables(tables)

        assert "Failed to create table 'test_table'" in str(exc_info.value)

    async def test_get_table_info_success(self) -> None:
        """Test successful table information retrieval."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        # Mock schema query result
        mock_connection.fetch.return_value = [
            {"column_name": "id", "data_type": "integer", "is_nullable": "NO"}
        ]

        # Mock stats query result
        mock_stats = {"schemaname": "public", "tablename": "test_table"}
        mock_connection.fetch.return_value = [mock_stats]

        result = await db_manager.get_table_info("test_table")

        assert result["table_name"] == "test_table"
        assert len(result["columns"]) == 1
        assert result["statistics"]["schemaname"] == "public"

    async def test_backup_table_success(self) -> None:
        """Test successful table backup."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        backup_name = await db_manager.backup_table("test_table")

        mock_connection.execute.assert_called_once()
        assert backup_name.startswith("test_table_backup_")

    async def test_backup_table_custom_name(self) -> None:
        """Test table backup with custom name."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )

        backup_name = await db_manager.backup_table("test_table", "my_backup")

        assert backup_name == "my_backup"
        mock_connection.execute.assert_called_once()

    async def test_backup_table_failure(self) -> None:
        """Test table backup failure."""
        db_manager = self.db_manager()
        mock_pool = AsyncMock()
        mock_connection = AsyncMock()
        db_manager.pool = mock_pool

        mock_pool.acquire.return_value.__aenter__ = AsyncMock(
            return_value=mock_connection
        )
        mock_connection.execute.side_effect = Exception("Backup failed")

        with pytest.raises(QueryExecutionError) as exc_info:
            await db_manager.backup_table("test_table")

        assert "Failed to backup table 'test_table'" in str(exc_info.value)


class TestDatabaseManagerIntegration:
    """Integration tests for DatabaseManager."""

    async def test_full_lifecycle(self) -> None:
        """Test full database manager lifecycle."""
        config = {
            "url": "postgresql://test:test@localhost:5432/test_db",
            "pool_size": 5,
            "max_overflow": 10
        }

        db_manager = DatabaseManager(config)

        # Test initialization
        assert not db_manager.pool

        # Test close without initialization
        await db_manager.close()
        assert not db_manager.pool

    async def test_error_handling_isolation(self) -> None:
        """Test that errors in one operation don't affect others."""
        db_manager = DatabaseManager({
            "url": "postgresql://test:test@localhost:5432/test_db",
            "pool_size": 5,
            "max_overflow": 10
        })

        # Test that operations work independently
        with pytest.raises(ConnectionPoolError):
            async with db_manager.get_connection():
                pass

        # Test that health check still works after error
        result = await db_manager.health_check()
        assert result["status"] == "unhealthy"
        assert "Database pool not initialized" in result["error"]

    async def test_query_types(self) -> None:
        """Test different query types work correctly."""
        db_manager = DatabaseManager({
            "url": "postgresql://test:test@localhost:5432/test_db",
            "pool_size": 5,
            "max_overflow": 10
        })

        # Test that all query methods are properly defined
        assert hasattr(db_manager, 'execute_query')
        assert hasattr(db_manager, 'execute_transaction')
        assert hasattr(db_manager, 'create_tables')
        assert hasattr(db_manager, 'get_table_info')
        assert hasattr(db_manager, 'backup_table')