"""Strategy Service main entry point with observability."""

import asyncio
import signal
from decimal import Decimal
from contextlib import asynccontextmanager

import redis.asyncio as redis
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
import structlog

from .config import get_settings
from .auth import AuthService, AuthContext, AuthError
from .models import Base
from .nats_client import AuthenticatedNatsClient
from .strategies import MomentumStrategy
from .observability import init_tracing, init_metrics, MetricsServer, configure_logging
from .resilience import CircuitBreakerManager, with_retry, RetryConfig


class StrategyService:
    """Main strategy service application with observability."""

    def __init__(self):
        self.settings = get_settings()
        self.running = False
        self.logger = structlog.get_logger()
        
        # Observability
        configure_logging("strategy-service")
        init_tracing("strategy-service")
        init_metrics()
        
        # Resilience
        self.circuit_breakers = CircuitBreakerManager()
        
        # Database
        self.engine = create_async_engine(
            self.settings.database_url.replace("postgres://", "postgresql+asyncpg://"),
            pool_size=10,
            max_overflow=20,
        )
        self.session_factory = async_sessionmaker(self.engine, expire_on_commit=False)
        
        # Redis
        self.redis = redis.from_url(self.settings.redis_url)
        
        # Auth
        self.auth_service = AuthService(self.redis)
        
        # NATS
        self.nats = AuthenticatedNatsClient()
        
        # Strategies
        self.strategies = {
            "momentum": MomentumStrategy(
                lookback_period=20,
                entry_threshold=0.02,
                exit_threshold=-0.01
            )
        }
        
        # Metrics server
        self.metrics_server = MetricsServer(
            port=int(self.settings.metrics_port if hasattr(self.settings, 'metrics_port') else 9102)
        )

    @asynccontextmanager
    async def get_session(self):
        async with self.session_factory() as session:
            yield session

    async def start(self):
        self.logger.info("starting_strategy_service")
        
        # Start metrics server
        await self.metrics_server.start()
        self.logger.info("metrics_server_started", port=9102)
        
        # Connect to NATS with retry
        await with_retry(
            "nats_connect",
            self.nats.connect,
            config=RetryConfig(max_retries=5),
        )
        
        # Subscribe to channels
        await self.nats.subscribe("market.ticks", self.handle_market_tick)
        await self.nats.subscribe("strategy.signals.request", self.handle_signal_request)
        
        self.logger.info("strategy_service_started")
        self.running = True
        self.metrics_server.set_healthy(True)

    async def stop(self):
        self.logger.info("stopping_strategy_service")
        self.running = False
        self.metrics_server.set_healthy(False)
        await self.nats.close()
        await self.redis.close()
        await self.engine.dispose()
        self.logger.info("strategy_service_stopped")

    async def handle_market_tick(self, msg):
        """Handle incoming market tick data."""
        import json
        try:
            data = json.loads(msg.data.decode())
            from .strategies.momentum import BarData
            
            bar = BarData(
                symbol=data.get("symbol", ""),
                open=Decimal(str(data.get("open", 0))),
                high=Decimal(str(data.get("high", 0))),
                low=Decimal(str(data.get("low", 0))),
                close=Decimal(str(data.get("last_price", 0))),
                volume=Decimal(str(data.get("volume", 0))),
                timestamp=data.get("timestamp", 0),
            )
            
            for strategy in self.strategies.values():
                strategy.update_bar(bar)
                
        except Exception as e:
            self.logger.error("market_tick_error", error=str(e))

    async def handle_signal_request(self, msg):
        """Handle signal generation request."""
        import json
        from .observability.metrics import get_metrics
        
        metrics = get_metrics()
        
        try:
            data = json.loads(msg.data.decode())
            auth_data = data.get("auth", {})
            
            auth = AuthContext(
                account_id=auth_data.get("account_id", ""),
                username=auth_data.get("username", ""),
                role=auth_data.get("role", ""),
                permissions=set(auth_data.get("permissions", [])),
                token_jti=""
            )
            
            strategy_name = data.get("strategy", "momentum")
            symbol = data.get("symbol", "")
            current_position = Decimal(str(data.get("current_position", 0)))
            
            with metrics.strategy_execution_duration.labels(strategy=strategy_name).time():
                strategy = self.strategies.get(strategy_name)
                if not strategy:
                    response = {"success": False, "error": f"Unknown strategy: {strategy_name}"}
                else:
                    signal = await strategy.generate_signal(auth, symbol, current_position)
                    if signal:
                        metrics.strategy_signals.labels(
                            strategy=strategy_name,
                            side=signal.side,
                            symbol=signal.symbol
                        ).inc()
                        response = {
                            "success": True,
                            "signal": {
                                "symbol": signal.symbol,
                                "side": signal.side,
                                "strength": signal.strength,
                                "reason": signal.reason,
                            }
                        }
                    else:
                        response = {"success": True, "signal": None}
                        
        except AuthError as e:
            response = {"success": False, "error": e.message, "code": e.code}
        except Exception as e:
            self.logger.error("signal_request_error", error=str(e))
            response = {"success": False, "error": str(e)}
        
        if msg.reply:
            await self.nats._client.publish(msg.reply, json.dumps(response).encode())


async def main():
    service = StrategyService()
    
    loop = asyncio.get_event_loop()
    for sig in (signal.SIGTERM, signal.SIGINT):
        loop.add_signal_handler(sig, lambda: asyncio.create_task(service.stop()))
    
    await service.start()
    
    while service.running:
        await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
