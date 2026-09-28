# End-to-End Test

`frontend/tests/e2e/investigation.pw.ts` exercises browser login and case creation against the real FastAPI service. Run `npm run test:e2e` from `frontend/` after installing dependencies and Chromium with `npx playwright install chromium`. The Playwright configuration starts the API and dashboard when they are not already running.

`compose_smoke.py` verifies the Docker Compose stack, including a real Linux agent collection and MinIO evidence retrieval. Run `docker compose up --build -d`, then `python tests/e2e/compose_smoke.py` from the repository root.
