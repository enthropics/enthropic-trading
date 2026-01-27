"""Pytest fixtures for Strategy Service tests."""

import pytest
from decimal import Decimal
from unittest.mock import AsyncMock, MagicMock
from dataclasses import dataclass


@dataclass
class MockAuthContext:
    account_id: str = "test-account-123"
    username: str = "testuser"
    role: str = "trader"
    permissions: set = None
    token_jti: str = "test-jti"
    
    def __post_init__(self):
        if self.permissions is None:
            self.permissions = {"orders:create", "strategies:execute"}
    
    def has_permission(self, permission: str) -> bool:
        return permission in self.permissions or "admin:full" in self.permissions


@pytest.fixture
def auth_context():
    return MockAuthContext()


@pytest.fixture
def admin_auth_context():
    return MockAuthContext(
        role="admin",
        permissions={"admin:full"}
    )


@pytest.fixture
def mock_nats_client():
    client = AsyncMock()
    client.publish = AsyncMock()
    client.request = AsyncMock()
    client.subscribe = AsyncMock()
    return client


@pytest.fixture
def mock_redis():
    redis = AsyncMock()
    redis.get = AsyncMock(return_value=None)
    redis.setex = AsyncMock()
    redis.exists = AsyncMock(return_value=False)
    return redis
