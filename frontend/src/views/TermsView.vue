<template>
  <main class="page">
    <div class="container">
      <h1>Conditions générales d'utilisation</h1>
      <p class="muted intro">Dernière mise à jour : 26 octobre 2026</p>

      <section class="panel">
        <h2>Objet</h2>
        <p>
          SyncWatch est un projet pédagogique réalisé dans le cadre du cursus 42. Il
          permet à plusieurs utilisateurs de regarder ensemble, en temps réel, des
          contenus issus de sources légales (Internet Archive, PeerTube), avec un
          chat et un système d'amis. En créant un compte, vous acceptez les présentes
          conditions.
        </p>
      </section>

      <section class="panel">
        <h2>Compte utilisateur</h2>
        <p>
          Vous êtes responsable de la confidentialité de votre mot de passe et des
          actions effectuées depuis votre compte. Les informations fournies à
          l'inscription doivent être exactes.
        </p>
      </section>

      <section class="panel">
        <h2>Usage autorisé</h2>
        <ul>
          <li>Le chat et les salons sont réservés à un usage respectueux : pas de
            harcèlement, de propos haineux, ni de contenu illégal.</li>
          <li>SyncWatch n'héberge aucun contenu vidéo protégé par des droits
            d'auteur : seuls les contenus du domaine public (Internet Archive) ou
            sous licence libre (PeerTube) sont accessibles via l'agrégateur.</li>
          <li>Toute tentative de contournement des mesures de sécurité du service
            est interdite.</li>
        </ul>
      </section>

      <section class="panel">
        <h2>Contenu que vous fournissez</h2>
        <p>
          L'avatar que vous téléversez reste votre propriété ; vous accordez à
          SyncWatch le droit de l'afficher dans le cadre du service. Les messages
          envoyés dans un salon sont conservés pour afficher l'historique de ce
          salon et peuvent être supprimés avec la salle.
        </p>
      </section>

      <section class="panel">
        <h2>Disponibilité du service</h2>
        <p>
          SyncWatch est un projet pédagogique, pas un service commercial : aucune
          garantie de disponibilité continue ou d'absence d'interruption n'est
          fournie.
        </p>
      </section>

      <section class="panel">
        <h2>Résiliation</h2>
        <p>
          Vous pouvez cesser d'utiliser le service et demander la suppression de
          votre compte à tout moment. L'accès à un compte peut être suspendu en cas
          d'usage abusif du service ou de non-respect des présentes conditions.
        </p>
      </section>

      <section class="panel">
        <h2>Limitation de responsabilité</h2>
        <p>
          Le service est fourni "en l'état", dans un cadre pédagogique, sans
          garantie d'aucune sorte quant à son fonctionnement ininterrompu ou
          l'exactitude des contenus provenant de sources tierces.
        </p>
      </section>

      <section class="panel">
        <h2>Modifications</h2>
        <p>
          Ces conditions peuvent évoluer avec le projet. La date de dernière mise à
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
</style>
