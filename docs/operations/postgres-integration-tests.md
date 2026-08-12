# PostgreSQL Integration Tests

The repository integration suite uses an explicitly configured disposable
database. It never chooses a database automatically and is guarded by a
confirmation variable.

```bash
export QERVON_TEST_DATABASE_URL='postgres://qervon_test:password@127.0.0.1:5432/qervon_test'
export QERVON_RUN_POSTGRES_INTEGRATION_TESTS=confirm
make test-postgres
```

The command applies governed migrations, then verifies PostgreSQL round trips
for users, customer profiles and addresses, couriers, vehicles, and courier
payouts. Do not point `QERVON_TEST_DATABASE_URL` at production data.
