-- =============================================================================
-- Enthropic Trading Platform - Seed Data
-- Roles, Permissions, and Demo Accounts
-- =============================================================================

-- =============================================================================
-- ROLES
-- =============================================================================

INSERT INTO roles (id, name, description) VALUES
                                              ('a0000000-0000-0000-0000-000000000001', 'admin', 'Full system access - manage users, configs, trading'),
                                              ('a0000000-0000-0000-0000-000000000002', 'trader', 'Submit orders, view own positions, market data'),
                                              ('a0000000-0000-0000-0000-000000000003', 'viewer', 'Read-only access to market data'),
                                              ('a0000000-0000-0000-0000-000000000004', 'risk_manager', 'View all positions, set risk limits, no trading'),
                                              ('a0000000-0000-0000-0000-000000000005', 'system', 'Internal system service account')
ON CONFLICT (name) DO NOTHING;

-- =============================================================================
-- PERMISSIONS
-- =============================================================================

INSERT INTO permissions (id, name, description, resource, action) VALUES
                                                                      -- Order permissions
                                                                      ('b0000000-0000-0000-0000-000000000001', 'orders:create', 'Submit new orders', 'orders', 'create'),
                                                                      ('b0000000-0000-0000-0000-000000000002', 'orders:read', 'View own orders', 'orders', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000003', 'orders:cancel', 'Cancel own orders', 'orders', 'delete'),
                                                                      ('b0000000-0000-0000-0000-000000000004', 'orders:read_all', 'View all users orders', 'orders', 'read_all'),
                                                                      -- Position permissions
                                                                      ('b0000000-0000-0000-0000-000000000010', 'positions:read', 'View own positions', 'positions', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000011', 'positions:read_all', 'View all positions', 'positions', 'read_all'),
                                                                      -- Market data permissions
                                                                      ('b0000000-0000-0000-0000-000000000020', 'market:read', 'View market data', 'market', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000021', 'market:subscribe', 'Subscribe to market streams', 'market', 'subscribe'),
                                                                      -- Account permissions
                                                                      ('b0000000-0000-0000-0000-000000000030', 'accounts:read', 'View own account', 'accounts', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000031', 'accounts:read_all', 'View all accounts', 'accounts', 'read_all'),
                                                                      ('b0000000-0000-0000-0000-000000000032', 'accounts:create', 'Create new accounts', 'accounts', 'create'),
                                                                      ('b0000000-0000-0000-0000-000000000033', 'accounts:update', 'Update account settings', 'accounts', 'update'),
                                                                      ('b0000000-0000-0000-0000-000000000034', 'accounts:delete', 'Deactivate accounts', 'accounts', 'delete'),
                                                                      -- Risk permissions
                                                                      ('b0000000-0000-0000-0000-000000000040', 'risk:read', 'View risk metrics', 'risk', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000041', 'risk:manage', 'Set risk limits', 'risk', 'manage'),
                                                                      -- Strategy permissions
                                                                      ('b0000000-0000-0000-0000-000000000050', 'strategies:read', 'View strategies', 'strategies', 'read'),
                                                                      ('b0000000-0000-0000-0000-000000000051', 'strategies:create', 'Create strategies', 'strategies', 'create'),
                                                                      ('b0000000-0000-0000-0000-000000000052', 'strategies:execute', 'Execute strategies', 'strategies', 'execute'),
                                                                      -- Admin permissions
                                                                      ('b0000000-0000-0000-0000-000000000090', 'admin:full', 'Full administrative access', 'admin', 'full'),
                                                                      ('b0000000-0000-0000-0000-000000000091', 'system:internal', 'Internal system operations', 'system', 'internal')
ON CONFLICT (name) DO NOTHING;

-- =============================================================================
-- ROLE-PERMISSION MAPPINGS
-- =============================================================================

-- Admin: all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000000-0000-0000-0000-000000000001', id FROM permissions
ON CONFLICT DO NOTHING;

-- Trader: trading + own data
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000000-0000-0000-0000-000000000002', id FROM permissions
WHERE name IN (
               'orders:create', 'orders:read', 'orders:cancel',
               'positions:read',
               'market:read', 'market:subscribe',
               'accounts:read',
               'risk:read',
               'strategies:read', 'strategies:execute'
    )
ON CONFLICT DO NOTHING;

-- Viewer: read-only market data
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000000-0000-0000-0000-000000000003', id FROM permissions
WHERE name IN ('market:read', 'market:subscribe')
ON CONFLICT DO NOTHING;

-- Risk Manager: view all, manage risk
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000000-0000-0000-0000-000000000004', id FROM permissions
WHERE name IN (
               'orders:read', 'orders:read_all',
               'positions:read', 'positions:read_all',
               'market:read', 'market:subscribe',
               'accounts:read', 'accounts:read_all',
               'risk:read', 'risk:manage'
    )
ON CONFLICT DO NOTHING;

-- System: internal operations
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000000-0000-0000-0000-000000000005', id FROM permissions
WHERE name IN ('system:internal', 'admin:full')
ON CONFLICT DO NOTHING;

-- =============================================================================
-- DEMO ACCOUNTS (DEVELOPMENT)
-- =============================================================================
-- ALL PASSWORDS: admin123
-- Hash generated: node -e "console.log(require('bcryptjs').hashSync('admin123', 12))"
-- =============================================================================

-- System account
INSERT INTO accounts (
    id, username, email, password_hash, role_id,
    is_active, is_verified, balance, available_balance
) VALUES (
             '00000000-0000-0000-0000-000000000001',
             'system',
             'system@enthropic.local',
             '$2a$12$yz1JoWVQQQIV174l2iWJWeLEHCy/IBM0bsl1tFTGruM6wnR2Oc2r.',
             'a0000000-0000-0000-0000-000000000005',
             true, true, 10000000.00000000, 10000000.00000000
         ) ON CONFLICT (username) DO UPDATE SET
                                                password_hash = EXCLUDED.password_hash,
                                                failed_login_attempts = 0,
                                                locked_until = NULL;

-- Admin account - Password: admin123
INSERT INTO accounts (
    id, username, email, password_hash, role_id,
    is_active, is_verified, balance, available_balance
) VALUES (
             '00000000-0000-0000-0000-000000000002',
             'admin',
             'admin@enthropic.local',
             '$2a$12$yz1JoWVQQQIV174l2iWJWeLEHCy/IBM0bsl1tFTGruM6wnR2Oc2r.',
             'a0000000-0000-0000-0000-000000000001',
             true, true, 1000000.00000000, 1000000.00000000
         ) ON CONFLICT (username) DO UPDATE SET
                                                password_hash = EXCLUDED.password_hash,
                                                failed_login_attempts = 0,
                                                locked_until = NULL;

-- Trader account - Password: admin123
INSERT INTO accounts (
    id, username, email, password_hash, role_id,
    is_active, is_verified, balance, available_balance
) VALUES (
             '00000000-0000-0000-0000-000000000003',
             'trader1',
             'trader1@enthropic.local',
             '$2a$12$yz1JoWVQQQIV174l2iWJWeLEHCy/IBM0bsl1tFTGruM6wnR2Oc2r.',
             'a0000000-0000-0000-0000-000000000002',
             true, true, 100000.00000000, 100000.00000000
         ) ON CONFLICT (username) DO UPDATE SET
                                                password_hash = EXCLUDED.password_hash,
                                                failed_login_attempts = 0,
                                                locked_until = NULL;

-- Viewer account - Password: admin123
INSERT INTO accounts (
    id, username, email, password_hash, role_id,
    is_active, is_verified, balance, available_balance
) VALUES (
             '00000000-0000-0000-0000-000000000004',
             'viewer1',
             'viewer1@enthropic.local',
             '$2a$12$yz1JoWVQQQIV174l2iWJWeLEHCy/IBM0bsl1tFTGruM6wnR2Oc2r.',
             'a0000000-0000-0000-0000-000000000003',
             true, true, 0.00000000, 0.00000000
         ) ON CONFLICT (username) DO UPDATE SET
                                                password_hash = EXCLUDED.password_hash,
                                                failed_login_attempts = 0,
                                                locked_until = NULL;

-- Risk Manager account - Password: admin123
INSERT INTO accounts (
    id, username, email, password_hash, role_id,
    is_active, is_verified, balance, available_balance
) VALUES (
             '00000000-0000-0000-0000-000000000005',
             'riskmanager',
             'risk@enthropic.local',
             '$2a$12$yz1JoWVQQQIV174l2iWJWeLEHCy/IBM0bsl1tFTGruM6wnR2Oc2r.',
             'a0000000-0000-0000-0000-000000000004',
             true, true, 0.00000000, 0.00000000
         ) ON CONFLICT (username) DO UPDATE SET
                                                password_hash = EXCLUDED.password_hash,
                                                failed_login_attempts = 0,
                                                locked_until = NULL;

-- =============================================================================
-- ACCOUNTS VIEW (Backward Compatibility)
-- =============================================================================

DROP VIEW IF EXISTS accounts_with_roles CASCADE;

CREATE VIEW accounts_with_roles AS
SELECT
    a.id,
    a.username,
    a.email,
    a.password_hash,
    a.role_id,
    r.name AS role,
    r.description AS role_description,
    a.is_active,
    a.is_verified,
    a.balance,
    a.available_balance,
    a.created_at,
    a.updated_at,
    a.last_login_at
FROM accounts a
         LEFT JOIN roles r ON a.role_id = r.id;

GRANT SELECT ON accounts_with_roles TO PUBLIC;

DO $$ BEGIN RAISE NOTICE ' Seed data loaded successfully - All passwords: admin123'; END $$;