import { useEffect, useRef, useCallback, useState } from 'react';
import { useAuth } from './useAuth';
import { Order, Position, MarketTick } from '../types/trading';

interface WebSocketMessage {
  type: string;
  data?: any;
  channel?: string;
  error?: string;
  code?: string;
}

interface UseNatsWebSocketReturn {
  connected: boolean;
  authenticated: boolean;
  positions: Position[];
  orders: Order[];
  marketTicks: Map<string, MarketTick>;
  subscribe: (channel: string) => void;
  unsubscribe: (channel: string) => void;
  submitOrder: (order: Omit<Order, 'id' | 'accountId' | 'createdAt' | 'status' | 'filledQuantity'>) => void;
  cancelOrder: (orderId: string) => void;
}

export function useNatsWebSocket(): UseNatsWebSocketReturn {
  const { accessToken, isAuthenticated } = useAuth();
  const wsRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [authenticated, setAuthenticated] = useState(false);
  const [positions, setPositions] = useState<Position[]>([]);
  const [orders, setOrders] = useState<Order[]>([]);
  const [marketTicks, setMarketTicks] = useState<Map<string, MarketTick>>(new Map());

  const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:3002';

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(WS_URL);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('WebSocket connected');
      setConnected(true);

      // Authenticate if we have a token
      if (accessToken) {
        ws.send(JSON.stringify({ type: 'authenticate', token: accessToken }));
      }
    };

    ws.onmessage = (event) => {
      const msg: WebSocketMessage = JSON.parse(event.data);
      handleMessage(msg);
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setConnected(false);
      setAuthenticated(false);

      // Reconnect after delay
      setTimeout(connect, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }, [accessToken, WS_URL]);

  const handleMessage = useCallback((msg: WebSocketMessage) => {
    switch (msg.type) {
      case 'authenticated':
        setAuthenticated(true);
        console.log('WebSocket authenticated');
        break;

      case 'message':
        if (msg.channel?.startsWith('positions.')) {
          setPositions(msg.data?.positions || []);
        } else if (msg.channel?.startsWith('orders.')) {
          setOrders((prev) => {
            const updated = [...prev];
            const index = updated.findIndex((o) => o.id === msg.data?.id);
            if (index >= 0) {
              updated[index] = msg.data;
            } else {
              updated.unshift(msg.data);
            }
            return updated.slice(0, 100);
          });
        } else if (msg.channel?.startsWith('market.')) {
          setMarketTicks((prev) => {
            const updated = new Map(prev);
            updated.set(msg.data?.symbol, msg.data);
            return updated;
          });
        }
        break;

      case 'order_submitted':
        console.log('Order submitted:', msg);
        break;

      case 'error':
        console.error('WebSocket error:', msg.code, msg.error);
        break;
    }
  }, []);

  const subscribe = useCallback((channel: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN && authenticated) {
      wsRef.current.send(JSON.stringify({ type: 'subscribe', channel }));
    }
  }, [authenticated]);

  const unsubscribe = useCallback((channel: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'unsubscribe', channel }));
    }
  }, []);

  const submitOrder = useCallback((order: Omit<Order, 'id' | 'accountId' | 'createdAt' | 'status' | 'filledQuantity'>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN && authenticated) {
      wsRef.current.send(JSON.stringify({ type: 'order', data: order }));
    }
  }, [authenticated]);

  const cancelOrder = useCallback((orderId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN && authenticated) {
      wsRef.current.send(JSON.stringify({ type: 'cancel', orderId }));
    }
  }, [authenticated]);

  useEffect(() => {
    if (isAuthenticated) {
      connect();
    }

    return () => {
      wsRef.current?.close();
    };
  }, [isAuthenticated, connect]);

  // Re-authenticate when token changes
  useEffect(() => {
    if (connected && accessToken && wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: 'authenticate', token: accessToken }));
    }
  }, [accessToken, connected]);

  return {
    connected,
    authenticated,
    positions,
    orders,
    marketTicks,
    subscribe,
    unsubscribe,
    submitOrder,
    cancelOrder,
  };
}
