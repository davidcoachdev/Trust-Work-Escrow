-- job_packages: paquetes escalonados del aviso (estilo Fiverr/PeoplePerHour Hourlies)
CREATE TABLE job_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address) ON DELETE CASCADE,
    tier TEXT NOT NULL CHECK (tier IN ('basic','standard','premium')),
    price BIGINT NOT NULL,
    delivery_time_days INTEGER,
    revisions INTEGER DEFAULT 0,
    description TEXT,
    UNIQUE (job_pda, tier)
);
CREATE INDEX idx_job_packages_job ON job_packages(job_pda);
