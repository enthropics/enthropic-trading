import { Decimal } from '@prisma/client/runtime/library';

export interface RiskLimits {
  maxPositionSize: Decimal;
  maxOrderSize: Decimal;
  maxDailyLoss: Decimal;
}

export interface RiskCheckResult {
  allowed: boolean;
  reason?: string;
  code?: string;
}

export interface AccountRiskSummary {
  accountId: string;
  totalExposure: Decimal;
  unrealizedPnl: Decimal;
  realizedPnl: Decimal;
  marginUtilization: number;
  positionCount: number;
  openOrderCount: number;
}

export interface OrderRiskCheck {
  accountId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: Decimal;
  price?: Decimal;
}
