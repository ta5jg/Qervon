# PostgreSQL Backup and Restore

Production backups run directly on the VPS; Docker is not required.

`qervon-backup.timer` runs the same command daily at 02:15 with up to 15
minutes of random delay. Check its next/last run with:

```sh
systemctl list-timers qervon-backup.timer
```

```sh
sudo -u qervon DATABASE_URL='postgres://...' /opt/qervon/scripts/backup-postgres.sh
```

The command creates a PostgreSQL custom-format dump under
`/var/lib/qervon/backups` and verifies it with `pg_restore --list` before it is
reported as successful. Copy encrypted backups to the approved off-host backup
location through the VPS operations process.

Restore is deliberately guarded. Stop both services, verify the selected dump,
and run:

```sh
sudo -u qervon QERVON_RESTORE_CONFIRM=restore DATABASE_URL='postgres://...' \
  /opt/qervon/scripts/restore-postgres.sh /var/lib/qervon/backups/qervon-<timestamp>.dump
```

Start the migration runner and services only after restore completion. A
restore overwrites the target database; never run it against an unknown or
production database without an approved recovery decision.
