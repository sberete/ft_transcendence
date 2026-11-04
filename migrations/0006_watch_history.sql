-- Historique de visionnage : alimente depuis GET /media/{provider}/{id}
-- (appele par WatchView ET par RoomView des que la video d'un salon change),
-- donc un seul endroit a instrumenter pour couvrir les deux facons de
-- regarder une video sur ce site. Donnees denormalisees (titre/annee/
-- vignette au moment du visionnage) comme le reste du projet (messages,
-- notifications) : pas besoin de re-interroger la source a chaque calcul
-- de recommandation.
--
-- PRIMARY KEY (user_id, provider, media_id) plutot qu'un id de ligne : on ne
-- veut qu'une ligne par (utilisateur, video), revue ou pas - "watched_at" se
-- met juste a jour. C'est exactement la forme dont a besoin le systeme de
-- recommandation (un ensemble, pas un journal d'evenements).
CREATE TABLE watch_history (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(16) NOT NULL,
    media_id VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    year INT,
    thumbnail_url TEXT,
    watched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, provider, media_id)
);

-- Pour "qui d'autre a regarde cette video" (filtrage collaboratif).
CREATE INDEX idx_watch_history_media ON watch_history(provider, media_id);
