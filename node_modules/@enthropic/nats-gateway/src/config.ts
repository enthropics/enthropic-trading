export interface Config {
  port: number;
  natsUrl: string;
  redisUrl: string;
  databaseUrl: string;
  jwtSecret: string;
  corsOrigin: string;
  rateLimitRequests: number;
  rateLimitWindowSeconds: number;
}

export function loadConfig(): Config {
  const required = ['JWT_SECRET', 'DATABASE_URL'];
  for (const key of required) {
    if (!process.env[key]) {
      throw new Error(`Missing required environment variable: ${key}`);
    }
  }

  return {
    port: parseInt(process.env.PORT || '3002', 10),
    natsUrl: process.env.NATS_URL || 'nats://localhost:4222',
    redisUrl: process.env.REDIS_URL || 'redis://localhost:6379',
    databaseUrl: process.env.DATABASE_URL!,
    jwtSecret: process.env.JWT_SECRET!,
    corsOrigin: process.env.CORS_ORIGIN || 'http://localhost:5173',
    rateLimitRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10),
    rateLimitWindowSeconds: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '60000', 10) / 1000,
  };
}
