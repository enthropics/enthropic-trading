import { Injectable } from '@nestjs/common';
import { Decimal } from '@prisma/client/runtime/library';
import { PrismaService } from '../prisma/prisma.service';
import { RiskCheckResult, AccountRiskSummary, OrderRiskCheck } from './risk.types';
import { AuthenticatedUser } from '../auth/auth.types';

@Injectable()
export class RiskService {
  constructor(private prisma: PrismaService) {}

  async checkOrderRisk(user: AuthenticatedUser, order: OrderRiskCheck): Promise<RiskCheckResult> {
    // Get account with limits
    const account = await this.prisma.account.findUnique({
      where: { id: order.accountId },
    });

    if (!account) {
      return { allowed: false, reason: 'Account not found', code: 'ACCOUNT_NOT_FOUND' };
    }

    // Check authorization
    if (user.accountId !== order.accountId && !user.permissions.has('admin:full')) {
      return { allowed: false, reason: 'Unauthorized', code: 'UNAUTHORIZED' };
    }

    // Check order size limit
    if (order.quantity.greaterThan(account.maxOrderSize)) {
      return {
        allowed: false,
        reason: `Order size ${order.quantity} exceeds limit ${account.maxOrderSize}`,
        code: 'ORDER_SIZE_EXCEEDED'
      };
    }

    // Get current position for the symbol
    const position = await this.prisma.position.findUnique({
      where: { accountId_symbol: { accountId: order.accountId, symbol: order.symbol } },
    });

    // Calculate new position size after order
    const currentQty = position?.netQuantity || new Decimal(0);
    const orderQty = order.side === 'buy' ? order.quantity : order.quantity.negated();
    const newQty = currentQty.add(orderQty);

    // Check position size limit
    if (newQty.abs().greaterThan(account.maxPositionSize)) {
      return {
        allowed: false,
        reason: `Resulting position ${newQty.abs()} exceeds limit ${account.maxPositionSize}`,
        code: 'POSITION_SIZE_EXCEEDED'
      };
    }

    // Get total exposure
    const positions = await this.prisma.position.findMany({
      where: { accountId: order.accountId },
    });

    const totalExposure = positions.reduce(
        (sum: Decimal, pos: any) => sum.add(pos.netQuantity.abs().mul(pos.avgPrice)),
        new Decimal(0)
    );

    const orderValue = order.quantity.mul(order.price || new Decimal(0));
    const newExposure = totalExposure.add(orderValue);

    // Check if exposure exceeds balance
    if (newExposure.greaterThan(account.balance)) {
      return {
        allowed: false,
        reason: `Insufficient balance. Required: ${newExposure}, Available: ${account.balance}`,
        code: 'INSUFFICIENT_BALANCE'
      };
    }

    return { allowed: true };
  }

  async getAccountRiskSummary(user: AuthenticatedUser, accountId: string): Promise<AccountRiskSummary> {
    // Check authorization
    if (user.accountId !== accountId &&
        !user.permissions.has('risk:read') &&
        !user.permissions.has('admin:full')) {
      throw new Error('Unauthorized to view risk summary');
    }

    const account = await this.prisma.account.findUnique({
      where: { id: accountId },
    });

    if (!account) {
      throw new Error('Account not found');
    }

    const positions = await this.prisma.position.findMany({
      where: { accountId },
    });

    const openOrders = await this.prisma.order.count({
      where: { accountId, status: { in: ['pending', 'partially_filled'] } },
    });

    const totalExposure = positions.reduce(
        (sum: Decimal, pos: any) => sum.add(pos.netQuantity.abs().mul(pos.avgPrice)),
        new Decimal(0)
    );

    const unrealizedPnl = positions.reduce(
        (sum: Decimal, pos: any) => sum.add(pos.unrealizedPnl),
        new Decimal(0)
    );

    const realizedPnl = positions.reduce(
        (sum: Decimal, pos: any) => sum.add(pos.realizedPnl),
        new Decimal(0)
    );

    const marginUtilization = account.balance.greaterThan(0)
        ? totalExposure.div(account.balance).mul(100).toNumber()
        : 0;

    return {
      accountId,
      totalExposure,
      unrealizedPnl,
      realizedPnl,
      marginUtilization,
      positionCount: positions.length,
      openOrderCount: openOrders,
    };
  }

  async getPositions(user: AuthenticatedUser, accountId?: string) {
    const targetAccount = accountId || user.accountId;

    // Check authorization
    if (targetAccount !== user.accountId &&
        !user.permissions.has('positions:read_all') &&
        !user.permissions.has('admin:full')) {
      throw new Error('Unauthorized to view positions');
    }

    return this.prisma.position.findMany({
      where: { accountId: targetAccount },
    });
  }

  async getOrders(user: AuthenticatedUser, accountId?: string, status?: string) {
    const targetAccount = accountId || user.accountId;

    // Check authorization
    if (targetAccount !== user.accountId &&
        !user.permissions.has('orders:read_all') &&
        !user.permissions.has('admin:full')) {
      throw new Error('Unauthorized to view orders');
    }

    const where: any = { accountId: targetAccount };
    if (status) {
      where.status = status;
    }

    return this.prisma.order.findMany({
      where,
      orderBy: { createdAt: 'desc' },
      take: 100,
    });
  }
}