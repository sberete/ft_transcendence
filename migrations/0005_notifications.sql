-- Notifications persistantes (amis, salons). Lues par polling cote front
-- (pas de diffusion WebSocket : cette connexion n'est ouverte que dans un
-- salon, pas globalement sur l'app - voir idea.txt). payload denormalise
-- (username/avatar au moment de l'evenement) pour ne pas dependre d'un
-- second appel a chaque affichage, comme ChatMessageView le fait deja pour
-- les messages de chat.
CREATE TABLE notifications (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notifications_user_id_created_at ON notifications(user_id, created_at DESC);
