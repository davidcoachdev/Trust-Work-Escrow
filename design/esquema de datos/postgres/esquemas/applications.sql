-- applications: postulaciones (proposal es texto libre, va aca)
CREATE TABLE applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_pda TEXT UNIQUE NOT NULL,              -- PDA Application on-chain
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    applicant_id UUID NOT NULL REFERENCES users(id),
    proposal TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','accepted','rejected','withdrawn')),
    applied_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_apps_job ON applications(job_pda);
CREATE INDEX idx_apps_applicant ON applications(applicant_id);
