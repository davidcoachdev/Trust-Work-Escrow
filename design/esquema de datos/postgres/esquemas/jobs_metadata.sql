-- jobs_metadata: metadatos del Job on-chain (title/description viven aca, no en el contrato)
CREATE TABLE jobs_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pda_address TEXT UNIQUE NOT NULL,                  -- PDA Job on-chain (FK logica)
    client_id UUID NOT NULL REFERENCES users(id),
    freelancer_id UUID REFERENCES users(id),
    team_id UUID REFERENCES teams(id),
    room_id UUID REFERENCES rooms(id),                  -- sala de distribucion/filtrado
    category_id UUID REFERENCES categories(id),         -- taxonomia
    title TEXT NOT NULL,
    description TEXT,
    skills TEXT[],                                     -- tags granulares para matching
    budget_type TEXT CHECK (budget_type IN ('fixed','hourly','package')),
    employment_type TEXT CHECK (employment_type IN ('full_time','part_time','contract','internship')),
    engagement_type TEXT CHECK (engagement_type IN ('project','full_time','hourly')),
    publication_type TEXT CHECK (publication_type IN ('bidding','fixed_offer')),
    language TEXT,
    location TEXT,
    is_remote BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_jobs_client ON jobs_metadata(client_id);
CREATE INDEX idx_jobs_freelancer ON jobs_metadata(freelancer_id);
CREATE INDEX idx_jobs_team ON jobs_metadata(team_id);
CREATE INDEX idx_jobs_room ON jobs_metadata(room_id);
CREATE INDEX idx_jobs_category ON jobs_metadata(category_id);
