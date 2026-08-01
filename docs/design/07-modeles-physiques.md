# 7. Modèles physiques

## 7.1 Géométrie solaire

- Position du soleil (azimut, hauteur) par l'algorithme SPA (§6.1).
- Angle d'incidence sur plan incliné :
  `cos(θ) = cos(z)·cos(β) + sin(z)·sin(β)·cos(γs − γ)`
  avec z = angle zénithal, β = inclinaison du plan, γs = azimut solaire,
  γ = azimut du plan.

## 7.2 Rayonnement solaire

- **Rayonnement extraterrestre** : `G0 = Gsc·(1 + 0.033·cos(360·n/365))`,
  Gsc = constante solaire (1361 W/m²).
- **Modèles de ciel pour la composante diffuse transposée** : isotrope (Liu-Jordan),
  Klucher, Hay-Davies, Reindl, **Perez (référence par défaut)**.
- **Composante réfléchie au sol** : `Gr = GHI·ρ·(1 − cos β)/2`, ρ = albédo du sol
  (paramétrable, variable saisonnière pour agrivoltaïsme/neige).
- **Bifacial** : rayonnement face arrière estimé par modèle de facteurs de vue
  (view-factor) entre le plan de module, le sol et le ciel, pondéré par
  l'albédo et la hauteur de structure ; couplage optionnel à un calcul
  Monte-Carlo simplifié pour les rangées denses (validation croisée avec
  modèles bifaciaux publiés, ex. approche « view factor » de Marion et al.).

## 7.3 Modèle thermique de cellule

- **NOCT (Nominal Operating Cell Temperature)** : modèle simple par défaut pour
  pré-dimensionnement.
- **Modèle Faiman (2008)** (référence par défaut simulation certifiée) :
  `Tcell = Tamb + G/(U0 + U1·v)`, U0/U1 = coefficients thermiques (paramétrables
  par type de montage — toiture ventilée, intégré, ombrière, sol), v = vitesse
  du vent.
- **Modèle Sandia (SAPM thermal)** en option pour cohérence avec bases de
  données de coefficients existantes.

## 7.4 Modèle électrique du module/générateur

- **Modèle à 5 paramètres (single-diode)**, De Soto et al. (2006) :
  `I = IL − I0·[exp((V + I·Rs)/(n·Vt)) − 1] − (V + I·Rs)/Rsh`
  avec IL (photocourant), I0 (courant de saturation inverse), Rs/Rsh (résistances
  série/parallèle), n (facteur d'idéalité), Vt = kT/q (tension thermique).
  Les 5 paramètres sont extrapolés aux conditions réelles (G, T) à partir des
  valeurs STC de la fiche technique (Isc, Voc, Impp, Vmpp, coefficients de
  température α, β, γ) selon les équations de translation de De Soto.
- **Résolution** : forme close par fonction de Lambert W (implémentation Rust
  pour performance), ou Newton-Raphson en repli.
- **Effets de l'ombrage partiel** : décomposition du module en sous-chaînes
  protégées par diodes de bypass, sommation des courbes I-V de sous-chaînes en
  série avec activation des diodes lorsque la tension d'une sous-chaîne devient
  négative (méthode de reconstruction de courbe composite).
- **Mismatch** : perte forfaitaire paramétrable (typiquement 1-3 %) + effet
  explicite de l'ombrage partiel modélisé ci-dessus (non redondant).

## 7.5 Pertes du système

| Poste de perte | Modèle |
|---|---|
| Réflexion angulaire (IAM) | ASHRAE : `IAM = 1 − b0·(1/cos θ − 1)` ou Martin-Ruiz |
| Salissure (soiling) | Facteur mensuel paramétrable, option modèle dynamique pluie-dépôt (type Kimber et al.) |
| Dégradation initiale (LID/LeTID) | Perte ponctuelle à la mise en service (% paramétrable par technologie) |
| Dégradation annuelle | Taux linéaire ou non-linéaire paramétrable par technologie de cellule |
| Mismatch électrique | Cf. §7.4 |
| Câblage DC | Pertes Joule calculées (R·I², §6.11) à partir des sections réelles |
| Onduleur | Courbe de rendement dynamique (§6.5) |
| Câblage AC / transformateur | Pertes Joule + pertes fer/cuivre transformateur (modèle simplifié) |
| Indisponibilité (maintenance, panne) | Taux paramétrable ou planning d'indisponibilité explicite |
| Effet spectral | Modèle SAPM spectral (optionnel, technologies à bande interdite non standard) |

## 7.6 Modèle onduleur

- **Modèle Sandia Inverter Model** (ou équivalent EN 50530 « rendement européen »)
  pour le rendement en fonction de la puissance d'entrée et de la tension DC.
- Prise en compte du seuil de démarrage (P_start), de la consommation de veille
  nocturne, et de l'écrêtage (clipping) au-delà de P_AC,max.

## 7.7 Modèle batterie

- **Modèle de capacité** : Kinetic Battery Model (KiBaM) pour la disponibilité
  d'énergie selon le régime de charge/décharge (deux compartiments : disponible
  / lié, taux de transfert k).
- **Modèle tension/rendement** : modèle de circuit équivalent simplifié (tension
  à vide fonction du SOC + résistance interne) pour les technologies Li-ion,
  rendement coulombique et rendement aller-retour paramétrables par technologie
  (Li-ion NMC/LFP, plomb-acide).
- **Vieillissement** : combinaison cyclage (Rainflow, §6.6) et calendaire
  (fonction Arrhenius simplifiée de la température moyenne et du SOC de stockage).

## 7.8 Modèle hydraulique (pompage solaire)

- **Hauteur manométrique totale (HMT)** : `HMT = Hg + ΔHf`, Hg = hauteur
  géométrique (aspiration + refoulement), ΔHf = pertes de charge linéaires et
  singulières (formule de Darcy-Weisbach / Hazen-Williams selon le diamètre et
  le matériau de conduite).
- **Courbe pompe** : interpolation des courbes constructeur H(Q) et η(Q) à la
  vitesse de rotation réelle imposée par l'électronique (loi d'affinité pour
  variation de vitesse).
- **NPSH** (Net Positive Suction Head) : vérification de non-cavitation par
  comparaison au NPSH requis constructeur.

## 7.9 Modèle micro-réseau

- Bilan de puissance instantané par pas de temps : `P_PV + P_batt,dech + P_gen +
  P_réseau,import = P_charge + P_batt,charge + P_réseau,export + P_pertes`.
- Contraintes de stabilité simplifiées (réserve tournante minimale si présence
  groupe électrogène, limites de rampe de charge/décharge batterie).

## 7.10 Validation scientifique des modèles

Chaque modèle physique fait l'objet d'un test de non-régression comparant sa
sortie à des jeux de données de référence publiés (données NREL/Sandia/PVGIS en
accès libre, sites de mesure documentés dans la littérature), avec tolérance
définie par modèle (voir §14.4). Aucune donnée ni algorithme propriétaire d'un
concurrent n'est utilisé comme référence — uniquement des publications
scientifiques ouvertes et des mesures de terrain publiques.
