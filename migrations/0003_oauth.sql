-- Un compte cree via Google/GitHub n'a pas de mot de passe.
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Lien entre un compte SyncWatch et une identite externe.
-- La cle (provider, provider_user_id) est l'identifiant stable fourni par le
-- fournisseur : on ne se fie jamais a l'email seul, qui peut changer de main.
CREATE TABLE oauth_accounts (
    provider VARCHAR(16) NOT NULL,
    provider_user_id TEXT NOT NULL,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, provider_user_id)
);

CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);
