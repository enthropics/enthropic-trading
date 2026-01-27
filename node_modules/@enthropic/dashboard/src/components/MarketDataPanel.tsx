// import React from 'react';
import { MarketTick } from '../types/trading';

interface Props {
  ticks: Map<string, MarketTick>;
  onSubscribe: (symbol: string) => void;
}

export function MarketDataPanel({ ticks, onSubscribe }: Props) {
  const symbols = ['BTC-USD', 'ETH-USD', 'SPY', 'AAPL', 'GOOGL'];

  const formatPrice = (price: string) => {
    return parseFloat(price).toFixed(2);
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold text-white mb-4">Market Data</h2>
      
      <div className="space-y-2">
        {symbols.map((symbol) => {
          const tick = ticks.get(symbol);
          return (
            <div key={symbol} className="bg-gray-700 p-3 rounded flex justify-between items-center">
              <div>
                <span className="text-white font-medium">{symbol}</span>
                {!tick && (
                  <button
                    onClick={() => onSubscribe(`market.${symbol}`)}
                    className="ml-2 text-xs text-blue-400 hover:text-blue-300"
                  >
                    Subscribe
                  </button>
                )}
              </div>
              {tick ? (
                <div className="text-right">
                  <div className="text-white font-mono">${formatPrice(tick.lastPrice)}</div>
                  <div className="text-xs text-gray-400">
                    Bid: ${formatPrice(tick.bidPrice)} / Ask: ${formatPrice(tick.askPrice)}
                  </div>
                </div>
              ) : (
                <span className="text-gray-500">--</span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
