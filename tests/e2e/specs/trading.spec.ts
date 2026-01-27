import { test, expect } from '@playwright/test';

test.describe('Trading Operations', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/');
    await page.getByLabel(/username/i).fill('trader1');
    await page.getByLabel(/password/i).fill('trader123');
    await page.getByRole('button', { name: /login/i }).click();
    await expect(page.getByText(/positions/i)).toBeVisible({ timeout: 10000 });
  });

  test('should display positions table', async ({ page }) => {
    const positionsTable = page.locator('table').filter({ hasText: /symbol/i });
    await expect(positionsTable).toBeVisible();
  });

  test('should display order form', async ({ page }) => {
    await expect(page.getByLabel(/symbol/i)).toBeVisible();
    await expect(page.getByLabel(/quantity/i)).toBeVisible();
    await expect(page.getByRole('button', { name: /submit|buy|sell/i })).toBeVisible();
  });

  test('should submit market order', async ({ page }) => {
    // Fill order form
    await page.getByLabel(/symbol/i).fill('BTC-USD');
    await page.getByLabel(/quantity/i).fill('0.1');
    
    // Select market order type if dropdown exists
    const orderTypeSelect = page.getByLabel(/type/i);
    if (await orderTypeSelect.isVisible()) {
      await orderTypeSelect.selectOption('market');
    }
    
    // Select buy side
    const sideSelect = page.getByLabel(/side/i);
    if (await sideSelect.isVisible()) {
      await sideSelect.selectOption('buy');
    }
    
    // Submit order
    await page.getByRole('button', { name: /submit|buy/i }).first().click();
    
    // Should see confirmation or order in list
    await expect(
      page.getByText(/submitted|pending|filled|order/i)
    ).toBeVisible({ timeout: 5000 });
  });

  test('should display market data', async ({ page }) => {
    // Look for market data section
    const marketSection = page.locator('[class*="market"]').or(
      page.getByText(/market data/i)
    );
    
    // Should show market data or subscription buttons
    await expect(
      page.getByText(/btc|eth|spy|subscribe/i)
    ).toBeVisible({ timeout: 5000 });
  });
});
