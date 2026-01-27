# Authentication & Authorization Guide

## Overview

The Enthropic Trading Platform implements a comprehensive authentication and authorization system designed for enterprise trading environments.

## JWT Token Structure

```json
{
  "sub": "account-uuid",
  "username": "trader1",
  "role": "trader",
  "permissions": ["orders:create", "orders:read", "positions:read"],
  "exp": 1706284800,
  "iat": 1706283900,
  "jti": "unique-token-id"
}
```

## Token Lifecycle

### Access Token
- **Expiry**: 15 minutes
- **Usage**: Include in Authorization header
- **Storage**: Memory only (frontend)

### Refresh Token
- **Expiry**: 7 days
- **Usage**: Exchange for new access token
- **Storage**: HttpOnly cookie (recommended) or secure storage

## Security Features

### Account Locking
- Accounts are locked after 5 consecutive failed login attempts
- Lock duration: 15 minutes
- Prevents brute-force attacks

### Token Revocation
- Tokens can be revoked via logout
- Revoked tokens are stored in Redis blacklist
- Blacklist entries expire with token TTL

### Audit Logging
All authentication events are logged:
- Login success/failure
- Token refresh
- Logout
- Account lock/unlock

### Rate Limiting
- Per-account rate limits
- Configurable via environment variables
- Sliding window algorithm

## Implementation Details

### Password Hashing
- Algorithm: bcrypt
- Cost factor: 12
- Salts automatically generated

### Token Signing
- Algorithm: HS256 (HMAC SHA-256)
- Secret: Minimum 32 characters
- Configurable via JWT_SECRET environment variable

## Service Integration

### NATS Gateway (Entry Point)
1. Client connects via WebSocket
2. Sends authenticate message with JWT
3. Gateway validates token
4. Auth context attached to all subsequent messages

### Execution Core (Rust)
1. Receives NATS message with auth context
2. Validates permissions before processing
3. Rejects unauthorized requests

### Risk Service (NestJS)
1. JWT strategy validates tokens
2. Guards check permissions on routes
3. Service methods verify ownership

### Strategy Service (Python)
1. Auth context extracted from NATS messages
2. Permission decorator validates access
3. Service methods check authorization

## Troubleshooting

### "Token expired" Error
- Token has exceeded 15-minute lifetime
- Solution: Use refresh token to get new access token

### "Token revoked" Error
- Token has been blacklisted (user logged out)
- Solution: Re-authenticate with credentials

### "Insufficient permissions" Error
- User's role doesn't include required permission
- Solution: Contact admin to adjust role/permissions

### "Account locked" Error
- Too many failed login attempts
- Solution: Wait 15 minutes or contact admin
