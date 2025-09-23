"""
Database Manager for Gordon Gekko Trading System

This module provides centralized database management with PostgreSQL support,
async operations, connection pooling, and comprehensive error handling.
"""

import asyncio
import logging
from contextlib import asynccontextmanager
from typing import Any, Dict, List, Optional, AsyncGenerator

try:
    import asyncpg
    from asyncpg import Pool, Connection
    from asyncpg.exceptions import (
        ConnectionError,
        ConnectionFailureError,
        UndefinedTableError,
        UniqueViolationError,
        ForeignKeyViolationError,
        CheckViolationError,
        NotNullViolationError,
        InvalidTextRepresentationError
    )
except ImportError:
    # Fallback for when asyncpg is not available
    asyncpg = None
    Pool = None
    Connection = None
    ConnectionError = Exception
    ConnectionFailureError = Exception
    UndefinedTableError = Exception
    UniqueViolationError = Exception
    ForeignKeyViolationError = Exception
    CheckViolationError = Exception
    NotNullViolationError = Exception
    InvalidTextRepresentationError = Exception

from ..config.settings import settings


class DatabaseError(Exception):
    """Base exception for database operations."""
    pass


class ConnectionPoolError(DatabaseError):
    """Exception raised for connection pool issues."""
    pass


class QueryExecutionError(DatabaseError):
    """Exception raised for query execution issues."""
    pass


class DatabaseManager:
    """
    Asynchronous database manager with connection pooling and error handling.

    Provides PostgreSQL connectivity with:
    - Connection pooling for optimal performance
    - Automatic retry logic for transient failures
    - Comprehensive error handling and logging
    - Health check capabilities
    - Proper resource cleanup
    """

    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """
        Initialize the database manager.

        Args:
            config: Optional database configuration override
        """
        self.config = config or settings.get_database_config()
        self.pool: Optional[Pool] = None
        self.logger = logging.getLogger(__name__)

    async def initialize(self) -> None:
        """
        Initialize the database connection pool.

        Raises:
            ConnectionPoolError: If pool creation fails
        """
        try:
            self.logger.info("Initializing database connection pool")

            # Create connection pool with optimized settings
            self.pool = await asyncpg.create_pool(
                self.config["url"],
                min_size=self.config["pool_size"] // 2,
                max_size=self.config["pool_size"],
                max_overflow=self.config["max_overflow"],
                command_timeout=settings.api_timeout_seconds,
                server_settings={
                    'application_name': settings.system_name,
                    'client_min_messages': 'warning'
                },
                # Connection health check
                init=self._init_connection
            )

            self.logger.info(
                f"Database connection pool initialized with "
                f"min_size={self.config['pool_size'] // 2}, "
                f"max_size={self.config['pool_size']}"
            )

        except Exception as e:
            error_msg = f"Failed to initialize database connection pool: {e}"
            self.logger.error(error_msg)
            raise ConnectionPoolError(error_msg) from e

    async def close(self) -> None:
        """
        Close the database connection pool and cleanup resources.
        """
        if self.pool:
            self.logger.info("Closing database connection pool")
            await self.pool.close()
            self.pool = None
            self.logger.info("Database connection pool closed")

    async def health_check(self) -> Dict[str, Any]:
        """
        Perform database health check.

        Returns:
            Dict containing health check results
        """
        if not self.pool:
            return {
                "status": "unhealthy",
                "error": "Database pool not initialized"
            }

        try:
            # Execute simple query to test connection
            async with self.pool.acquire() as connection:
                await connection.fetchval("SELECT 1")

            return {
                "status": "healthy",
                "pool_size": self.pool.get_size(),
                "free_connections": self.pool.get_idle_size(),
                "used_connections": self.pool.get_size() - self.pool.get_idle_size()
            }

        except Exception as e:
            return {
                "status": "unhealthy",
                "error": str(e)
            }

    @asynccontextmanager
    async def get_connection(self) -> AsyncGenerator[Connection, None]:
        """
        Get a database connection from the pool.

        Yields:
            Database connection

        Raises:
            ConnectionPoolError: If connection cannot be acquired
        """
        if not self.pool:
            raise ConnectionPoolError("Database pool not initialized")

        try:
            async with self.pool.acquire() as connection:
                # Set connection-specific settings
                await connection.set_type_codec(
                    'jsonb',
                    encoder=lambda x: x,
                    decoder=lambda x: x,
                    schema='pg_catalog'
                )
                yield connection

        except Exception as e:
            error_msg = f"Failed to acquire database connection: {e}"
            self.logger.error(error_msg)
            raise ConnectionPoolError(error_msg) from e

    async def execute_query(
        self,
        query: str,
        *args,
        fetch: bool = False,
        fetch_single: bool = False,
        commit: bool = False
    ) -> Any:
        """
        Execute a database query with automatic retry logic.

        Args:
            query: SQL query to execute
            *args: Query parameters
            fetch: Whether to fetch results
            fetch_single: Whether to fetch single result
            commit: Whether to commit transaction

        Returns:
            Query results or None

        Raises:
            QueryExecutionError: If query execution fails
        """
        max_retries = 3
        retry_delay = 0.1

        for attempt in range(max_retries):
            try:
                async with self.get_connection() as connection:
                    if fetch_single:
                        result = await connection.fetchval(query, *args)
                    elif fetch:
                        result = await connection.fetch(query, *args)
                    else:
                        result = await connection.execute(query, *args)

                    if commit:
                        await connection.commit()

                    return result

            except (ConnectionError, ConnectionFailureError) as e:
                if attempt < max_retries - 1:
                    self.logger.warning(
                        f"Database connection error on attempt {attempt + 1}, "
                        f"retrying in {retry_delay}s: {e}"
                    )
                    await asyncio.sleep(retry_delay)
                    retry_delay *= 2  # Exponential backoff
                else:
                    error_msg = f"Database query failed after {max_retries} attempts: {e}"
                    self.logger.error(error_msg)
                    raise QueryExecutionError(error_msg) from e

            except (UndefinedTableError, UniqueViolationError,
                    ForeignKeyViolationError, CheckViolationError,
                    NotNullViolationError, InvalidTextRepresentationError) as e:
                # These are application-level errors, don't retry
                error_msg = f"Database validation error: {e}"
                self.logger.error(error_msg)
                raise QueryExecutionError(error_msg) from e

            except Exception as e:
                error_msg = f"Unexpected database error: {e}"
                self.logger.error(error_msg)
                raise QueryExecutionError(error_msg) from e

    async def execute_transaction(self, queries: List[tuple]) -> None:
        """
        Execute multiple queries in a single transaction.

        Args:
            queries: List of (query, args) tuples

        Raises:
            QueryExecutionError: If transaction fails
        """
        try:
            async with self.get_connection() as connection:
                async with connection.transaction():
                    for query, args in queries:
                        await connection.execute(query, *args)

                    self.logger.debug(f"Transaction executed successfully with {len(queries)} queries")

        except Exception as e:
            error_msg = f"Transaction failed: {e}"
            self.logger.error(error_msg)
            raise QueryExecutionError(error_msg) from e

    async def create_tables(self, tables: Dict[str, str]) -> None:
        """
        Create database tables if they don't exist.

        Args:
            tables: Dictionary mapping table names to CREATE statements

        Raises:
            QueryExecutionError: If table creation fails
        """
        for table_name, create_statement in tables.items():
            try:
                await self.execute_query(create_statement)
                self.logger.info(f"Table '{table_name}' created or already exists")

            except Exception as e:
                error_msg = f"Failed to create table '{table_name}': {e}"
                self.logger.error(error_msg)
                raise QueryExecutionError(error_msg) from e

    async def get_table_info(self, table_name: str) -> Dict[str, Any]:
        """
        Get information about a database table.

        Args:
            table_name: Name of the table

        Returns:
            Dictionary containing table information

        Raises:
            QueryExecutionError: If query fails
        """
        try:
            # Get table schema information
            schema_query = """
                SELECT
                    column_name,
                    data_type,
                    is_nullable,
                    column_default
                FROM information_schema.columns
                WHERE table_name = $1
                ORDER BY ordinal_position
            """

            columns = await self.execute_query(schema_query, table_name, fetch=True)

            # Get table statistics
            stats_query = """
                SELECT
                    schemaname,
                    tablename,
                    tableowner,
                    tablespace,
                    hasindexes,
                    hasrules,
                    hastriggers
                FROM pg_tables
                WHERE tablename = $1
            """

            stats = await self.execute_query(stats_query, table_name, fetch=True)

            return {
                "table_name": table_name,
                "columns": columns or [],
                "statistics": stats[0] if stats else None
            }

        except Exception as e:
            error_msg = f"Failed to get table info for '{table_name}': {e}"
            self.logger.error(error_msg)
            raise QueryExecutionError(error_msg) from e

    async def backup_table(self, table_name: str, backup_name: Optional[str] = None) -> str:
        """
        Create a backup of a table.

        Args:
            table_name: Name of table to backup
            backup_name: Optional name for backup table

        Returns:
            Name of the backup table

        Raises:
            QueryExecutionError: If backup fails
        """
        if backup_name is None:
            backup_name = f"{table_name}_backup_{asyncio.get_event_loop().time()}"

        try:
            backup_query = f"CREATE TABLE {backup_name} AS SELECT * FROM {table_name}"

            await self.execute_query(backup_query)

            self.logger.info(f"Table '{table_name}' backed up to '{backup_name}'")

            return backup_name

        except Exception as e:
            error_msg = f"Failed to backup table '{table_name}': {e}"
            self.logger.error(error_msg)
            raise QueryExecutionError(error_msg) from e

    async def _init_connection(self, connection: Connection) -> None:
        """
        Initialize a new database connection.

        Args:
            connection: Database connection to initialize
        """
        # Set connection-level optimizations
        await connection.execute("SET timezone = 'UTC'")

        # Enable query timing for performance monitoring
        if settings.debug:
            await connection.execute("SET log_statement = 'all'")
        else:
            await connection.execute("SET log_statement = 'none'")


# Global database manager instance
database_manager = DatabaseManager()


async def get_database_manager() -> DatabaseManager:
    """
    Dependency injection function for database manager.

    Returns:
        Configured database manager instance
    """
    return database_manager