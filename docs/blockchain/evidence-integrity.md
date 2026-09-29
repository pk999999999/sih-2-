# Evidence integrity

The contract stores a hashed evidence identifier, SHA-256 evidence digest, hashed
custodian and block timestamps. It never receives filenames, file bytes, host
payloads, human-readable identities or report contents. Hash commitments are not
encryption and may permit guessing of low-entropy identifiers.

Registration hashes the actual stored bytes and compares the result with the
collection metadata before submitting. Duplicate successful API registrations
return the original record. The contract itself rejects duplicate identifiers.
Verification compares four values: current object bytes, contract/mock digest,
registration-row digest and collection digest. All must agree for VERIFIED.
Any difference produces MISMATCH. Missing storage/chain connectivity yields an
error, never VERIFIED or an invented mismatch. Verification records are dated.

The contract registrar is immutable. Only that address may register or transfer;
the contract checks the expected previous custodian. Backend identities, role
checks and registrar custody are part of the trust boundary. Ganache uses unlocked
development accounts; do not expose its RPC publicly or use it for real assets.

## Local EVM setup

1. `docker compose up -d ganache`
2. `npm --prefix blockchain ci`
3. `npm --prefix blockchain run deploy`
4. Set `BLOCKCHAIN_MOCK=false`, `BLOCKCHAIN_CONTRACT_ADDRESS` and
   `BLOCKCHAIN_ACCOUNT` to deployment output. Set `BLOCKCHAIN_ABI_PATH` to the
   deployment.json file for a non-Docker backend.
5. `docker compose up -d --build backend`

Use separate workspaces/databases for mock and EVM evidence. Changing modes cannot
convert an existing mock commitment into real attestation. Tests use both the
Python mock and an in-process Ganache EVM with unauthorized/replay attempts.
