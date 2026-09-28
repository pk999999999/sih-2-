# logs

`logs.read(path)` reads up to 1 MiB or 1,000 lines from an existing UTF-8 log file under `JOCKY_READ_ROOTS`. It does not clear, truncate, or tamper with logs. Native Windows event-channel and Linux journal readers are not yet implemented.
