// import React from 'react';
import { Position } from '../types/trading';

interface Props {
  positions: Position[];
}

export function PositionsTable({ positions }: Props) {
  const formatNumber = (value: string, decimals = 2) => {
    return parseFloat(value).toFixed(decimals);
  };

  const getPnlColor = (pnl: string) => {
    const value = parseFloat(pnl);
    if (value > 0) return 'text-green-400';
    if (value < 0) return 'text-red-400';
    return 'text-gray-400';
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold text-white mb-4">Positions</h2>
      
      {positions.length === 0 ? (
        <p className="text-gray-400">No open positions</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-gray-400 border-b border-gray-700">
                <th className="text-left py-2">Symbol</th>
                <th className="text-right py-2">Quantity</th>
                <th className="text-right py-2">Avg Price</th>
                <th className="text-right py-2">Unrealized P&L</th>
                <th className="text-right py-2">Realized P&L</th>
              </tr>
            </thead>
            <tbody>
              {positions.map((pos) => (
                <tr key={`${pos.accountId}-${pos.symbol}`} className="text-white border-b border-gray-700">
                  <td className="py-2 font-medium">{pos.symbol}</td>
                  <td className={`text-right ${parseFloat(pos.netQuantity) >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {formatNumber(pos.netQuantity, 4)}
                  </td>
                  <td className="text-right">${formatNumber(pos.avgPrice)}</td>
                  <td className={`text-right ${getPnlColor(pos.unrealizedPnl)}`}>
                    ${formatNumber(pos.unrealizedPnl)}
                  </td>
                  <td className={`text-right ${getPnlColor(pos.realizedPnl)}`}>
                    ${formatNumber(pos.realizedPnl)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
