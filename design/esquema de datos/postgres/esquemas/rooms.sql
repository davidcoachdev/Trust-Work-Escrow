-- rooms: salas de distribucion/filtrado donde se publican los jobs (tablon/canal)
CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES categories(id),
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    created_by UUID REFERENCES users(id),
    is_public BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_rooms_category ON rooms(category_id);
CREATE INDEX idx_rooms_created_by ON rooms(created_by);
