import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('should display login form', async ({ page }) => {
    await page.goto('/');
    
    // Should see login form when not authenticated
    await expect(page.getByRole('heading', { name: /login/i })).toBeVisible();
    await expect(page.getByLabel(/username/i)).toBeVisible();
    await expect(page.getByLabel(/password/i)).toBeVisible();
  });

  test('should login successfully with valid credentials', async ({ page }) => {
    await page.goto('/');
    
    // Fill login form
    await page.getByLabel(/username/i).fill('trader1');
    await page.getByLabel(/password/i).fill('trader123');
    await page.getByRole('button', { name: /login/i }).click();
    
    // Should see dashboard after login
    await expect(page.getByText(/positions/i)).toBeVisible({ timeout: 10000 });
  });

  test('should show error for invalid credentials', async ({ page }) => {
    await page.goto('/');
    
    // Fill login form with wrong password
    await page.getByLabel(/username/i).fill('trader1');
    await page.getByLabel(/password/i).fill('wrongpassword');
    await page.getByRole('button', { name: /login/i }).click();
    
    // Should see error message
    await expect(page.getByText(/invalid|error|failed/i)).toBeVisible({ timeout: 5000 });
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.getByLabel(/username/i).fill('trader1');
    await page.getByLabel(/password/i).fill('trader123');
    await page.getByRole('button', { name: /login/i }).click();
    
    // Wait for dashboard
    await expect(page.getByText(/positions/i)).toBeVisible({ timeout: 10000 });
    
    // Logout
    await page.getByRole('button', { name: /logout/i }).click();
    
    // Should see login form again
    await expect(page.getByRole('heading', { name: /login/i })).toBeVisible();
  });
});
