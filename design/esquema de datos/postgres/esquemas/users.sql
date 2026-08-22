-- users: cache de perfil sincronizado con la wallet on-chain (source of truth)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_principal TEXT UNIQUE NOT NULL,          -- wallet principal (on-chain)
    username TEXT UNIQUE,
    bio TEXT,
    avatar_url TEXT,
    reputation_score DECIMAL(3,2) DEFAULT 0.00,      -- 0..5
    jobs_completed INTEGER DEFAULT 0,
    disputes_won INTEGER DEFAULT 0,
    disputes_lost INTEGER DEFAULT 0,
    verified BOOLEAN DEFAULT FALSE,                  -- verificacion de talento (estilo Toptal/PPH)
    hourly_rate BIGINT,                              -- tarifa por hora (lamports/centavos)
    open_to_work BOOLEAN DEFAULT FALSE,              -- disponibilidad (estilo LinkedIn Open To Work)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_reputation CHECK (reputation_score >= 0 AND reputation_score <= 5)
);
CREATE INDEX idx_users_wallet ON users(wallet_principal);
CREATE INDEX idx_users_username ON users(username);
