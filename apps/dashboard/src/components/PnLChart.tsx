// import React from 'react';
import { Position } from '../types/trading';

interface Props {
  positions: Position[];
}

export function PnLChart({ positions }: Props) {
  const totalUnrealized = positions.reduce(
    (sum, pos) => sum + parseFloat(pos.unrealizedPnl),
    0
  );
  const totalRealized = positions.reduce(
    (sum, pos) => sum + parseFloat(pos.realizedPnl),
    0
  );
  const total = totalUnrealized + totalRealized;

  const formatCurrency = (value: number) => {
    const prefix = value >= 0 ? '+' : '';
    return `${prefix}$${value.toFixed(2)}`;
  };

  const getColor = (value: number) => {
    if (value > 0) return 'text-green-400';
    if (value < 0) return 'text-red-400';
    return 'text-gray-400';
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h2 className="text-lg font-semibold text-white mb-4">P&L Summary</h2>
      
      <div className="grid grid-cols-3 gap-4 text-center">
        <div>
          <div className="text-gray-400 text-sm">Unrealized</div>
          <div className={`text-xl font-bold ${getColor(totalUnrealized)}`}>
            {formatCurrency(totalUnrealized)}
          </div>
        </div>
        <div>
          <div className="text-gray-400 text-sm">Realized</div>
          <div className={`text-xl font-bold ${getColor(totalRealized)}`}>
            {formatCurrency(totalRealized)}
          </div>
        </div>
        <div>
          <div className="text-gray-400 text-sm">Total</div>
          <div className={`text-xl font-bold ${getColor(total)}`}>
            {formatCurrency(total)}
          </div>
        </div>
      </div>

      {/* Simple bar visualization */}
      <div className="mt-4">
        <div className="h-4 bg-gray-700 rounded overflow-hidden flex">
          {positions.map((pos, i) => {
            const pnl = parseFloat(pos.unrealizedPnl);
            const maxPnl = Math.max(...positions.map(p => Math.abs(parseFloat(p.unrealizedPnl))), 1);
            const width = Math.abs(pnl / maxPnl) * 50;
            return (
              <div
                key={i}
                className={`h-full ${pnl >= 0 ? 'bg-green-500' : 'bg-red-500'}`}
                style={{ width: `${width}%` }}
                title={`${pos.symbol}: ${formatCurrency(pnl)}`}
              />
            );
          })}
        </div>
      </div>
    </div>
  );
}
