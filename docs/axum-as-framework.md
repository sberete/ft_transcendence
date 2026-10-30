# Défendre Axum comme framework backend

Notes pour l'oral et pour justifier le module bonus **"Use a framework for
both the frontend and backend"** (Web, majeur, 2 pts, chap. VII). Vue
couvre le frontend sans discussion possible (cité explicitement comme
framework en III.3 du sujet). Axum, lui, n'apparaît pas dans la liste
d'exemples du sujet (Express, Fastify, NestJS, Django, Flask, Rails) — ce
document explique pourquoi il remplit quand même la définition donnée,
avec des exemples précis tirés du code plutôt que des généralités.

## Le critère exact du sujet (III.3)

> *A framework must have the following features: a structured
> architecture, with conventions for organizing code; built-in
> functionalities for common tasks (e.g. routing, database access,
> session management...); a complete ecosystem of tools and libraries.*

Trois critères, pas une liste fermée de noms — le sujet dit "such as"
devant Express/Django/Rails, donc ce sont des exemples, pas une liste
exhaustive. Axum est pris sur ces trois critères, un par un.

## 1. Architecture structurée, avec des conventions

Axum impose une forme de code précise, pas juste une bibliothèque de
fonctions qu'on appelle à sa convenance :

- **Un handler est une fonction `async`, ses paramètres sont des
  extracteurs, son retour implémente `IntoResponse`.** On ne décide pas
  cette forme, Axum l'exige — un handler qui ne s'y conforme pas ne
  compile pas.
- **Le routeur assemble des sous-routeurs par `.nest()`**, pas par un
  système de dispatch ad hoc. Exemple réel (`src/routes/mod.rs:9-15`) :
  chaque module (`auth`, `users`, `friends`, `media`, `rooms`, `ws`)
  expose son propre `Router`, assemblé par l'application au démarrage.
  C'est la convention qui a permis à ce projet de garder 8 modules
  backend séparés, chacun testable et lisible seul.
- **L'inversion de contrôle est le point qui distingue vraiment un
  framework d'une bibliothèque** (voir le manuel en ligne, section
  « Framework et bibliothèque ») : une bibliothèque, c'est du code que
  *le mien* appelle ; un framework, c'est une structure qui appelle *le
  mien* au bon moment. Avec Axum, c'est le routeur qui décide quand et
  avec quels arguments appeler chaque handler — jamais l'inverse.

## 2. Fonctionnalités intégrées pour les tâches courantes

Le sujet cite explicitement *routage*, *accès aux données*, *gestion de
session* comme exemples de ce qu'un framework doit fournir de base.

| Tâche courante | Ce qu'Axum fournit | Exemple dans le code |
|---|---|---|
| Routage | `Router`, `.route()`, `.nest()`, extraction de paramètres de chemin (`Path<T>`) | `src/routes/mod.rs` |
| État partagé / session | `State<T>` (pool PostgreSQL partagé), extracteur **personnalisé** `AuthUser` qui vérifie le token sur toute route privée | `src/auth/extractor.rs` (`impl FromRequestParts<PgPool> for AuthUser`) |
| Accès aux données | Extracteurs de corps/requête (`Json<T>`, `Query<T>`), composés avec sqlx pour le reste | tous les handlers |
| WebSocket | `WebSocketUpgrade`, gestion native de l'upgrade de connexion | `src/ws/handler.rs:26-36` |
| Fichiers statiques | `ServeDir` (tower-http), branché en une ligne | `src/server/mod.rs` |

Le point le plus parlant est l'**extracteur personnalisé** `AuthUser`
(`src/auth/extractor.rs`) : écrire `auth: AuthUser` en paramètre d'un
handler suffit à le rendre privé, sans réécrire la vérification à chaque
fois. C'est exactement le genre de "fonctionnalité intégrée pour une
tâche courante" que le critère demande — sauf qu'Axum permet de
l'étendre proprement au lieu de l'imposer tout fait, ce qui est cohérent
avec un framework bas niveau plutôt que haut niveau (voir section 4).

## 3. Écosystème complet d'outils et de bibliothèques

Axum n'est pas isolé : il est conçu autour de **Tower** (middleware,
appelés *layers*) et s'intègre nativement avec Tokio (runtime async) et
Serde (sérialisation). Dans ce projet :

```rust
// src/server/mod.rs
let app = routes::router(pool.clone(), rooms, frontend_origins)
    .nest_service("/uploads", ServeDir::new("uploads"))   // tower-http
    .layer(Extension(MediaClient::new(&config.peertube_url)))
    .layer(Extension(OAuthConfig::new(&config)))
    .layer(Extension(LoginRateLimiter::new()))
    .layer(cors)                                           // tower-http CorsLayer
    .with_state(pool);
```

Chaque `.layer()` est un middleware Tower qui s'empile sur toutes les
routes sans toucher aux handlers (CORS, état partagé additionnel). C'est
le même mécanisme qui permettrait d'ajouter demain de la compression,
des métriques ou un rate-limiter générique — pas une bibliothèque isolée,
un écosystème qui compose.

## 4. Pourquoi Axum n'est pas dans la liste du sujet (et pourquoi ça n'exclut rien)

Argument anticipé côté correcteur : *"Axum est minimaliste, pas complet
comme Rails ou Django."* Réponse :

- Le sujet ne demande pas d'être "aussi complet que Rails" — il liste
  trois critères précis (ci-dessus), pas un niveau d'abstraction donné.
  Axum les remplit tous les trois, documenté avec des exemples réels du
  code, pas des affirmations vagues.
- Express — cité en exemple dans le sujet — est lui-même minimaliste
  (pas d'ORM, pas de gestion de session intégrée, tout vient de
  middlewares tiers). Si Express compte comme framework malgré ça,
  "minimaliste" n'est pas un critère d'exclusion.
- La différence entre Axum et Rust historiquement : avant des crates
  comme Axum/Actix, écrire un serveur HTTP en Rust voulait dire gérer
  soi-même le parsing TCP/HTTP bas niveau. Axum apporte exactement ce
  que le sujet attend d'un framework par rapport à "écrire sans
  framework" : routage déclaratif, extracteurs, middleware composable.

## Ce qu'il faut pouvoir dire à l'oral, en une phrase

*"Axum remplit les trois critères du sujet — architecture avec
conventions, fonctionnalités intégrées pour le routage/l'état/les
sessions, écosystème Tower/Tokio complet — même s'il n'est pas dans la
liste d'exemples, qui n'est pas fermée. L'extracteur `AuthUser` est
l'exemple le plus concret : il montre l'inversion de contrôle qui
distingue un framework d'une bibliothèque."*
