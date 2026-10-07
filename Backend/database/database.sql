-- Initial development schema and seed data for the test product.

CREATE TABLE IF NOT EXISTS members (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO members (name) VALUES
    ('Member 1'),
    ('Member 2'),
    ('Member 3')
ON CONFLICT DO NOTHING;
