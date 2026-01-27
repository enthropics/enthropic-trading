import { Test, TestingModule } from '@nestjs/testing';
import { JwtService } from '@nestjs/jwt';
import { AuthService } from '../src/auth/auth.service';
import { PrismaService } from '../src/prisma/prisma.service';
import { ConfigService } from '@nestjs/config';
import Redis from 'ioredis';
import * as bcrypt from 'bcryptjs';

describe('AuthService', () => {
  let service: AuthService;
  let prisma: PrismaService;
  let jwtService: JwtService;
  let redis: Redis;

  const mockPrismaService = {
    account: {
      findUnique: jest.fn(),
      update: jest.fn(),
    },
    refreshToken: {
      create: jest.fn(),
      findFirst: jest.fn(),
      delete: jest.fn(),
    },
    tokenBlacklist: {
      create: jest.fn(),
      findUnique: jest.fn(),
    },
    auditLog: {
      create: jest.fn(),
    },
  };

  const mockJwtService = {
    sign: jest.fn().mockReturnValue('mock-jwt-token'),
    verify: jest.fn(),
  };

  const mockRedis = {
    setex: jest.fn(),
    get: jest.fn(),
    exists: jest.fn().mockResolvedValue(0),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        AuthService,
        {
          provide: PrismaService,
          useValue: mockPrismaService,
        },
        {
          provide: JwtService,
          useValue: mockJwtService,
        },
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn((key: string) => {
              const config: Record<string, any> = {
                TOKEN_EXPIRY_MINUTES: '15',
                REFRESH_TOKEN_EXPIRY_DAYS: '7',
              };
              return config[key];
            }),
          },
        },
        {
          provide: 'REDIS_CLIENT',
          useValue: mockRedis,
        },
      ],
    }).compile();

    service = module.get<AuthService>(AuthService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('login', () => {
    it('should return tokens for valid credentials', async () => {
      const hashedPassword = await bcrypt.hash('password123', 12);
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: '123e4567-e89b-12d3-a456-426614174000',
        username: 'testuser',
        password_hash: hashedPassword,
        is_active: true,
        locked_until: null,
        failed_login_attempts: 0,
        role: {
          name: 'trader',
          permissions: [{ permission: { name: 'orders:create' } }],
        },
      });

      mockPrismaService.refreshToken.create.mockResolvedValue({
        id: 'refresh-id',
      });

      const result = await service.login('testuser', 'password123', '127.0.0.1');

      expect(result.accessToken).toBe('mock-jwt-token');
      expect(result.user.username).toBe('testuser');
    });

    it('should reject invalid password', async () => {
      const hashedPassword = await bcrypt.hash('password123', 12);
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: '123e4567-e89b-12d3-a456-426614174000',
        username: 'testuser',
        password_hash: hashedPassword,
        is_active: true,
        locked_until: null,
        failed_login_attempts: 0,
      });

      await expect(
        service.login('testuser', 'wrongpassword', '127.0.0.1')
      ).rejects.toThrow('Invalid credentials');
    });

    it('should reject locked account', async () => {
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: '123e4567-e89b-12d3-a456-426614174000',
        username: 'testuser',
        is_active: true,
        locked_until: new Date(Date.now() + 3600000), // Locked for 1 hour
        failed_login_attempts: 5,
      });

      await expect(
        service.login('testuser', 'password123', '127.0.0.1')
      ).rejects.toThrow('locked');
    });
  });
});
