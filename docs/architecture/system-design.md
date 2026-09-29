# System design

JOCKY is a local-development, single-organization investigation workspace.
Authenticated users share case visibility; analyst/admin roles mutate cases and
findings. Registration requires the case owner/admin, custody transfer requires
the current custodian, and the global audit endpoint requires admin. This is not
a multi-tenant isolation model.

React calls FastAPI with short-lived JWTs held in session storage. Agents use a
development shared bearer key, claim queued jobs and upload bounded JSON results.
PostgreSQL stores metadata, findings, audit and custody records. Object storage
holds JSON evidence. Collection stores a SHA-256 digest and a collection custody
event. Multi-machine case creation validates all agents and creates one system
information job per distinct selected agent in a single SQL transaction.

The registry service submits only commitments to an EVM contract. The backend is
the authorized registrar, mapping authenticated custodians to hashed identifiers.
Mock mode persists its independent comparison records in a separate SQLite file
and labels every transaction `mock:`. Neither a mock registry nor a local Ganache
chain provides an independently governed evidentiary trust anchor.

PDFs use the original report evidence inventory, current case findings, dated
verification results and current custody history. Downloads do not silently
refresh chain verification. Re-verify explicitly when fresh attestation matters.

Known distributed-systems boundary: a successful chain transaction and failed
SQL commit are not atomic. Operations fail closed with reconciliation guidance.
Production needs a durable transaction-intent/outbox worker, confirmations,
reorg handling and an operator reconciliation workflow before high availability.
