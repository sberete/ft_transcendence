# Ressources

Document de référence pour comprendre chaque techno utilisée dans SyncWatch —
utile pour toi, pour un futur coéquipier qui reprend une branche, et comme
base pour la section "Resources" du README final (exigée par le sujet).

Chaque lien a été vérifié avant d'être ajouté ici (pas de lien deviné). Pour
une vidéo YouTube précise dont je n'ai pas pu confirmer l'URL exacte, le
lien pointe vers la chaîne elle-même avec le titre à chercher, plutôt que
de risquer un lien mort.

---

## Rust (langage)

- **Le livre officiel ("The Book")** — https://doc.rust-lang.org/book/
  Le point de départ incontournable. Ownership, borrowing, structs, enums,
  traits, generics, gestion d'erreurs, concurrence. Accessible aussi hors
  ligne via `rustup doc --book`.
- **Rust by Example** — https://doc.rust-lang.org/rust-by-example/
  Même matière que le livre, mais par l'exemple de code plutôt que par la
  théorie — complémentaire si tu apprends mieux en lisant du code.
- **Chaîne YouTube "Let's Get Rusty"** — https://www.youtube.com/@letsgetrusty
  Playlist qui suit le livre officiel chapitre par chapitre, en vidéo.
  Bon complément si la lecture seule ne suffit pas pour un concept donné
  (l'ownership et les lifetimes, notamment).

## Tokio (runtime asynchrone)

- **Tutoriel officiel** — https://tokio.rs/tokio/tutorial
  Explique `async`/`.await`, les tâches (`tokio::spawn`), l'état partagé,
  les channels (`mpsc`) — exactement les briques utilisées dans `src/ws/`.
  Basé sur la construction progressive d'un mini-Redis.

## Axum (framework backend)

- **Dépôt officiel + exemples** — https://github.com/tokio-rs/axum
  Le dossier `examples/` du dépôt contient des mini-projets complets et à
  jour, dont `examples/chat` (chat WebSocket avec un canal `broadcast`) —
  une base de comparaison utile pour `src/ws/handler.rs`.
- **Documentation API (docs.rs)** — https://docs.rs/axum
  Référence de chaque type/fonction (`State`, `Router`, `WebSocketUpgrade`,
  extracteurs...). Axum construit sur l'écosystème Tower plutôt que
  d'avoir son propre système de middleware — voir aussi `docs.rs/tower`.

## SQLx + PostgreSQL

- **Dépôt officiel SQLx** — https://github.com/launchbadge/sqlx
  SQL asynchrone en Rust pur, requêtes vérifiées à la compilation, pas
  d'ORM/DSL — exactement l'approche du projet (`sqlx::query_as`,
  `#[derive(sqlx::FromRow)]` partout dans `src/*/handlers.rs`).
- **Documentation PostgreSQL** — https://www.postgresql.org/docs/
  Le tutoriel (Part I) pour les bases du relationnel/SQL ; le reste du
  manuel sert de référence (types, contraintes, `INTERVAL`, etc. —
  utilisés dans les migrations et dans `src/ws/registry.rs` pour le calcul
  du statut "en ligne").

## WebSockets (temps réel — Phase 3)

- **RFC 6455 — The WebSocket Protocol** — https://www.rfc-editor.org/info/rfc6455
  La norme elle-même : handshake, trames, codes de fermeture. Utile pour
  comprendre pourquoi on utilise des codes `4400`-`4429` personnalisés
  dans `docs/websocket.md` (la plage 4000-4999 est réservée aux
  applications par la norme).
- **MDN — WebSocket API (navigateur)** — https://developer.mozilla.org/en-US/docs/Web/API/WebSocket
  Côté client : `new WebSocket(url)`, événements `open`/`message`/`close`,
  utile pour la branche `frontend-chat`/`frontend-sync`.

## Sécurité web

- **MDN — CORS** — https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
  Explique pourquoi les WebSockets ne passent PAS par le CORS (d'où la
  vérification manuelle de l'en-tête `Origin` dans `src/ws/handler.rs`) et
  pourquoi le joker `*` ne couvre pas `Authorization` (vu plus tôt sur le
  switch Actix → Axum).
- **OWASP Cheat Sheet Series** — https://github.com/OWASP/CheatSheetSeries
  Fiches pratiques, notamment *CSRF Prevention* et *XSS Prevention* —
  pertinent pour le `state` anti-CSRF de l'OAuth et pour le choix de ne
  jamais faire `v-html` sur du contenu externe (descriptions Archive.org).
- **RFC 9106 — Argon2** — https://www.rfc-editor.org/rfc/rfc9106
  La référence derrière `argon2` (crate utilisée dans
  `src/auth/password.rs`).

## OAuth 2.0 (Google / GitHub)

- **RFC 6749 — The OAuth 2.0 Authorization Framework** — https://www.rfc-editor.org/rfc/rfc6749
  Section 4.1 : le flux "Authorization Code", celui utilisé dans
  `src/auth/oauth/`. Dense mais c'est LA référence normative.
- **Documentation Google Identity (OAuth 2.0, Web Server Apps)** — https://developers.google.com/identity/protocols/oauth2/web-server
- **Documentation GitHub — Authorizing OAuth Apps** — https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps
  Les deux expliquent les spécificités de chaque fournisseur (scopes,
  endpoints, format des réponses) par-dessus le RFC générique.

## Frontend : Vue 3 / Vite / Vue Router

- **Documentation Vue 3** — https://vuejs.org/guide/introduction
  Composition API (`<script setup>`), réactivité (`ref`/`reactive`),
  composants — tout le style utilisé dans `frontend/src/`.
- **Documentation Vite** — https://vite.dev
  Le serveur de dev et le build utilisés (`npm run dev`, `npm run build`).
- **Documentation Vue Router** — https://router.vuejs.org/introduction
  Routes, navigation guards (`router.beforeEach`, utilisé dans
  `frontend/src/router/index.js` pour `requiresAuth`/`guestOnly`).

## Lecture vidéo : hls.js

- **Dépôt officiel** — https://github.com/video-dev/hls.js
  Lecteur HLS pur JS par-dessus `<video>`. Le README du dépôt couvre la
  mise en route (`new Hls()`, `attachMedia`) — utilisé par
  `frontend/src/components/VideoPlayer.vue`.

## Sources du catalogue (module `media`)

- **Internet Archive — Advanced Search API** — https://archive.org/advancedsearch.php
  Recherche par mots-clés + filtres (`mediatype:movies`, etc.), syntaxe
  façon Lucene. Pagination limitée à 10 000 résultats (sans incidence ici
  vu `MAX_RESULTS_PER_PROVIDER`).
- **PeerTube — API REST** — https://docs.joinpeertube.org/api/rest-reference.html
  Spécification OpenAPI complète. Authentification OAuth2 propre à chaque
  instance (non utilisée ici : le projet ne fait que des lectures
  publiques, pas d'upload).

## Docker / Docker Compose

- **Référence du fichier Compose** — https://docs.docker.com/reference/compose-file/
  La spec complète (`services`, `volumes`, `healthcheck`, `depends_on`...)
  utilisée dans `docker-compose.yml`.
- **Chaîne YouTube NetworkChuck** — https://www.youtube.com/@NetworkChuck
  Vidéo à chercher sur la chaîne : *"Docker Compose will BLOW your
  MIND!!"* (~16 min, n'ai pas pu confirmer l'URL exacte de la vidéo donc
  je donne la chaîne) — concentré sur Compose spécifiquement plutôt que
  Docker en général.
- **Chaîne YouTube TechWorld with Nana** — https://www.youtube.com/@TechWorldwithNana
  Cours plus complet (images, conteneurs, volumes, Compose) si le concept
  général de conteneurisation n'est pas encore clair.

---

## Utilisation de l'IA sur ce projet

*Brouillon pour la section README correspondante — à relire/compléter
avant le rendu final, notamment pour la partie catalogue vidéo/OAuth faite
dans une session séparée (voir le journal dans `idea.txt`, entrée du
2026-10-10).*

L'IA (Claude, via Claude Code) a été utilisée tout au long du projet comme
assistant de pair-programming, pas comme auteur autonome :

- **Conception** : discussion du concept (SyncWatch), des choix
  techniques (Rust/Axum, Postgres, Vue), et des arbitrages produit
  (ex : les 4 décisions de `docs/websocket.md` section 9) — toujours
  validés ou tranchés par moi, jamais décidés seul par l'IA.
- **Implémentation** : écriture assistée des modules backend (`auth`,
  `users`, `friends`, `rooms`, `ws`, `media`) et frontend, avec relecture
  et tests systématiques après chaque étape (`curl`, scripts de test
  WebSocket, Docker) avant de committer.
- **Debug** : plusieurs bugs réels trouvés et corrigés en binôme avec
  l'IA (ex : `media: null` manquant dans `playback.state` après
  `set_media`, cache navigateur sur `index.html`, bug de cache de build
  Docker).
- **Apprentissage Rust** : la piscine Rust (`rust-piscine/`) a servi à
  apprendre le langage avant d'attaquer le projet — exercices faits et
  codés entièrement par moi, l'IA n'a fourni que les énoncés et jamais les
  solutions.
- **Ce que je peux expliquer** : l'architecture du backend (modules,
  extracteur `AuthUser`, pool de connexions, migrations), le protocole
  WebSocket et ses décisions, le fonctionnement de Docker Compose pour ce
  projet — je dois être capable de défendre chaque partie à l'oral, pas
  seulement de l'avoir fait fonctionner.
