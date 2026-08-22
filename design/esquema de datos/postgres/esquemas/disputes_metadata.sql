-- disputes_metadata: metadatos de la disputa (reason vive aca, no on-chain)
CREATE TABLE disputes_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_pda TEXT UNIQUE NOT NULL,                  -- PDA Dispute on-chain
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);
CREATE INDEX idx_disputes_job ON disputes_metadata(job_pda);
