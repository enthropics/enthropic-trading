import { Injectable } from '@nestjs/common';

@Injectable()
export class ConfigService {
  get databaseUrl(): string {
    return process.env.DATABASE_URL || '';
  }

  get redisUrl(): string {
    return process.env.REDIS_URL || 'redis://localhost:6379';
  }

  get jwtSecret(): string {
    const secret = process.env.JWT_SECRET;
    if (!secret) throw new Error('JWT_SECRET required');
    return secret;
  }

  get jwtAccessExpiry(): string {
    return process.env.JWT_ACCESS_EXPIRY || '15m';
  }

  get jwtRefreshExpiry(): string {
    return process.env.JWT_REFRESH_EXPIRY || '7d';
  }

  get port(): number {
    return parseInt(process.env.PORT || '3000', 10);
  }
}
