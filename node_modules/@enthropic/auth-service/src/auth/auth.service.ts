import { Injectable, UnauthorizedException, BadRequestException, ForbiddenException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import * as bcrypt from 'bcryptjs';
import { createHash, randomUUID } from 'crypto';
import { PrismaService } from '../prisma/prisma.service';
import { ConfigService } from '../config/config.service';
import { LoginDto, RegisterDto, RefreshTokenDto } from './auth.dto';

export interface JwtPayload {
  sub: string;
  username: string;
  role: string;
  permissions: string[];
  jti: string;
  iat: number;
  exp: number;
}

export interface AuthResponse {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
  user: {
    id: string;
    username: string;
    email: string;
    role: string;
    permissions: string[];
  };
}

@Injectable()
export class AuthService {
  constructor(
    private prisma: PrismaService,
    private jwtService: JwtService,
    private config: ConfigService,
  ) {}

  async validateUser(username: string, password: string): Promise<any> {
    const account = await this.prisma.account.findUnique({
      where: { username },
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
      await this.logAuditEvent(null, 'login_failed', { username, reason: 'not_found' }, false);
      throw new UnauthorizedException('Invalid credentials');
    }

    // Check if account is locked
    if (account.lockedUntil && account.lockedUntil > new Date()) {
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'account_locked' }, false);
      throw new ForbiddenException('Account is locked. Try again later.');
    }

    // Check if account is active
    if (!account.isActive) {
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'account_disabled' }, false);
      throw new ForbiddenException('Account is disabled');
    }

    // Verify password
    const isValid = await bcrypt.compare(password, account.passwordHash);
    if (!isValid) {
      await this.incrementFailedLogins(account.id);
      await this.logAuditEvent(account.id, 'login_failed', { reason: 'invalid_password' }, false);
      throw new UnauthorizedException('Invalid credentials');
    }

    // Reset failed attempts on success
    await this.prisma.account.update({
      where: { id: account.id },
      data: { 
        failedLoginAttempts: 0, 
        lastLoginAt: new Date(),
        lockedUntil: null,
      },
    });

    return account;
  }

  async login(dto: LoginDto, ipAddress?: string, userAgent?: string): Promise<AuthResponse> {
    const account = await this.validateUser(dto.username, dto.password);
    
    const permissions = account.role?.permissions.map((rp: any) => rp.permission.name) || [];
    const jti = randomUUID();
    
    const payload: Omit<JwtPayload, 'iat' | 'exp'> = {
      sub: account.id,
      username: account.username,
      role: account.role?.name || 'viewer',
      permissions,
      jti,
    };

    const accessToken = this.jwtService.sign(payload);
    const refreshToken = await this.createRefreshToken(account.id, ipAddress, userAgent);
    
    const decoded = this.jwtService.decode(accessToken) as JwtPayload;
    
    await this.logAuditEvent(account.id, 'login_success', { jti }, true, ipAddress, userAgent);

    return {
      accessToken,
      refreshToken,
      expiresAt: decoded.exp,
      user: {
        id: account.id,
        username: account.username,
        email: account.email,
        role: account.role?.name || 'viewer',
        permissions,
      },
    };
  }

  async register(dto: RegisterDto): Promise<{ message: string }> {
    const existing = await this.prisma.account.findFirst({
      where: {
        OR: [{ username: dto.username }, { email: dto.email }],
      },
    });

    if (existing) {
      throw new BadRequestException('Username or email already exists');
    }

    const passwordHash = await bcrypt.hash(dto.password, 12);
    
    // Get default trader role
    const traderRole = await this.prisma.role.findUnique({
      where: { name: 'trader' },
    });

    await this.prisma.account.create({
      data: {
        username: dto.username,
        email: dto.email,
        passwordHash,
        roleId: traderRole?.id,
        balance: 100000, // Demo balance
        availableBalance: 100000,
      },
    });

    return { message: 'Account created successfully' };
  }

  async refreshTokens(dto: RefreshTokenDto): Promise<{ accessToken: string; expiresAt: number }> {
    const tokenHash = this.hashToken(dto.refreshToken);
    
    const storedToken = await this.prisma.refreshToken.findUnique({
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

    if (!storedToken) {
      throw new UnauthorizedException('Invalid refresh token');
    }

    if (storedToken.revokedAt) {
      throw new UnauthorizedException('Refresh token has been revoked');
    }

    if (storedToken.expiresAt < new Date()) {
      throw new UnauthorizedException('Refresh token expired');
    }

    if (!storedToken.account.isActive) {
      throw new ForbiddenException('Account is disabled');
    }

    const permissions = storedToken.account.role?.permissions.map((rp: any) => rp.permission.name) || [];
    const jti = randomUUID();

    const payload: Omit<JwtPayload, 'iat' | 'exp'> = {
      sub: storedToken.account.id,
      username: storedToken.account.username,
      role: storedToken.account.role?.name || 'viewer',
      permissions,
      jti,
    };

    const accessToken = this.jwtService.sign(payload);
    const decoded = this.jwtService.decode(accessToken) as JwtPayload;

    return { accessToken, expiresAt: decoded.exp };
  }

  async logout(jti: string, accountId: string): Promise<void> {
    // Blacklist the access token
    await this.prisma.tokenBlacklist.create({
      data: {
        tokenJti: jti,
        accountId,
        expiresAt: new Date(Date.now() + 24 * 60 * 60 * 1000), // 1 day
        reason: 'logout',
      },
    });

    // Revoke all refresh tokens for this account
    await this.prisma.refreshToken.updateMany({
      where: { accountId, revokedAt: null },
      data: { revokedAt: new Date(), revokedReason: 'logout' },
    });

    await this.logAuditEvent(accountId, 'logout', { jti }, true);
  }

  async isTokenBlacklisted(jti: string): Promise<boolean> {
    const blacklisted = await this.prisma.tokenBlacklist.findUnique({
      where: { tokenJti: jti },
    });
    return !!blacklisted;
  }

  async validateJwtPayload(payload: JwtPayload): Promise<any> {
    // Check if token is blacklisted
    if (await this.isTokenBlacklisted(payload.jti)) {
      throw new UnauthorizedException('Token has been revoked');
    }

    const account = await this.prisma.account.findUnique({
      where: { id: payload.sub },
      include: {
        role: {
          include: {
            permissions: { include: { permission: true } },
          },
        },
      },
    });

    if (!account || !account.isActive) {
      throw new UnauthorizedException('Account not found or disabled');
    }

    if (account.lockedUntil && account.lockedUntil > new Date()) {
      throw new ForbiddenException('Account is locked');
    }

    return {
      id: account.id,
      username: account.username,
      role: account.role?.name || 'viewer',
      permissions: account.role?.permissions.map((rp: any) => rp.permission.name) || [],
    };
  }

  private async createRefreshToken(accountId: string, ipAddress?: string, userAgent?: string): Promise<string> {
    const token = randomUUID();
    const tokenHash = this.hashToken(token);
    
    // 7 days expiry
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + 7);

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
    const account = await this.prisma.account.findUnique({
      where: { id: accountId },
    });

    if (!account) return;

    const newAttempts = account.failedLoginAttempts + 1;
    const lockedUntil = newAttempts >= 5 
      ? new Date(Date.now() + 15 * 60 * 1000) // Lock for 15 minutes
      : null;

    await this.prisma.account.update({
      where: { id: accountId },
      data: {
        failedLoginAttempts: newAttempts,
        lockedUntil,
      },
    });
  }

  private async logAuditEvent(
    accountId: string | null,
    eventType: string,
    eventData: Record<string, any>,
    success: boolean,
    ipAddress?: string,
    userAgent?: string,
  ): Promise<void> {
    await this.prisma.auditLog.create({
      data: {
        accountId,
        eventType,
        eventData,
        success,
        ipAddress,
        userAgent,
      },
    });
  }
}
