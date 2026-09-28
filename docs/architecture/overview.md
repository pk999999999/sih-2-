# Platform Architecture

JOCKY has four planes:

- Language plane: compiler, IR, runtime, and CLI.
- Collection plane: cross-platform read-only agents.
- Control plane: FastAPI backend, PostgreSQL metadata, and object storage.
- Analyst plane: React dashboard for investigations, agents, jobs, evidence, and reports.

Evidence is modeled with provenance, collection timestamp, capability name, and optional object-storage location.

