# Contributing

JOCKY accepts changes that support authorized defensive forensics, incident response, malware analysis, and security research.

Before contributing:

1. Keep collection APIs read-only unless a maintainer explicitly approves an administrative feature.
2. Do not add bypass, evasion, persistence, credential theft, exploitation, or destructive capabilities.
3. Add tests for compiler, backend, frontend, or agent changes.
4. Document new language syntax and standard-library APIs.

Run checks where available:

```bash
cargo test
pytest backend/tests
npm --prefix frontend test
```

