# process

`process.list()` enumerates visible processes using documented platform APIs.

`process.modules(pid)` inventories executable file mappings from `/proc/<pid>/maps` on Linux and ToolHelp module snapshots on Windows. It is limited to modules visible under normal OS permissions and returns an error on access denial.
