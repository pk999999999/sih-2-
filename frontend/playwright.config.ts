import { defineConfig, devices } from '@playwright/test'
const apiPort = process.env.E2E_API_PORT || '8000'
const webPort = process.env.E2E_WEB_PORT || '5173'

export default defineConfig({
  timeout: 60000,
  testDir: './tests/e2e',
  testMatch: '*.pw.ts',
  retries: process.env.CI ? 1 : 0,
  use: { ...devices['Desktop Chrome'], baseURL: `http://127.0.0.1:${webPort}`, ...(process.env.PLAYWRIGHT_CHANNEL ? { channel: process.env.PLAYWRIGHT_CHANNEL } : {}) },
  webServer: [
    {
      command: `python -m uvicorn app.main:app --app-dir backend --host 127.0.0.1 --port ${apiPort}`,
      cwd: '..',
      url: `http://127.0.0.1:${apiPort}/health`,
      reuseExistingServer: !process.env.CI,
      timeout: 60000,
    },
    {
      command: `npm run dev -- --port ${webPort}`,
      env: { VITE_API_URL: `http://127.0.0.1:${apiPort}` },
      cwd: '.',
      url: `http://127.0.0.1:${webPort}`,
      reuseExistingServer: !process.env.CI,
      timeout: 60000,
    },
  ],
})
