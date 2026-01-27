import { WebSocketServer, WebSocket } from 'ws';
import { Pool } from 'pg';
import Redis from 'ioredis';
import { connect, NatsConnection, Subscription } from 'nats';
import { AuthService, AuthContext, Permissions } from './auth';
import { Config } from './config';

interface AuthenticatedClient {
  ws: WebSocket;
  auth: AuthContext | null;
  subscriptions: Map<string, Subscription>;
  connectedAt: Date;
  ipAddress: string;
}

interface WebSocketMessage {
  type: string;
  [key: string]: any;
}

export class WebSocketHandler {
  private wss: WebSocketServer | null = null;
  private clients: Map<WebSocket, AuthenticatedClient> = new Map();
  private nats: NatsConnection | null = null;
  private authService: AuthService;
  private pool: Pool;
  private redis: Redis;
  private config: Config;

  constructor(pool: Pool, redis: Redis, config: Config) {
    this.pool = pool;
    this.redis = redis;
    this.config = config;
    this.authService = new AuthService(pool, redis, config);
  }

  async start(): Promise<void> {
    // Connect to NATS
    this.nats = await connect({ servers: this.config.natsUrl });
    console.log('Connected to NATS');

    // Start WebSocket server
    this.wss = new WebSocketServer({ port: this.config.port });

    this.wss.on('connection', (ws, req) => {
      const ipAddress = req.socket.remoteAddress || 'unknown';
      this.handleConnection(ws, ipAddress);
    });

    // Heartbeat
    setInterval(() => this.heartbeat(), 30000);
  }

  stop(): void {
    this.wss?.close();
    this.nats?.close();
    this.clients.clear();
  }

  private handleConnection(ws: WebSocket, ipAddress: string): void {
    const client: AuthenticatedClient = {
      ws,
      auth: null,
      subscriptions: new Map(),
      connectedAt: new Date(),
      ipAddress,
    };

    this.clients.set(ws, client);
    console.log(`Client connected from ${ipAddress}`);

    ws.on('message', async (data) => {
      try {
        const message: WebSocketMessage = JSON.parse(data.toString());
        await this.handleMessage(client, message);
      } catch (error) {
        this.sendError(ws, 'INVALID_MESSAGE', 'Failed to parse message');
      }
    });

    ws.on('close', () => {
      this.handleDisconnect(client);
    });

    ws.on('error', (error) => {
      console.error(`WebSocket error for ${ipAddress}:`, error);
    });
  }

  private async handleMessage(client: AuthenticatedClient, message: WebSocketMessage): Promise<void> {
    switch (message.type) {
      case 'login':
        await this.handleLogin(client, message.username, message.password);
        break;

      case 'authenticate':
        await this.handleAuthenticate(client, message.token);
        break;

      case 'refresh':
        await this.handleRefresh(client, message.refreshToken);
        break;

      case 'logout':
        await this.handleLogout(client);
        break;

      case 'subscribe':
        await this.handleSubscribe(client, message.channel);
        break;

      case 'unsubscribe':
        await this.handleUnsubscribe(client, message.channel);
        break;

      case 'order':
        await this.handleOrder(client, message.data);
        break;

      case 'cancel':
        await this.handleCancel(client, message.orderId);
        break;

      default:
        this.sendError(client.ws, 'UNKNOWN_TYPE', `Unknown message type: ${message.type}`);
    }
  }

  private async handleLogin(client: AuthenticatedClient, username: string, password: string): Promise<void> {
    try {
      const result = await this.authService.login(username, password, client.ipAddress);
      client.auth = result.authContext;
      
      this.send(client.ws, {
        type: 'login_success',
        accessToken: result.accessToken,
        refreshToken: result.refreshToken,
        expiresAt: result.expiresAt,
        user: {
          id: result.authContext.accountId,
          username: result.authContext.username,
          role: result.authContext.role,
          permissions: Array.from(result.authContext.permissions),
        },
      });
    } catch (error: any) {
      this.sendError(client.ws, 'LOGIN_FAILED', error.message);
    }
  }

  private async handleAuthenticate(client: AuthenticatedClient, token: string): Promise<void> {
    try {
      const authContext = await this.authService.validateToken(token);
      client.auth = authContext;
      this.send(client.ws, { type: 'authenticated', success: true });
    } catch (error: any) {
      this.sendError(client.ws, 'AUTH_FAILED', error.message);
    }
  }

  private async handleRefresh(client: AuthenticatedClient, refreshToken: string): Promise<void> {
    try {
      const result = await this.authService.refreshAccessToken(refreshToken);
      this.send(client.ws, {
        type: 'token_refreshed',
        accessToken: result.accessToken,
        expiresAt: result.expiresAt,
      });
    } catch (error: any) {
      this.sendError(client.ws, 'REFRESH_FAILED', error.message);
    }
  }

  private async handleLogout(client: AuthenticatedClient): Promise<void> {
    if (client.auth) {
      await this.authService.revokeToken(client.auth.tokenJti, client.auth.accountId, 'logout');
      client.auth = null;
    }
    
    // Unsubscribe from all
    for (const [channel, sub] of client.subscriptions) {
      sub.unsubscribe();
    }
    client.subscriptions.clear();

    this.send(client.ws, { type: 'logged_out', success: true });
  }

  private async handleSubscribe(client: AuthenticatedClient, channel: string): Promise<void> {
    if (!client.auth) {
      this.sendError(client.ws, 'UNAUTHORIZED', 'Must authenticate first');
      return;
    }

    // Check permissions based on channel
    const hasPermission = this.checkChannelPermission(client.auth, channel);
    if (!hasPermission) {
      this.sendError(client.ws, 'FORBIDDEN', `No permission for channel: ${channel}`);
      return;
    }

    // Check rate limit
    const allowed = await this.authService.checkRateLimit(client.auth.accountId, 'subscribe');
    if (!allowed) {
      this.sendError(client.ws, 'RATE_LIMITED', 'Too many requests');
      return;
    }

    if (!this.nats || client.subscriptions.has(channel)) {
      return;
    }

    const sub = this.nats.subscribe(channel, {
      callback: (err, msg) => {
        if (err) {
          console.error(`Subscription error for ${channel}:`, err);
          return;
        }
        this.send(client.ws, {
          type: 'message',
          channel,
          data: JSON.parse(msg.data.toString()),
        });
      },
    });

    client.subscriptions.set(channel, sub);
    this.send(client.ws, { type: 'subscribed', channel });
  }

  private async handleUnsubscribe(client: AuthenticatedClient, channel: string): Promise<void> {
    const sub = client.subscriptions.get(channel);
    if (sub) {
      sub.unsubscribe();
      client.subscriptions.delete(channel);
    }
    this.send(client.ws, { type: 'unsubscribed', channel });
  }

  private async handleOrder(client: AuthenticatedClient, orderData: any): Promise<void> {
    if (!client.auth) {
      this.sendError(client.ws, 'UNAUTHORIZED', 'Must authenticate first');
      return;
    }

    if (!this.authService.hasPermission(client.auth, Permissions.ORDERS_CREATE)) {
      this.sendError(client.ws, 'FORBIDDEN', 'orders:create permission required');
      return;
    }

    // Rate limit check
    const allowed = await this.authService.checkRateLimit(client.auth.accountId, 'order');
    if (!allowed) {
      this.sendError(client.ws, 'RATE_LIMITED', 'Too many orders');
      return;
    }

    // Publish to NATS with auth context
    if (this.nats) {
      const message = {
        auth: {
          account_id: client.auth.accountId,
          username: client.auth.username,
          role: client.auth.role,
          permissions: Array.from(client.auth.permissions),
        },
        ...orderData,
      };

      await this.nats.publish('orders.submit', JSON.stringify(message));
      this.send(client.ws, { type: 'order_submitted', data: orderData });
    }
  }

  private async handleCancel(client: AuthenticatedClient, orderId: string): Promise<void> {
    if (!client.auth) {
      this.sendError(client.ws, 'UNAUTHORIZED', 'Must authenticate first');
      return;
    }

    if (!this.authService.hasPermission(client.auth, Permissions.ORDERS_CANCEL)) {
      this.sendError(client.ws, 'FORBIDDEN', 'orders:cancel permission required');
      return;
    }

    if (this.nats) {
      const message = {
        auth: {
          account_id: client.auth.accountId,
          username: client.auth.username,
          role: client.auth.role,
          permissions: Array.from(client.auth.permissions),
        },
        order_id: orderId,
      };

      await this.nats.publish('orders.cancel', JSON.stringify(message));
      this.send(client.ws, { type: 'cancel_submitted', orderId });
    }
  }

  private checkChannelPermission(auth: AuthContext, channel: string): boolean {
    if (auth.permissions.has(Permissions.ADMIN_FULL)) {
      return true;
    }

    if (channel.startsWith('market.')) {
      return auth.permissions.has(Permissions.MARKET_SUBSCRIBE);
    }

    if (channel.startsWith('orders.')) {
      return auth.permissions.has(Permissions.ORDERS_READ);
    }

    if (channel.startsWith('positions.')) {
      return auth.permissions.has(Permissions.POSITIONS_READ);
    }

    return false;
  }

  private handleDisconnect(client: AuthenticatedClient): void {
    for (const [, sub] of client.subscriptions) {
      sub.unsubscribe();
    }
    this.clients.delete(client.ws);
    console.log(`Client disconnected from ${client.ipAddress}`);
  }

  private heartbeat(): void {
    for (const [ws, client] of this.clients) {
      if (ws.readyState === WebSocket.OPEN) {
        ws.ping();
      }
    }
  }

  private send(ws: WebSocket, data: any): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(data));
    }
  }

  private sendError(ws: WebSocket, code: string, message: string): void {
    this.send(ws, { type: 'error', code, error: message });
  }
}
