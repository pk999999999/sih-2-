# Integrated local deployment

Requirements: Docker Compose, or Python 3.11+, Node 22+, Rust stable; clang for
native executable generation. Use the repository root as working directory.

```sh
docker compose up --build -d
```

Open http://localhost:5173 and log in with analyst@jocky.local / jocky-demo.
API: http://localhost:8000/docs. The backend image builds and installs the Rust
CLI, applies Alembic migrations and supports the web editor. Default registry
mode is mock. PostgreSQL, S3-compatible object storage and mock-chain state use
named volumes. Ganache RPC binds localhost:8545. Do not delete volumes containing
real evidence. Compose credentials are development-only.

## Without Docker

```sh
python -m pip install -r backend/requirements.txt
npm --prefix frontend ci
npm --prefix blockchain ci
cargo build -p jocky-cli
```

Set `JOCKY_CLI` to the absolute path of target/debug/jocky-cli (Windows: .exe).
For the backend set DATABASE_URL=sqlite:///./jocky.db, BLOCKCHAIN_MOCK=true and
STORAGE_BACKEND=local. Run `python -m uvicorn app.main:app --app-dir backend`
and `npm --prefix frontend run dev`. For existing deployments run
`cd backend` then `python -m alembic upgrade head` before startup. A compatibility
check also adds the nullable registry reference to old local SQLite schemas.

## Demo and tests

`python scripts/demo_e2e.py` creates an isolated temporary workspace, nine
synthetic evidence records across three machines, two findings, registry
commitments and a deliberately modified synthetic object. It asserts PC-002
VERIFIED and PC-001 MISMATCH and writes output/pdf/demo-report.pdf. It does not
seed or alter your running server. Browser E2E tests seed their own mock records.

`make setup/test/lint/build/docker/demo` provides shortcuts (GNU Make required).
Without Make use the corresponding commands in the root Makefile. CI runs Rust
on Linux/Windows, native LLVM parity, Python tests/demo, frontend build/unit/E2E,
Solidity contract tests, and a Docker Compose smoke test.
