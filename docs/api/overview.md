# API Overview

Primary endpoints:

- `POST /api/auth/login`
- `GET, POST /api/investigations`
- `PATCH /api/investigations/{id}`
- `GET /api/agents`
- `GET, POST /api/jobs`
- `GET /api/evidence`
- `GET /api/evidence/{id}/content`
- `GET, POST /api/reports`
- `POST /api/agent/register`
- `GET /api/agent/{agent_id}/next`
- `POST /api/agent/{agent_id}/jobs/{job_id}/complete`

Analyst endpoints use a JWT bearer token. Agent endpoints use the development agent key. Only analyst and admin roles can create investigations, jobs, and reports. Supported collection capabilities are `system.info`, `process.list`, and `network.connections`.

OpenAPI documentation is available at `/docs` when the backend is running.
