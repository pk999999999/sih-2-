# Security Policy

JOCKY is defensive software. Reports about vulnerabilities, unsafe forensic behavior, or risky APIs are welcome.

## Out of Scope Features

The project must not implement EDR bypass, privilege escalation, stealth, persistence, credential theft, malware deployment, destructive actions, or unauthorized collection.

## Reporting

Open a private security advisory or contact the maintainers with:

- A concise description
- Reproduction steps
- Impact assessment
- Suggested mitigation, if known

## Design Principles

- Prefer explicit authorization and auditable actions.
- Prefer read-only collection APIs.
- Record evidence provenance and hashes.
- Avoid disrupting endpoint security products.
- Fail closed when permissions or target identity are unclear.

