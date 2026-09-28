# End-to-End Test

`frontend/tests/e2e/investigation.pw.ts` exercises browser login and case creation against the real FastAPI service. Run `npm run test:e2e` from `frontend/` after installing dependencies and Chromium with `npx playwright install chromium`. The Playwright configuration starts the API and dashboard when they are not already running.
