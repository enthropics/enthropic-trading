import { Test, TestingModule } from '@nestjs/testing';
import { RiskService } from '../src/risk/risk.service';
import { PrismaService } from '../src/prisma/prisma.service';
import { ConfigService } from '@nestjs/config';
import { Decimal } from '@prisma/client/runtime/library';

describe('RiskService', () => {
  let service: RiskService;
  let prisma: PrismaService;

  const mockPrismaService = {
    account: {
      findUnique: jest.fn(),
    },
    position: {
      findMany: jest.fn(),
      aggregate: jest.fn(),
    },
    order: {
      findMany: jest.fn(),
      count: jest.fn(),
    },
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        RiskService,
        {
          provide: PrismaService,
          useValue: mockPrismaService,
        },
        {
          provide: ConfigService,
          useValue: {
            get: jest.fn().mockReturnValue('100000'),
          },
        },
      ],
    }).compile();

    service = module.get<RiskService>(RiskService);
    prisma = module.get<PrismaService>(PrismaService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('checkOrderRisk', () => {
    it('should approve order within risk limits', async () => {
      const accountId = '123e4567-e89b-12d3-a456-426614174000';
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: accountId,
        balance: new Decimal('100000'),
        is_active: true,
      });

      mockPrismaService.position.aggregate.mockResolvedValue({
        _sum: { quantity: new Decimal('10') },
      });

      const result = await service.checkOrderRisk(accountId, {
        symbol: 'BTC-USD',
        side: 'buy',
        quantity: 1,
        price: 50000,
      });

      expect(result.approved).toBe(true);
      expect(result.checks).toBeDefined();
    });

    it('should reject order exceeding max order size', async () => {
      const accountId = '123e4567-e89b-12d3-a456-426614174000';
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: accountId,
        balance: new Decimal('100000'),
        is_active: true,
      });

      const result = await service.checkOrderRisk(accountId, {
        symbol: 'BTC-USD',
        side: 'buy',
        quantity: 10000, // Exceeds typical limit
        price: 50000,
      });

      expect(result.approved).toBe(false);
      expect(result.reason).toContain('order size');
    });

    it('should reject order for inactive account', async () => {
      const accountId = '123e4567-e89b-12d3-a456-426614174000';
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: accountId,
        balance: new Decimal('100000'),
        is_active: false,
      });

      const result = await service.checkOrderRisk(accountId, {
        symbol: 'BTC-USD',
        side: 'buy',
        quantity: 1,
        price: 50000,
      });

      expect(result.approved).toBe(false);
      expect(result.reason).toContain('inactive');
    });
  });

  describe('getAccountRiskSummary', () => {
    it('should return risk summary for account', async () => {
      const accountId = '123e4567-e89b-12d3-a456-426614174000';
      
      mockPrismaService.account.findUnique.mockResolvedValue({
        id: accountId,
        balance: new Decimal('100000'),
        is_active: true,
      });

      mockPrismaService.position.findMany.mockResolvedValue([
        {
          symbol: 'BTC-USD',
          quantity: new Decimal('1'),
          avg_price: new Decimal('50000'),
          unrealized_pnl: new Decimal('1000'),
          realized_pnl: new Decimal('500'),
        },
      ]);

      mockPrismaService.order.count.mockResolvedValue(5);

      const result = await service.getAccountRiskSummary(accountId);

      expect(result.accountId).toBe(accountId);
      expect(result.totalExposure).toBeDefined();
      expect(result.totalUnrealizedPnl).toBeDefined();
      expect(result.openOrderCount).toBe(5);
    });
  });
});
