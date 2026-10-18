# Contrat temps réel SyncWatch (WebSocket)

> **Statut : proposition.** Rien de ce document n'est encore implémenté. Il
> fixe le format des échanges *avant* d'écrire le module `ws` (phase 3), pour
> que backend et frontend puissent être développés chacun de leur côté.
> Le contrat REST, lui, décrit l'existant : voir `openapi.yaml`.

## 1. Vue d'ensemble

Une **salle** (room) regroupe des utilisateurs qui regardent la même vidéo.
Le serveur est la seule source de vérité sur l'état de lecture : un client ne
dit jamais aux autres quoi faire, il **demande** au serveur, qui valide puis
diffuse à tous.

```
Client A ── playback.pause ──► Serveur ── playback.state ──► A, B, C
```

Les tables `rooms` et `messages` existent déjà (migration 0001).

## 2. Routes REST à ajouter

La création et la consultation d'une salle restent en REST : ce sont des
actions ponctuelles, pas du temps réel.

| Méthode | Chemin | Rôle | Réponse |
|---|---|---|---|
| `POST` | `/rooms` | Créer une salle. Corps : `{"name"}` | `201` `Room` |
| `GET` | `/rooms/{id}` | Lire une salle | `200` `Room` · `404` |
| `DELETE` | `/rooms/{id}` | Supprimer sa salle | `204` · `403` si pas propriétaire |
| `GET` | `/rooms/{id}/messages?before={id}&limit=50` | Historique du chat, du plus récent au plus ancien | `200` `[ChatMessage]` |

```json
// Room
{ "id": 12, "name": "Soirée Chaplin", "owner_id": 3, "created_at": "2026-10-10T20:00:00Z" }
```

Toutes privées (`Authorization: Bearer`). À ajouter dans `openapi.yaml` au
moment de l'implémentation.

## 3. Connexion

```
GET /ws        (Upgrade: websocket)
```

### Authentification par premier message

Un navigateur ne peut pas ajouter d'en-tête `Authorization` à une connexion
WebSocket. Deux options existent : mettre le token dans l'URL
(`/ws?token=…`), ou l'envoyer dans le premier message. **On retient le
premier message**, car une URL finit dans les logs du serveur, des proxys et
l'historique du navigateur, et un token n'a rien à y faire.

1. Le client ouvre la connexion.
2. Il envoie `auth` dans les **5 secondes**.
3. Le serveur répond `auth.ok`, ou ferme avec le code `4401`.

Tant que `auth.ok` n'a pas été reçu, tout autre message ferme la connexion.

Le serveur doit aussi vérifier l'en-tête `Origin` de la requête d'ouverture
et refuser les origines inconnues : les WebSockets ne sont pas soumis au CORS.

## 4. Format des messages

Trames **texte**, JSON, un objet par trame. Le champ `type` identifie le
message ; les autres champs dépendent du type.

```json
{ "type": "chat.send", "content": "Salut !" }
```

- Noms en `domaine.action`, en minuscules.
- Dates en ISO 8601 UTC. Positions de lecture en **secondes**, nombre décimal.
- Horloge serveur (`server_time`) en **millisecondes** depuis l'epoch Unix.
- Un champ inconnu est ignoré ; un `type` inconnu provoque un message `error`.
- Taille maximale d'une trame : **4 Ko**. Au-delà, fermeture `4400`.

## 5. Messages client → serveur

| `type` | Champs | Effet |
|---|---|---|
| `auth` | `token` | Authentifie la connexion. |
| `room.join` | `room_id` | Rejoint une salle. Quitte la précédente : une connexion = une salle. |
| `room.leave` | — | Quitte la salle courante. |
| `chat.send` | `content` (1 à 500 caractères) | Envoie un message dans la salle. |
| `playback.set_media` | `provider`, `id` | Change la vidéo de la salle. Remet la position à 0, en pause. |
| `playback.play` | `position` | Lance la lecture depuis cette position. |
| `playback.pause` | `position` | Met en pause à cette position. |
| `playback.seek` | `position` | Saute à cette position sans changer lecture/pause. |
| `ping` | `t` (horloge du client, ms) | Mesure de latence et de décalage d'horloge. |

`provider` et `id` suivent exactement les règles de `GET /media/{provider}/{id}`.
Le serveur les valide de la même façon, et n'accepte jamais d'URL.

## 6. Messages serveur → client

| `type` | Champs | Quand |
|---|---|---|
| `auth.ok` | `user` (`{id, username, avatar_url}`) | Après un `auth` valide. |
| `room.state` | `room`, `members`, `media`, `playback` | Juste après `room.join` : l'état complet. |
| `room.member_joined` | `user` | Quelqu'un entre dans la salle. |
| `room.member_left` | `user_id` | Quelqu'un sort ou se déconnecte. |
| `room.closed` | — | La salle a été supprimée. |
| `chat.message` | `id`, `sender`, `content`, `created_at` | Nouveau message, y compris pour son auteur. |
| `playback.state` | `media`, `playing`, `position`, `server_time`, `by` | À chaque changement de lecture, à tous. |
| `pong` | `t`, `server_time` | Réponse à `ping`. |
| `error` | `code`, `message` | Une demande a été refusée. La connexion reste ouverte. |

```json
// room.state
{
  "type": "room.state",
  "room": { "id": 12, "name": "Soirée Chaplin", "owner_id": 3 },
  "members": [{ "id": 3, "username": "alice", "avatar_url": null }],
  "media": { "provider": "archive", "id": "notld_201610" },
  "playback": { "playing": true, "position": 754.2, "server_time": 1791590000000 }
}
```

`media` vaut `null` tant qu'aucune vidéo n'a été choisie.

### Codes d'erreur (`error.code`)

| Code | Signification |
|---|---|
| `bad_message` | JSON invalide, `type` inconnu ou champ manquant. |
| `not_in_room` | Action de salle envoyée sans avoir rejoint de salle. |
| `room_not_found` | La salle n'existe pas. |
| `forbidden` | Action réservée au propriétaire de la salle. |
| `invalid_media` | `provider` ou `id` au mauvais format. |
| `rate_limited` | Trop de messages. |

## 7. Synchronisation de la lecture

### Principe

Le serveur ne diffuse pas « où en est la vidéo » en continu. Il mémorise un
**point de référence** et chaque client calcule la position attendue :

```
si playing : position_attendue = position + (maintenant_serveur - server_time) / 1000
sinon      : position_attendue = position
```

Un seul message suffit donc pour qu'un nouvel arrivant se cale, même en cours
de lecture.

### Décalage d'horloge

`maintenant_serveur` n'est pas l'horloge du client. Le client l'estime avec
`ping` / `pong` :

```
aller_retour = reception - t
decalage     = server_time + aller_retour / 2 - reception
maintenant_serveur = Date.now() + decalage
```

Faire 5 mesures à la connexion et garder celle dont l'aller-retour est le plus
court (la moins perturbée), puis une mesure toutes les 30 secondes.

### Correction de la dérive

Toutes les 2 secondes, le client compare `video.currentTime` à la position
attendue :

| Écart | Action |
|---|---|
| moins de 0,25 s | Rien. |
| de 0,25 s à 2 s | Ajuster `video.playbackRate` (0,95 ou 1,05) jusqu'à rattrapage : imperceptible. |
| plus de 2 s | `video.currentTime = position_attendue` (saut visible). |

C'est le même principe que le moteur WASM de Movix (statuts *calibrating →
adjusting → perfect*), en JavaScript simple. Un module WASM pourra le
remplacer plus tard sans changer ce contrat.

### Éviter les boucles

Quand le client applique un `playback.state`, la balise `<video>` émet ses
propres événements `play`, `pause`, `seeked`. Il ne faut **pas** les renvoyer
au serveur, sinon chaque client relance tous les autres à l'infini. Le client
garde un drapeau « changement venant du serveur » et ignore les événements
qu'il a lui-même provoqués.

## 8. Limites et fermetures

| Règle | Valeur |
|---|---|
| Délai pour s'authentifier | 5 s |
| Taille d'une trame | 4 Ko |
| Messages de chat | 5 par 5 secondes et par utilisateur |
| Commandes de lecture | 10 par 5 secondes et par utilisateur |
| Inactivité (ni message ni `ping`) | fermeture après 60 s |

Dépasser une limite renvoie `error` avec `rate_limited`. Insister ferme la
connexion.

| Code de fermeture | Raison |
|---|---|
| `1000` | Fermeture normale. |
| `4400` | Message illisible ou trame trop grande. |
| `4401` | Token absent, invalide ou expiré, ou délai d'authentification dépassé. |
| `4408` | Inactivité. |
| `4429` | Limites dépassées de façon répétée. |

Les codes 4000 à 4999 sont réservés aux applications par la norme WebSocket.

Après une fermeture autre que `4401`, le client se reconnecte avec un délai
croissant (1 s, 2 s, 4 s… plafonné à 30 s), renvoie `auth` puis `room.join`.

## 9. Décisions prises (2026-10-19)

1. **Qui contrôle la lecture ?** Tous les membres peuvent lire, mettre en
   pause et avancer. Seul le propriétaire peut changer de vidéo
   (`playback.set_media` refusé avec `forbidden` si l'appelant n'est pas
   `owner_id`).
2. **Qui peut rejoindre une salle ?** Toute personne connectée qui a le
   lien (`room_id`). Pas de vérification d'amitié.
3. **Que devient une salle vide ?** **Suppression automatique** dès que le
   dernier membre quitte ou se déconnecte (différent de la proposition
   initiale). Le serveur diffuse `room.closed` avant de fermer les
   connexions restantes, puis supprime la ligne `rooms` (cascade sur
   `messages` via la contrainte existante). Une salle qui n'a jamais eu de
   membre connecté (créée puis jamais rejointe) n'est pas concernée tant
   que personne n'a déclenché `room.join` puis quitté.
4. **Le chat est-il conservé ?** Oui, dans la table `messages`. Texte brut
   uniquement, jamais interprété comme HTML (`v-text`/interpolation côté
   Vue, jamais `v-html`).
