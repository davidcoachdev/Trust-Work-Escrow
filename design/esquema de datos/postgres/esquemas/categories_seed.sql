-- categories_seed: categorias comunes de freelance
-- Las gestiona el administrador desde su panel; el backend hace seed de las mas usadas.
-- Fuente: taxonomia observada en LinkedIn + Fiverr / Workana / Freelancer (analisis de competencia).
INSERT INTO categories (id, parent_id, name, slug, descripcion) VALUES
    (gen_random_uuid(), NULL, 'Desarrollo Web', 'web-development', 'Sitios y aplicaciones web frontend/backend.'),
    (gen_random_uuid(), NULL, 'Desarrollo Móvil', 'mobile-development', 'Apps iOS, Android y multiplataforma.'),
    (gen_random_uuid(), NULL, 'Desarrollo de Software', 'software-development', 'Sistemas, APIs y arquitectura de software.'),
    (gen_random_uuid(), NULL, 'Diseño Gráfico', 'graphic-design', 'Identidad visual, branding e ilustracion.'),
    (gen_random_uuid(), NULL, 'Diseño UX/UI', 'ux-ui-design', 'Experiencia de usuario e interfaces.'),
    (gen_random_uuid(), NULL, 'Marketing Digital', 'digital-marketing', 'Campanas, growth y presencia online.'),
    (gen_random_uuid(), NULL, 'SEO / SEM', 'seo-sem', 'Posicionamiento y publicidad en buscadores.'),
    (gen_random_uuid(), NULL, 'Redacción y Traducción', 'writing-translation', 'Contenido y traduccion de textos.'),
    (gen_random_uuid(), NULL, 'Redacción de Contenidos', 'content-writing', 'Articulos, blogs y copywriting.'),
    (gen_random_uuid(), NULL, 'Video y Animación', 'video-animation', 'Edicion, motion graphics y animacion.'),
    (gen_random_uuid(), NULL, 'Música y Audio', 'music-audio', 'Produccion, mezcla y podcasts.'),
    (gen_random_uuid(), NULL, 'Fotografía', 'photography', 'Sesiones, edicion y bancos de imagenes.'),
    (gen_random_uuid(), NULL, 'Data Science / IA', 'data-science', 'ML, analitica y modelos de IA.'),
    (gen_random_uuid(), NULL, 'DevOps / Infraestructura', 'devops', 'CI/CD, nube y operaciones.'),
    (gen_random_uuid(), NULL, 'Ciberseguridad', 'cybersecurity', 'Pentesting, auditoria y proteccion.'),
    (gen_random_uuid(), NULL, 'Blockchain / Web3', 'blockchain-web3', 'Contratos inteligentes y dApps.'),
    (gen_random_uuid(), NULL, 'Contabilidad y Finanzas', 'finance-accounting', 'Libros, impuestos y analisis financiero.'),
    (gen_random_uuid(), NULL, 'Legal y Jurídico', 'legal', 'Contratos, sociedades y cumplimiento.'),
    (gen_random_uuid(), NULL, 'Recursos Humanos', 'hr', 'Reclutamiento y gestion de equipos.'),
    (gen_random_uuid(), NULL, 'Ventas', 'sales', 'Comercial, B2B y cierre de deals.'),
    (gen_random_uuid(), NULL, 'Atención al Cliente', 'customer-support', 'Soporte y experiencia de cliente.'),
    (gen_random_uuid(), NULL, 'Community Management', 'community-management', 'Redes sociales y comunidades.'),
    (gen_random_uuid(), NULL, 'Consultoría de Negocios', 'business-consulting', 'Estrategia y operaciones.'),
    (gen_random_uuid(), NULL, 'Producto / Project Management', 'product-management', 'Roadmaps, agile y entrega.'),
    (gen_random_uuid(), NULL, 'Educación y Tutoría', 'education', 'Clases, cursos y mentoría.'),
    (gen_random_uuid(), NULL, 'Diseño de Juegos', 'game-design', 'Mecanicas, niveles y mundos.')
ON CONFLICT (slug) DO NOTHING;
