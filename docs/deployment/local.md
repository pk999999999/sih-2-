# Local Deployment

Use Docker Compose:

```bash
docker compose up --build
```

For backend-only development:

```bash
cd backend
python -m venv .venv
.venv/Scripts/pip install -r requirements.txt
.venv/Scripts/uvicorn app.main:app --reload
```

For frontend-only development:

```bash
cd frontend
npm install
npm run dev
```

The Docker Compose stack runs a Linux agent that registers as `demo-agent` and polls for queued jobs. Use the login shown in the root README, create an investigation, then schedule `system.info`, `process.list`, or `network.connections` from the Jobs view. The agent submits JSON evidence to the API; the API hashes it and stores it through the S3 API in SeaweedFS. Backend-only development uses local evidence files by default.

To run the agent outside Docker, set `JOCKY_AGENT_KEY`, `JOCKY_API_URL`, and `JOCKY_AGENT_ID`, then run `cargo run -p jocky-agent`. The agent is read-only and polls every five seconds. Agent credentials and demo login values in this repository are for local development only.

Apply migrations with `cd backend && alembic upgrade head`. The API also creates missing tables on startup for an easy prototype launch. For tests, run `cd backend && pytest -q`, `cargo test --workspace`, and `cd frontend && npm test && npm run build`.
