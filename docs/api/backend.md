# Backend API

Interactive schema: `/docs`; machine-readable schema: `/openapi.json`.
`POST /api/auth/login` accepts email/password and returns a bearer JWT.
Shared workspace readers require authentication; writes require analyst/admin.

| Route | Request / result |
|---|---|
| POST /api/investigations | title, description, optional agent_ids; atomic case + one system.info job per selected agent |
| GET /api/investigations/{id} | case, agents, jobs, evidence, findings |
| GET/POST /api/findings | list filters investigation_id/severity/category/search; create finding |
| GET/PATCH/DELETE /api/findings/{id} | read; replace validated finding fields; delete with audit snapshot |
| POST /api/blockchain/register | evidence_id; owner/admin only; hash derived from object |
| GET /api/blockchain/records | registry rows, mode and last verification status |
| GET /api/blockchain/verify/{evidence_id} | recomputed hashes, VERIFIED/MISMATCH and mode; writes verification audit |
| GET /api/custody/history?evidence_id=... | chronological collection/registry/transfer events |
| GET /api/custody/recipients | eligible user id/email pairs for analysts |
| POST /api/custody/transfer | evidence_id, from_entity, to_entity; current custodian only |
| GET /api/audit?limit=200 | admin-only; maximum 1000 latest records |
| POST /api/editor/check, /run, /compile | source (max 16000 chars); success, output, errors, ir, mode |
| GET /api/reports/{report_id}/pdf | authenticated PDF download |

Finding body: investigation_id, category (process/network/logs/filesystem/system/
other), severity (LOW/MEDIUM/HIGH/CRITICAL), title, description, evidence_ids,
confidence 0..1. PATCH currently requires the full editable body, not JSON Merge
Patch. Evidence references must belong to the finding's investigation.

Errors: 401 invalid token, 403 insufficient permission, 404 missing resource,
409 stale custody/integrity/state conflict, 413 size exceeded, 422 validation,
429 rate/concurrency limit, 503 compiler/chain/storage unavailable.

Existing endpoints for jobs, agent registration/claim/completion, evidence and
JSON reports remain compatible. Scheduling only allows system.info,
process.list and network.connections. Findings can be authored by an analyst;
CLI analysis findings are returned in runtime output, not silently auto-imported
as backend findings.
