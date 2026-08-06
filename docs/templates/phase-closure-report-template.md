# Modèle de rapport de clôture de phase

**Usage** : copier ce fichier vers `docs/design/NN-rapport-cloture-<nom-de-phase>.md`
(numéro suivant dans la séquence des documents de conception) au moment
de clôturer toute phase ou sous-phase du projet — que ce soit un nouveau
moteur scientifique (`docs/design/22-roadmap-scientifique.md`), une phase
applicative (`docs/design/15-feuille-de-route.md`), ou toute autre étape
majeure. Remplacer chaque section `[À compléter]` ; ne supprimer aucune
section même si elle semble ne pas s'appliquer — écrire explicitement
« Sans objet pour cette phase » plutôt que d'omettre, conformément au
principe de traçabilité (`docs/scientific-principles.md` §5).

Voir `docs/design/20-rapport-cloture-phase1.md` pour un exemple rempli
(rapport de clôture de la Phase 1).

---

# NN. Rapport de clôture — [Nom de la phase]

**Document officiel de clôture de [nom de la phase].** Tag associé, le
cas échéant : `[tag]`.

## 1. Résumé exécutif

[2-3 paragraphes : ce qui a été livré, pourquoi, et le verdict global —
un lecteur qui ne lit que cette section doit comprendre l'essentiel.]

## 2. Objectifs

[Liste des objectifs fixés au démarrage de la phase, tels qu'énoncés dans
le brief ou la roadmap qui l'a déclenchée — cités, pas reformulés après
coup pour paraître atteints.]

## 3. Travaux réalisés

[Description factuelle de ce qui a été construit : modules/crates
livrés, fonctionnalités implémentées, décisions d'architecture prises
(avec renvoi vers les ADR correspondants,
`docs/governance/adr-policy.md`).]

## 4. Validation scientifique

[Pour chaque modèle physique livré dans cette phase, résultat des six
critères de `docs/governance/scientific-review-process.md` :
publications originales, normes, jeux de validation, benchmarks, revue
documentaire, analyse des limites. Chiffres exacts (erreur max/moyenne/
RMS), jamais estimés — citer le rapport auto-généré source.]

## 5. Benchmarks

[Temps moyen, écart-type, mémoire/allocations, pour chaque opération
critique de la phase — méthodologie suivant
`engine/crates/tests/tests/benchmarks.rs`. Citer le rapport auto-généré
source (`engine/validation-data/latest_*.md` ou équivalent pour la
phase).]

## 6. Couverture

[Couverture de tests mesurée (`cargo llvm-cov`), couverture de
documentation (`RUSTFLAGS="-D missing_docs" cargo doc`), résultat de
`cargo clippy -D warnings` et `cargo fmt --check`. Chiffres exacts.]

## 7. Limites connues

[Toute limite, approximation ou domaine de validité restreint introduit
par cette phase — chacune doit avoir une entrée correspondante dans
`docs/scientific-debt-register.md` ; lister ici les ID `DEBT-NNN`
concernés plutôt que de dupliquer le détail.]

## 8. Travaux futurs

[Ce que cette phase laisse explicitement à faire, avec renvoi vers la
phase suivante de `docs/design/15-feuille-de-route.md` et/ou
`docs/design/22-roadmap-scientifique.md`. Distinguer ce qui est *requis*
avant la phase suivante de ce qui est une *amélioration future*
optionnelle.]

## 9. Documents associés

[Table des documents produits ou mis à jour par cette phase — même
format que §8 de `docs/design/20-rapport-cloture-phase1.md`.]

## 10. Déclaration de clôture

[Confirmation explicite que les critères de sortie fixés au démarrage de
la phase sont satisfaits, avec renvoi vers chacun. Déclaration de ce qui
est désormais gelé (si applicable) et de la prochaine étape autorisée.]
