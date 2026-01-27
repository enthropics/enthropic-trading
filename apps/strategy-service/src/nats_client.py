"""NATS client wrapper with authentication support."""

import json
import asyncio
from typing import Callable, Optional, Any
import nats
from nats.aio.client import Client as NatsClient
from nats.aio.msg import Msg

from .auth import AuthContext
from .config import get_settings


class AuthenticatedNatsClient:
    """NATS client that includes auth context in messages."""

    def __init__(self):
        self.settings = get_settings()
        self._client: Optional[NatsClient] = None
        self._subscriptions: dict = {}

    async def connect(self) -> None:
        """Connect to NATS server."""
        self._client = await nats.connect(self.settings.nats_url)
        print(f"Connected to NATS at {self.settings.nats_url}")

    async def close(self) -> None:
        """Close NATS connection."""
        if self._client:
            await self._client.close()

    def _build_message(self, data: dict, auth: AuthContext) -> bytes:
        """Build message with auth context."""
        message = {
            "auth": {
                "account_id": auth.account_id,
                "username": auth.username,
                "role": auth.role,
                "permissions": list(auth.permissions),
            },
            **data
        }
        return json.dumps(message).encode()

    async def publish(self, subject: str, data: dict, auth: AuthContext) -> None:
        """Publish message with auth context."""
        if not self._client:
            raise RuntimeError("Not connected to NATS")
        
        message = self._build_message(data, auth)
        await self._client.publish(subject, message)

    async def request(
        self,
        subject: str,
        data: dict,
        auth: AuthContext,
        timeout: float = 5.0
    ) -> dict:
        """Send request with auth context and wait for response."""
        if not self._client:
            raise RuntimeError("Not connected to NATS")
        
        message = self._build_message(data, auth)
        response = await self._client.request(subject, message, timeout=timeout)
        return json.loads(response.data.decode())

    async def subscribe(
        self,
        subject: str,
        callback: Callable[[Msg], Any],
        queue: Optional[str] = None
    ) -> None:
        """Subscribe to a subject."""
        if not self._client:
            raise RuntimeError("Not connected to NATS")
        
        sub = await self._client.subscribe(subject, queue=queue, cb=callback)
        self._subscriptions[subject] = sub

    async def unsubscribe(self, subject: str) -> None:
        """Unsubscribe from a subject."""
        if subject in self._subscriptions:
            await self._subscriptions[subject].unsubscribe()
            del self._subscriptions[subject]

    async def submit_order(self, auth: AuthContext, order: dict) -> dict:
        """Submit an order through NATS."""
        return await self.request("orders.submit", order, auth)

    async def cancel_order(self, auth: AuthContext, order_id: str) -> dict:
        """Cancel an order through NATS."""
        return await self.request("orders.cancel", {"order_id": order_id}, auth)

    async def get_positions(self, auth: AuthContext) -> dict:
        """Query positions through NATS."""
        return await self.request("positions.query", {}, auth)
