/**
 * Authentication & Authorization Module for NATS Gateway
 * Phase 2: JWT validation, RBAC, rate limiting, token blacklist
 */

import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';
import { v4 as uuidv4 } from 'uuid';
import { createHash } from 'crypto';
import { Pool } from 'pg';
import Redis from 'ioredis';
import { Config } from './config';

export interface JwtClaims {
  sub: string;
  username: string;
  role: string;
  permissions: string[];
  exp: number;
  iat: number;
  jti: string;
}

export interface AuthContext {
  accountId: string;
  username: string;
  role: string;
  permissions: Set<string>;
  tokenJti: string;
}

export class AuthError extends Error {
  constructor(
      message: string,
      public code: string,
      public statusCode: number = 401
  ) {
    super(message);
    this.name = 'AuthError';
  }
}

export class AuthService {
  private pool: Pool;
  private redis: Redis;
  private config: Config;

  constructor(pool: Pool, redis: Redis, config: Config) {
    this.pool = pool;
    this.redis = redis;
    this.config = config;
  }

  /**
   * Validate JWT token and return auth context
   */
  async validateToken(token: string): Promise<AuthContext> {
    let claims: JwtClaims;

    try {
      claims = jwt.verify(token, this.config.jwtSecret) as JwtClaims;
    } catch (err) {
      if (err instanceof jwt.TokenExpiredError) {
        throw new AuthError('Token expired', 'TOKEN_EXPIRED');
      }
      throw new AuthError('Invalid token', 'INVALID_TOKEN');
    }

    // Check if token is blacklisted (Redis first for speed)
    const blacklistKey = `token_blacklist:${claims.jti}`;
    const isBlacklisted = await this.redis.exists(blacklistKey);
    if (isBlacklisted) {
      throw new AuthError('Token revoked', 'TOKEN_REVOKED');
    }

    // Also check database (in case Redis was restarted)
    const dbBlacklist = await this.pool.query(
        'SELECT 1 FROM token_blacklist WHERE token_jti = $1',
        [claims.jti]
    );
    if (dbBlacklist.rows.length > 0) {
      // Re-add to Redis
      await this.redis.setex(blacklistKey, 86400, '1');
      throw new AuthError('Token revoked', 'TOKEN_REVOKED');
    }

    // Verify account still exists and is active
    const result = await this.pool.query(
        'SELECT id, is_active, locked_until FROM accounts WHERE id = $1',
        [claims.sub]
    );

    if (result.rows.length === 0) {
      throw new AuthError('Account not found', 'ACCOUNT_NOT_FOUND');
    }

    const account = result.rows[0];
    if (!account.is_active) {
      throw new AuthError('Account disabled', 'ACCOUNT_DISABLED', 403);
    }

    if (account.locked_until && new Date(account.locked_until) > new Date()) {
      throw new AuthError('Account locked', 'ACCOUNT_LOCKED', 403);
    }

    return {
      accountId: claims.sub,
      username: claims.username,
      role: claims.role,
      permissions: new Set(claims.permissions),
      tokenJti: claims.jti,
    };
  }

  /**
   * Check if context has a specific permission
   */
  hasPermission(ctx: AuthContext, permission: string): boolean {
    return ctx.permissions.has(permission) || ctx.permissions.has('admin:full');
  }

  /**
   * Check if context has any of the specified permissions
   */
  hasAnyPermission(ctx: AuthContext, permissions: string[]): boolean {
    return permissions.some(p => this.hasPermission(ctx, p));
  }

  /**
   * Check if context can access a specific account's resources
   */
  canAccessAccount(ctx: AuthContext, targetAccountId: string): boolean {
    return ctx.accountId === targetAccountId ||
        this.hasPermission(ctx, 'admin:full') ||
        this.hasPermission(ctx, 'accounts:read_all');
  }

  /**
   * Check rate limit for an account/action
   */
  async checkRateLimit(accountId: string, action: string): Promise<boolean> {
    const key = `rate_limit:${accountId}:${action}`;
    const count = await this.redis.incr(key);

    if (count === 1) {
      await this.redis.expire(key, this.config.rateLimitWindowSeconds);
    }

    return count <= this.config.rateLimitRequests;
  }

  /**
   * Revoke a token by adding to blacklist
   */
  async revokeToken(jti: string, accountId: string, reason: string = 'logout'): Promise<void> {
    // Add to database
    await this.pool.query(
        `INSERT INTO token_blacklist (token_jti, account_id, expires_at, reason)
       VALUES ($1, $2, NOW() + INTERVAL '1 day', $3)
       ON CONFLICT (token_jti) DO NOTHING`,
        [jti, accountId, reason]
    );

    // Add to Redis (TTL 1 day)
    await this.redis.setex(`token_blacklist:${jti}`, 86400, '1');
  }

  /**
   * Login user with username/email and password
   */
  async login(username: string, password: string, ipAddress: string): Promise<{
    authContext: AuthContext;
    accessToken: string;
    refreshToken: string;
    expiresAt: Date;
  }> {
    // Query user by username or email
    const result = await this.pool.query(
        `SELECT account_id, username, email, password_hash, role, is_active, locked_until 
       FROM accounts 
       WHERE (username = $1 OR email = $1) AND is_active = true`,
        [username]
    );

    if (result.rows.length === 0) {
      throw new AuthError('Invalid credentials', 'INVALID_CREDENTIALS', 401);
    }

    const account = result.rows[0];

    // Check if account is locked
    if (account.locked_until && new Date(account.locked_until) > new Date()) {
      throw new AuthError('Account locked', 'ACCOUNT_LOCKED', 403);
    }

    // Verify password with bcrypt
    const isValid = await bcrypt.compare(password, account.password_hash);

    if (!isValid) {
      throw new AuthError('Invalid credentials', 'INVALID_CREDENTIALS', 401);
    }

    // Get user permissions from database
    const permissionsResult = await this.pool.query(
        `SELECT permission FROM account_permissions WHERE account_id = $1`,
        [account.account_id]
    );

    const permissions = permissionsResult.rows.map(row => row.permission);

    // Generate JWT tokens
    const accessTokenJti = uuidv4();
    const refreshTokenJti = uuidv4();
    const expiresAt = new Date(Date.now() + 3600000); // 1 hour

    const accessToken = jwt.sign(
        {
          accountId: account.account_id,
          username: account.username,
          role: account.role,
          permissions,
          jti: accessTokenJti,
          type: 'access',
        },
        this.config.jwtSecret,
        { expiresIn: '1h' }
    );

    const refreshToken = jwt.sign(
        {
          accountId: account.account_id,
          username: account.username,
          jti: refreshTokenJti,
          type: 'refresh',
        },
        this.config.jwtSecret,
        { expiresIn: '7d' }
    );

    // Store refresh token in database
    await this.pool.query(
        `INSERT INTO refresh_tokens (token_jti, account_id, expires_at, ip_address)
       VALUES ($1, $2, NOW() + INTERVAL '7 days', $3)`,
        [refreshTokenJti, account.account_id, ipAddress]
    );

    const authContext: AuthContext = {
      accountId: account.account_id,
      username: account.username,
      role: account.role,
      permissions: new Set(permissions),
      tokenJti: accessTokenJti,
    };

    return {
      authContext,
      accessToken,
      refreshToken,
      expiresAt,
    };
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken: string): Promise<{
    accessToken: string;
    expiresAt: Date;
  }> {
    let claims: any;

    try {
      claims = jwt.verify(refreshToken, this.config.jwtSecret);
    } catch (err) {
      throw new AuthError('Invalid refresh token', 'INVALID_TOKEN', 401);
    }

    // Validate it's a refresh token
    if (claims.type !== 'refresh') {
      throw new AuthError('Not a refresh token', 'INVALID_TOKEN', 401);
    }

    // Check if refresh token is revoked
    const blacklistKey = `token_blacklist:${claims.jti}`;
    const isBlacklisted = await this.redis.exists(blacklistKey);

    if (isBlacklisted) {
      throw new AuthError('Token revoked', 'TOKEN_REVOKED', 401);
    }

    // Check in database
    const dbCheck = await this.pool.query(
        `SELECT account_id, expires_at FROM refresh_tokens 
       WHERE token_jti = $1 AND revoked_at IS NULL`,
        [claims.jti]
    );

    if (dbCheck.rows.length === 0) {
      throw new AuthError('Token not found or revoked', 'TOKEN_REVOKED', 401);
    }

    const tokenData = dbCheck.rows[0];

    // Check if expired
    if (new Date(tokenData.expires_at) < new Date()) {
      throw new AuthError('Refresh token expired', 'TOKEN_EXPIRED', 401);
    }

    // Get fresh user data and permissions
    const userResult = await this.pool.query(
        `SELECT account_id, username, role, is_active, locked_until 
       FROM accounts 
       WHERE account_id = $1`,
        [tokenData.account_id]
    );

    if (userResult.rows.length === 0) {
      throw new AuthError('Account not found', 'ACCOUNT_NOT_FOUND', 404);
    }

    const account = userResult.rows[0];

    if (!account.is_active) {
      throw new AuthError('Account disabled', 'ACCOUNT_DISABLED', 403);
    }

    if (account.locked_until && new Date(account.locked_until) > new Date()) {
      throw new AuthError('Account locked', 'ACCOUNT_LOCKED', 403);
    }

    // Get current permissions
    const permissionsResult = await this.pool.query(
        `SELECT permission FROM account_permissions WHERE account_id = $1`,
        [account.account_id]
    );

    const permissions = permissionsResult.rows.map(row => row.permission);

    // Generate new access token
    const newAccessTokenJti = uuidv4();
    const expiresAt = new Date(Date.now() + 3600000); // 1 hour

    const accessToken = jwt.sign(
        {
          accountId: account.account_id,
          username: account.username,
          role: account.role,
          permissions,
          jti: newAccessTokenJti,
          type: 'access',
        },
        this.config.jwtSecret,
        { expiresIn: '1h' }
    );

    return {
      accessToken,
      expiresAt,
    };
  }
}

// Permission constants
export const Permissions = {
  ORDERS_CREATE: 'orders:create',
  ORDERS_READ: 'orders:read',
  ORDERS_CANCEL: 'orders:cancel',
  ORDERS_READ_ALL: 'orders:read_all',
  POSITIONS_READ: 'positions:read',
  POSITIONS_READ_ALL: 'positions:read_all',
  MARKET_READ: 'market:read',
  MARKET_SUBSCRIBE: 'market:subscribe',
  ACCOUNTS_READ: 'accounts:read',
  ACCOUNTS_READ_ALL: 'accounts:read_all',
  RISK_READ: 'risk:read',
  RISK_MANAGE: 'risk:manage',
  ADMIN_FULL: 'admin:full',
} as const;