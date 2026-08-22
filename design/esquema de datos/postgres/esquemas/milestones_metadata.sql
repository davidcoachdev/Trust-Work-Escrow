-- milestones_metadata: metadatos de cada hito (el amount/status estan on-chain)
CREATE TABLE milestones_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    index INTEGER NOT NULL,                           -- indice del hito en el PDA
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (job_pda, index)
);
CREATE INDEX idx_milestones_job ON milestones_metadata(job_pda);
