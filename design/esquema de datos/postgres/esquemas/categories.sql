-- categories: taxonomia jerarquica de publicaciones (estilo LinkedIn/Freelancer)
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES categories(id) ON DELETE CASCADE,  -- subcategorias
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    descripcion TEXT
);
CREATE INDEX idx_categories_parent ON categories(parent_id);
