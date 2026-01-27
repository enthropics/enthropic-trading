import React, { useState } from 'react';
import { Order } from '../types/trading';

interface Props {
  orders: Order[];
  onSubmitOrder: (order: any) => void;
  onCancelOrder: (orderId: string) => void;
}

export function OrderStatus({ orders, onSubmitOrder, onCancelOrder }: Props) {
  const [symbol, setSymbol] = useState('BTC-USD');
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [orderType, setOrderType] = useState<'market' | 'limit'>('limit');
  const [quantity, setQuantity] = useState('1');
  const [price, setPrice] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmitOrder({
      clientOrderId: `order-${Date.now()}`,
      symbol,
      side,
      orderType,
      quantity,
      price: orderType === 'limit' ? price : undefined,
    });
    setQuantity('1');
    setPrice('');
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'filled': return 'text-green-400';
      case 'cancelled': return 'text-gray-400';
      case 'rejected': return 'text-red-400';
      case 'pending': return 'text-yellow-400';
      default: return 'text-blue-400';
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold text-white mb-4">Orders</h2>
      
      {/* Order Form */}
      <form onSubmit={handleSubmit} className="mb-4 space-y-3">
        <div className="grid grid-cols-2 gap-2">
          <input
            type="text"
            value={symbol}
            onChange={(e) => setSymbol(e.target.value)}
            placeholder="Symbol"
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          />
          <select
            value={side}
            onChange={(e) => setSide(e.target.value as 'buy' | 'sell')}
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          >
            <option value="buy">Buy</option>
            <option value="sell">Sell</option>
          </select>
        </div>
        
        <div className="grid grid-cols-2 gap-2">
          <select
            value={orderType}
            onChange={(e) => setOrderType(e.target.value as 'market' | 'limit')}
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          >
            <option value="limit">Limit</option>
            <option value="market">Market</option>
          </select>
          <input
            type="number"
            value={quantity}
            onChange={(e) => setQuantity(e.target.value)}
            placeholder="Qty"
            className="px-2 py-1 bg-gray-700 text-white rounded text-sm"
          />
        </div>

        {orderType === 'limit' && (
          <input
            type="number"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            placeholder="Price"
            className="w-full px-2 py-1 bg-gray-700 text-white rounded text-sm"
          />
        )}

        <button
          type="submit"
          className={`w-full py-2 rounded text-sm font-medium ${
            side === 'buy' ? 'bg-green-600 hover:bg-green-700' : 'bg-red-600 hover:bg-red-700'
          } text-white`}
        >
          {side === 'buy' ? 'Buy' : 'Sell'} {symbol}
        </button>
      </form>

      {/* Orders List */}
      <div className="space-y-2 max-h-64 overflow-y-auto">
        {orders.slice(0, 20).map((order) => (
          <div key={order.id} className="bg-gray-700 p-2 rounded text-xs">
            <div className="flex justify-between items-center">
              <span className="text-white font-medium">{order.symbol}</span>
              <span className={getStatusColor(order.status)}>{order.status}</span>
            </div>
            <div className="flex justify-between text-gray-400 mt-1">
              <span className={order.side === 'buy' ? 'text-green-400' : 'text-red-400'}>
                {order.side.toUpperCase()} {order.quantity}
              </span>
              {order.price && <span>${parseFloat(order.price).toFixed(2)}</span>}
            </div>
            {(order.status === 'pending' || order.status === 'partially_filled') && (
              <button
                onClick={() => onCancelOrder(order.id)}
                className="mt-1 text-red-400 hover:text-red-300 text-xs"
              >
                Cancel
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
