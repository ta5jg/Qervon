CREATE TABLE tenancy.courier_tenants (
    courier_id uuid PRIMARY KEY REFERENCES couriers.couriers (id),
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants (id)
);
CREATE INDEX courier_tenants_tenant_idx ON tenancy.courier_tenants (tenant_id);

CREATE TABLE tenancy.order_tenants (
    order_id uuid PRIMARY KEY REFERENCES orders.orders (id),
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants (id)
);
CREATE INDEX order_tenants_tenant_idx ON tenancy.order_tenants (tenant_id);
