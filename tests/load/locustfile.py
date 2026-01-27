"""
Load Tests for Enthropic Trading Platform
Phase 4: Performance testing for 10k orders/sec target

Run with: locust -f locustfile.py --headless -u 100 -r 10 --run-time 2m
"""

import json
import random
import string
from locust import HttpUser, task, between, events
from locust.runners import MasterRunner, WorkerRunner


class TradingUser(HttpUser):
    """Simulates a trader using the platform."""
    
    wait_time = between(0.1, 0.5)  # Fast trading simulation
    
    def on_start(self):
        """Login and get authentication token."""
        # Login
        response = self.client.post("/api/auth/login", json={
            "username": f"loadtest_{random.randint(1, 100)}",
            "password": "loadtest123"
        })
        
        if response.status_code == 200:
            data = response.json()
            self.token = data.get("accessToken")
            self.account_id = data.get("user", {}).get("id")
        else:
            # Use demo token for testing
            self.token = "demo-token"
            self.account_id = "demo-account"
        
        self.headers = {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }
        
        self.symbols = ["BTC-USD", "ETH-USD", "SPY", "AAPL", "GOOGL", "MSFT", "AMZN"]
    
    @task(10)
    def submit_market_order(self):
        """Submit a market order - most common operation."""
        order = {
            "symbol": random.choice(self.symbols),
            "side": random.choice(["buy", "sell"]),
            "type": "market",
            "quantity": round(random.uniform(0.01, 10), 4),
            "client_order_id": f"load_{self._generate_id()}"
        }
        
        with self.client.post(
            "/api/orders",
            json=order,
            headers=self.headers,
            catch_response=True,
            name="POST /api/orders (market)"
        ) as response:
            if response.status_code in [200, 201]:
                response.success()
            elif response.status_code == 429:
                response.failure("Rate limited")
            else:
                response.failure(f"Failed: {response.status_code}")
    
    @task(5)
    def submit_limit_order(self):
        """Submit a limit order."""
        symbol = random.choice(self.symbols)
        base_price = self._get_base_price(symbol)
        
        order = {
            "symbol": symbol,
            "side": random.choice(["buy", "sell"]),
            "type": "limit",
            "quantity": round(random.uniform(0.01, 10), 4),
            "price": round(base_price * random.uniform(0.95, 1.05), 2),
            "client_order_id": f"load_{self._generate_id()}"
        }
        
        with self.client.post(
            "/api/orders",
            json=order,
            headers=self.headers,
            catch_response=True,
            name="POST /api/orders (limit)"
        ) as response:
            if response.status_code in [200, 201]:
                response.success()
            elif response.status_code == 429:
                response.failure("Rate limited")
            else:
                response.failure(f"Failed: {response.status_code}")
    
    @task(8)
    def get_positions(self):
        """Check positions - frequent read operation."""
        with self.client.get(
            "/api/risk/positions",
            headers=self.headers,
            catch_response=True,
            name="GET /api/risk/positions"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Failed: {response.status_code}")
    
    @task(5)
    def get_orders(self):
        """Get open orders."""
        with self.client.get(
            "/api/risk/orders?status=open",
            headers=self.headers,
            catch_response=True,
            name="GET /api/risk/orders"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Failed: {response.status_code}")
    
    @task(3)
    def get_risk_summary(self):
        """Get account risk summary."""
        with self.client.get(
            "/api/risk/summary",
            headers=self.headers,
            catch_response=True,
            name="GET /api/risk/summary"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Failed: {response.status_code}")
    
    @task(2)
    def cancel_order(self):
        """Cancel a random order."""
        # First get orders
        response = self.client.get(
            "/api/risk/orders?status=open",
            headers=self.headers
        )
        
        if response.status_code == 200:
            orders = response.json().get("orders", [])
            if orders:
                order_id = random.choice(orders).get("id")
                with self.client.delete(
                    f"/api/orders/{order_id}",
                    headers=self.headers,
                    catch_response=True,
                    name="DELETE /api/orders/{id}"
                ) as del_response:
                    if del_response.status_code in [200, 204]:
                        del_response.success()
                    else:
                        del_response.failure(f"Failed: {del_response.status_code}")
    
    @task(1)
    def health_check(self):
        """Health check endpoint."""
        with self.client.get(
            "/api/health",
            catch_response=True,
            name="GET /api/health"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Failed: {response.status_code}")
    
    def _generate_id(self):
        return ''.join(random.choices(string.ascii_lowercase + string.digits, k=12))
    
    def _get_base_price(self, symbol):
        prices = {
            "BTC-USD": 50000,
            "ETH-USD": 2500,
            "SPY": 500,
            "AAPL": 180,
            "GOOGL": 140,
            "MSFT": 400,
            "AMZN": 180,
        }
        return prices.get(symbol, 100)


class WebSocketUser(HttpUser):
    """Simulates WebSocket connections for real-time data."""
    
    wait_time = between(1, 3)
    
    @task
    def websocket_ping(self):
        """Simulate WebSocket connection check via HTTP health."""
        with self.client.get(
            "/health",
            catch_response=True,
            name="GET /health (gateway)"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Gateway unhealthy: {response.status_code}")


# Custom metrics reporting
@events.request.add_listener
def on_request(request_type, name, response_time, response_length, response, **kwargs):
    """Log detailed metrics for analysis."""
    pass


@events.test_start.add_listener
def on_test_start(environment, **kwargs):
    """Setup before test run."""
    print("="*60)
    print("Enthropic Trading Platform Load Test")
    print("Target: 10,000 orders/second")
    print("="*60)


@events.test_stop.add_listener
def on_test_stop(environment, **kwargs):
    """Cleanup after test run."""
    print("="*60)
    print("Load Test Complete")
    print("="*60)
