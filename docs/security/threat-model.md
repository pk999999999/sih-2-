# Threat model and integration review

Assets: evidence bytes, collection hashes, custody ownership, case metadata,
credentials, registrar signing authority and audit history. Threats include an
untrusted endpoint, malicious authenticated analyst, stolen bearer token, object
tampering, forged custody claims, registry outages and resource exhaustion.

Implemented controls:

- Collection allowlist; no injection, exploits, evasion or destructive APIs.
- Path-root restrictions and bounded collectors; evidence hashes checked on read.
- JWT validation and analyst/admin roles. Custodian checks on transfers.
- On-chain registrar restriction, duplicate prevention and expected-owner checks.
- Cross-case evidence references rejected for findings. Confidence bounded 0..1.
- Editor mock-only execution, no shell, sanitized environment, source/output
  limits, timeout, fixed compiler binary and concurrency ceiling.
- Per-IP bounded in-memory rate limits: login 20/min, editor 30/min, other 600/min.
  Forwarded headers are not trusted. Production requires a shared edge limiter.
- PDF content escaped before ReportLab markup. Hash-mismatched system content
  is withheld. Tokens and raw evidence are not included in audit details.
- Demo tampering occurs only inside a temporary synthetic workspace.

Residual risks and release gates:

- Shared-team visibility is intentional; no tenant isolation or case membership.
- Agent shared development key must be replaced with per-agent mTLS and enrollment.
- Default local credentials, HTTP and unlocked Ganache accounts are development
  only. Deploy behind TLS; rotate secrets; disable demo seeding in production.
- Chain/SQL dual writes need durable intents and reconciliation; handle reorgs.
- Mock SQLite and Ganache volumes can be changed by administrators. Hash agreement
  does not prove origin, truthful acquisition, completeness or admissibility.
- Source checking is bounded, not an OS sandbox. Internet-exposed editor workers
  require resource-isolated containers and independent security review.
- The bundled Ganache development toolchain is legacy; dependency audit output
  must be reviewed before exposing it beyond localhost.

No production security certification is claimed. Tests exercise tampering,
unauthorized roles, stale custody, cross-case references and contract access.
