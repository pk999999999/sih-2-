# Chain of custody

Collection records agent -> case owner off-chain. Registry registration adds a
blue blockchain event with a transaction reference. Transfers add yellow events
and update the current custodian after successful contract execution. Ordinary
collection events are green in the dashboard. Events are returned chronologically.

Transfers require an authenticated analyst/admin who is the current custodian,
the expected `from_entity`, and a different existing analyst/admin recipient.
Admin status does not override custody ownership. Stale-owner requests return
409; unauthorized requests return 403. Evidence must be registered first.

SQL row locking serializes operations on PostgreSQL. A process lock also covers
single-worker SQLite development. Multi-worker SQLite is not supported for
custody mutation. A database and chain cannot share an ACID transaction; after
an ambiguous failure stop transfers and reconcile chain history, receipt and SQL
audit records. Never silently overwrite an existing registration.

Custody history is append-only through the public API. Database administrators
can still modify SQL and mock state. Production deployments need independent
audit retention and signed exports; this prototype does not claim legal
admissibility or tamper-proof administrative storage.
