import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  testMatch: '*.pw.ts',
  retries: process.env.CI ? 1 : 0,
  use: { ...devices['Desktop Chrome'], baseURL: 'http://127.0.0.1:5173', ...(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {}) },
  webServer: [
    {
      command: 'python -m uvicorn app.main:app --app-dir backend --host 127.0.0.1 --port 8000',
      cwd: '..',
      url: 'http://127.0.0.1:8000/health',
      reuseExistingServer: !process.env.CI,
      timeout: 60000,
    },
    {
      command: 'npm run dev',
      cwd: '.',
      url: 'http://127.0.0.1:5173',
      reuseExistingServer: !process.env.CI,
      timeout: 60000,
    },
  ],
})
