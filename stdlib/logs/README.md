# logs

`logs.read(path)` reads up to 1 MiB or 1,000 lines from an existing UTF-8 log file under `JOCKY_READ_ROOTS`. `logs.syslog(path)` exposes those lines as events without guessing their year or timezone. `logs.journal()` retrieves up to 100 entries from the Linux journal. `logs.windows_events(channel)` retrieves up to 100 events from the Windows `System`, `Application`, or `Security` channel. Access denials are errors; none of these calls clear, truncate, or tamper with logs.
