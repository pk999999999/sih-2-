# JOCKY

JOCKY is a defensive digital-forensics programming language and investigation platform. It is designed for authorized collection, triage, malware analysis, and security operations workflows.

This prototype includes:

- A Rust DSL toolchain: lexer, parser, AST, semantic analyzer, IR, runtime, CLI, and safe code-generation stubs.
- Read-only collectors for system, process, and network workflows, with standard-library directories reserved for further forensic capabilities.
- Cross-platform forensic agent crates and a polling agent executable for Windows and Linux.
- A FastAPI backend with JWT authentication, investigations, agents, jobs, evidence metadata, and report endpoints.
- A React + TypeScript dashboard.
- Docker Compose for PostgreSQL, MinIO, backend, and frontend.
- Tests, examples, and documentation.

## Safety Scope

JOCKY is for authorized defensive security and digital forensics only. The implementation intentionally excludes:

- BYOVD exploitation
- EDR/antivirus bypass or disabling
- Kernel callback disabling
- Process hollowing
- Reflective DLL injection
- API unhooking
- Direct syscall evasion
- Credential theft
- Persistence mechanisms
- Privilege escalation exploits
- Stealth mechanisms
- Domain-fronting for command-and-control
- Malware payload generation
- Destructive actions

The language and runtime expose read-only collection and analysis primitives. Collection APIs should minimize false positives and avoid interfering with security tooling, but must not bypass or disable security controls.

## Quick Start

```bash
docker compose up --build
```

Services:

- Frontend: http://localhost:5173
- Backend API: http://localhost:8000
- API docs: http://localhost:8000/docs
- MinIO console: http://localhost:9001

Default demo login (local development only):

- Email: `analyst@jocky.local`
- Password: `jocky-demo`

The Compose stack starts a Linux agent registered as `demo-agent`. Create an investigation and schedule a supported collection from the Jobs view. Evidence is stored as JSON with a SHA-256 digest. The API checks the digest when evidence is opened.

This is a prototype. The LLVM output is a code generation stub, so the executable path is the interpreted IR runtime. Agent authentication uses a shared development key; production deployments need per-agent credentials, TLS, audited enrollment, and an explicit authorization policy before connecting real endpoints.

## Language Example

```jocky
investigation "Suspicious process triage" {
  target host("workstation-17")

  collect system.info() as sys
  collect process.list() as processes
  collect network.connections() as connections

  analyze processes where name contains "powershell"
  report "triage-report" {
    include sys
    include processes
    include connections
  }
}
```

Compile to JOCKY IR:

```bash
cargo run -p jocky-cli -- compile examples/system_info.jky
```

Run semantic checks:

```bash
cargo run -p jocky-cli -- check examples/full_investigation.jky
```

## Repository Map

- `compiler/` - Rust DSL implementation.
- `stdlib/` - Safe forensic API contracts and documentation.
- `agent/` - Rust forensic agent crates.
- `backend/` - FastAPI service.
- `frontend/` - React dashboard.
- `examples/` - JOCKY scripts.
- `tests/` - Cross-component test assets.
- `docs/` - Language, architecture, API, and deployment docs.
