-- job_interests: senales de interes livianas (estilo Torre signal / LinkedIn Open To Work)
CREATE TABLE job_interests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (job_pda, user_id)
);
CREATE INDEX idx_job_interests_job ON job_interests(job_pda);
CREATE INDEX idx_job_interests_user ON job_interests(user_id);
