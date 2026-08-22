-- reviews: ratings entre usuarios al cerrar un job
CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_pda TEXT NOT NULL REFERENCES jobs_metadata(pda_address),
    from_user UUID NOT NULL REFERENCES users(id),
    to_user UUID NOT NULL REFERENCES users(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_reviews_to ON reviews(to_user);
CREATE INDEX idx_reviews_from ON reviews(from_user);
