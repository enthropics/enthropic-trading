import { Injectable, UnauthorizedException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { ConfigService } from '@nestjs/config';
import * as bcrypt from 'bcryptjs';
import { createHash, randomUUID } from 'crypto';
import { PrismaService } from '../prisma/prisma.service';
import Redis from 'ioredis';
import { LoginDto, TokenResponseDto } from './auth.types';

@Injectable()
export class AuthService {
  private readonly redis: Redis;
  private readonly tokenExpiryMinutes: number;
  private readonly refreshTokenExpiryDays: number;

  constructor(
      private readonly prisma: PrismaService,
      private readonly jwtService: JwtService,
      configService: ConfigService,
  ) {
    this.redis = new Redis(configService.get<string>('REDIS_URL') || 'redis://localhost:6379');
    this.tokenExpiryMinutes = configService.get<number>('TOKEN_EXPIRY_MINUTES', 15);
    this.refreshTokenExpiryDays = configService.get<number>('REFRESH_TOKEN_EXPIRY_DAYS', 7);
  }

  async login(dto: LoginDto, ipAddress?: string, userAgent?: string): Promise<TokenResponseDto> {
    // Get account with role and permissions
    const account = await this.prisma.account.findUnique({
      where: { username: dto.username },
      include: {
        role: {
          include: {
            permissions: {
              include: { permission: true },
            },
          },
        },
      },
    });

    if (!account) {
      await this.logAuditEvent(null, 'login_failed', { username: dto.username, reason: 'not_found' }, ipAddress, userAgent, false);
      throw new UnauthorizedException('Invalid credentials');
    }

    // Check if locked
    if (account.lockedUntil && account.lockedUntil > new Date()) {
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'locked' }, ipAddress, userAgent, false);
      throw new UnauthorizedException('Account is locked');
    }

    // Check if active
    if (!account.isActive) {
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'disabled' }, ipAddress, userAgent, false);
      throw new UnauthorizedException('Account is disabled');
    }

    // Verify password
    const validPassword = await bcrypt.compare(dto.password, account.passwordHash);
    if (!validPassword) {
      await this.incrementFailedLogins(account.id);
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'invalid_password' }, ipAddress, userAgent, false);
      throw new UnauthorizedException('Invalid credentials');
    }

    // Reset failed login attempts
    await this.prisma.account.update({
      where: { id: account.id },
      data: { failedLoginAttempts: 0, lastLoginAt: new Date() },
    });

    // Generate tokens
    const permissions = account.role?.permissions.map((rp: any) => rp.permission.name) || [];
    const jti = randomUUID();

    const payload = {
      sub: account.id,
      username: account.username,
      role: account.role?.name || 'viewer',
      permissions,
      jti,
    };

    const accessToken = this.jwtService.sign(payload, {
      expiresIn: `${this.tokenExpiryMinutes}m`,
    });

    const refreshToken = await this.generateRefreshToken(account.id, ipAddress, userAgent);

    await this.logAuditEvent(account.id, 'login_success', { jti }, ipAddress, userAgent, true);

    // Calculate expiresAt
    const expiresAt = Math.floor(Date.now() / 1000) + (this.tokenExpiryMinutes * 60);

    return {
      accessToken,
      refreshToken,
      expiresAt,
      user: {
        id: account.id,
        username: account.username,
        role: account.role?.name || 'viewer',
        permissions,
      },
    };
  }

  async refreshAccessToken(refreshToken: string): Promise<{ accessToken: string; expiresAt: number }> {
    const tokenHash = this.hashToken(refreshToken);

    const token = await this.prisma.refreshToken.findUnique({
      where: { tokenHash },
      include: {
        account: {
          include: {
            role: {
              include: {
                permissions: { include: { permission: true } },
              },
            },
          },
        },
      },
    });

    if (!token) {
      throw new UnauthorizedException('Invalid refresh token');
    }

    if (token.revokedAt) {
      throw new UnauthorizedException('Refresh token revoked');
    }

    if (token.expiresAt < new Date()) {
      throw new UnauthorizedException('Refresh token expired');
    }

    if (!token.account.isActive) {
      throw new UnauthorizedException('Account disabled');
    }

    const permissions = token.account.role?.permissions.map((rp: any) => rp.permission.name) || [];
    const jti = randomUUID();

    const payload = {
      sub: token.account.id,
      username: token.account.username,
      role: token.account.role?.name || 'viewer',
      permissions,
      jti,
    };

    const accessToken = this.jwtService.sign(payload, {
      expiresIn: `${this.tokenExpiryMinutes}m`,
    });

    const expiresAt = Math.floor(Date.now() / 1000) + (this.tokenExpiryMinutes * 60);

    return { accessToken, expiresAt };
  }

  async logout(jti: string, accountId: string): Promise<void> {
    // Add to blacklist
    await this.prisma.tokenBlacklist.create({
      data: {
        tokenJti: jti,
        accountId,
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000),
        reason: 'logout',
      },
    });

    // Add to Redis for fast lookup
    await this.redis.setex(`token_blacklist:${jti}`, 86400, '1');
  }

  private async generateRefreshToken(accountId: string, ipAddress?: string, userAgent?: string): Promise<string> {
    const token = randomUUID();
    const tokenHash = this.hashToken(token);
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + this.refreshTokenExpiryDays);

    await this.prisma.refreshToken.create({
      data: {
        accountId,
        tokenHash,
        expiresAt,
        ipAddress,
        userAgent,
      },
    });

    return token;
  }

  private hashToken(token: string): string {
    return createHash('sha256').update(token).digest('hex');
  }

  private async incrementFailedLogins(accountId: string): Promise<void> {
    const account = await this.prisma.account.update({
      where: { id: accountId },
      data: { failedLoginAttempts: { increment: 1 } },
    });

    if (account.failedLoginAttempts >= 5) {
      await this.prisma.account.update({
        where: { id: accountId },
        data: { lockedUntil: new Date(Date.now() + 15 * 60 * 1000) },
      });
    }
  }

  private async logAuditEvent(
      accountId: string | null,
      eventType: string,
      eventData: Record<string, any>,
      ipAddress?: string,
      userAgent?: string,
      success: boolean = true,
  ): Promise<void> {
    await this.prisma.auditLog.create({
      data: {
        accountId,
        eventType,
        eventData,
        ipAddress,
        userAgent,
        success,
      },
    });
  }
}