-- V1 init: users, wa_links, invoices, permits, audit_log, outbox
-- sqlx migrate compatible (postgres). Money as NUMERIC(14,2).
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "citext";

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email CITEXT UNIQUE NOT NULL,
  phone_e164 VARCHAR(20) UNIQUE,
  password_hash TEXT NOT NULL,
  cohort VARCHAR(20),
  perms BIGINT[] NOT NULL DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE wa_links (
  phone_e164 VARCHAR(20) PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  linked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE invoices (
  tx_ref UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  student_id UUID NOT NULL REFERENCES users(id),
  amount NUMERIC(14,2) NOT NULL CHECK (amount > 0),
  status VARCHAR(16) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','paid','expired','void')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  paid_at TIMESTAMPTZ
);
CREATE INDEX ON invoices (student_id, status);

CREATE TABLE permits (
  nonce VARCHAR(64) PRIMARY KEY,
  student_id UUID NOT NULL REFERENCES users(id),
  course_code VARCHAR(16) NOT NULL,
  session_label VARCHAR(32) NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  revoked BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX ON permits (student_id, course_code);

CREATE TABLE audit_log (
  id BIGSERIAL PRIMARY KEY,
  actor UUID REFERENCES users(id),
  action VARCHAR(64) NOT NULL,
  entity VARCHAR(64) NOT NULL,
  entity_id VARCHAR(64) NOT NULL,
  at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON audit_log (entity, entity_id);

-- Outbox for partition-tolerant publish (DB commit -> relay -> RabbitMQ)
CREATE TABLE outbox (
  id BIGSERIAL PRIMARY KEY,
  topic VARCHAR(128) NOT NULL,
  payload JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at TIMESTAMPTZ
);
CREATE INDEX ON outbox (published_at) WHERE published_at IS NULL;
