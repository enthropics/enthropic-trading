import { Pool } from 'pg';
import Redis from 'ioredis';
import { WebSocketHandler } from './websocket-handler';
import { loadConfig } from './config';

async function main() {
  const config = loadConfig();

  console.log('Starting NATS Gateway...');

  // Initialize PostgreSQL pool
  const pool = new Pool({
    connectionString: config.databaseUrl,
    max: 20,
  });

  // Test database connection
  try {
    const client = await pool.connect();
    console.log('Connected to PostgreSQL');
    client.release();
  } catch (error) {
    console.error('Failed to connect to PostgreSQL:', error);
    process.exit(1);
  }

  // Initialize Redis
  const redis = new Redis(config.redisUrl, {
    maxRetriesPerRequest: 3,
    retryStrategy: (times) => Math.min(times * 50, 2000),
  });

  redis.on('connect', () => {
    console.log('Connected to Redis');
  });

  redis.on('error', (error) => {
    console.error('Redis error:', error);
  });

  // Initialize WebSocket handler
  const wsHandler = new WebSocketHandler(pool, redis, config);
  await wsHandler.start();

  console.log(`NATS Gateway running on port ${config.port}`);

  // Graceful shutdown
  const shutdown = async () => {
    console.log('Shutting down...');
    wsHandler.stop();
    await pool.end();
    redis.disconnect();
    process.exit(0);
  };

  process.on('SIGTERM', shutdown);
  process.on('SIGINT', shutdown);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
