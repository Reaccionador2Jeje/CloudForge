-- Self-service accounts use a least-privilege member role.
INSERT INTO roles (name, description, is_system)
VALUES ('member', 'Self-service account with access only to personal workload and storage resources.', TRUE)
ON CONFLICT (name) DO NOTHING;

INSERT INTO permissions (resource, action, description)
VALUES
    ('instances', 'read', 'Read personal instance metadata.'),
    ('instances', 'create', 'Create personal instance metadata records.'),
    ('storage', 'read', 'Read personal storage quota and reservations.'),
    ('storage', 'create', 'Create personal storage reservations and volume plans.'),
    ('storage', 'manage', 'Release personal reservations and delete eligible volume plans.'),
    ('images', 'read', 'Read the shared image catalog.'),
    ('compute', 'read', 'Read compute provider and guest profile catalogs.'),
    ('installations', 'read', 'Read personal installation plans.'),
    ('installations', 'create', 'Create personal installation plans.')
ON CONFLICT (resource, action) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON
    (p.resource, p.action) IN (
        ('instances', 'read'), ('instances', 'create'),
        ('storage', 'read'), ('storage', 'create'), ('storage', 'manage'),
        ('images', 'read'), ('compute', 'read'),
        ('installations', 'read'), ('installations', 'create')
    )
WHERE r.name = 'member'
ON CONFLICT DO NOTHING;
