-- V2: survey responses (append-only, PII-minimized, 90-180d retention)
CREATE TABLE IF NOT EXISTS survey_responses (
  id BIGSERIAL PRIMARY KEY,
  cat VARCHAR(16) NOT NULL CHECK (cat IN ('student','staff','stakeholder')),
  details JSONB NOT NULL,
  answers JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ON survey_responses (cat, created_at);
