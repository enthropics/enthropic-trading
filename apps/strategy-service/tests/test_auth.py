"""Unit tests for Auth Service."""

import pytest
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime, timedelta

from src.auth import AuthService, AuthContext, AuthError, Permissions


@pytest.fixture
def mock_redis():
    redis = AsyncMock()
    redis.get = AsyncMock(return_value=None)
    redis.setex = AsyncMock()
    return redis


@pytest.fixture
def auth_service(mock_redis):
    with patch.dict('os.environ', {'JWT_SECRET': 'test-secret-key-minimum-32-characters'}):
        return AuthService(mock_redis)


class TestAuthService:
    @pytest.mark.asyncio
    async def test_validate_token_success(self, auth_service, mock_redis):
        # Create a valid JWT token
        import jwt
        payload = {
            "sub": "acc-123",
            "username": "trader1",
            "role": "trader",
            "permissions": ["orders:create", "orders:read"],
            "exp": datetime.utcnow() + timedelta(hours=1),
            "iat": datetime.utcnow(),
            "jti": "jti-123"
        }
        token = jwt.encode(payload, "test-secret-key-minimum-32-characters", algorithm="HS256")
        
        auth_context = await auth_service.validate_token(token)
        
        assert auth_context.account_id == "acc-123"
        assert auth_context.username == "trader1"
        assert auth_context.role == "trader"
        assert "orders:create" in auth_context.permissions

    @pytest.mark.asyncio
    async def test_validate_token_expired(self, auth_service, mock_redis):
        import jwt
        payload = {
            "sub": "acc-123",
            "username": "trader1",
            "role": "trader",
            "permissions": [],
            "exp": datetime.utcnow() - timedelta(hours=1),  # Expired
            "iat": datetime.utcnow() - timedelta(hours=2),
            "jti": "jti-123"
        }
        token = jwt.encode(payload, "test-secret-key-minimum-32-characters", algorithm="HS256")
        
        with pytest.raises(AuthError) as exc_info:
            await auth_service.validate_token(token)
        
        assert exc_info.value.code == "TOKEN_EXPIRED"

    @pytest.mark.asyncio
    async def test_validate_token_blacklisted(self, auth_service, mock_redis):
        import jwt
        mock_redis.get.return_value = b"1"  # Token is blacklisted
        
        payload = {
            "sub": "acc-123",
            "username": "trader1",
            "role": "trader",
            "permissions": [],
            "exp": datetime.utcnow() + timedelta(hours=1),
            "iat": datetime.utcnow(),
            "jti": "jti-123"
        }
        token = jwt.encode(payload, "test-secret-key-minimum-32-characters", algorithm="HS256")
        
        with pytest.raises(AuthError) as exc_info:
            await auth_service.validate_token(token)
        
        assert exc_info.value.code == "TOKEN_REVOKED"

    @pytest.mark.asyncio
    async def test_revoke_token(self, auth_service, mock_redis):
        await auth_service.revoke_token("jti-123")
        
        mock_redis.setex.assert_called_once()


class TestAuthContext:
    def test_has_permission_direct(self):
        auth = AuthContext(
            account_id="acc-123",
            username="trader1",
            role="trader",
            permissions={"orders:create", "orders:read"},
            token_jti="jti-123"
        )
        
        assert auth.has_permission("orders:create") is True
        assert auth.has_permission("admin:full") is False

    def test_has_permission_admin_bypass(self):
        auth = AuthContext(
            account_id="acc-123",
            username="admin",
            role="admin",
            permissions={"admin:full"},
            token_jti="jti-123"
        )
        
        # Admin can access everything
        assert auth.has_permission("orders:create") is True
        assert auth.has_permission("risk:manage") is True
        assert auth.has_permission("anything:else") is True

    def test_can_access_account_own(self):
        auth = AuthContext(
            account_id="acc-123",
            username="trader1",
            role="trader",
            permissions={"orders:create"},
            token_jti="jti-123"
        )
        
        assert auth.can_access_account("acc-123") is True
        assert auth.can_access_account("acc-456") is False

    def test_can_access_account_admin(self):
        auth = AuthContext(
            account_id="acc-admin",
            username="admin",
            role="admin",
            permissions={"admin:full"},
            token_jti="jti-123"
        )
        
        # Admin can access any account
        assert auth.can_access_account("acc-123") is True
        assert auth.can_access_account("acc-456") is True
