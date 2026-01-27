import { useEffect, useState } from 'react';
import { useAuth } from './hooks/useAuth';
import { useNatsWebSocket } from './hooks/useNatsWebSocket';
import { Login } from './components/Login';
import { PositionsTable } from './components/PositionsTable';
import { OrderStatus } from './components/OrderStatus';
import { MarketDataPanel } from './components/MarketDataPanel';
import { PnLChart } from './components/PnLChart';

function Dashboard() {
  const { user, logout } = useAuth();
  const {
    connected,
    authenticated,
    positions,
    orders,
    marketTicks,
    subscribe,
    submitOrder,
    cancelOrder,
  } = useNatsWebSocket();

  useEffect(() => {
    if (authenticated) {
      subscribe('positions.*');
      subscribe('orders.*');
    }
  }, [authenticated, subscribe]);

  return (
    <div className="min-h-screen bg-gray-900">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3">
        <div className="flex justify-between items-center">
          <div className="flex items-center space-x-4">
            <h1 className="text-xl font-bold text-white">Enthropic Trading</h1>
            <div className="flex items-center space-x-2">
              <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-400' : 'bg-red-400'}`} />
              <span className="text-sm text-gray-400">
                {connected ? (authenticated ? 'Connected' : 'Connecting...') : 'Disconnected'}
              </span>
            </div>
          </div>
          <div className="flex items-center space-x-4">
            <span className="text-gray-400">
              {user?.username} ({user?.role})
            </span>
            <button
              onClick={logout}
              className="px-3 py-1 bg-gray-700 text-white rounded hover:bg-gray-600 text-sm"
            >
              Logout
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="p-4">
        <div className="grid grid-cols-12 gap-4">
          {/* Left Column */}
          <div className="col-span-3 space-y-4">
            <MarketDataPanel ticks={marketTicks} onSubscribe={subscribe} />
            <OrderStatus
              orders={orders}
              onSubmitOrder={submitOrder}
              onCancelOrder={cancelOrder}
            />
          </div>

          {/* Center Column */}
          <div className="col-span-6 space-y-4">
            <PnLChart positions={positions} />
            <PositionsTable positions={positions} />
          </div>

          {/* Right Column */}
          <div className="col-span-3">
            <div className="bg-gray-800 rounded-lg p-4">
              <h2 className="text-lg font-semibold text-white mb-4">Account Info</h2>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">Username:</span>
                  <span className="text-white">{user?.username}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Role:</span>
                  <span className="text-white">{user?.role}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Positions:</span>
                  <span className="text-white">{positions.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Open Orders:</span>
                  <span className="text-white">
                    {orders.filter(o => o.status === 'pending' || o.status === 'partially_filled').length}
                  </span>
                </div>
              </div>

              <div className="mt-4 pt-4 border-t border-gray-700">
                <h3 className="text-sm font-medium text-white mb-2">Permissions</h3>
                <div className="flex flex-wrap gap-1">
                  {user?.permissions.map((perm) => (
                    <span
                      key={perm}
                      className="px-2 py-0.5 bg-gray-700 text-gray-300 rounded text-xs"
                    >
                      {perm}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

function App() {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Login />;
  }

  return <Dashboard />;
}

export default App;
