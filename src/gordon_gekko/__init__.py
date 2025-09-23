"""
Gordon Gekko Autonomous Trading Agent

A sophisticated autonomous trading system with advanced machine learning capabilities,
multi-platform trading integration, and enterprise-grade security.

This package provides a complete algorithmic trading solution with:
- Multi-platform trading (Coinbase, Binance.US, OANDA)
- GPU-accelerated machine learning pipeline
- Real-time risk management and position tracking
- Enterprise security with zero-trust architecture
- Comprehensive monitoring and observability
- RESTful API and WebSocket support

Version: 1.0.0
Author: Gordon Gekko Development Team
License: MIT
"""

from typing import Optional
from .core import TradingSystem

__version__ = "1.0.0"
__author__ = "Gordon Gekko Development Team"
__description__ = "Autonomous Trading Agent with Advanced ML and Multi-Platform Integration"
__license__ = "MIT"

# Main entry point for the trading system
__all__ = ["TradingSystem", "__version__", "__author__", "__description__"]

# Global trading system instance
_trading_system: Optional[TradingSystem] = None

def get_trading_system() -> TradingSystem:
    """Get the global trading system instance."""
    global _trading_system
    if _trading_system is None:
        _trading_system = TradingSystem()
    return _trading_system

def create_trading_system(config_path: Optional[str] = None) -> TradingSystem:
    """Create a new trading system instance with optional configuration."""
    return TradingSystem(config_path=config_path)