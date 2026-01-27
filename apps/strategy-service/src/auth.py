"""Authentication & Authorization for Strategy Service.

Phase 2: JWT validation, RBAC, token blacklist checking.
"""

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Optional, Set
import jwt
import redis.asyncio as redis
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from .config import get_settings


@dataclass
class AuthContext:
    """Authenticated user context."""
    account_id: str
    username: str
    role: str
    permissions: Set[str]
    token_jti: str

    def has_permission(self, permission: str) -> bool:
        """Check if user has a specific permission."""
        return permission in self.permissions or "admin:full" in self.permissions

    def can_access_account(self, target_account_id: str) -> bool:
        """Check if user can access another account's resources."""
        return (
            self.account_id == target_account_id
            or self.has_permission("admin:full")
            or self.has_permission("accounts:read_all")
        )


class AuthError(Exception):
    """Authentication/Authorization error."""
    def __init__(self, message: str, code: str):
        self.message = message
        self.code = code
        super().__init__(message)


class AuthService:
    """JWT authentication service."""

    def __init__(self, redis_client: redis.Redis):
        self.settings = get_settings()
        self.redis = redis_client

    async def validate_token(self, token: str, db: AsyncSession) -> AuthContext:
        """Validate JWT token and return auth context."""
        try:
            payload = jwt.decode(
                token,
                self.settings.jwt_secret,
                algorithms=["HS256"],
                options={"verify_exp": True}
            )
        except jwt.ExpiredSignatureError:
            raise AuthError("Token expired", "TOKEN_EXPIRED")
        except jwt.InvalidTokenError as e:
            raise AuthError(f"Invalid token: {e}", "INVALID_TOKEN")

        # Check if token is blacklisted
        jti = payload.get("jti", "")
        is_blacklisted = await self.redis.exists(f"token_blacklist:{jti}")
        if is_blacklisted:
            raise AuthError("Token revoked", "TOKEN_REVOKED")

        # Verify account exists and is active
        from .models import Account  # Import here to avoid circular imports
        
        result = await db.execute(
            select(Account).where(Account.id == payload["sub"])
        )
        account = result.scalar_one_or_none()

        if not account:
            raise AuthError("Account not found", "ACCOUNT_NOT_FOUND")

        if not account.is_active:
            raise AuthError("Account disabled", "ACCOUNT_DISABLED")

        if account.locked_until and account.locked_until > datetime.now(timezone.utc):
            raise AuthError("Account locked", "ACCOUNT_LOCKED")

        return AuthContext(
            account_id=payload["sub"],
            username=payload["username"],
            role=payload["role"],
            permissions=set(payload.get("permissions", [])),
            token_jti=jti
        )

    async def revoke_token(self, jti: str, account_id: str, reason: str = "logout") -> None:
        """Revoke a token by adding it to the blacklist."""
        # Add to Redis with 24h TTL
        await self.redis.setex(f"token_blacklist:{jti}", 86400, "1")


class Permissions:
    """Permission constants."""
    ORDERS_CREATE = "orders:create"
    ORDERS_READ = "orders:read"
    ORDERS_CANCEL = "orders:cancel"
    ORDERS_READ_ALL = "orders:read_all"
    POSITIONS_READ = "positions:read"
    POSITIONS_READ_ALL = "positions:read_all"
    MARKET_READ = "market:read"
    MARKET_SUBSCRIBE = "market:subscribe"
    STRATEGIES_READ = "strategies:read"
    STRATEGIES_CREATE = "strategies:create"
    STRATEGIES_EXECUTE = "strategies:execute"
    ADMIN_FULL = "admin:full"


def require_permission(*permissions: str):
    """Decorator to require specific permissions."""
    def decorator(func):
        async def wrapper(self, auth: AuthContext, *args, **kwargs):
            has_permission = any(auth.has_permission(p) for p in permissions)
            if not has_permission:
                raise AuthError(
                    f"Missing permission: {', '.join(permissions)}",
                    "FORBIDDEN"
                )
            return await func(self, auth, *args, **kwargs)
        return wrapper
    return decorator
