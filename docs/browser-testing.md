# Support de navigateurs additionnels — plan de test

Pour le module **"Support for additional browsers"** (Accessibility,
mineur, 1 pt, chap. IV). Le sujet demande une compatibilité **testée et
documentée** avec au moins deux navigateurs en plus du principal — pas
juste "ça devrait marcher". Ce document liste quoi tester, pourquoi
chaque point est pertinent pour ce projet précis, et une grille à
remplir en le faisant pour de vrai.

Je ne peux pas ouvrir de navigateur moi-même : cette passe doit être
faite à la main. Ce document sert de checklist pour ne rien oublier et
de justificatif pour le README/l'oral une fois rempli.

## Navigateurs visés

| Navigateur | Moteur | Statut avant cette passe |
|---|---|---|
| Chrome | Blink | Référence, utilisé pendant tout le développement |
| Brave | Blink (même moteur que Chrome) | Déjà utilisé plusieurs fois pendant le débogage de la synchro de lecture |
| Safari | WebKit | Déjà utilisé plusieurs fois (débogage OAuth, débogage sync) — mais jamais de passe systématique |
| Firefox | Gecko | **Jamais testé** sur ce projet |

Brave étant Chromium, le tester n'apporte presque rien de neuf par
rapport à Chrome (même moteur de rendu, mêmes APIs) — Safari et Firefox
sont les deux qui comptent vraiment pour ce module, parce que ce sont
des moteurs différents avec des comportements différents sur les points
sensibles ci-dessous.

## Pourquoi ces points précis (pas une liste générique)

Chaque ligne correspond à quelque chose de spécifique au code de ce
projet, pas à un test générique "le site s'affiche" :

- **Lecture vidéo HLS** : `VideoPlayer.vue` utilise hls.js partout
  *sauf* sur Safari, qui lit le HLS nativement (`canPlayType`). Deux
  chemins de code différents selon le navigateur — doivent être
  vérifiés séparément, pas supposés équivalents.
- **Autoplay avec le son** : les navigateurs bloquent `video.play()`
  tant qu'il n'y a pas eu de vrai clic sur CET élément précis. C'est la
  cause directe de deux bugs déjà trouvés et corrigés cette session
  (voir le manuel, section « Salons et temps réel ») — la politique
  exacte varie d'un moteur à l'autre, donc le comportement du bouton
  "Rejoindre la vidéo en cours" doit être revérifié par navigateur.
- **Popup OAuth + `BroadcastChannel`** : `OAuthButtons.vue` ouvre une
  popup et communique avec elle via `BroadcastChannel`. Un bug Safari
  précis a déjà été observé ici (popup muette en navigation normale,
  plus jamais reproduit depuis sans cause confirmée — voir le JOURNAL).
  `BroadcastChannel` vient d'être rendu tolérant à l'absence de support
  (repli direct en pleine page) : à vérifier que ce repli ne se
  déclenche jamais par erreur sur un navigateur qui le supporte bien.
- **`navigator.clipboard`** (copier le lien d'invitation d'un salon) :
  déjà avec un repli (`catch` silencieux, le lien reste sélectionnable),
  mais jamais vu échouer ni réussir en vrai sur Firefox/Safari.
- **WebSocket + synchro de lecture** : le moteur de synchro
  (`playbackSync.js`) manipule directement `<video>` (play/pause/seek,
  `playbackRate`). Chaque navigateur a sa propre implémentation des
  événements `play`/`pause`/`seeked` et de la précision de `currentTime`
  — la fenêtre de grâce et de suppression (voir le manuel) a été
  calibrée en observant Chrome/Brave/Safari, jamais Firefox.
- **Upload d'avatar avec barre de progression** : `XMLHttpRequest.upload.onprogress`,
  ajouté le 2026-10-30, jamais vérifié visuellement nulle part.
- **Design (CSS)** : variables CSS, `aspect-ratio`, `backdrop-filter`
  éventuel — à l'œil, pas de vérification automatisée possible ici.

## Grille à remplir

Une ligne par navigateur testé. Mettre ✅ / ❌ / ⚠️ (partiel, préciser en
note) dans chaque colonne.

| Navigateur (version) | Signup/Login | OAuth popup | Recherche + filtres | Upload avatar + barre | Salon : rejoindre par code | Chat | Sync vidéo (lecture en cours) | Sync vidéo (rejoindre en cours) | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Firefox | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Passe faite le 2026-11-04, confirmée par l'utilisateur en vrai sur sa machine |
| Safari | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Passe faite le 2026-11-04, confirmée par l'utilisateur en vrai sur sa machine |

## Comment tester la synchro vidéo (le point le plus facile à mal tester)

Un seul onglet ne suffit pas : il faut deux sessions qui voient le même
salon en même temps, sinon il n'y a rien à synchroniser.

1. Ouvrir le même salon dans **deux navigateurs différents** (ou le même
   navigateur en navigation normale + privée, pour deux comptes), un
   compte propriétaire et un invité.
2. Propriétaire : choisir une vidéo, lancer la lecture.
3. Invité : doit voir le bouton "Rejoindre la vidéo en cours" (pas de
   lecture automatique) — cliquer, vérifier que la lecture démarre à la
   bonne position.
4. Mettre en pause côté propriétaire : l'invité doit suivre en quelques
   secondes, sans action de sa part.
5. Avancer manuellement côté invité (glisser la barre de progression) :
   le propriétaire doit suivre aussi.
6. Propriétaire quitte le salon (reste l'invité) : revérifier que
   l'invité devient propriétaire (peut changer de vidéo) et que rien ne
   redémarre à zéro.

## Une fois la grille remplie

- Mettre à jour le tableau "Modules" du README (`Support for additional
  browsers` : In progress → Done, avec un résumé d'une phrase).
- Si un vrai bug spécifique à un navigateur est trouvé : le corriger
  d'abord, puis seulement ensuite marquer le module acquis — un module
  incomplet ou buggé vaut 0 point, pas de demi-point.
