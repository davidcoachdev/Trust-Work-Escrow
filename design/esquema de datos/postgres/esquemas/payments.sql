-- payments: espejo de transacciones on-chain (signature es la fuente de verdad)
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature TEXT UNIQUE NOT NULL,                    -- tx signature on-chain
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address),
    payer TEXT,
    payee TEXT,
    amount BIGINT NOT NULL,                            -- lamports
    type TEXT NOT NULL
        CHECK (type IN ('deposit','release','refund','fee','arbitration')),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_payments_job ON payments(job_pda);
