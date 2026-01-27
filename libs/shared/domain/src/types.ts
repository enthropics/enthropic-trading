export interface Order {
  id: string;
  accountId: string;
  clientOrderId: string;
  symbol: string;
  side: 'buy' | 'sell';
  orderType: 'market' | 'limit' | 'stop' | 'stop_limit';
  timeInForce: 'gtc' | 'ioc' | 'fok' | 'day';
  quantity: string;
  price?: string;
  stopPrice?: string;
  filledQuantity: string;
  avgFillPrice?: string;
  status: 'pending' | 'partially_filled' | 'filled' | 'cancelled' | 'rejected' | 'expired';
  rejectReason?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Position {
  accountId: string;
  symbol: string;
  netQuantity: string;
  avgPrice: string;
  unrealizedPnl: string;
  realizedPnl: string;
  costBasis: string;
  updatedAt: string;
}

export interface Trade {
  id: string;
  buyOrderId: string;
  sellOrderId: string;
  buyerAccountId: string;
  sellerAccountId: string;
  symbol: string;
  quantity: string;
  price: string;
  executedAt: string;
}

export interface Account {
  id: string;
  username: string;
  email: string;
  role: string;
  balance: string;
  availableBalance: string;
  marginUsed: string;
  maxPositionSize: string;
  maxOrderSize: string;
  maxDailyLoss: string;
  isActive: boolean;
  createdAt: string;
}

export interface MarketTick {
  symbol: string;
  bidPrice: string;
  askPrice: string;
  bidSize: string;
  askSize: string;
  lastPrice: string;
  lastSize: string;
  volume: string;
  timestamp: number;
}

export interface RiskLimits {
  maxPositionSize: string;
  maxOrderSize: string;
  maxDailyLoss: string;
}

export interface RiskSummary {
  accountId: string;
  totalExposure: string;
  unrealizedPnl: string;
  realizedPnl: string;
  marginUtilization: number;
  positionCount: number;
  openOrderCount: number;
}

export const Permissions = {
  ORDERS_CREATE: 'orders:create',
  ORDERS_READ: 'orders:read',
  ORDERS_CANCEL: 'orders:cancel',
  ORDERS_READ_ALL: 'orders:read_all',
  POSITIONS_READ: 'positions:read',
  POSITIONS_READ_ALL: 'positions:read_all',
  MARKET_READ: 'market:read',
  MARKET_SUBSCRIBE: 'market:subscribe',
  ACCOUNTS_READ: 'accounts:read',
  ACCOUNTS_READ_ALL: 'accounts:read_all',
  RISK_READ: 'risk:read',
  RISK_MANAGE: 'risk:manage',
  STRATEGIES_READ: 'strategies:read',
  STRATEGIES_CREATE: 'strategies:create',
  STRATEGIES_EXECUTE: 'strategies:execute',
  ADMIN_FULL: 'admin:full',
} as const;
