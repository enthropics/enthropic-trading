// User & Authentication
export interface User {
  id: string;
  username: string;
  email?: string;
  role: string;
  permissions: string[];
}

export interface AuthState {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  expiresAt: Date | null;  // Changed from number to Date for consistency
  isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  refresh: () => Promise<void>;
}

// Order Types
export type OrderSide = 'buy' | 'sell';
export type OrderType = 'market' | 'limit' | 'stop' | 'stop_limit';
export type OrderStatus =
    | 'pending'
    | 'accepted'
    | 'partially_filled'
    | 'filled'
    | 'cancelled'
    | 'rejected'
    | 'expired';

export interface Order {
  id: string;
  accountId: string;
  clientOrderId: string;
  symbol: string;
  side: OrderSide;
  orderType: OrderType;
  quantity: string;
  price?: string;
  stopPrice?: string;
  filledQuantity: string;
  avgFillPrice?: string;
  status: OrderStatus;
  timeInForce?: 'GTC' | 'IOC' | 'FOK' | 'DAY';
  createdAt: string;
  updatedAt?: string;
  filledAt?: string;
}

// Position
export interface Position {
  accountId: string;
  symbol: string;
  netQuantity: string;
  avgPrice: string;
  currentPrice?: string;
  unrealizedPnl: string;
  realizedPnl: string;
  costBasis: string;
  marketValue?: string;
  updatedAt?: string;
}

// Market Data
export interface MarketTick {
  symbol: string;
  bidPrice: string;
  bidSize?: string;
  askPrice: string;
  askSize?: string;
  lastPrice: string;
  lastSize?: string;
  volume: string;
  high24h?: string;
  low24h?: string;
  change24h?: string;
  changePercent24h?: string;
  timestamp: number;
}

// Risk Management
export interface RiskSummary {
  accountId: string;
  totalExposure: string;
  unrealizedPnl: string;
  realizedPnl: string;
  totalPnl?: string;
  marginUsed: string;
  marginAvailable: string;
  marginUtilization: number; // 0-100 percentage
  buyingPower?: string;
  positionCount: number;
  openOrderCount: number;
  riskLevel?: 'low' | 'medium' | 'high' | 'critical';
  updatedAt?: string;
}

// WebSocket Message Types
export interface WSMessage {
  type: string;
  [key: string]: any;
}

export interface WSAuthMessage extends WSMessage {
  type: 'authenticate';
  token: string;
}

export interface WSSubscribeMessage extends WSMessage {
  type: 'subscribe';
  channel: string;
}

export interface WSOrderMessage extends WSMessage {
  type: 'order';
  data: Partial<Order>;
}

export interface WSMarketDataMessage extends WSMessage {
  type: 'message';
  channel: string;
  data: MarketTick;
}

// API Response Types
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
  timestamp?: number;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    pageSize: number;
    total: number;
    totalPages: number;
  };
}

// Chart Data
export interface ChartDataPoint {
  timestamp: number;
  value: number;
  label?: string;
}

export interface PnLChartData {
  realized: ChartDataPoint[];
  unrealized: ChartDataPoint[];
  total: ChartDataPoint[];
}

// Trading Types
export type TradingAction = 'buy' | 'sell' | 'hold';

export interface TradingStrategy {
  id: string;
  name: string;
  description?: string;
  status: 'active' | 'inactive' | 'paused';
  symbols: string[];
  parameters?: Record<string, any>;
}

// Account
export interface Account {
  id: string;
  username: string;
  email: string;
  role: string;
  balance: string;
  equity: string;
  marginAvailable: string;
  isActive: boolean;
  createdAt: string;
  lastLoginAt?: string;
}