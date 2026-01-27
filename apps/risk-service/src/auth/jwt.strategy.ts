import { Injectable, UnauthorizedException } from '@nestjs/common';
import { PassportStrategy } from '@nestjs/passport';
import { Strategy, ExtractJwt } from 'passport-jwt';
import { ConfigService } from '@nestjs/config';
import { PrismaService } from '../prisma/prisma.service';
import { JwtPayload, AuthenticatedUser } from './auth.types';
import Redis from 'ioredis';

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  private redis: Redis;

  constructor(
    private configService: ConfigService,
    private prisma: PrismaService,
  ) {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      ignoreExpiration: false,
      secretOrKey: configService.get<string>('JWT_SECRET'),
    });
    this.redis = new Redis(configService.get<string>('REDIS_URL') || 'redis://localhost:6379');
  }

  async validate(payload: JwtPayload): Promise<AuthenticatedUser> {
    // Check if token is blacklisted
    const isBlacklisted = await this.redis.exists(`token_blacklist:${payload.jti}`);
    if (isBlacklisted) {
      throw new UnauthorizedException('Token has been revoked');
    }

    // Verify account exists and is active
    const account = await this.prisma.account.findUnique({
      where: { id: payload.sub },
      select: { id: true, isActive: true, lockedUntil: true },
    });

    if (!account) {
      throw new UnauthorizedException('Account not found');
    }

    if (!account.isActive) {
      throw new UnauthorizedException('Account is disabled');
    }

    if (account.lockedUntil && account.lockedUntil > new Date()) {
      throw new UnauthorizedException('Account is locked');
    }

    return {
      accountId: payload.sub,
      username: payload.username,
      role: payload.role,
      permissions: new Set(payload.permissions),
      tokenJti: payload.jti,
    };
  }
}
