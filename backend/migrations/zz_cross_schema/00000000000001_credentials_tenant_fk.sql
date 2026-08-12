-- Adds the tenant relationship after both identity and tenancy schemas exist.
ALTER TABLE identity.refresh_sessions
    ADD CONSTRAINT refresh_sessions_tenant_fk
    FOREIGN KEY (tenant_id) REFERENCES tenancy.tenants (id);
