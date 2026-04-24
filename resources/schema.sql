CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS vetores_rinha (
    id     BIGSERIAL PRIMARY KEY,
    vector vector(14) NOT NULL,
    label  TEXT       NOT NULL
);
