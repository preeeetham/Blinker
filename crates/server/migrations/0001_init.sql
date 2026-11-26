-- Blinks: one row per shareable Solana Action.
CREATE TABLE IF NOT EXISTS blinks (
    id            TEXT PRIMARY KEY,
    kind          TEXT NOT NULL CHECK (kind IN ('sol', 'spl')),
    title         TEXT NOT NULL,
    description   TEXT NOT NULL,
    icon_url      TEXT NOT NULL,
    label         TEXT NOT NULL,
    recipient     TEXT NOT NULL,
    creator       TEXT NOT NULL,
    mint          TEXT,
    token_program TEXT,
    token_symbol  TEXT,
    decimals      SMALLINT NOT NULL,
    amounts       DOUBLE PRECISION[] NOT NULL,
    allow_custom  BOOLEAN NOT NULL DEFAULT TRUE,
    theme         TEXT NOT NULL DEFAULT 'aurora',
    fee_lamports  BIGINT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active')),
    fee_signature TEXT UNIQUE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    activated_at  TIMESTAMPTZ,
    CHECK (kind = 'sol' OR (mint IS NOT NULL AND token_program IS NOT NULL))
);
CREATE INDEX IF NOT EXISTS blinks_creator_idx ON blinks (creator, created_at DESC);

-- Confirmed payments through a blink. The signature primary key makes
-- verification idempotent: the watcher and the client callback can both
-- report the same transaction, and only the first insert emits an event.
CREATE TABLE IF NOT EXISTS payments (
    signature  TEXT PRIMARY KEY,
    blink_id   TEXT NOT NULL REFERENCES blinks (id) ON DELETE CASCADE,
    payer      TEXT NOT NULL,
    amount     BIGINT NOT NULL CHECK (amount > 0),
    slot       BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS payments_blink_idx ON payments (blink_id, created_at DESC);
