<template>
  <main class="page">
    <div class="container">
      <h1>Politique de confidentialité</h1>
      <p class="muted intro">Dernière mise à jour : 26 octobre 2026</p>

      <section class="panel">
        <h2>Qui nous sommes</h2>
        <p>
          SyncWatch est un projet pédagogique réalisé dans le cadre du cursus 42.
          Cette page explique quelles données sont collectées lorsque vous utilisez
          le service, pourquoi, et comment elles sont protégées.
        </p>
      </section>

      <section class="panel">
        <h2>Données collectées</h2>
        <ul>
          <li><strong>Compte</strong> : nom d'utilisateur, adresse email, mot de passe
            (haché avec Argon2, jamais stocké en clair), avatar si vous en téléversez un.</li>
          <li><strong>Connexion via Google ou GitHub</strong> : identifiant de compte
            externe et adresse email vérifiée par le fournisseur, utilisés uniquement
            pour créer ou relier votre compte SyncWatch.</li>
          <li><strong>Réseau social</strong> : votre liste d'amis, les demandes d'ami
            envoyées ou reçues, votre statut en ligne.</li>
          <li><strong>Salons de visionnage</strong> : les salons que vous créez ou
            rejoignez, et les messages de chat que vous y envoyez (conservés pour
            afficher l'historique de la salle).</li>
          <li><strong>Technique</strong> : un jeton de session (haché en base, jamais
            stocké en clair), et les journaux techniques standard du serveur
            (adresse IP, horodatage des requêtes) à des fins de sécurité.</li>
        </ul>
      </section>

      <section class="panel">
        <h2>Pourquoi ces données sont collectées</h2>
        <p>
          Uniquement pour faire fonctionner le service : vous authentifier, afficher
          votre profil et vos relations, faire fonctionner les salons partagés et le
          chat, et assurer la sécurité du compte. Aucune donnée n'est vendue, louée,
          ni partagée avec des régies publicitaires ou des traceurs tiers.
        </p>
      </section>

      <section class="panel">
        <h2>Sources vidéo externes</h2>
        <p>
          SyncWatch n'héberge aucune vidéo : il interroge Internet Archive et
          PeerTube, puis votre navigateur lit directement le flux chez ces
          hébergeurs. Vos termes de recherche sont donc transmis à ces services au
          moment où vous effectuez une recherche, comme pour n'importe quel site qui
          interroge une API publique.
        </p>
      </section>

      <section class="panel">
        <h2>Cookies et stockage local</h2>
        <p>
          Le jeton de session est conservé dans le stockage local de votre
          navigateur (<code>localStorage</code>), pas dans un cookie de suivi. Un
          cookie technique temporaire, protégé (HttpOnly), est utilisé uniquement
          pendant la connexion via Google ou GitHub pour vérifier l'origine de la
          requête ; il est supprimé juste après. Aucun cookie publicitaire ou de
          mesure d'audience n'est utilisé.
        </p>
      </section>

      <section class="panel">
        <h2>Conservation des données</h2>
        <p>
          Vos données sont conservées tant que votre compte existe. Vous pouvez
          demander la suppression de votre compte et des données associées à tout
          moment (voir Contact ci-dessous).
        </p>
      </section>

      <section class="panel">
        <h2>Sécurité</h2>
        <p>
          Les mots de passe sont hachés avec Argon2, les jetons de session sont
          hachés avant stockage, et les échanges entre votre navigateur et le
          serveur sont destinés à être chiffrés (HTTPS).
        </p>
      </section>

      <section class="panel">
        <h2>Vos droits</h2>
        <p>
          Vous pouvez demander l'accès, la rectification ou la suppression de vos
          données personnelles en nous contactant à l'adresse ci-dessous.
        </p>
      </section>

      <section class="panel">
        <h2>Modifications</h2>
        <p>
          Cette politique peut évoluer avec le projet. La date de dernière mise à
          jour est indiquée en haut de cette page.
        </p>
      </section>

      <section class="panel">
        <h2>Contact</h2>
        <p class="muted">À compléter avec une adresse de contact.</p>
      </section>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 720px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.intro {
  margin-top: -0.5rem;
}

.panel {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.panel p,
.panel li {
  color: var(--text);
  line-height: 1.6;
}

.panel ul {
  padding-left: 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

code {
  background: var(--bg-hover);
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-size: 0.9em;
}
</style>
