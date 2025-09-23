"""
Core Business Logic Module

This module contains the core business logic for the Gordon Gekko
trading system, including the main TradingSystem class and core
service components.

The core module is responsible for:
- System orchestration and lifecycle management
- Trading strategy execution and management
- Risk management and position tracking
- Market data processing and analysis
- Order execution and portfolio optimization
- ML model integration and inference
- API management and external integrations

Architecture follows microservices patterns with clear service
boundaries and event-driven processing for real-time trading
operations.
"""

from .trading_system import TradingSystem

__all__ = [
    "TradingSystem"
]