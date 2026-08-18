-- support_tickets_metadata: metadatos del ticket de soporte (razón/resolución off-chain)
CREATE TABLE support_tickets_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ticket_pda TEXT UNIQUE NOT NULL,                   -- PDA SupportTicket on-chain
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);
CREATE INDEX idx_support_tickets_job ON support_tickets_metadata(job_pda);
