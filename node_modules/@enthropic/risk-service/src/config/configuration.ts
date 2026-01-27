export default () => ({
  port: parseInt(process.env.PORT || '3001', 10),
  database: {
    url: process.env.DATABASE_URL,
  },
  jwt: {
    secret: process.env.JWT_SECRET,
    expiresIn: process.env.TOKEN_EXPIRY_MINUTES || '15',
  },
  redis: {
    url: process.env.REDIS_URL || 'redis://localhost:6379',
  },
  nats: {
    url: process.env.NATS_URL || 'nats://localhost:4222',
  },
});
