<a name="other-languages"></a>

# PyGameZ - Autres langues / Otros idiomas / Outros idiomas

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](README.md#-deutsch)** · **🇬🇧 [English](README.md#-english)** · **🇫🇷 [Français](#-francais)** · **🇪🇸 [Español](#-espanol)** · **🇵🇹 [Português](#-portugues)**

---

<a name="-francais"></a>

## 🇫🇷 Français

Une collection de jeux de bureau en Python : **Tkinter** fournit la fenêtre et
le menu, **Pygame** est intégré comme écran de jeu à l'intérieur de la fenêtre
Tkinter. Vingt-neuf jeux avec des options partagées, des commandes entièrement
réassignables, des meilleurs scores, des effets sonores procéduraux et, pour
plusieurs titres, un mode multijoueur. L'interface est **multilingue**
(allemand / anglais / français / espagnol / portugais) ; la langue se choisit au
premier démarrage sur un **écran d'accueil** qui permet aussi de régler la
**résolution** et le **son** (désactivé par défaut) ; l'espagnol et le portugais
se cachent derrière le bouton **« Plus »**. Tout reste modifiable dans les options.

### Démarrage rapide

#### Windows

```bat
install-python.bat    :: une seule fois : installe Python 3.13 + .venv + pygame
start.bat             :: lance la collection de jeux
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # démarre avec .venv, sinon avec le python3 du système
```

`start.bat` / `start.sh` utilisent automatiquement l'environnement virtuel
`.venv` s'il existe, sinon le Python du système. Un guide détaillé pas à pas se
trouve tout en bas : **[Guide d'installation](#guide-dinstallation)**.

### Les jeux

| Jeu          | Modes           | Brève description |
|--------------|-----------------|-------------------|
| **Snake**    | 1 / 2 joueurs   | Snake de luxe avec vue 2D et 3D, turbo, 6 modes de jeu (dont Compétitif), pommes dorées et prestige |
| **Pong**     | 1 / 2 joueurs   | Le classique contre l'IA ou le joueur 2, mode de déplacement commutable |
| **Air Hockey** | 1 / 2 joueurs | Physique 2D avec transfert d'impulsion, contrôle à la souris, IA et power-ups |
| **Tic-Tac-Toe** | 1 / 2 joueurs | Jeu m,n,k de 3x3 à 9x9, trois niveaux d'IA **ou** X contre O en local |
| **Breakout** | 1 joueur        | Casse-briques avec types de briques, power-ups, combos et de nombreux niveaux |
| **Tetris**   | 1 / 2 joueurs   | Classique ou Versus (deux champs côte à côte) |
| **Invaders** | 1 joueur        | Space Invaders : vide les vagues, protège tes vies |
| **Asteroids** | 1 / 2 joueurs  | Physique d'inertie, vagues, OVNIs, power-ups, hyperespace - en solo ou duel coopératif |
| **Pac-Man**  | 1 joueur        | Clone fidèle : 4 IA de fantômes, pilules de pouvoir, tunnels, fruits, niveaux |
| **Flappy Bird** | 1 joueur     | Vol gravitationnel entre les tuyaux, pièces, bouclier, jour/nuit, médailles |
| **Doodle Jump** | 1 joueur     | Saut automatique vers le haut, types de plateformes, ressorts, hélice, monstres |
| **2048**     | 1 joueur        | Puzzle de nombres à faire glisser, objectif : la tuile 2048 |
| **Minesweeper** | 1 joueur     | Le classique avec premier clic sûr, chording, smiley et meilleurs temps |
| **Sudoku**      | 1 joueur     | 400 niveaux à graine (4 difficultés x 100), 4 modes d'assistance avec multiplicateur de score, notes, indices, limite de 3 erreurs |
| **Frogger**     | 1 joueur     | Route + rivière + 5 abris, mouche bonus, crocodiles, limite de temps, 3 difficultés |
| **Memory**      | 1 / 2 joueurs | Trouve les paires sur 4x4 à 8x6, animation de retournement, score en solo ou duel |
| **Solitaire**   | 1 joueur     | 5 variantes (Klondike, Spider, FreeCell, Pyramide, TriPeaks) avec glisser-déposer et annulation |
| **Aim Trainer** | 1 joueur     | Tir sur cible 3D détendu : la souris dirige la caméra, 4 modes (précision/réflexes/mobiles/chill), 3 thèmes dont un trou noir |
| **Puissance 4** | 1 / 2 joueurs | Le classique avec animation de chute : 3 niveaux d'IA (minimax) ou duel local |
| **Duel de tanks** | 1 / 2 joueurs | Duel 2D en arène avec tirs à ricochet, power-ups, 4 arènes, IA à 3 niveaux |
| **Blackjack**    | 1 joueur    | Blackjack de casino avec sabot de 4 jeux, doubler/partager, blackjack 3:2 et solde de jetons persistant |
| **Tunnel Racer** | 1 joueur    | Vol 3D dans un tube néon : mode sans fin + 30 niveaux, pilotage au clavier ou à la souris, motion blur |
| **Labyrinthe 3D** | 1 joueur   | Raycaster à la première personne (style Wolfenstein) avec 50 niveaux à graine, orbes, minicarte - ou vue 2D de dessus |
| **Reversi**      | 1 / 2 joueurs | Othello 8x8 : encercler et retourner les pions, 3 forces d'IA (minimax) ou un duel local |
| **Yams**         | 1 / 2 joueurs | Classique de dés à 13 catégories, bonus supérieur et Yams ; course au score ou hotseat à 2 |
| **Wordle**       | 1 joueur   | Devine le mot de 5 lettres en 6 essais, série sans fin, indices colorés, 5 langues |
| **T-Rex Runner** | 1 joueur   | Course infinie dans le désert : saut variable, s'accroupir, cactus & ptérodactyles, cycle jour/nuit, vitesse croissante, 3 difficultés |
| **Dames**        | 1 / 2 joueurs | 3 règles au choix (allemandes 8×8, internationales 10×10, checkers), prise obligatoire & dame volante, 3 niveaux d'IA (minimax) ou duel local |
| **Poker**        | 1 joueur   | 3 variantes au choix : Texas Hold'em contre l'IA, 5 Card Draw et Vidéo Poker ; tours d'enchères, blinds, compte de jetons persistant |
| **Taquin**         | 1 joueur   | Jeu du 15 en 3x3/4x4/5x5 : glisse les tuiles numérotées dans le trou, contrôle souris ou flèches, score selon coups & temps |
| **Mastermind**     | 1 joueur   | Perce le code couleur secret (3 modes : 4×6, classique, 5×8), pions indicateurs noirs/blancs, série sans fin |
| **Bubble Shooter** | 1 joueur   | Clone de Puzzle Bobble : tire des couleurs identiques par groupes de trois, rebonds sur les parois, grappes qui tombent, 3 difficultés |
| **Hangman**        | 1 joueur   | Devine le mot avant que la potence soit complète ; clavier à l'écran, listes de mots par langue, 3 modes de longueur, série sans fin |
| **Block Jump**     | 1 joueur   | Jeu de plateforme 3D façon Minecraft : monde de blocs (voxels) avec échelles, barrières & blocs-ressorts, caméra 1re/3e personne, flou de mouvement, niveaux générés |

**Le multijoueur (2 joueurs en local)** est disponible pour **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duel
coopératif)**, **Memory (duel)**, **Puissance 4**, **Duel de tanks**,
**Reversi**, **Yams** et **Dames**.
Le mode se choisit directement sur l'écran de préparation
(*Un joueur / Multijoueur*).

#### Détails par jeu

**Snake**
- **NOUVEAU - Vue 3D** (touche **V** dans le setup ou clic sur *Vue*) : le
  plateau est rendu comme une scène 3D en temps réel - une **caméra de
  poursuite** flotte derrière le serpent et se dirige **par rapport au regard**
  (gauche/droite = tourner, deux pressions rapides = demi-tour). Avec brouillard
  de distance, ciel étoilé, sol en damier, bordures, cristaux de nourriture
  rotatifs, particules 3D et secousse de caméra au crash ; après le game over la
  caméra orbite autour du serpent. Le turbo élargit le champ de vision. En 3D :
  *Classique* et *Obstacles* (les murs y sont toujours fixes, la 3D n'existe
  qu'en un joueur). La vue est mémorisée dans `settings.json`.
- **NOUVEAU - Options de caméra 3D** (dans le setup 3D, la ligne *Caméra 3D /
  Smooth-Shake* ou touche **K**) : un menu dédié avec **Smooth-Shake** (caméra
  plus douce, beaucoup moins de secousses en bougeant/tournant), **champ de
  vision (FOV)** et **hauteur de caméra** réglables, plus un interrupteur
  **secousse en tournant** (screen shake aux virages gauche/droite oui/non).
  Tout est mémorisé dans `settings.json`.
- **Turbo** : **maintenir** la touche turbo = vitesse double, consomme de
  l'endurance (barre) ; une fois vide, le turbo se coupe et se recharge. Par
  défaut J1 = Espace/Maj gauche, J2 = Entrée/Maj droite.
- **6 modes de jeu** (dans le setup) : *Classique*, *Speed-Rush* (plus rapide à
  chaque pomme), *Obstacles* (blocs mortels), *Portails* (paires de
  téléporteurs), *Contre-la-montre* (60 secondes, autant de pommes que possible)
  et *Compétitif* (voir ci-dessous).
- **NOUVEAU - Compétitif** (un joueur) : mode sans fin avec **montée de
  niveau** - tu commences avec exactement **une** pomme et tu ne peux pas en
  avoir plus au début ; plus tu en collectes au total, plus ton **niveau**
  monte, ce qui pose une pomme simultanée de plus sur le terrain et augmente le
  multiplicateur de points. Les **pommes bleues** ouvrent une **machine à
  sous** : la mise est ta longueur, le résultat des rouleaux la multiplie ou la
  réduit et fait apparaître un moment des **pommes supplémentaires** (jackpot
  avec trois symboles identiques). Les **pommes violettes** (pari) mettent en
  jeu une part de ta **taille** et multiplient cette part au hasard, le reste
  est à l'abri (nouvelle taille = taille·(1-p) + taille·p·facteur) : en
  **normal** 50 % fixes avec **x0.5 .. x1.5**, en **HARDCORE** plus risqué avec
  **75-90 %** de mise et **x0.25 .. x2.25**. La **taille** s'affiche en
  **décimal en haut à gauche** et est reportée exactement, si bien que les paris
  suivants s'appuient dessus. Il y a **15 niveaux** (multiplicateur jusqu'à x16,
  jusqu'à 16 pommes à la fois) ; les niveaux vivent dans
  `games/levels/snake-comp.json` et s'y étendent sans toucher au code, le reste
  du réglage fin est dans `competitive.py`.
- **NOUVEAU - HARDCORE** (interrupteur dans le setup Compétitif, touche **H**) :
  chaque **turbo dévore la longueur** de ton serpent ; une **inscription
  HARDCORE** rouge lumineuse marque le mode. Uniquement en Compétitif ; la
  longueur ne descend jamais sous le minimum. Mémorisé dans `settings.json`.
- Les **pommes dorées** (temporaires) donnent beaucoup de points et rechargent
  le turbo instantanément.
- En option : **traverser les murs**, pommes bonus, **prestige** (un joueur,
  touche **P**).
- **NOUVEAU - Personnaliser** (bouton pinceau tout en haut à droite du setup, ou
  touche **C**) : un menu purement visuel (des « mods » qui ne changent *jamais*
  le jeu) avec des onglets :
  - **Tête** : la **couleur de tête** du serpent - 4 modèles bleu-turquoise (du
    plus bleu au plus turquoise), rouge, orange et une **couleur personnalisée**
    via des curseurs RVB.
  - **Grille (repère)** : superpose une **grille de coordonnées** sur le
    terrain - **numéros de ligne** (bords gauche et droit) et **lettres de
    colonne** (haut/bas). Sur les grands terrains tu vois ainsi tout de suite
    que la pomme en *8a* est sur la même ligne *8* que ta position *8z*. La
    séquence de couleurs (5 modèles + deux couleurs personnalisées A/B) définit
    le thème.
  - **Bannière** : activer/désactiver la bannière de multiplicateur (p. ex. de
    la pomme violette) et régler sa **taille** et son **opacité** - avec aperçu
    en direct.
  Tout est enregistré dans `mem-ngb.json` ; la personnalisation visuelle passe
  par le module `ngb.py`.
- Esthétique : serpent arrondi avec des yeux (tête turquoise par défaut), halo
  de turbo, particules.

**Pong**
- Un joueur contre l'IA, multijoueur = joueur 2 à droite. Jusqu'à 5 points.
- **Mode de déplacement commutable par jeu de commandes** : *Continu* (une
  pression -> continue d'avancer, par défaut) ou *Maintenir* (ne bouge que tant
  que la touche est enfoncée). Basculer : **X** = commandes 1, **N** =
  commandes 2 (mémorisé dans `settings.json`).
- Physique de balle avec accélération et angle selon le point d'impact.

**Air Hockey**
- **Vraie physique 2D** : maillets ronds et palet avec transfert d'impulsion -
  le palet reprend la vitesse du maillet à l'impact ; bandes avec restitution,
  légère friction de glace, buts en ouvertures dans les parois latérales.
- **Contrôle à la souris** en un joueur : le maillet suit la souris (toute
  touche revient au clavier). Clavier : 8 directions, multijoueur = J1 à gauche
  (WASD), J2 à droite (IJKL).
- **IA à trois niveaux** (Facile/Moyen/Difficile) : défend son but, attaque dans
  sa moitié et contourne le palet pour éviter les buts contre son camp.
- **Power-ups** (désactivables) : *XL* (maillet plus grand), *BUT* (le but
  adverse rétrécit), *>>* (maillet plus rapide) - ils appartiennent au dernier
  joueur ayant touché le palet.
- Setup : difficulté, **buts pour gagner** (3/5/7/10), power-ups oui/non
  (enregistré dans `settings.json`). Après chaque but, l'engagement revient à
  celui qui l'a encaissé.
- Esthétique : traînée lumineuse du palet, particules, bouches de but pulsantes,
  indicateurs d'effets.

**Tic-Tac-Toe**
- Setup : difficulté (Facile/Moyen/Difficile) et taille du plateau 3x3..9x9 ;
  longueur gagnante K = 3 (3x3), 4 (4x4), sinon 5.
- **1 joueur** contre l'IA (Difficile sur 3x3 est imbattable) **ou 2 joueurs**
  en local (X contre O, à tour de rôle au clic). Après la fin : Entrée/clic =
  nouvelle manche, **S** = réglages.

**Breakout**
- Types de briques : Normale, **Acier** (indestructible), **Bombe** (explose),
  **Or** (points bonus).
- Power-ups : laser, boule de feu, collante, bouclier, pièce et plus ;
  **multiplicateur de combo**.
- Effets : particules, traînées de balle, screen shake, pop-ups de points,
  nombreux motifs de niveaux.
- Setup : **1/2/3** = difficulté, **Gauche/Droite** = couleur de balle,
  **Haut/Bas** = niveau de départ, **M** = disposition. Jeu : souris/flèches,
  **Espace** lance la balle (tire le laser), **P/Échap** = pause.

**Tetris**
- Gauche/Droite déplace, Haut = pivoter, Bas = soft drop, Action = hard drop.
- Les lignes complètes donnent des points ; le niveau monte toutes les 10
  lignes.
- **Versus** : perd celui dont la pile touche le haut en premier.

**Invaders** – deux modes (sur l'écran de préparation) :
- **Classique** : le bloc d'aliens classique ; ensuite dans le setup :
  **déplacement** (gauche/droite seulement *ou* libre avec WASD) et **visée**
  (toujours vers le haut *ou* vers la **souris** – tu tires alors là où se
  trouve le curseur). Les aliens détruits lâchent parfois des power-ups.
- **Arène (libre)** : déplacement libre dans toutes les directions, les ennemis
  affluent par tous les bords ; on vise dans la direction du mouvement, arme
  avec **1–4**.
En commun : système de niveaux avec **boss** tous les 4 niveaux, quatre armes
(blaster, tir dispersé, tir rapide, laser), power-ups (vie supplémentaire,
bouclier, amélioration d'arme), effets d'explosion, meilleur score.

**Asteroids**
- **Physique d'inertie** : Haut = poussée dans la direction du regard,
  Gauche/Droite = pivoter, le vaisseau continue de dériver (léger
  amortissement) ; tout traverse les bords de l'écran. **Esthétique
  vectorielle** classique avec flamme de propulsion et ciel étoilé ; chaque
  rocher a son propre polygone aléatoire.
- Les rochers se brisent en deux plus petits (3 tailles, **20/50/100 points**),
  **vagues** croissantes avec annonce en bannière.
- **OVNI** (désactivable) : traverse régulièrement l'écran et vise les vaisseaux
  (erreur de visée selon la difficulté) - 200 points pour l'abattre.
- **Power-ups** (désactivables), lâchés par les rochers détruits : bouclier
  **S** (6 s invulnérable), tir **T**riple, tir **R**apide.
- **Hyperespace** (touche Bas) : saut d'urgence vers une position aléatoire avec
  4 s de recharge - et 12 % de risque de s'y écraser.
- 3 vies, réapparition sûre avec clignotement d'invulnérabilité, **vie
  supplémentaire tous les 5000 points** ; particules d'explosion et secousse de
  caméra.
- **Duel coopératif** (multijoueur) : les deux vaisseaux volent en même temps
  avec vies et points séparés - celui qui a le plus de points gagne.
- Setup : difficulté, OVNIs oui/non, power-ups oui/non (dans `settings.json`).

**Pac-Man**
- **Labyrinthe classique 28x31** au look néon avec pastilles, 4 pilules de
  pouvoir, tunnels latéraux et maison des fantômes au centre.
- **Quatre fantômes aux comportements originaux** (IA de case cible) : *Blinky*
  poursuit directement, *Pinky* tend une embuscade (4 cases devant), *Inky*
  utilise un vecteur passant par Blinky, *Clyde* s'écarte de près.
- **Phases scatter/chase** en alternance (les fantômes font demi-tour à chaque
  changement) ; la **pilule de pouvoir** les rend bleus et comestibles (chaîne
  200/400/800/1600), puis les yeux rentrent à la maison.
- Maison des fantômes avec **sortie échelonnée**, **fruits** bonus (par niveau),
  **3 vies**, **vie supplémentaire à 10 000**, système de niveaux (de plus en
  plus rapide), animation de mort, écrans READY/GAME OVER.
- Setup : **difficulté** (Normal/Difficile/Extrême) – vitesse des fantômes et
  durée de frayeur.
- Commandes : **flèches ou WASD**.  Entrée = nouveau, S = setup.

**Flappy Bird**
- **Physique de gravité** : Espace / Haut / W / **clic** fait battre des ailes à
  l'oiseau ; il s'incline selon la vitesse de montée/descente.
- **Paires de tuyaux** sans fin avec un passage (+1 par tuyau) ; des **pièces**
  (bonus) et un power-up **bouclier** (survit à une collision) apparaissent dans
  les passages.
- Les **thèmes jour/nuit** changent avec le score ; nuages à la dérive
  (parallaxe), sol défilant.
- Difficulté (Facile/Normal/Difficile) : taille du passage, vitesse, espacement
  des tuyaux – le passage se resserre un peu quand le score monte.
- **Médailles** (bronze/argent/or/platine) après le game over, animation de
  crash avec secousse de caméra, meilleur score.

**Doodle Jump**
- Le doodler **saute automatiquement** à l'atterrissage ; tu ne diriges que
  gauche/droite (avec inertie), les bords se rejoignent (**wrap-around**) ; la
  caméra monte avec toi.
- **Types de plateformes** : verte (normale), bleue (mobile), marron (se casse),
  blanche (disparaît). Les **ressorts** donnent un super-saut, le **chapeau à
  hélice** te porte un instant automatiquement vers le haut (et rend
  invulnérable).
- **Monstres** : le contact est mortel – mais tu peux les **abattre** avec
  Haut / Espace (points bonus).
- Points = hauteur atteinte ; la difficulté monte avec la hauteur. Meilleur
  score.
- Commandes : gauche/droite = bouger, Haut / Espace = tirer.

**2048** – flèches/WASD font glisser toutes les tuiles ; les nombres égaux
fusionnent.

**Minesweeper**
- Trois niveaux : **Débutant** (9x9, 10 mines), **Avancé** (16x16, 40),
  **Expert** (30x16, 99) - le **meilleur temps par niveau** est enregistré et
  affiché dans le setup.
- Le **premier clic est toujours sûr** (les mines ne sont réparties qu'après,
  la zone 3x3 autour du clic reste libre).
- **Clic gauche** = révéler, **clic droit** = drapeau (en option avec cycle de
  point d'interrogation), **F** = drapeau sous le curseur, **R** = nouvelle
  partie.
- **Chording** : cliquer sur un chiffre satisfait révèle les voisins restants.
- HUD classique : compteur de mines, **smiley cliquable** (étonné/lunettes de
  soleil/mort), chronomètre ; les faux drapeaux sont barrés à la fin, confettis
  à la victoire.
- Points = valeur de base du niveau moins les secondes.

**Sudoku**
- **400 niveaux** : 4 difficultés (Facile/Normal/Difficile/Expert) x 100
  niveaux. Les puzzles sont **générés par graine et à solution unique** - le
  niveau 12 de « Difficile » est le même puzzle sur chaque PC. Les niveaux
  résolus sont enregistrés et cochés dans la sélection.
- **4 modes de jeu** (choisis avant de commencer) avec multiplicateur de
  score : **Classique** (x2,0 - sans aides), **Notes** (x1,5 - + notes au
  crayon), **Confort** (x1,0 - + erreurs en rouge, surlignage des conflits et
  des chiffres identiques, les entrées correctes se verrouillent),
  **Assistant** (x0,7 - + indice, max. 3).
- Chaque entrée est vérifiée immédiatement contre la solution ; avec la
  **limite de 3 erreurs** activée (option du setup), la troisième erreur met
  fin à la partie.
- Commandes : flèches/WASD = case, **1-9** = chiffre (pavé numérique aussi),
  **0/Retour arrière/clic droit** = effacer, **N** = notes, **H** = indice,
  **R** = recommencer le niveau, **Q** = sélection des niveaux ; entièrement
  jouable à la souris (pavé numérique à droite). Après la fin, **A** masque le
  bandeau et affiche la **solution** complète sur le plateau (A à nouveau =
  retour).
- Points = (base de la difficulté - temps - erreurs - indices) x multiplicateur
  du mode.

**Frogger**
- 5 voies de circulation (voitures/camions) et 5 voies de rivière (troncs,
  tortues qui **plongent** aux niveaux supérieurs) ; en haut 5 abris - tous les
  remplir = niveau suivant, tout s'accélère.
- Extras : **mouche bonus** (+200) dans les abris vides, des **crocodiles**
  occupent des abris aux niveaux supérieurs, **barre de temps** par grenouille,
  vie supplémentaire à 10 000.
- 3 difficultés (vitesse, densité du trafic, temps) ; points par nouvelle
  rangée, abri = 50 + bonus de temps, niveau complet = +1000.

**Memory**
- Tailles de plateau **4x4, 6x6, 8x6** ; motifs issus de combinaisons
  forme-couleur, dessinés entièrement avec des primitives ; **animation de
  retournement**, les paires ratées se retournent toutes seules.
- **Solo** : base - 15 par coup - 2 par seconde (min. 100). **Duel** (local) :
  à tour de rôle, une paire trouvée = on rejoue, gagne celui qui a le plus de
  paires.

**Solitaire**
- **5 variantes** sur l'écran de préparation : Klondike (tirage 1/3 en option),
  Spider (1/2/4 couleurs), FreeCell (limite de super-déplacements), Pyramide
  (paires de 13, 2 redistributions) et TriPeaks (chaîne ±1 avec multiplicateur
  de combo).
- **Glisser-déposer** ou clic-clic, **clic droit** = vers la fondation,
  **U** = annulation illimitée, **R** = nouvelle donne, Espace = talon.
- Les cartes sont rendues sans fichiers d'images (`games/cards.py`) ; toutes
  les variantes partagent une liste de meilleurs scores avec des formules
  spécifiques.

**Aim Trainer**
- **Vraie 3D logicielle** (comme le mode 3D de Snake) : réticule fixe au centre
  de l'écran, **visée directe 1:1 à la souris comme dans un shooter** (capture
  du pointeur : le curseur est retenu dans la fenêtre, Échap le libère ;
  sensibilité réglable, yaw illimité, pitch ±60°). Le clic gauche tire
  exactement par le centre, avec flash de bouche, balle traçante et particules
  d'impact.
- **4 modes** : Précision (60 s, 3 balles, bonus de précision), Réflexes (30
  cibles une à une, statistiques de temps de réaction), Cibles mobiles
  (trajectoires + multiplicateur de combo jusqu'à x4) et Chill (sans fin, sans
  pénalité, **E** termine).
- **3 thèmes** (dans le setup, enregistrés) : **Espace** avec sphère d'étoiles,
  un **trou noir à l'anneau lumineux** et une planète (par défaut), arène néon
  avec grille au sol et soleil synthwave, et un stand de tir intérieur.
- La sensibilité se change aussi en pleine partie avec **+/-** ; en plus un
  **motion blur réglable** (0-80 %) pour un look extra chill - les deux sont
  enregistrés.

**Puissance 4**
- Plateau 7x6 avec **animation de chute**, aperçu au survol et ligne gagnante
  pulsante ; souris, flèches ou choix direct **1-7**.
- **3 niveaux d'IA** (minimax avec élagage alpha-bêta) : Facile rate exprès des
  menaces, Moyen bloque avec fiabilité, Difficile planifie en profondeur - ou
  **2 joueurs** en local sur le même appareil.
- Le joueur qui commence change à chaque manche ; le meilleur score compte les
  **victoires contre l'IA** d'une session.

**Duel de tanks**
- Duel 2D en arène : **les tirs rebondissent une fois sur les murs**
  (ricochet) - touche dans les angles (ou toi-même !). Premier à 5 manches avec
  compte à rebours.
- **4 arènes** (Ouverte, Croix, Colonnes, Labyrinthe) ou rotation aléatoire ;
  **power-ups** : tir rapide, bouclier, tir triple.
- **IA à 3 niveaux** - la difficile vise avec anticipation et tire exprès par
  la bande - ou **2 joueurs** sur un clavier (J1 WASD+Espace,
  J2 flèches+Entrée).

**Blackjack**
- Vraies règles de casino : **sabot de 4 jeux**, le croupier reste à 17, le
  **blackjack paie 3:2**, peek du croupier avec as/10 ; **doubler** et **un
  partage** (les as partagés reçoivent une carte chacun).
- **Solde de jetons persistant** : départ avec 500, solde et **record**
  survivent à chaque redémarrage (`mem.json`) ; sous 10 jetons tu en reçois 500
  neufs - le record reste.
- Manipulation par boutons de jetons et touches (**H**it/**S**tand/**D**ouble/
  partager **X**, **1-4** = mise, Entrée = distribuer) avec animations de
  cartes et retournement de la carte cachée.

**Tunnel Racer**
- **Vol 3D dans un tube néon** (rendu logiciel comme l'Aim Trainer) : barres,
  blocs et **diaphragmes en anneau à enfiler**, pièces sur la trajectoire
  idéale.
- **Deux modes** : Sans fin (la vitesse monte jusqu'à un plafond, meilleur
  score) et **30 niveaux à graine** avec arrivée, bonus de temps et progression
  cochée.
- **Pilotage au clavier** (par défaut) ou **pilotage direct à la souris**
  (capture du pointeur, touche **C**) ; en plus un **motion blur réglable**
  (touche **B**, 0-80 %) - tout est enregistré.

**Labyrinthe 3D**
- **Raycaster à la première personne style Wolfenstein** (DDA, brouillard de
  distance, sprites) avec mouselook + WASD, **minicarte** (touche **M**) et
  sortie verte pulsante - ou une **vue 2D de dessus** classique (touche **V**
  dans le setup).
- **50 niveaux à graine** qui ne cessent de grandir ; la sortie se trouve
  toujours au point le plus éloigné, les **orbes** en chemin donnent des points
  bonus.
- Points : 500 par niveau + 100 par orbe + bonus de temps ; les niveaux résolus
  sont cochés et la session s'additionne au meilleur score.

**Reversi**
- **Othello sur 8x8** : pose des pions qui encerclent les rangées adverses et
  retourne tout ce qui est enfermé ; les coups illégaux sont bloqués et un tour
  sans coup possible est **passé automatiquement**.
- **Solo contre l'IA** (3 forces : negamax avec alpha-bêta, pondération de
  position + mobilité) **ou un duel local**, Noir contre Blanc.
- Les cases jouables sont mises en évidence ; joue à la **souris** ou avec le
  cadre de sélection (flèches + Espace/Entrée). Chaque victoire contre l'IA vaut
  un point pour le meilleur score.

**Yams**
- **Classique de dés** : 5 dés, jusqu'à 3 lancers par tour, **garde** les dés un
  à un, puis inscris l'une des **13 catégories** (avec un aperçu des points
  possibles).
- Feuille complète : section haute avec **bonus de 63 (+35)**, brelan/carré,
  full, petite/grande suite, **Yams (50)** et Chance.
- **Solo en course au meilleur total** ou **hotseat à 2 joueurs** avec deux
  feuilles côte à côte ; joue à la souris ou aux touches (Espace, 1-5, flèches,
  Entrée).

**Wordle**
- Devine le **mot de 5 lettres en 6 essais** ; retour coloré (vert/jaune/gris)
  avec un **comptage correct des lettres doubles** et un clavier à l'écran qui se
  colore.
- **Série sans fin** : chaque mot résolu rapporte des points (moins d'essais =
  plus), le premier mot non trouvé met fin à la partie - total = meilleur score.
- **Listes de mots par langue** (A-Z uniquement) ; les essais ne sont pas
  vérifiés dans un dictionnaire. Tape au clavier ou clique les touches à l'écran.

Les meilleurs scores sont enregistrés dans la section `highscores` de
`mem.json` (à côté du code) – avec la langue (section `mem`).

### L'interface

Toute l'interface est dessinée à la main (Tkinter pur + Pygame, sans paquets
supplémentaires) et soignée façon lanceur de jeux moderne :

- **Barre latérale avec liste de jeux** : chaque ligne a son
  **mini-pictogramme** dans la couleur d'accent du jeu, montre le **meilleur
  score actuel (★)** et réagit avec des effets de survol animés en douceur. Le
  jeu en cours reste marqué en couleur ; dans les petites fenêtres la liste
  **défile** à la molette.
- **Carte d'état** en bas à gauche avec **LED d'état** (gris = menu, vert = en
  cours, doré = pause, rouge = game over) et **affichage FPS en direct**.
- **Écran d'accueil** avec lumières aurorales, champ d'étoiles en parallaxe
  avec étoiles filantes, logo flottant avec étincelles en orbite, une **grille
  de jeux cliquable** juste sous le logo (tous les jeux avec effet de survol
  dans leur couleur d'accent) et un **bandeau défilant des meilleurs scores**.
- **Effets partout** : transitions d'écran douces, étincelles à la confirmation
  dans le menu, **pluie de confettis pour un nouveau record** et un vrai
  **flou** derrière la superposition de pause.
- L'**écran de préparation** de chaque jeu apparaît dans sa couleur d'accent et
  affiche le record précédent sous forme de puce.
- **Wiki intégré** (« LamaWiki ») : aide détaillée pour chaque jeu (commandes,
  modes, points, astuces) plus des pages générales - avec **champ de
  recherche**, catégories, articles défilables et puces de touches, dans les
  cinq langues. Accessible via le bouton **« Wiki / Aide »** de la barre
  latérale et depuis l'écran de préparation de chaque jeu (ouvre directement sa
  page).

### Prise en main

- Choisis le jeu avec le bouton du menu de gauche. Ensuite apparaît l'**écran
  de préparation** : choisir **Un joueur** ou **Multijoueur**, aller aux
  **options** ou revenir. Flèches/souris pour choisir, Entrée démarre.
- **ÉCHAP** = pause / reprendre (dans les menus : retour).
- **F11** (ou le bouton « Plein écran oui/non ») = plein écran. L'affichage
  Pygame reste intégré et est agrandi en conservant les proportions (bandes
  noires si le rapport diffère). La fenêtre se redimensionne librement.
- **« Retour au menu »** termine le jeu et enregistre le meilleur score.
- **« Quitter »** ferme proprement Pygame et Tkinter.

### Options, commandes et son

L'écran d'options s'ouvre avec le bouton **« Options / Commandes »** (à gauche)
ou depuis l'écran de préparation. Il est organisé en **trois onglets**
(**Général / Commandes / Apparence** ; changer par clic ou avec la touche Tab) :

- **Général** : **son** oui/non, **volume** et **vibration** (vibration de la
  manette, effective seulement avec une manette branchée) ainsi que
  **résolution auto**, **résolution**, **FPS** et **langue** – chacun avec
  Gauche/Droite.
- **Commandes** : **modèles** (*WASD + Flèches*, *WASD + IJKL*,
  *Flèches + WASD*) et **chaque touche individuelle** des joueurs 1 et 2 est
  réassignable : choisir la ligne, appuyer sur Entrée, appuyer sur la touche
  voulue (Échap annule).
- **Apparence** : choisir le **design de l'interface** – **UI v4** (par
  défaut : un look graphite épuré et plat avec un seul accent indigo) ou
  **UI v3** (l'ancienne interface classique avec ciel étoilé, aurores et
  halos lumineux). Les deux cartes montrent un petit aperçu ; le choix
  s'applique immédiatement à toute l'interface (zone de jeu **et** barre
  latérale) et est enregistré.

Les réglages sont enregistrés durablement dans `settings.json`. En **un
joueur** les deux assignations contrôlent le même personnage (par défaut :
WASD *et* flèches), en **multijoueur** une chacun. Tous les jeux ont des
**effets sonores** (générés procéduralement, sans fichiers supplémentaires)
qui peuvent être coupés globalement.

### Structure du projet

```
install-python.bat  Installation Windows : Python 3.13 + .venv + pygame
start.bat            Script de lancement (Windows)
start.sh             Script de lancement (Linux / macOS / Git Bash)
pyinstall.bat        Build EXE (Windows) : met tout dans builds\PyGameZ.exe
main.py              Interface Tkinter, intégration Pygame, boucle de jeu centrale
game_base.py         Classe de base des jeux (update/draw/handle_event) + InputEvent + assistants
settings.py          Charger/enregistrer les réglages (son/vibration/touches) (JSON)
audio.py             Effets sonores procéduraux + vibration de manette
menu.py              Écrans de langue, de préparation (mode) et d'options (son/commandes)
highscore.py         Charger/enregistrer les meilleurs scores (section dans mem.json)
store.py             Fichier de sauvegarde central mem.json (sections : mem, highscores)
prestige.py          Système de prestige de Snake
competitive.py       Paramètres du mode Compétitif de Snake (niveaux, machine à sous, pommes de pari)
ngb.py               Personnalisation visuelle (« mods ») : couleur de tête + grille + menu (mem-ngb.json)
i18n.py              Moteur de traduction (charge lang/*.json, t("clé"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textes (une clé par texte)
lamawiki/
  lamawiki.py          Wiki intégré (recherche, catégories, rendu d'articles)
  de.json  en.json  fr.json  es.json  pt.json   Contenu du wiki (une page par jeu + pages générales)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py
  trexrunner.py  dame.py  poker.py
```

La langue choisie est enregistrée dans `mem.json` (dans la section `mem`, à
côté de la section `highscores` du même fichier) et chargée automatiquement au
prochain démarrage.

### Notes de plateforme

L'affichage tourne **off-screen** : pygame utilise le pilote vidéo dummy
(`SDL_VIDEODRIVER=dummy`), rend donc dans une surface, et chaque image est
dessinée dans un widget Tkinter. Il n'y a **pas de fenêtre SDL native** qui
pourrait se disputer la taille/position avec Tkinter. La fenêtre se comporte
ainsi partout de la même façon et de manière stable :

- **Windows** : le processus est en plus marqué DPI-aware pour que l'affichage
  reste net sur les écrans mis à l'échelle (125/150/200 %) et ne « tremble »
  pas.
- **Linux/X11 et Wayland** : fonctionne sans cas particuliers (pas de
  `SDL_WINDOWID`).
- **macOS** : fonctionne aussi (avant, la fenêtre intégrée ne s'affichait pas
  du tout ici).

---

### Guide d'installation

Prérequis : **Python 3.9+** (recommandé 3.12 ou 3.13) et **pygame ≥ 2.6**.

#### Windows (recommandé : automatique)

1. Ouvre le dossier du projet et lance **`install-python.bat`** par
   double-clic. Le script
   - vérifie si **Python 3.13** est présent et, sinon, l'installe via
     **winget** (`winget install Python.Python.3.13`),
   - crée l'environnement virtuel **`.venv`**,
   - installe **pygame** depuis `requirements.txt`.
2. Lance ensuite la collection avec **`start.bat`** (double-clic).

> Remarque : si le script indique « pas encore disponible dans cette
> fenêtre », Python vient d'être installé – ouvre simplement **un nouveau
> terminal/fenêtre** et relance `install-python.bat`. Si **winget** n'est pas
> disponible, installe Python 3.13 manuellement depuis
> <https://www.python.org/downloads/> en cochant
> **« Add python.exe to PATH »**.

#### Windows / Linux / macOS (manuel)

```bash
# 1. Vérifier Python (3.9+)
python --version

# 2. Créer et activer un environnement virtuel
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Installer les dépendances
pip install -r requirements.txt
#   ou :  pip install "pygame>=2.6" (ou pygame-ce)
#                                    pip install pygame-ce
# 4. Lancer
python main.py
```

#### Linux / macOS avec start.sh

```bash
# Préparer Python + venv comme ci-dessus (étapes 2 et 3), puis :
chmod +x start.sh      # une fois, si pas encore exécutable
./start.sh
```

Sous Linux, installe au besoin Python via le gestionnaire de paquets, p. ex.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu) ; sous
macOS p. ex. `brew install python`.

#### Utiliser une autre version de Python

`install-python.bat` installe Python 3.13 par défaut. Si tu préfères 3.12 (ou
une autre version), change dans le fichier la ligne `set "PYVER=3.13"` vers la
version voulue et l'ID winget en conséquence (`Python.Python.3.12`).

#### Créer une EXE autonome (Windows)

```bat
pyinstall.bat         :: crée builds\PyGameZ.exe (tout dans un seul fichier)
```

`pyinstall.bat` utilise la `.venv` (et la crée au besoin), installe
automatiquement **PyInstaller** et empaquette le jeu complet - Python,
pygame, tous les jeux, les langues, le wiki et les logos - dans **une seule
`PyGameZ.exe`** dans le dossier **`builds\`**. Le fichier tourne sur
n'importe quel PC Windows sans Python installé et peut être copié librement.
Les réglages et meilleurs scores (`settings.json`, `mem.json`,
`mem-ngb.json`) sont créés à côté de la .exe pendant le jeu.

#### Dépannage

- **`pygame` introuvable** → venv activé ? Répète l'étape 3
  (`pip install -r requirements.txt`).
- **`python` n'est pas reconnu (Windows)** → Python a été installé sans « Add
  to PATH » ; réinstalle en cochant la case, ou utilise `py` au lieu de
  `python`.
- **Pas de son** → vérifie « Son » dans les options ; la vibration ne
  fonctionne qu'avec une manette.
- **Fenêtre/intégration sous Linux** → voir *Notes de plateforme*
  (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ retour en haut / back to top</a></b></div>

---

<a name="-espanol"></a>

## 🇪🇸 Español

Una colección de juegos de escritorio en Python: **Tkinter** aporta la ventana y
el menú, **Pygame** va incrustado como pantalla de juego dentro de la ventana de
Tkinter. Veintinueve juegos con opciones compartidas, controles totalmente
reasignables, récords, efectos de sonido procedurales y, en varios títulos, modo
multijugador. La interfaz es **multilingüe** (alemán / inglés / francés /
español / portugués); el idioma se elige en una **pantalla de bienvenida** en el
primer arranque, que también permite ajustar la **resolución** y el **sonido**
(desactivado por defecto); el español y el portugués están tras el botón
**«Más»**. Todo se puede cambiar en cualquier momento en las opciones.

### Inicio rápido

#### Windows

```bat
install-python.bat    :: una vez: instala Python 3.13 + .venv + pygame
start.bat             :: inicia la colección de juegos
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # arranca con .venv, si no con python3 del sistema
```

`start.bat` / `start.sh` usan automáticamente el entorno virtual `.venv` si
existe, y si no el Python del sistema. Al final del documento hay una guía
detallada paso a paso: **[Guía de instalación](#guía-de-instalación)**.

### Los juegos

| Juego        | Modos           | Descripción breve |
|--------------|-----------------|-------------------|
| **Snake**    | 1 / 2 jugadores | Snake de lujo con vista 2D y 3D, turbo, 6 modos (incl. Competitivo), manzanas doradas y prestigio |
| **Pong**     | 1 / 2 jugadores | El clásico contra la IA o el jugador 2, modo de movimiento conmutable |
| **Air Hockey** | 1 / 2 jugadores | Física 2D con transferencia de impulso, control con ratón, IA y power-ups |
| **Tic-Tac-Toe** | 1 / 2 jugadores | Juego m,n,k de 3x3 a 9x9, tres niveles de IA **o** X contra O en local |
| **Breakout** | 1 jugador       | Rompe-ladrillos con tipos de ladrillo, power-ups, combos y muchos niveles |
| **Tetris**   | 1 / 2 jugadores | Clásico o Versus (dos campos lado a lado) |
| **Invaders** | 1 jugador       | Space Invaders: vacía las oleadas, protege tus vidas |
| **Asteroids** | 1 / 2 jugadores | Física de inercia, oleadas, OVNIs, power-ups, hiperespacio - solo o duelo cooperativo |
| **Pac-Man**  | 1 jugador       | Clon fiel: 4 IAs de fantasmas, píldoras de poder, túneles, frutas, niveles |
| **Flappy Bird** | 1 jugador    | Vuelo con gravedad entre tuberías, monedas, escudo, día/noche, medallas |
| **Doodle Jump** | 1 jugador    | Salto automático hacia arriba, tipos de plataforma, muelles, hélice, monstruos |
| **2048**     | 1 jugador       | Puzle de deslizar números, objetivo: la ficha 2048 |
| **Minesweeper** | 1 jugador    | El clásico con primer clic seguro, chording, smiley y mejores tiempos |
| **Sudoku**      | 1 jugador    | 400 niveles con semilla (4 dificultades x 100), 4 modos de ayuda con multiplicador, notas, pistas, límite de 3 errores |
| **Frogger**     | 1 jugador    | Carretera + río + 5 bahías, mosca de bonus, cocodrilos, límite de tiempo, 3 dificultades |
| **Memory**      | 1 / 2 jugadores | Encuentra parejas en 4x4 hasta 8x6, animación de volteo, solo o duelo |
| **Solitario**   | 1 jugador    | 5 variantes (Klondike, Spider, FreeCell, Pirámide, TriPeaks) con arrastrar y soltar y deshacer |
| **Aim Trainer** | 1 jugador    | Tiro al blanco 3D relajado: el ratón dirige la cámara, 4 modos (precisión/reflejos/móviles/chill), 3 temas incl. un agujero negro |
| **Cuatro en raya** | 1 / 2 jugadores | El clásico con animación de caída: 3 niveles de IA (minimax) o duelo local |
| **Duelo de tanques** | 1 / 2 jugadores | Duelo 2D en arena con disparos con rebote, power-ups, 4 arenas, IA con 3 niveles |
| **Blackjack**    | 1 jugador    | Blackjack de casino con zapato de 4 barajas, doblar/dividir, blackjack 3:2 y saldo de fichas persistente |
| **Tunnel Racer** | 1 jugador    | Vuelo 3D por un tubo de neón: modo sin fin + 30 niveles, control por teclas o ratón, motion blur |
| **Laberinto 3D** | 1 jugador    | Raycaster en primera persona (estilo Wolfenstein) con 50 niveles con semilla, orbes, minimapa - o vista cenital 2D |
| **Reversi**      | 1 / 2 jugadores | Othello en 8x8: atrapar y voltear fichas, 3 fuerzas de IA (minimax) o un duelo local |
| **Yahtzee**      | 1 / 2 jugadores | Clásico de dados con 13 categorías, bono superior y Yahtzee; carrera por el récord o hotseat a 2 |
| **Wordle**       | 1 jugador    | Adivina la palabra de 5 letras en 6 intentos, racha sin fin, pistas de color, 5 idiomas |
| **T-Rex Runner** | 1 jugador    | Carrera infinita por el desierto: salto variable, agacharse, cactus y pterodáctilos, ciclo día/noche, velocidad creciente, 3 dificultades |
| **Damas**        | 1 / 2 jugadores | 3 reglamentos a elegir (alemanas 8×8, internacionales 10×10, checkers), captura obligatoria y dama voladora, 3 fuerzas de IA (minimax) o duelo local |
| **Póker**        | 1 jugador    | 3 variantes a elegir: Texas Hold'em contra la IA, 5 Card Draw y Video Poker; rondas de apuestas, ciegas, cuenta de fichas persistente |
| **Puzle deslizante** | 1 jugador  | Puzle-15 en 3x3/4x4/5x5: desliza las fichas numeradas al hueco, control con ratón o flechas, puntos por movimientos y tiempo |
| **Mastermind**       | 1 jugador  | Descifra el código de color secreto (3 modos: 4×6, clásico, 5×8), fichas de pista negras/blancas, racha sin fin |
| **Bubble Shooter**   | 1 jugador  | Clon de Puzzle Bobble: dispara colores iguales en grupos de tres, rebotes en las paredes, racimos que caen, 3 dificultades |
| **Hangman**          | 1 jugador  | Adivina la palabra antes de completar la horca; teclado en pantalla, listas de palabras por idioma, 3 modos de longitud, racha sin fin |
| **Block Jump**       | 1 jugador  | Plataformas 3D estilo Minecraft: mundo de bloques (vóxeles) con escaleras, vallas y bloques-resorte, cámara 1ª/3ª persona, desenfoque, niveles generados |

**El multijugador (2 jugadores en local)** está disponible en **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Cuatro en raya**, **Duelo de tanques**,
**Reversi**, **Yahtzee** y **Damas**.
El modo se elige directamente en la pantalla previa (*Un jugador / Multijugador*).

#### Detalles por juego

**Snake**
- **NUEVO - Vista 3D** (tecla **V** en el setup o clic en *Vista*): el tablero se
  renderiza como escena 3D en tiempo real - una **cámara de persecución** flota
  tras la serpiente y se dirige **relativo a la vista** (izq/der = girar, dos
  pulsaciones rápidas = media vuelta). Con niebla de distancia, cielo estrellado,
  suelo de ajedrez, bandas, cristales de comida giratorios, partículas 3D y
  sacudida de cámara al chocar; tras el game over la cámara orbita la serpiente.
  El turbo amplía el campo de visión. En 3D: *Clásico* y *Obstáculos* (allí los
  muros siempre son fijos, 3D solo en un jugador). La vista se guarda en
  `settings.json`.
- **NUEVO - Opciones de cámara 3D** (en el setup 3D, la fila *Cámara 3D /
  Smooth-Shake* o tecla **K**): menú propio con **Smooth-Shake** (cámara más
  suave, mucho menos traqueteo), **campo de visión (FOV)** y **altura de cámara**
  ajustables y un interruptor de **sacudida al girar**. Todo se guarda en
  `settings.json`.
- **Turbo**: **mantén** la tecla de turbo = velocidad doble, consume resistencia
  (barra); si se vacía, el turbo se apaga y se recarga. Estándar J1 =
  Espacio/Shift izq., J2 = Enter/Shift der.
- **6 modos** (en el setup): *Clásico*, *Speed-Rush* (más rápido con cada
  manzana), *Obstáculos* (bloques mortales), *Portales* (pares de teletransporte),
  *Contrarreloj* (60 segundos, tantas manzanas como puedas) y *Competitivo* (ver abajo).
- **NUEVO - Competitivo** (un jugador): modo sin fin con **subida de nivel** -
  empiezas con exactamente **una** manzana; cuantas más recojas en total, más alto
  tu **nivel**, que añade otra manzana simultánea al campo y sube el multiplicador.
  Las **manzanas azules** abren una **tragaperras**: apuestas tu longitud, el
  resultado la multiplica o la reduce y hace aparecer **manzanas extra** un rato
  (jackpot con tres símbolos iguales). Las **manzanas lilas** (apuesta) ponen en
  juego una parte de tu **tamaño** y multiplican esa parte al azar, el resto queda
  a salvo (nuevo tamaño = tamaño·(1-p) + tamaño·p·factor): **normal** 50 % fijo
  con **x0.5 .. x1.5**, en **HARDCORE** más arriesgado con **75-90 %** y
  **x0.25 .. x2.25**. El **tamaño** aparece como **decimal arriba a la izquierda**
  y se arrastra con exactitud, así las siguientes apuestas parten de él. Hay
  **15 niveles** (multiplicador hasta x16, hasta 16 manzanas a la vez); los
  niveles viven en `games/levels/snake-comp.json` y se amplían sin tocar código,
  el resto del ajuste fino está en `competitive.py`.
- **NUEVO - HARDCORE** (interruptor en el setup de Competitivo, tecla **H**): cada
  **turbo consume longitud** de tu serpiente; un **letrero HARDCORE** rojo marca
  el modo. Solo en Competitivo; la longitud nunca baja del mínimo. Se guarda en
  `settings.json`.
- Las **manzanas doradas** (temporales) dan muchos puntos y recargan el turbo.
- Opcional: **atravesar muros**, manzanas extra, **prestigio** (un jugador, tecla **P**).
- **NUEVO - Personalizar** (botón del pincel arriba a la derecha del setup, o
  tecla **C**): un menú solo visual ("mods" que *nunca* cambian el juego) con
  pestañas:
  - **Cabeza**: el **color de la cabeza** - 4 plantillas azul-turquesa, rojo,
    naranja y un **color propio** con deslizadores RGB.
  - **Rejilla (guía)**: superpone una **cuadrícula de coordenadas** - **números
    de fila** (bordes izquierdo y derecho) y **letras de columna** (arriba/abajo).
    Así en campos grandes ves al instante que la manzana en *8a* está en la misma
    fila *8* que tu posición *8z*. La secuencia de colores (5 plantillas + dos
    colores propios A/B) define el tema.
  - **Banner**: activar/desactivar el banner de multiplicador (p. ej. de la
    manzana lila) y ajustar **tamaño** y **opacidad** - con vista previa en vivo.
  Todo se guarda en `mem-ngb.json`; la personalización visual pasa por el módulo
  `ngb.py`.
- Estética: serpiente redondeada con ojos (cabeza turquesa por defecto), brillo
  de turbo, partículas.

**Pong**
- Un jugador contra la IA, multijugador = jugador 2 a la derecha. Hasta 5 puntos.
- **Modo de movimiento conmutable por control**: *Continuo* (pulsa una vez ->
  sigue moviéndose, estándar) o *Mantener* (solo mientras pulsas).
  Cambiar: **X** = control 1, **N** = control 2 (se guarda en `settings.json`).
- Física de bola con aceleración y ángulo según el punto de impacto.

**Air Hockey**
- **Física 2D real**: mazos redondos y puck con transferencia de impulso - el
  puck hereda la velocidad del mazo al golpear; bandas con restitución, ligera
  fricción de hielo, porterías como huecos en las paredes laterales.
- **Control con ratón** en un jugador: el mazo sigue al ratón (cualquier tecla
  vuelve al teclado). Teclado: 8 direcciones, multijugador = J1 izquierda (WASD),
  J2 derecha (IJKL).
- **IA con tres niveles** (Fácil/Medio/Difícil): defiende su portería, ataca en
  su mitad y rodea el puck para evitar goles en propia.
- **Power-ups** (desactivables): *XL* (mazo más grande), *GOL* (la portería rival
  encoge), *>>* (mazo más rápido) - pertenecen al último jugador que tocó el puck.
- Setup: dificultad, **goles para ganar** (3/5/7/10), power-ups sí/no (guardado
  en `settings.json`). Tras cada gol saca quien lo encajó.
- Estética: estela del puck, partículas, bocas de portería pulsantes, indicadores.

**Tic-Tac-Toe**
- Setup: dificultad (Fácil/Medio/Difícil) y tamaño del tablero 3x3..9x9; longitud
  ganadora K = 3 (3x3), 4 (4x4), si no 5.
- **1 jugador** contra la IA (Difícil en 3x3 es imbatible) **o 2 jugadores** en
  local (X contra O, por turnos con clic). Tras el final: Enter/clic = nueva
  ronda, **S** = ajustes.

**Breakout**
- Tipos de ladrillo: Normal, **Acero** (indestructible), **Bomba** (explota),
  **Oro** (puntos extra).
- Power-ups: láser, bola de fuego, pegajosa, escudo, moneda y más;
  **multiplicador de combo**.
- Efectos: partículas, estelas, screen shake, pop-ups de puntos, muchos patrones.
- Setup: **1/2/3** = dificultad, **Izq/Der** = color de bola, **Arriba/Abajo** =
  nivel inicial, **M** = diseño. Juego: ratón/flechas, **Espacio** lanza la bola
  (dispara láser), **P/Esc** = pausa.

**Tetris**
- Izq/Der mueve, Arriba = girar, Abajo = soft drop, Acción = hard drop.
- Las líneas completas dan puntos; cada 10 líneas sube el nivel.
- **Versus**: pierde aquel cuya pila toca antes arriba.

**Invaders** – dos modos (en la pantalla previa):
- **Clásico**: bloque de aliens clásico; luego en el setup: **movimiento** (solo
  izq/der *o* libre con WASD) y **apuntado** (siempre arriba *o* hacia el
  **ratón** – disparas adonde esté el cursor). Los aliens destruidos a veces
  sueltan power-ups.
- **Arena (libre)**: movimiento libre, los enemigos entran por todos los bordes;
  se apunta en la dirección del movimiento, arma con **1–4**.
En común: sistema de niveles con **jefe** cada 4.º nivel, cuatro armas (bláster,
disparo múltiple, fuego rápido, láser), power-ups (vida extra, escudo, mejora de
arma), efectos de explosión, récord.

**Asteroids**
- **Física de inercia**: Arriba = propulsión en la dirección de la vista,
  Izq/Der = girar, la nave sigue derivando (leve amortiguación); todo cruza los
  bordes de la pantalla. **Estética vectorial** clásica con llama de propulsión
  y cielo estrellado; cada roca tiene su propio polígono aleatorio.
- Las rocas se parten en dos más pequeñas (3 tamaños, **20/50/100 puntos**),
  **oleadas** crecientes con anuncio de banner.
- **OVNI** (desactivable): cruza la pantalla y apunta a las naves (error de
  puntería según dificultad) - 200 puntos por derribarlo.
- **Power-ups** (desactivables), caen de las rocas destruidas: **E**scudo (6 s
  invulnerable), disparo **T**riple, fuego **R**ápido.
- **Hiperespacio** (tecla Abajo): salto de emergencia a una posición aleatoria
  con 4 s de recarga - y 12 % de riesgo de estrellarte.
- 3 vidas, reaparición segura con parpadeo de invulnerabilidad, **vida extra
  cada 5000 puntos**; partículas de explosión y sacudida de cámara.
- **Duelo cooperativo** (multijugador): ambas naves vuelan a la vez con vidas y
  puntos separados - gana quien tenga más puntos.
- Setup: dificultad, OVNIs sí/no, power-ups sí/no (en `settings.json`).

**Pac-Man**
- **Laberinto clásico de 28x31** con estética neón, píldoras, 4 píldoras de
  poder, túneles laterales y casa de fantasmas central.
- **Cuatro fantasmas con los comportamientos originales** (IA de casilla
  objetivo): *Blinky* persigue directo, *Pinky* embosca (4 casillas por delante),
  *Inky* usa un vector a través de Blinky, *Clyde* se aparta de cerca.
- **Fases scatter/chase** alternas (los fantasmas dan media vuelta en cada
  cambio); la **píldora de poder** los vuelve azules y comestibles (cadena
  200/400/800/1600), luego los ojos vuelven a casa.
- Casa de fantasmas con **salida escalonada**, **frutas** de bonus (por nivel),
  **3 vidas**, **vida extra a los 10.000**, sistema de niveles (más rápido),
  animación de muerte, pantallas READY/GAME OVER.
- Setup: **dificultad** (Normal/Difícil/Extremo) – velocidad de fantasmas y
  tiempo de miedo.
- Controles: **flechas o WASD**.  Enter = nuevo, S = setup.

**Flappy Bird**
- **Física de gravedad**: Espacio / Arriba / W / **clic** hace aletear al pájaro;
  se inclina según el ritmo de subida/bajada.
- **Pares de tuberías** sin fin con hueco (+1 por tubería); **monedas** (bonus)
  y power-up de **escudo** (sobrevive una colisión) aparecen en los huecos.
- **Temas de día/noche** cambian con la puntuación; nubes a la deriva (parallax),
  suelo en desplazamiento.
- Dificultad (Fácil/Normal/Difícil): tamaño del hueco, velocidad, separación –
  el hueco se estrecha al subir la puntuación.
- **Medallas** (bronce/plata/oro/platino) tras el game over, animación de choque
  con sacudida de cámara, récord.

**Doodle Jump**
- El doodler **salta automáticamente** al aterrizar; solo diriges izq/der (con
  inercia), los bordes envuelven (**wrap-around**); la cámara sube contigo.
- **Tipos de plataforma**: verde (normal), azul (móvil), marrón (se rompe),
  blanca (desaparece). **Muelles** dan un supersalto, el **gorro-hélice** te
  lleva arriba un momento (e invulnerable).
- **Monstruos**: el contacto es mortal – pero puedes **derribarlos** con
  Arriba / Espacio (puntos extra).
- Puntos = altura alcanzada; la dificultad sube con la altura. Récord.
- Controles: izq/der = mover, Arriba / Espacio = disparar.

**2048** – flechas/WASD deslizan todas las fichas; los números iguales se fusionan.

**Minesweeper**
- Tres niveles: **Principiante** (9x9, 10 minas), **Avanzado** (16x16, 40),
  **Experto** (30x16, 99) - el **mejor tiempo por nivel** se guarda y se muestra
  en el setup.
- El **primer clic siempre es seguro** (las minas se reparten después, el área
  3x3 alrededor queda libre).
- **Clic izquierdo** = destapar, **clic derecho** = bandera (opcional con ciclo
  de interrogante), **F** = bandera bajo el cursor, **R** = nuevo.
- **Chording**: clic en un número completado destapa el resto de vecinos.
- HUD clásico: contador de minas, **smiley clicable** (sorprendido/gafas de
  sol/muerto), cronómetro; las banderas falsas se tachan al final, confeti al
  ganar.
- Puntos = valor base del nivel menos segundos.

**Sudoku**
- **400 niveles**: 4 dificultades (Fácil/Normal/Difícil/Experto) x 100 niveles.
  Los puzles se **generan por semilla y tienen solución única** - el nivel 12 de
  "Difícil" es el mismo puzle en cualquier PC. Los resueltos se guardan y se
  marcan en la selección.
- **4 modos de juego** (antes de empezar) con multiplicador: **Clásico** (x2,0 -
  sin ayudas), **Notas** (x1,5 - + notas a lápiz), **Confort** (x1,0 - + errores
  en rojo, resalte de conflictos y de dígitos iguales, entradas correctas se
  fijan), **Asistente** (x0,7 - + pista, máx. 3).
- Cada entrada se comprueba al momento contra la solución; con el **límite de 3
  errores** activo (opción del setup) el tercer error acaba la partida.
- Controles: flechas/WASD = celda, **1-9** = dígito (también teclado numérico),
  **0/Backspace/clic derecho** = borrar, **N** = notas, **H** = pista,
  **R** = reiniciar nivel, **Q** = selección; jugable por completo con el ratón
  (panel numérico a la derecha). Tras el final, **A** oculta el cartel y muestra
  la **solución** completa (A de nuevo = volver).
- Puntos = (base del nivel - tiempo - errores - pistas) x multiplicador del modo.

**Frogger**
- 5 carriles de tráfico (coches/camiones) y 5 vías de río (troncos, tortugas que
  **se sumergen** en niveles altos); arriba 5 bahías - llenarlas todas = siguiente
  nivel, todo se acelera.
- Extras: **mosca de bonus** (+200) en bahías vacías, **cocodrilos** ocupan
  bahías en niveles altos, **barra de tiempo** por rana, vida extra a los 10 000.
- 3 dificultades (velocidad, densidad del tráfico, tiempo); puntos por fila
  nueva, bahía = 50 + bonus de tiempo, nivel completo = +1000.

**Memory**
- Tamaños de tablero **4x4, 6x6, 8x6**; motivos de combinaciones forma-color,
  dibujados por completo con primitivas; **animación de volteo**, los fallos se
  voltean solos.
- **Solo**: base - 15 por movimiento - 2 por segundo (mín. 100). **Duelo**
  (local): por turnos, acierto = repites, gana quien más parejas tenga.

**Solitario**
- **5 variantes** en la pantalla previa: Klondike (robar 1/3 como opción), Spider
  (1/2/4 palos), FreeCell (límite de supermovimientos), Pirámide (parejas de 13,
  2 redeals) y TriPeaks (cadena ±1 con multiplicador de combo).
- **Arrastrar y soltar** o clic-clic, **clic derecho** = a la fundación,
  **U** = deshacer ilimitado, **R** = mano nueva, Espacio = mazo.
- Las cartas se renderizan sin archivos de imagen (`games/cards.py`); todas las
  variantes comparten una lista de récords con fórmulas específicas.

**Aim Trainer**
- **3D por software real** (como el modo 3D de Snake): mira fija en el centro,
  **control de ratón directo 1:1 como en un shooter** (captura de puntero: el
  cursor queda retenido en la ventana, Esc lo libera; sensibilidad ajustable,
  yaw ilimitado, pitch ±60°). El clic izquierdo dispara exacto por el centro,
  con fogonazo, trazadora y partículas de impacto.
- **4 modos**: Precisión (60 s, 3 esferas, bonus de precisión), Reflejos (30
  objetivos de uno en uno, estadística de reacción), Objetivos móviles
  (trayectorias + multiplicador de combo hasta x4) y Chill (sin fin, sin
  castigo, **E** termina).
- **3 temas** (en el setup, guardados): **Espacio** con esfera de estrellas, un
  **agujero negro con anillo brillante** y un planeta (estándar), arena neón con
  rejilla en el suelo y sol synthwave, y una galería de tiro interior.
- La sensibilidad se cambia también en plena partida con **+/-**; además un
  **motion blur ajustable** (0-80 %) para una estética extra chill - ambos se
  guardan.

**Cuatro en raya**
- Tablero 7x6 con **animación de caída**, vista previa al pasar el ratón y línea
  ganadora pulsante; ratón, flechas o selección directa **1-7**.
- **3 niveles de IA** (minimax con poda alfa-beta): Fácil pasa por alto amenazas
  a propósito, Medio bloquea con fiabilidad, Difícil planifica en profundidad -
  o **2 jugadores** en local en el mismo equipo.
- Tras cada ronda cambia quien empieza; el récord cuenta las **victorias contra
  la IA** de una sesión.

**Duelo de tanques**
- Duelo 2D en arena: **los disparos rebotan una vez en las paredes** (ricochet) -
  acierta por la esquina (¡o a ti mismo!). Al mejor de 5 rondas con cuenta atrás.
- **4 arenas** (Abierta, Cruz, Columnas, Laberinto) o rotación aleatoria;
  **power-ups**: fuego rápido, escudo, disparo triple.
- **IA con 3 niveles** - la difícil apunta con anticipación y dispara adrede con
  rebote - o **2 jugadores** en un teclado (J1 WASD+Espacio, J2 flechas+Enter).

**Blackjack**
- Reglas de casino reales: **zapato de 4 barajas**, el crupier se planta en 17,
  el **blackjack paga 3:2**, peek del crupier con as/10; **doblar** y **una
  división** (los ases divididos reciben una carta cada uno).
- **Saldo de fichas persistente**: empiezas con 500, saldo y **récord**
  sobreviven a cada reinicio (`mem.json`); con menos de 10 fichas recibes 500
  nuevas - el récord se queda.
- Manejo con botones de fichas y teclas (**H**it/**S**tand/**D**ouble/dividir
  **X**, **1-4** = apuesta, Enter = repartir) con animaciones de cartas y volteo
  de la carta tapada.

**Tunnel Racer**
- **Vuelo 3D por un tubo de neón** (renderizador por software como el Aim
  Trainer): barras, bloques y **diafragmas de anillo para enhebrar**, monedas en
  la línea ideal.
- **Dos modos**: Sin fin (la velocidad sube hasta un tope, récord) y **30 niveles
  con semilla** con meta, bonus de tiempo y progreso marcado.
- **Control por teclas** (estándar) o **control directo con ratón** (captura de
  puntero, tecla **C**); además **motion blur ajustable** (tecla **B**, 0-80 %) -
  todo se guarda.

**Laberinto 3D**
- **Raycaster en primera persona estilo Wolfenstein** (DDA, niebla de distancia,
  sprites) con mouselook + WASD, **minimapa** (tecla **M**) y salida verde
  pulsante - o una **vista cenital 2D** clásica (tecla **V** en el setup).
- **50 niveles con semilla** que van creciendo; la salida siempre está en el
  punto más alejado, los **orbes** del camino dan puntos extra.
- Puntos: 500 por nivel + 100 por orbe + bonus de tiempo; los niveles resueltos
  se marcan y la sesión se suma al récord.

**Reversi**
- **Othello en 8x8**: coloca fichas que atrapen las filas rivales y voltea todo
  lo encerrado; los movimientos ilegales están bloqueados y un turno sin jugada
  se **pasa automáticamente**.
- **Un jugador contra la IA** (3 fuerzas: negamax con alfa-beta, ponderación de
  posición + movilidad) **o un duelo local**, Negras contra Blancas.
- Las casillas válidas se resaltan; juega con el **ratón** o con el marco de
  selección (flechas + Espacio/Enter). Cada victoria contra la IA suma un punto
  al récord.

**Yahtzee**
- **Clásico de dados**: 5 dados, hasta 3 tiradas por turno, **retén** los dados
  uno a uno, luego anota una de las **13 categorías** (con vista previa de los
  puntos posibles).
- Hoja completa: sección superior con **bono de 63 (+35)**, trío/póker, full,
  escalera menor/mayor, **Yahtzee (50)** y Suerte.
- **Un jugador como carrera por el mayor total** o **hotseat a 2 jugadores** con
  dos hojas en paralelo; juega con ratón o teclas (Espacio, 1-5, flechas, Enter).

**Wordle**
- Adivina la **palabra de 5 letras en 6 intentos**; respuesta de color
  (verde/amarillo/gris) con **conteo correcto de letras repetidas** y un teclado
  en pantalla que se colorea.
- **Racha sin fin**: cada palabra resuelta da puntos (menos intentos = más), la
  primera palabra no resuelta termina la partida - total = récord.
- **Listas de palabras por idioma** (solo A-Z); los intentos no se comprueban
  con un diccionario. Escribe con el teclado o pulsa las teclas en pantalla.

Los récords se guardan en la sección `highscores` de `mem.json` (junto al
código) – junto con el idioma (sección `mem`).

### La interfaz

Toda la interfaz está dibujada a mano (Tkinter puro + Pygame, sin paquetes
extra) y pulida con aspecto de lanzador moderno:

- **Barra lateral con lista de juegos**: cada fila tiene su **mini-pictograma**
  en el color de acento del juego, muestra el **récord actual (★)** y reacciona
  con efectos hover suavemente animados. El juego en curso queda marcado en
  color; en ventanas pequeñas la lista **se desplaza** con la rueda.
- **Tarjeta de estado** abajo a la izquierda con **LED de estado** (gris = menú,
  verde = en marcha, dorado = pausa, rojo = game over) e **indicador de FPS en
  vivo**.
- **Pantalla de inicio** con luces aurora, campo de estrellas con parallax y
  estrellas fugaces, logo flotante con chispas en órbita, una **cuadrícula de
  juegos clicable** justo bajo el logo (todos los juegos con efecto hover en su
  color) y una **cinta de récords**.
- **Efectos por todas partes**: transiciones suaves entre pantallas, chispas al
  confirmar en el menú, **lluvia de confeti con un nuevo récord** y un
  **desenfoque real** tras la superposición de pausa.
- La **pantalla previa** de cada juego aparece en su color de acento y muestra
  el récord anterior como chip.
- **Wiki integrado** ("LamaWiki"): ayuda detallada de cada juego (controles,
  modos, puntos, consejos) más páginas generales - con **buscador**, categorías,
  artículos desplazables y chips de teclas, en los cinco idiomas. Accesible por
  el botón **«Wiki / Ayuda»** de la barra lateral y desde la pantalla previa de
  cada juego (abre directamente su página).

### Manejo

- Elige el juego con el botón del menú izquierdo. Después aparece la **pantalla
  previa**: elegir **Un jugador** o **Multijugador**, ir a las **opciones** o
  volver. Flechas/ratón para elegir, Enter empieza.
- **ESC** = pausa / continuar (en menús: volver).
- **F11** (o el botón «Pantalla completa sí/no») = pantalla completa. La pantalla
  de Pygame sigue incrustada y se escala conservando la proporción (bandas negras
  si la proporción difiere). La ventana se puede redimensionar libremente.
- **«Volver al menú»** termina el juego y guarda el récord.
- **«Salir»** cierra Pygame y Tkinter limpiamente.

### Opciones, controles y sonido

La pantalla de opciones se abre con el botón **«Opciones / Controles»** (a la
izquierda) o desde la pantalla previa. Está organizada en **tres pestañas**
(**General / Controles / Apariencia**; se cambia con clic o con la tecla Tab):

- **General**: **sonido** sí/no, **volumen** y **vibración** (vibración del
  gamepad, solo efectiva con mando conectado) además de **resolución
  automática**, **resolución**, **FPS** e **idioma** – cada uno con Izq/Der.
- **Controles**: **plantillas** (*WASD + Flechas*, *WASD + IJKL*,
  *Flechas + WASD*) y **cada tecla individual** de los jugadores 1 y 2 es
  reasignable: elegir fila, pulsar Enter, pulsar la tecla deseada (Esc
  cancela).
- **Apariencia**: elegir el **diseño de la interfaz** – **UI v4**
  (predeterminado: un look grafito limpio y plano con un solo acento índigo) o
  **UI v3** (la interfaz clásica anterior con cielo estrellado, auroras y
  brillos). Ambas tarjetas muestran una pequeña vista previa; la elección se
  aplica al instante a toda la interfaz (área de juego **y** barra lateral) y
  se guarda.

Los ajustes se guardan de forma permanente en `settings.json`. En **un jugador**
ambas asignaciones controlan la misma figura (estándar: WASD *y* flechas), en
**multijugador** una cada uno. Todos los juegos tienen **efectos de sonido**
(generados por procedimientos, sin archivos extra) que se pueden silenciar
globalmente.

### Estructura del proyecto

```
install-python.bat  Instalación en Windows: Python 3.13 + .venv + pygame
start.bat            Script de arranque (Windows)
start.sh             Script de arranque (Linux / macOS / Git Bash)
pyinstall.bat        Build de EXE (Windows): lo empaqueta todo en builds\PyGameZ.exe
main.py              Interfaz Tkinter, incrustación de Pygame, bucle central
game_base.py         Clase base de juego (update/draw/handle_event) + InputEvent + ayudas
settings.py          Cargar/guardar ajustes (sonido/vibración/teclas) (JSON)
audio.py             Efectos de sonido procedurales + vibración de gamepad
menu.py              Pantallas de idioma, previa (modo) y opciones (sonido/controles)
highscore.py         Cargar/guardar récords (sección en mem.json)
store.py             Archivo central mem.json (secciones: mem, highscores)
prestige.py          Sistema de prestigio de Snake
competitive.py       Parámetros del modo Competitivo de Snake (niveles, tragaperras, manzanas de apuesta)
ngb.py               Personalización visual ("mods"): color de cabeza + rejilla + menú (mem-ngb.json)
i18n.py              Motor de traducción (carga lang/*.json, t("clave"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textos (una clave por texto)
lamawiki/
  lamawiki.py          Wiki integrado (búsqueda, categorías, renderizador)
  de.json  en.json  fr.json  es.json  pt.json   Contenido del wiki (una página por juego + generales)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py
  trexrunner.py  dame.py  poker.py
```

El idioma elegido se guarda en `mem.json` (en la sección `mem`, junto a la
sección `highscores` del mismo archivo) y se carga automáticamente en el
siguiente arranque.

### Notas de plataforma

La pantalla funciona **off-screen**: pygame usa el controlador de vídeo dummy
(`SDL_VIDEODRIVER=dummy`), renderiza en una superficie y cada fotograma se
dibuja como imagen en un widget de Tkinter. **No hay ventana SDL nativa** que
pelee con Tkinter por tamaño/posición. Así la ventana se comporta igual y de
forma estable en todas partes:

- **Windows**: el proceso se marca además como DPI-aware para que la imagen sea
  nítida en pantallas escaladas (125/150/200 %) y no "tiemble".
- **Linux/X11 y Wayland**: funciona sin casos especiales (sin `SDL_WINDOWID`).
- **macOS**: también funciona (antes la ventana incrustada ni se mostraba aquí).

---

### Guía de instalación

Requisito: **Python 3.9+** (recomendado 3.12 o 3.13) y **pygame ≥ 2.6**.

#### Windows (recomendado: automático)

1. Abre la carpeta del proyecto y ejecuta **`install-python.bat`** con doble
   clic. El script
   - comprueba si hay **Python 3.13** y, si no, lo instala con
     **winget** (`winget install Python.Python.3.13`),
   - crea el entorno virtual **`.venv`**,
   - instala **pygame** desde `requirements.txt`.
2. Después inicia la colección con **`start.bat`** (doble clic).

> Nota: si el script dice que "aún no está disponible en esta ventana", Python
> se acaba de instalar – abre **una terminal/ventana nueva** y ejecuta
> `install-python.bat` otra vez. Si no hay **winget**, instala Python 3.13 a
> mano desde <https://www.python.org/downloads/> marcando
> **"Add python.exe to PATH"**.

#### Windows / Linux / macOS (manual)

```bash
# 1. Comprobar Python (3.9+)
python --version

# 2. Crear y activar un entorno virtual
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Instalar dependencias
pip install -r requirements.txt
#   o:  pip install "pygame>=2.6" (o pygame-ce)
#                                  pip install pygame-ce
# 4. Iniciar
python main.py
```

#### Linux / macOS con start.sh

```bash
# Preparar Python + venv como arriba (pasos 2 y 3), luego:
chmod +x start.sh      # una vez, si aún no es ejecutable
./start.sh
```

En Linux, si hace falta, instala Python con el gestor de paquetes, p. ej.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); en macOS
p. ej. `brew install python`.

#### Usar otra versión de Python

`install-python.bat` instala Python 3.13 por defecto. Si prefieres 3.12 (u otra
versión), cambia en el archivo la línea `set "PYVER=3.13"` a la versión deseada
y el ID de winget en consecuencia (`Python.Python.3.12`).

#### Crear un EXE independiente (Windows)

```bat
pyinstall.bat         :: crea builds\PyGameZ.exe (todo en un solo archivo)
```

`pyinstall.bat` usa la `.venv` (y la crea si hace falta), instala
**PyInstaller** automáticamente y empaqueta el juego completo - Python,
pygame, todos los juegos, los idiomas, el wiki y los logos - en **un único
`PyGameZ.exe`** dentro de la carpeta **`builds\`**. El archivo funciona en
cualquier PC con Windows sin Python instalado y se puede copiar libremente.
Los ajustes y récords (`settings.json`, `mem.json`, `mem-ngb.json`) se crean
junto al .exe al jugar.

#### Solución de problemas

- **No se encuentra `pygame`** → ¿venv activado? Repite el paso 3
  (`pip install -r requirements.txt`).
- **`python` no se reconoce (Windows)** → Python se instaló sin "Add to PATH";
  reinstala marcando la casilla, o usa `py` en vez de `python`.
- **Sin sonido** → revisa "Sonido" en las opciones; la vibración solo funciona
  con mando.
- **Ventana/incrustación en Linux** → ver *Notas de plataforma* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ volver arriba / back to top</a></b></div>

---

<a name="-portugues"></a>

## 🇵🇹 Português

Uma coleção de jogos de desktop em Python: o **Tkinter** fornece a janela e o
menu, o **Pygame** é incorporado como ecrã de jogo dentro da janela do Tkinter.
Vinte e nove jogos com opções partilhadas, controlos totalmente reatribuíveis,
recordes, efeitos sonoros procedurais e, em vários títulos, modo multijogador.
A interface é **multilingue** (alemão / inglês / francês / espanhol /
português); o idioma escolhe-se num **ecrã de boas-vindas** no primeiro arranque,
que também permite definir a **resolução** e o **som** (desligado por omissão); o
espanhol e o português estão atrás do botão **«Mais»**. Tudo pode ser mudado a
qualquer momento nas opções.

### Início rápido

#### Windows

```bat
install-python.bat    :: uma vez: instala Python 3.13 + .venv + pygame
start.bat             :: inicia a coleção de jogos
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # arranca com .venv, senão com o python3 do sistema
```

`start.bat` / `start.sh` usam automaticamente o ambiente virtual `.venv` se
existir, senão o Python do sistema. No fim do documento há um guia detalhado
passo a passo: **[Guia de instalação](#guia-de-instalação)**.

### Os jogos

| Jogo         | Modos           | Descrição breve |
|--------------|-----------------|-----------------|
| **Snake**    | 1 / 2 jogadores | Snake de luxo com vista 2D e 3D, turbo, 6 modos (incl. Competitivo), maçãs douradas e prestígio |
| **Pong**     | 1 / 2 jogadores | O clássico contra a IA ou o jogador 2, modo de movimento comutável |
| **Air Hockey** | 1 / 2 jogadores | Física 2D com transferência de impulso, controlo com rato, IA e power-ups |
| **Tic-Tac-Toe** | 1 / 2 jogadores | Jogo m,n,k de 3x3 a 9x9, três níveis de IA **ou** X contra O em local |
| **Breakout** | 1 jogador       | Parte-tijolos com tipos de tijolo, power-ups, combos e muitos níveis |
| **Tetris**   | 1 / 2 jogadores | Clássico ou Versus (dois campos lado a lado) |
| **Invaders** | 1 jogador       | Space Invaders: limpa as vagas, protege as tuas vidas |
| **Asteroids** | 1 / 2 jogadores | Física de inércia, vagas, OVNIs, power-ups, hiperespaço - a solo ou duelo cooperativo |
| **Pac-Man**  | 1 jogador       | Clone fiel: 4 IAs de fantasmas, pílulas de poder, túneis, frutas, níveis |
| **Flappy Bird** | 1 jogador    | Voo com gravidade entre canos, moedas, escudo, dia/noite, medalhas |
| **Doodle Jump** | 1 jogador    | Salto automático para cima, tipos de plataforma, molas, hélice, monstros |
| **2048**     | 1 jogador       | Puzzle de deslizar números, objetivo: a peça 2048 |
| **Minesweeper** | 1 jogador    | O clássico com primeiro clique seguro, chording, smiley e melhores tempos |
| **Sudoku**      | 1 jogador    | 400 níveis com semente (4 dificuldades x 100), 4 modos de ajuda com multiplicador, notas, dicas, limite de 3 erros |
| **Frogger**     | 1 jogador    | Estrada + rio + 5 baías, mosca bónus, crocodilos, limite de tempo, 3 dificuldades |
| **Memory**      | 1 / 2 jogadores | Encontra pares em 4x4 até 8x6, animação de viragem, a solo ou em duelo |
| **Solitário**   | 1 jogador    | 5 variantes (Klondike, Spider, FreeCell, Pirâmide, TriPeaks) com arrastar e largar e anular |
| **Aim Trainer** | 1 jogador    | Tiro ao alvo 3D descontraído: o rato dirige a câmara, 4 modos (precisão/reflexos/móveis/chill), 3 temas incl. um buraco negro |
| **Quatro em linha** | 1 / 2 jogadores | O clássico com animação de queda: 3 níveis de IA (minimax) ou duelo local |
| **Duelo de tanques** | 1 / 2 jogadores | Duelo 2D em arena com tiros com ricochete, power-ups, 4 arenas, IA com 3 níveis |
| **Blackjack**    | 1 jogador    | Blackjack de casino com shoe de 4 baralhos, dobrar/dividir, blackjack 3:2 e saldo de fichas persistente |
| **Tunnel Racer** | 1 jogador    | Voo 3D num tubo de néon: modo sem fim + 30 níveis, controlo por teclas ou rato, motion blur |
| **Labirinto 3D** | 1 jogador    | Raycaster na primeira pessoa (estilo Wolfenstein) com 50 níveis com semente, orbes, minimapa - ou vista de cima 2D |
| **Reversi**      | 1 / 2 jogadores | Othello em 8x8: cercar e virar peças, 3 forças de IA (minimax) ou um duelo local |
| **Yahtzee**      | 1 / 2 jogadores | Clássico de dados com 13 categorias, bónus superior e Yahtzee; caça ao recorde ou hotseat a 2 |
| **Wordle**       | 1 jogador    | Adivinha a palavra de 5 letras em 6 tentativas, série sem fim, pistas coloridas, 5 idiomas |
| **T-Rex Runner** | 1 jogador    | Corrida infinita pelo deserto: salto variável, agachar, cactos e pterodáctilos, ciclo dia/noite, velocidade crescente, 3 dificuldades |
| **Damas**        | 1 / 2 jogadores | 3 regulamentos à escolha (alemãs 8×8, internacionais 10×10, checkers), captura obrigatória e dama voadora, 3 forças de IA (minimax) ou duelo local |
| **Póquer**       | 1 jogador    | 3 variantes à escolha: Texas Hold'em contra a IA, 5 Card Draw e Video Poker; rondas de apostas, blinds, saldo de fichas persistente |
| **Quebra-cabeça deslizante** | 1 jogador | Jogo do 15 em 3x3/4x4/5x5: deslize as peças numeradas para o vazio, controle por rato ou setas, pontos por jogadas e tempo |
| **Mastermind**       | 1 jogador  | Decifra o código de cor secreto (3 modos: 4×6, clássico, 5×8), pinos de dica pretos/brancos, série sem fim |
| **Bubble Shooter**   | 1 jogador  | Clone do Puzzle Bobble: atira cores iguais em grupos de três, ressaltos nas paredes, grupos que caem, 3 dificuldades |
| **Hangman**          | 1 jogador  | Adivinha a palavra antes de a forca ficar completa; teclado no ecrã, listas de palavras por idioma, 3 modos de tamanho, série sem fim |
| **Block Jump**       | 1 jogador  | Plataforma 3D estilo Minecraft: mundo de blocos (voxels) com escadas, cercas e blocos-mola, câmera 1ª/3ª pessoa, desfoque, níveis gerados |

**O multijogador (2 jogadores em local)** está disponível em **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Quatro em linha**, **Duelo de tanques**,
**Reversi**, **Yahtzee** e **Damas**.
O modo escolhe-se diretamente no ecrã de preparação (*Um jogador / Multijogador*).

#### Detalhes por jogo

**Snake**
- **NOVO - Vista 3D** (tecla **V** no setup ou clique em *Vista*): o tabuleiro é
  renderizado como cena 3D em tempo real - uma **câmara de perseguição** flutua
  atrás da cobra e conduz-se **em relação ao olhar** (esq/dir = virar, duas
  pressões rápidas = inversão de marcha). Com nevoeiro de distância, céu
  estrelado, chão de xadrez, bandas, cristais de comida rotativos, partículas 3D
  e abanão de câmara ao bater; depois do game over a câmara orbita a cobra. O
  turbo alarga o campo de visão. Em 3D: *Clássico* e *Obstáculos* (aí os muros
  são sempre fixos, 3D só em um jogador). A vista é guardada em `settings.json`.
- **NOVO - Opções de câmara 3D** (no setup 3D, a linha *Câmara 3D /
  Smooth-Shake* ou tecla **K**): menu próprio com **Smooth-Shake** (câmara mais
  suave, muito menos solavancos), **campo de visão (FOV)** e **altura da câmara**
  ajustáveis e um interruptor de **abanão ao virar**. Tudo guardado em
  `settings.json`.
- **Turbo**: **manter** a tecla de turbo = velocidade dupla, consome resistência
  (barra); vazia, o turbo desliga e recarrega. Padrão J1 = Espaço/Shift esq.,
  J2 = Enter/Shift dir.
- **6 modos** (no setup): *Clássico*, *Speed-Rush* (mais rápido a cada maçã),
  *Obstáculos* (blocos mortais), *Portais* (pares de teletransporte),
  *Contrarrelógio* (60 segundos, tantas maçãs quanto possível) e *Competitivo*
  (ver abaixo).
- **NOVO - Competitivo** (um jogador): modo sem fim com **subida de nível** -
  começas com exatamente **uma** maçã; quantas mais apanhares no total, maior o
  teu **nível**, que vai pondo mais uma maçã simultânea no campo e sobe o
  multiplicador de pontos. As **maçãs azuis** abrem uma **slot machine**: a
  aposta é o teu comprimento, o resultado multiplica-o ou encolhe-o e faz
  aparecer **maçãs extra** por instantes (jackpot com três símbolos iguais).
  As **maçãs lilás** (aposta) põem em jogo uma parte do teu **tamanho** e
  multiplicam essa parte ao acaso, o resto fica seguro (novo tamanho =
  tamanho·(1-p) + tamanho·p·fator): **normal** 50 % fixo com **x0.5 .. x1.5**,
  no **HARDCORE** mais arriscado com **75-90 %** e **x0.25 .. x2.25**. O
  **tamanho** aparece como **decimal em cima à esquerda** e é transportado com
  exatidão, para as apostas seguintes partirem dele. Há **15 níveis**
  (multiplicador até x16, até 16 maçãs ao mesmo tempo); os níveis vivem em
  `games/levels/snake-comp.json` e podem ser ampliados sem tocar no código, o
  resto da afinação está em `competitive.py`.
- **NOVO - HARDCORE** (interruptor no setup do Competitivo, tecla **H**): cada
  **turbo consome comprimento** da tua cobra; um **letreiro HARDCORE** vermelho
  marca o modo. Só no Competitivo; o comprimento nunca desce abaixo do mínimo.
  Guardado em `settings.json`.
- As **maçãs douradas** (temporárias) dão muitos pontos e recarregam o turbo.
- Opcional: **atravessar muros**, maçãs bónus, **prestígio** (um jogador, tecla **P**).
- **NOVO - Personalizar** (botão do pincel em cima à direita do setup, ou tecla
  **C**): um menu só visual ("mods" que *nunca* mudam o jogo) com separadores:
  - **Cabeça**: a **cor da cabeça** - 4 modelos azul-turquesa, vermelho, laranja
    e uma **cor própria** com reguladores RGB.
  - **Grelha (guia)**: sobrepõe uma **grelha de coordenadas** - **números de
    linha** (bordas esquerda e direita) e **letras de coluna** (cima/baixo).
    Assim em campos grandes vês logo que a maçã em *8a* está na mesma linha *8*
    que a tua posição *8z*. A sequência de cores (5 modelos + duas cores
    próprias A/B) define o tema.
  - **Banner**: ligar/desligar o banner de multiplicador (p. ex. da maçã lilás)
    e ajustar **tamanho** e **opacidade** - com pré-visualização ao vivo.
  Tudo é guardado em `mem-ngb.json`; a personalização visual passa pelo módulo
  `ngb.py`.
- Visual: cobra arredondada com olhos (cabeça turquesa por padrão), brilho de
  turbo, partículas.

**Pong**
- Um jogador contra a IA, multijogador = jogador 2 à direita. Até 5 pontos.
- **Modo de movimento comutável por controlo**: *Contínuo* (primes uma vez ->
  continua a andar, padrão) ou *Manter* (só se move enquanto primes).
  Mudar: **X** = controlo 1, **N** = controlo 2 (guardado em `settings.json`).
- Física da bola com aceleração e ângulo conforme o ponto de impacto.

**Air Hockey**
- **Física 2D verdadeira**: tacos redondos e puck com transferência de impulso -
  o puck herda a velocidade do taco no toque; bandas com restituição, ligeira
  fricção de gelo, balizas como aberturas nas paredes laterais.
- **Controlo com rato** em um jogador: o taco segue o rato (qualquer tecla volta
  ao teclado). Teclado: 8 direções, multijogador = J1 esquerda (WASD), J2
  direita (IJKL).
- **IA com três níveis** (Fácil/Médio/Difícil): defende a sua baliza, ataca na
  sua metade e contorna o puck para evitar autogolos.
- **Power-ups** (desativáveis): *XL* (taco maior), *GOLO* (a baliza adversária
  encolhe), *>>* (taco mais rápido) - pertencem ao último jogador que tocou o
  puck.
- Setup: dificuldade, **golos para vencer** (3/5/7/10), power-ups sim/não
  (guardado em `settings.json`). Após cada golo serve quem o sofreu.
- Visual: rasto do puck, partículas, bocas de baliza pulsantes, indicadores.

**Tic-Tac-Toe**
- Setup: dificuldade (Fácil/Médio/Difícil) e tamanho do tabuleiro 3x3..9x9;
  comprimento vencedor K = 3 (3x3), 4 (4x4), senão 5.
- **1 jogador** contra a IA (Difícil no 3x3 é imbatível) **ou 2 jogadores** em
  local (X contra O, à vez com clique). Depois do fim: Enter/clique = nova
  ronda, **S** = definições.

**Breakout**
- Tipos de tijolo: Normal, **Aço** (indestrutível), **Bomba** (explode), **Ouro**
  (pontos extra).
- Power-ups: laser, bola de fogo, pegajosa, escudo, moeda e mais;
  **multiplicador de combo**.
- Efeitos: partículas, rastos, screen shake, pop-ups de pontos, muitos padrões.
- Setup: **1/2/3** = dificuldade, **Esq/Dir** = cor da bola, **Cima/Baixo** =
  nível inicial, **M** = estrutura. Jogo: rato/setas, **Espaço** lança a bola
  (dispara laser), **P/Esc** = pausa.

**Tetris**
- Esq/Dir move, Cima = rodar, Baixo = soft drop, Ação = hard drop.
- Linhas completas dão pontos; a cada 10 linhas sobe o nível.
- **Versus**: perde aquele cuja pilha toca primeiro no topo.

**Invaders** – dois modos (no ecrã de preparação):
- **Clássico**: o clássico bloco de aliens; depois no setup: **movimento** (só
  esq/dir *ou* livre com WASD) e **mira** (sempre para cima *ou* para o **rato**
  – disparas para onde está o cursor). Os aliens destruídos às vezes largam
  power-ups.
- **Arena (livre)**: movimento livre, os inimigos entram por todas as bordas;
  aponta-se na direção do movimento, arma com **1–4**.
Em comum: sistema de níveis com **boss** a cada 4.º nível, quatro armas
(blaster, tiro disperso, fogo rápido, laser), power-ups (vida extra, escudo,
melhoria de arma), efeitos de explosão, recorde.

**Asteroids**
- **Física de inércia**: Cima = propulsão na direção do olhar, Esq/Dir = rodar,
  a nave continua à deriva (leve amortecimento); tudo atravessa as bordas do
  ecrã. **Visual vetorial** clássico com chama de propulsão e céu estrelado;
  cada rocha tem o seu polígono aleatório.
- As rochas partem-se em duas mais pequenas (3 tamanhos, **20/50/100 pontos**),
  **vagas** crescentes com anúncio em banner.
- **OVNI** (desativável): cruza o ecrã e aponta às naves (erro de pontaria
  conforme a dificuldade) - 200 pontos por abatê-lo.
- **Power-ups** (desativáveis), caem das rochas destruídas: **E**scudo (6 s
  invulnerável), tiro **T**riplo, fogo **R**ápido.
- **Hiperespaço** (tecla Baixo): salto de emergência para uma posição aleatória
  com 4 s de recarga - e 12 % de risco de te despedaçares.
- 3 vidas, reaparecimento seguro com piscar de invulnerabilidade, **vida extra
  a cada 5000 pontos**; partículas de explosão e abanão de câmara.
- **Duelo cooperativo** (multijogador): as duas naves voam ao mesmo tempo com
  vidas e pontos separados - vence quem tiver mais pontos.
- Setup: dificuldade, OVNIs sim/não, power-ups sim/não (em `settings.json`).

**Pac-Man**
- **Labirinto clássico 28x31** em visual néon com pílulas, 4 pílulas de poder,
  túneis laterais e casa de fantasmas ao centro.
- **Quatro fantasmas com os comportamentos originais** (IA de casa alvo):
  *Blinky* persegue direto, *Pinky* arma a emboscada (4 casas à frente), *Inky*
  usa um vetor através do Blinky, *Clyde* afasta-se de perto.
- **Fases scatter/chase** alternadas (os fantasmas invertem a marcha a cada
  mudança); a **pílula de poder** torna-os azuis e comestíveis (cadeia
  200/400/800/1600), depois os olhos voltam a casa.
- Casa de fantasmas com **saída escalonada**, **frutas** de bónus (por nível),
  **3 vidas**, **vida extra aos 10.000**, sistema de níveis (mais rápido),
  animação de morte, ecrãs READY/GAME OVER.
- Setup: **dificuldade** (Normal/Difícil/Extremo) – velocidade dos fantasmas e
  tempo de medo.
- Controlos: **setas ou WASD**.  Enter = novo, S = setup.

**Flappy Bird**
- **Física de gravidade**: Espaço / Cima / W / **clique** faz o pássaro bater as
  asas; ele inclina-se conforme o ritmo de subida/descida.
- **Pares de canos** sem fim com abertura (+1 por cano); **moedas** (bónus) e um
  power-up de **escudo** (sobrevive a uma colisão) aparecem nas aberturas.
- **Temas de dia/noite** mudam com a pontuação; nuvens à deriva (parallax), chão
  em deslocamento.
- Dificuldade (Fácil/Normal/Difícil): tamanho da abertura, velocidade, distância –
  a abertura estreita com a pontuação a subir.
- **Medalhas** (bronze/prata/ouro/platina) depois do game over, animação de
  choque com abanão de câmara, recorde.

**Doodle Jump**
- O doodler **salta automaticamente** ao aterrar; só diriges esq/dir (com
  inércia), as bordas dão a volta (**wrap-around**); a câmara sobe contigo.
- **Tipos de plataforma**: verde (normal), azul (móvel), castanha (parte-se),
  branca (desaparece). **Molas** dão um supersalto, o **chapéu-hélice** leva-te
  por instantes automaticamente para cima (e invulnerável).
- **Monstros**: o toque é mortal – mas podes **abatê-los** com Cima / Espaço
  (pontos extra).
- Pontos = altura alcançada; a dificuldade sobe com a altura. Recorde.
- Controlos: esq/dir = mover, Cima / Espaço = disparar.

**2048** – setas/WASD deslizam todas as peças; números iguais fundem-se.

**Minesweeper**
- Três níveis: **Principiante** (9x9, 10 minas), **Avançado** (16x16, 40),
  **Perito** (30x16, 99) - o **melhor tempo por nível** é guardado e mostrado no
  setup.
- O **primeiro clique é sempre seguro** (as minas só são distribuídas depois, a
  área 3x3 em volta fica livre).
- **Clique esquerdo** = revelar, **clique direito** = bandeira (opcional com
  ciclo de interrogação), **F** = bandeira sob o cursor, **R** = novo.
- **Chording**: clique num número completo revela os vizinhos restantes.
- HUD clássico: contador de minas, **smiley clicável** (espantado/óculos de
  sol/morto), cronómetro; bandeiras erradas são riscadas no fim, confetes na
  vitória.
- Pontos = valor base do nível menos segundos.

**Sudoku**
- **400 níveis**: 4 dificuldades (Fácil/Normal/Difícil/Perito) x 100 níveis. Os
  puzzles são **gerados por semente e têm solução única** - o nível 12 de
  "Difícil" é o mesmo puzzle em qualquer PC. Os resolvidos são guardados e
  assinalados na escolha de níveis.
- **4 modos de jogo** (antes de começar) com multiplicador: **Clássico** (x2,0 -
  sem ajudas), **Notas** (x1,5 - + notas a lápis), **Conforto** (x1,0 - + erros
  a vermelho, realce de conflitos e dígitos iguais, entradas corretas fixam-se),
  **Assistente** (x0,7 - + dica, máx. 3).
- Cada entrada é verificada logo contra a solução; com o **limite de 3 erros**
  ativo (opção do setup) o terceiro erro acaba a partida.
- Controlos: setas/WASD = célula, **1-9** = dígito (também teclado numérico),
  **0/Backspace/clique direito** = apagar, **N** = notas, **H** = dica,
  **R** = recomeçar nível, **Q** = escolha de níveis; totalmente jogável com o
  rato (painel numérico à direita). Depois do fim, **A** esconde a faixa e
  mostra a **solução** completa (A de novo = voltar).
- Pontos = (base do nível - tempo - erros - dicas) x multiplicador do modo.

**Frogger**
- 5 faixas de trânsito (carros/camiões) e 5 vias de rio (troncos, tartarugas que
  **mergulham** em níveis altos); no topo 5 baías - encher todas = próximo
  nível, tudo acelera.
- Extras: **mosca bónus** (+200) em baías vazias, **crocodilos** ocupam baías em
  níveis altos, **barra de tempo** por rã, vida extra aos 10 000.
- 3 dificuldades (velocidade, densidade do trânsito, tempo); pontos por fila
  nova, baía = 50 + bónus de tempo, nível completo = +1000.

**Memory**
- Tamanhos de tabuleiro **4x4, 6x6, 8x6**; motivos de combinações forma-cor,
  desenhados por completo com primitivas; **animação de viragem**, pares
  falhados viram-se sozinhos.
- **Solo**: base - 15 por jogada - 2 por segundo (mín. 100). **Duelo** (local):
  à vez, acerto = repetes, vence quem tiver mais pares.

**Solitário**
- **5 variantes** no ecrã de preparação: Klondike (tirar 1/3 como opção), Spider
  (1/2/4 naipes), FreeCell (limite de supermovimentos), Pirâmide (pares de 13,
  2 redeals) e TriPeaks (cadeia ±1 com multiplicador de combo).
- **Arrastar e largar** ou clique-clique, **clique direito** = para a fundação,
  **U** = anular ilimitado, **R** = mão nova, Espaço = baralho.
- As cartas são renderizadas sem ficheiros de imagem (`games/cards.py`); todas
  as variantes partilham uma lista de recordes com fórmulas específicas.

**Aim Trainer**
- **3D por software verdadeiro** (como o modo 3D do Snake): mira fixa no centro,
  **controlo de rato direto 1:1 como num shooter** (captura de ponteiro: o
  cursor fica preso na janela, Esc liberta-o; sensibilidade ajustável, yaw
  ilimitado, pitch ±60°). O clique esquerdo dispara exato pelo centro, com
  clarão, traçadora e partículas de impacto.
- **4 modos**: Precisão (60 s, 3 esferas, bónus de precisão), Reflexos (30 alvos
  um a um, estatística de reação), Alvos móveis (trajetórias + multiplicador de
  combo até x4) e Chill (sem fim, sem castigo, **E** termina).
- **3 temas** (no setup, guardados): **Espaço** com esfera de estrelas, um
  **buraco negro com anel brilhante** e um planeta (padrão), arena néon com
  grelha no chão e sol synthwave, e uma carreira de tiro interior.
- A sensibilidade também muda a meio do jogo com **+/-**; mais um **motion blur
  ajustável** (0-80 %) para um visual extra chill - ambos guardados.

**Quatro em linha**
- Tabuleiro 7x6 com **animação de queda**, pré-visualização ao passar o rato e
  linha vencedora pulsante; rato, setas ou escolha direta **1-7**.
- **3 níveis de IA** (minimax com poda alfa-beta): Fácil ignora ameaças de
  propósito, Médio bloqueia com fiabilidade, Difícil planeia fundo - ou
  **2 jogadores** em local no mesmo aparelho.
- Após cada ronda muda quem começa; o recorde conta as **vitórias contra a IA**
  de uma sessão.

**Duelo de tanques**
- Duelo 2D em arena: **os tiros ressaltam uma vez nas paredes** (ricochete) -
  acerta pela esquina (ou em ti próprio!). À melhor de 5 rondas com contagem
  decrescente.
- **4 arenas** (Aberta, Cruz, Colunas, Labirinto) ou rotação aleatória;
  **power-ups**: fogo rápido, escudo, tiro triplo.
- **IA com 3 níveis** - a difícil aponta com antecipação e dispara de propósito
  com ressalto - ou **2 jogadores** num teclado (J1 WASD+Espaço, J2 setas+Enter).

**Blackjack**
- Regras de casino verdadeiras: **shoe de 4 baralhos**, o dealer fica em 17, o
  **blackjack paga 3:2**, peek do dealer com ás/10; **dobrar** e **uma divisão**
  (ases divididos recebem uma carta cada).
- **Saldo de fichas persistente**: começas com 500, saldo e **recorde**
  sobrevivem a cada reinício (`mem.json`); com menos de 10 fichas recebes 500
  novas - o recorde fica.
- Manejo com botões de fichas e teclas (**H**it/**S**tand/**D**ouble/dividir
  **X**, **1-4** = aposta, Enter = dar) com animações de cartas e viragem da
  carta tapada.

**Tunnel Racer**
- **Voo 3D num tubo de néon** (renderizador por software como o Aim Trainer):
  barras, blocos e **diafragmas em anel para enfiar**, moedas na linha ideal.
- **Dois modos**: Sem fim (a velocidade sobe até um teto, recorde) e **30 níveis
  com semente** com meta, bónus de tempo e progresso assinalado.
- **Controlo por teclas** (padrão) ou **controlo direto com rato** (captura de
  ponteiro, tecla **C**); mais **motion blur ajustável** (tecla **B**, 0-80 %) -
  tudo guardado.

**Labirinto 3D**
- **Raycaster na primeira pessoa estilo Wolfenstein** (DDA, nevoeiro de
  distância, sprites) com mouselook + WASD, **minimapa** (tecla **M**) e saída
  verde pulsante - ou uma **vista de cima 2D** clássica (tecla **V** no setup).
- **50 níveis com semente** que vão crescendo; a saída fica sempre no ponto mais
  afastado, os **orbes** pelo caminho dão pontos extra.
- Pontos: 500 por nível + 100 por orbe + bónus de tempo; níveis resolvidos são
  assinalados e a sessão soma-se ao recorde.

**Reversi**
- **Othello em 8x8**: coloca peças que cercam as filas do adversário e vira tudo
  o que fica preso; jogadas ilegais estão bloqueadas e uma vez sem jogada é
  **passada automaticamente**.
- **Um jogador contra a IA** (3 forças: negamax com alfa-beta, ponderação de
  posição + mobilidade) **ou um duelo local**, Pretas contra Brancas.
- As casas válidas são realçadas; joga com o **rato** ou com o cursor de seleção
  (setas + Espaço/Enter). Cada vitória contra a IA vale um ponto para o recorde.

**Yahtzee**
- **Clássico de dados**: 5 dados, até 3 lançamentos por vez, **guarda** os dados
  um a um, depois marca uma das **13 categorias** (com pré-visualização dos
  pontos possíveis).
- Folha completa: secção de cima com **bónus de 63 (+35)**, trinca/quadra, full
  house, sequência baixa/alta, **Yahtzee (50)** e Chance.
- **Um jogador como caça ao maior total** ou **hotseat a 2 jogadores** com duas
  folhas lado a lado; joga com o rato ou teclas (Espaço, 1-5, setas, Enter).

**Wordle**
- Adivinha a **palavra de 5 letras em 6 tentativas**; resposta colorida
  (verde/amarelo/cinzento) com **contagem correta de letras repetidas** e um
  teclado no ecrã que se colore.
- **Série sem fim**: cada palavra resolvida dá pontos (menos tentativas = mais),
  a primeira palavra não resolvida termina a partida - total = recorde.
- **Listas de palavras por idioma** (só A-Z); as tentativas não são verificadas
  num dicionário. Escreve no teclado ou clica as teclas no ecrã.

Os recordes são guardados na secção `highscores` de `mem.json` (junto ao
código) – juntamente com o idioma (secção `mem`).

### A interface

Toda a interface é desenhada de raiz (Tkinter puro + Pygame, sem pacotes extra)
e polida com aspeto de launcher moderno:

- **Barra lateral com lista de jogos**: cada linha tem o seu **mini-pictograma**
  na cor de destaque do jogo, mostra o **recorde atual (★)** e reage com efeitos
  hover suavemente animados. O jogo em curso fica marcado a cores; em janelas
  pequenas a lista **desloca-se** com a roda do rato.
- **Cartão de estado** em baixo à esquerda com **LED de estado** (cinzento =
  menu, verde = a decorrer, dourado = pausa, vermelho = game over) e **FPS ao
  vivo**.
- **Ecrã inicial** com luzes aurora, campo de estrelas com parallax e estrelas
  cadentes, logótipo flutuante com faíscas em órbita, uma **grelha de jogos
  clicável** logo abaixo do logótipo (todos os jogos com efeito hover na sua
  cor) e uma **faixa de recordes**.
- **Efeitos por todo o lado**: transições suaves entre ecrãs, faíscas ao
  confirmar no menu, **chuva de confetes num novo recorde** e um **desfoque
  verdadeiro** atrás da sobreposição de pausa.
- O **ecrã de preparação** de cada jogo aparece na sua cor de destaque e mostra
  o recorde anterior como chip.
- **Wiki integrado** ("LamaWiki"): ajuda detalhada de cada jogo (controlos,
  modos, pontos, dicas) mais páginas gerais - com **campo de pesquisa**,
  categorias, artigos deslocáveis e chips de teclas, nos cinco idiomas.
  Acessível pelo botão **«Wiki / Ajuda»** da barra lateral e a partir do ecrã de
  preparação de cada jogo (abre logo a sua página).

### Utilização

- Escolhe o jogo com o botão no menu à esquerda. Depois aparece o **ecrã de
  preparação**: escolher **Um jogador** ou **Multijogador**, ir às **opções** ou
  voltar. Setas/rato para escolher, Enter começa.
- **ESC** = pausa / continuar (nos menus: voltar).
- **F11** (ou o botão «Ecrã inteiro sim/não») = ecrã inteiro. O ecrã do Pygame
  continua incorporado e é ampliado mantendo a proporção (barras pretas se a
  proporção diferir). A janela pode ser redimensionada livremente.
- **«Voltar ao menu»** termina o jogo e guarda o recorde.
- **«Sair»** fecha o Pygame e o Tkinter de forma limpa.

### Opções, controlos e som

O ecrã de opções abre com o botão **«Opções / Controlos»** (à esquerda) ou a
partir do ecrã de preparação. Está organizado em **três separadores**
(**Geral / Controlos / Aparência**; muda-se com clique ou com a tecla Tab):

- **Geral**: **som** sim/não, **volume** e **vibração** (vibração do gamepad,
  só com comando ligado) além de **resolução automática**, **resolução**,
  **FPS** e **idioma** – cada um com Esq/Dir.
- **Controlos**: **modelos** (*WASD + Setas*, *WASD + IJKL*, *Setas + WASD*) e
  **cada tecla individual** dos jogadores 1 e 2 é reatribuível: escolher a
  linha, premir Enter, premir a tecla desejada (Esc cancela).
- **Aparência**: escolher o **design da interface** – **UI v4** (padrão: um
  visual grafite limpo e plano com um único acento índigo) ou **UI v3** (a
  interface clássica anterior com céu estrelado, auroras e brilhos). Ambos os
  cartões mostram uma pequena pré-visualização; a escolha aplica-se de
  imediato a toda a interface (área de jogo **e** barra lateral) e é guardada.

As definições são guardadas permanentemente em `settings.json`. Em **um
jogador** ambas as atribuições controlam a mesma figura (padrão: WASD *e*
setas), no **multijogador** uma cada. Todos os jogos têm **efeitos sonoros**
(gerados proceduralmente, sem ficheiros extra) que podem ser silenciados
globalmente.

### Estrutura do projeto

```
install-python.bat  Instalação no Windows: Python 3.13 + .venv + pygame
start.bat            Script de arranque (Windows)
start.sh             Script de arranque (Linux / macOS / Git Bash)
pyinstall.bat        Build de EXE (Windows): empacota tudo em builds\PyGameZ.exe
main.py              Interface Tkinter, incorporação do Pygame, ciclo central
game_base.py         Classe base de jogo (update/draw/handle_event) + InputEvent + auxiliares
settings.py          Carregar/guardar definições (som/vibração/teclas) (JSON)
audio.py             Efeitos sonoros procedurais + vibração de gamepad
menu.py              Ecrãs de idioma, preparação (modo) e opções (som/controlos)
highscore.py         Carregar/guardar recordes (secção em mem.json)
store.py             Ficheiro central mem.json (secções: mem, highscores)
prestige.py          Sistema de prestígio do Snake
competitive.py       Parâmetros do modo Competitivo do Snake (níveis, slot machine, maçãs de aposta)
ngb.py               Personalização visual ("mods"): cor da cabeça + grelha + menu (mem-ngb.json)
i18n.py              Motor de tradução (carrega lang/*.json, t("chave"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textos (uma chave por texto)
lamawiki/
  lamawiki.py          Wiki integrado (pesquisa, categorias, renderizador)
  de.json  en.json  fr.json  es.json  pt.json   Conteúdo do wiki (uma página por jogo + gerais)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py  tetris.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py
  trexrunner.py  dame.py  poker.py
```

O idioma escolhido é guardado em `mem.json` (na secção `mem`, junto à secção
`highscores` do mesmo ficheiro) e carregado automaticamente no próximo arranque.

### Notas de plataforma

O ecrã corre **off-screen**: o pygame usa o controlador de vídeo dummy
(`SDL_VIDEODRIVER=dummy`), renderiza para uma superfície e cada fotograma é
desenhado como imagem num widget do Tkinter. **Não há janela SDL nativa** a
lutar com o Tkinter por tamanho/posição. Assim a janela comporta-se igual e de
forma estável em todo o lado:

- **Windows**: o processo é ainda marcado como DPI-aware para a imagem ficar
  nítida em ecrãs escalados (125/150/200 %) e não "tremer".
- **Linux/X11 e Wayland**: funciona sem casos especiais (sem `SDL_WINDOWID`).
- **macOS**: também funciona (antes a janela incorporada nem aparecia aqui).

---

### Guia de instalação

Requisito: **Python 3.9+** (recomendado 3.12 ou 3.13) e **pygame ≥ 2.6**.

#### Windows (recomendado: automático)

1. Abre a pasta do projeto e executa **`install-python.bat`** com duplo clique.
   O script
   - verifica se existe **Python 3.13** e, se não, instala-o via
     **winget** (`winget install Python.Python.3.13`),
   - cria o ambiente virtual **`.venv`**,
   - instala o **pygame** a partir de `requirements.txt`.
2. Depois inicia a coleção com **`start.bat`** (duplo clique).

> Nota: se o script disser "ainda não disponível nesta janela", o Python acabou
> de ser instalado – abre **um novo terminal/janela** e executa
> `install-python.bat` outra vez. Se não houver **winget**, instala o Python
> 3.13 manualmente em <https://www.python.org/downloads/> marcando
> **"Add python.exe to PATH"**.

#### Windows / Linux / macOS (manual)

```bash
# 1. Verificar o Python (3.9+)
python --version

# 2. Criar e ativar um ambiente virtual
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Instalar dependências
pip install -r requirements.txt
#   ou:  pip install "pygame>=2.6" (ou pygame-ce)
#                                   pip install pygame-ce
# 4. Iniciar
python main.py
```

#### Linux / macOS com start.sh

```bash
# Preparar Python + venv como acima (passos 2 e 3), depois:
chmod +x start.sh      # uma vez, se ainda não for executável
./start.sh
```

No Linux, se necessário, instala o Python com o gestor de pacotes, p. ex.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); no macOS
p. ex. `brew install python`.

#### Usar outra versão do Python

O `install-python.bat` instala por padrão o Python 3.13. Quem preferir o 3.12
(ou outra versão) muda no ficheiro a linha `set "PYVER=3.13"` para a versão
desejada e o ID do winget em conformidade (`Python.Python.3.12`).

#### Criar um EXE autónomo (Windows)

```bat
pyinstall.bat         :: cria builds\PyGameZ.exe (tudo num único ficheiro)
```

`pyinstall.bat` usa a `.venv` (e cria-a se necessário), instala o
**PyInstaller** automaticamente e empacota o jogo completo - Python, pygame,
todos os jogos, os idiomas, o wiki e os logótipos - num **único
`PyGameZ.exe`** na pasta **`builds\`**. O ficheiro corre em qualquer PC
Windows sem Python instalado e pode ser copiado livremente. As definições e
recordes (`settings.json`, `mem.json`, `mem-ngb.json`) são criados ao lado
do .exe durante o jogo.

#### Resolução de problemas

- **`pygame` não encontrado** → venv ativado? Repete o passo 3
  (`pip install -r requirements.txt`).
- **`python` não é reconhecido (Windows)** → o Python foi instalado sem "Add to
  PATH"; reinstala com a opção marcada, ou usa `py` em vez de `python`.
- **Sem som** → verifica "Som" nas opções; a vibração só funciona com comando.
- **Janela/incorporação no Linux** → ver *Notas de plataforma* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ voltar ao topo / back to top</a></b></div>
