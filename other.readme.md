<a name="other-languages"></a>

# PyGameZ - Autres langues / Otros idiomas / Outros idiomas

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](README.md#-deutsch)** · **🇬🇧 [English](README.md#-english)** · **🇫🇷 [Français](#-francais)** · **🇪🇸 [Español](#-espanol)** · **🇵🇹 [Português](#-portugues)** · **🇵🇱 [Polski](#-polski)** · **🇹🇷 [Türkçe](#-turkce)** · **🇩🇰 [Dansk](#-dansk)** · **🇳🇴 [Norsk](#-norsk)** · **🇸🇪 [Svenska](#-svenska)** · **🇫🇮 [Suomi](#-suomi)** · **🇨🇿 [Čeština](#-cestina)** · **🇸🇮 [Slovenščina](#-slovenscina)** · **🇭🇷 [Hrvatski](#-hrvatski)**

---

<a name="-francais"></a>

## 🇫🇷 Français

Une collection de jeux de bureau en Python : **Tkinter** fournit la fenêtre et
le menu, **Pygame** est intégré comme écran de jeu à l'intérieur de la fenêtre
Tkinter. Quarante-six jeux avec des options partagées, des commandes entièrement
réassignables, des meilleurs scores, des effets sonores procéduraux et, pour
plusieurs titres, un mode multijoueur. L'interface est **multilingue** –
**14 langues** (allemand / anglais / français / espagnol / portugais / polonais /
turc / danois / norvégien / suédois / finnois / tchèque / slovène / croate) ; la
langue se choisit au premier démarrage sur un **écran d'accueil** qui permet
aussi de régler la **résolution** et le **son** (désactivé par défaut) ; en
dehors des trois langues principales, toutes les autres se cachent derrière le
bouton **« Plus »**. Tout reste modifiable dans les options.

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
| **Tetris**   | 1 / 2 joueurs   | Règles Guideline modernes (SRS, réserve, aperçu de 5 pièces, T-Spins) : Marathon, Sprint 40, Ultra 2:00, duel contre l'IA (3 niveaux) ou à deux avec lignes de déchets |
| **Invaders** | 1 joueur        | Space Invaders : vide les vagues, protège tes vies |
| **Asteroids** | 1 / 2 joueurs  | Physique d'inertie, vagues, OVNIs, power-ups, hyperespace - en solo ou duel coopératif |
| **Pac-Man**  | 1 joueur        | Clone fidèle : 4 IA de fantômes, pilules de pouvoir, tunnels, fruits, niveaux |
| **Flappy Bird** | 1 joueur     | Vol gravitationnel entre les tuyaux, pièces, bouclier, jour/nuit, médailles |
| **Doodle Jump** | 1 joueur     | Saut automatique vers le haut, types de plateformes, ressorts, hélice, monstres |
| **2048**     | 1 joueur        | Puzzle de nombres à faire glisser de 3x3 à 8x8 : Classique, Contre-la-montre et Infini, annulation, animations fluides, parties sauvegardées |
| **Minesweeper** | 1 joueur     | Le classique avec premier clic sûr, chording, smiley et meilleurs temps |
| **Sudoku**      | 1 joueur     | 4 variantes (Classique, Sudoku X, Killer, Mini 6x6) de 400 niveaux chacune, Sudoku du jour, jusqu'à 3 étoiles par niveau, 4 modes d'assistance, annulation, sauvegarde |
| **Frogger**     | 1 joueur     | Route + rivière + 5 abris, mouche bonus, crocodiles, limite de temps, 3 difficultés |
| **Memory**      | 1 / 2 joueurs | Trouve les paires sur 4x4 à 8x6, animation de retournement, score en solo ou duel |
| **Solitaire**   | 1 joueur     | 5 variantes (Klondike, Spider, FreeCell, Pyramide, TriPeaks) avec glisser-déposer et annulation |
| **Aim Trainer** | 1 joueur     | Tir sur cible 3D détendu : la souris dirige la caméra, 4 modes (précision/réflexes/mobiles/chill), 3 thèmes dont un trou noir |
| **Puissance 4** | 1 / 2 joueurs | Le classique avec animation de chute : 3 niveaux d'IA (minimax) ou duel local |
| **Duel de tanks** | 1 / 2 joueurs | Duel 2D en arène avec tirs à ricochet, power-ups, 4 arènes, IA à 3 niveaux |
| **Blackjack**    | 1 joueur    | Blackjack de casino avec sabot de 4 jeux, doubler/partager et blackjack 3:2 ; joue avec les jetons Lama de la Banque Lama commune |
| **Tunnel Racer** | 1 joueur    | Vol 3D dans un tube néon : mode sans fin + 30 niveaux, pilotage au clavier ou à la souris, motion blur |
| **Labyrinthe 3D** | 1 joueur   | Raycaster à la première personne (style Wolfenstein) avec 50 niveaux à graine, orbes, minicarte - ou vue 2D de dessus |
| **Reversi**      | 1 / 2 joueurs | Othello 8x8 : encercler et retourner les pions, 3 forces d'IA (minimax) ou un duel local |
| **Yams**         | 1 / 2 joueurs | Classique de dés à 13 catégories, bonus supérieur et Yams ; course au score ou hotseat à 2 |
| **Wordle**       | 1 joueur   | Devine des mots de 4 à 7 lettres : Sans fin, Mot du jour, Dordle et Quordle, mode difficile, palette daltonien, statistiques en histogramme, partage du résultat, vraies listes de mots en 14 langues |
| **T-Rex Runner** | 1 joueur   | Course infinie dans le désert : saut variable, s'accroupir, cactus & ptérodactyles, cycle jour/nuit, vitesse croissante, 3 difficultés |
| **Dames**        | 1 / 2 joueurs | 3 règles au choix (allemandes 8×8, internationales 10×10, checkers), prise obligatoire & dame volante, 3 niveaux d'IA (minimax) ou duel local |
| **Poker**        | 1 joueur   | 3 variantes au choix : Texas Hold'em contre l'IA, 5 Card Draw et Vidéo Poker ; tours d'enchères, blinds, jetons Lama de la Banque Lama commune |
| **Échecs**      | 1 / 2 joueurs | Règles complètes, Chess960 et pendule, 6 niveaux d'IA, 200 problèmes de la base Lichess, annulation/indice, liste des coups, export PGN ou duel local |
| **Moulin**      | 1 / 2 joueurs | Phases de pose, de déplacement et de vol, moulins et prises, vol désactivable, 3 niveaux d'IA ou duel local |
| **Simon**       | 1 / 2 joueurs | Jeu de mémoire Senso : modes Classique/Speed/Reverse/Mixte + duel, son oui/non/mixte, 4/6/9 cases, record par mode |
| **Billard**     | 1 / 2 joueurs | 8-Ball, 9-Ball et entraînement en 2D, vue 3D fixe ou caméra 3D libre ; physique douce, aide à la visée, IA à 3 niveaux |
| **Taquin**         | 1 joueur   | Jeu du 15 en 3x3/4x4/5x5 : glisse les tuiles numérotées dans le trou, contrôle souris ou flèches, score selon coups & temps |
| **Mastermind**     | 1 joueur   | Perce le code couleur secret (3 modes : 4×6, classique, 5×8), pions indicateurs noirs/blancs, série sans fin |
| **Bubble Shooter** | 1 joueur   | Clone de Puzzle Bobble : tire des couleurs identiques par groupes de trois, rebonds sur les parois, grappes qui tombent, 3 difficultés |
| **Hangman**        | 1 joueur   | Devine le mot avant que la potence soit complète ; clavier à l'écran, listes de mots par langue, 3 modes de longueur, série sans fin |
| **Block Jump**     | 1 joueur   | Jeu de plateforme 3D façon Minecraft : monde de blocs texturés au skin Minecraft avec figurine Steve, échelles, barrières & blocs-ressorts, caméra 1re/3e personne, flou de mouvement, niveaux générés |
| **Tower Defense** | 1 joueur   | Repousse des vagues infinies sur 4 cartes : jusqu'à 11 types de tours avec améliorations, vente & spécialisation A/B, boss, 3 modes, capacités actives |
| **Minigolf**    | 1 / 2 joueurs | 360 trous sur 40 parcours (18 construits à la main, 342 générés) : sable, rampes, eau, pare-chocs, moulins & blocs mobiles ; carte de score avec par et bonus trou en un ; **éditeur de trous** avec 15 types d'objets, 12 modèles et partage en `.lamapgzmap` |
| **Pinball**     | 1 / 2 joueurs | Flipper avec 3 tables : bumpers, slingshots, cibles, couloirs L-A-M-A, multibille avec jackpot, sauvegarde de bille, secousse & tilt |
| **Bowling**     | 1 / 2 joueurs | 10 frames avec la vraie règle strike/spare, physique des quilles, effet hook et piste en perspective, 3 difficultés |
| **Crossy Road** | 1 joueur      | Sauter sans fin par-dessus prés, routes, rivières et voies ferrées en style voxel isométrique : jour/nuit, aigle, 10 personnages à acheter, parcours du jour |
| **Geometry Dash** | 1 joueur    | Plateforme rythmique avec cube, vaisseau, boule, OVNI et onde : 8 niveaux de Facile à Démon avec 3 pièces secrètes chacun, mode entraînement, bande-son par niveau ; **éditeur de niveaux** avec partage en `.lamapgzlevel` |
| **Battleship**  | 1 / 2 joueurs | Bataille navale 10x10 : flotte placée par glisser-déposer, 3 règles au choix (contact, salve, rejouer), IA à 3 niveaux ou duel local avec écran de passation |
| **Casino**      | 1 joueur      | Roulette européenne avec toutes les mises classiques et Machine Lama (5 rouleaux, 10 lignes, wild, tours gratuits) ; un seul compte de jetons Lama avec le Blackjack et le Poker |

**Le multijoueur (2 joueurs en local)** est disponible pour **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duel
coopératif)**, **Memory (duel)**, **Puissance 4**, **Duel de tanks**,
**Reversi**, **Yams**, **Dames**, **Échecs**, **Moulin**, **Simon (duel)**,
**Billard**, **Minigolf**, **Pinball**, **Bowling** et **Battleship** (avec un
écran de passation qui cache les flottes) - 20 jeux en tout. Le mode se choisit
directement sur l'écran de préparation (*Un joueur / Multijoueur*) ; Tetris
propose en plus un **duel contre l'IA**. La version web est uniquement en solo.

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
- **Règles Guideline modernes** : terrain de 10x20, pièces tirées d'un **sac de
  7**, **système de rotation SRS** avec de vrais wall kicks (y compris pour la
  pièce I), rotation dans les deux sens, **réserve** (une fois par pièce),
  **aperçu de 5 pièces**, pièce fantôme et **lock delay** (0,5 s, au plus 15
  remises à zéro).
- **Trois modes** sur l'écran de préparation : *Solo*, *Duel contre l'IA* et *2
  joueurs*. Le Solo propose dans le setup **Marathon** (niveau de départ 1-15,
  compte pour le record), **Sprint 40 lignes** (meilleur temps) et **Ultra 2
  minutes** (meilleur score) ; les records sont dans la section `tetris` de
  `mem.json`.
- **Score Guideline** : du Single au Tetris, **T-Spins** (complets et mini),
  **Back-to-Back** (x1,5), **combos** et **Perfect Clear** - avec annonces,
  animation d'effacement, traînée de hard drop, particules et effet de montée de
  niveau.
- **Duel avec lignes de déchets** : les lignes effacées envoient des déchets à
  l'adversaire (Tetris = 4, T-Spin Double = 4 …), les déchets entrants sont
  annoncés dans une barre d'alerte et **compensés** par tes propres attaques ;
  les deux terrains reçoivent la même suite de pièces. L'**IA** existe en 3
  niveaux et la vitesse augmente toutes les 40 secondes.
- **Commandes** : Gauche/Droite avec **DAS/ARR** personnalisés (réglables dans le
  setup), Haut = rotation à droite, Bas = soft drop, Action = hard drop ;
  **C**/Maj = réserve, **Z**/**Y** = rotation à gauche, **X** = rotation à
  droite. À deux, J1 met en réserve avec **Q** et tourne à gauche avec **E**, J2
  avec **Maj droite** / **Ctrl droite**. Après la partie : **R** = rejouer,
  **S** = setup.

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

**2048**
- **Écran de réglages dédié** avec des plateaux de **3x3 à 8x8** et trois modes :
  *Classique* (objectif 2048, puis « Continuer ? »), *Contre-la-montre* (3
  minutes, le chrono démarre au premier coup) et *Infini*.
- **Animations fluides** : les tuiles glissent, fusionnent avec un « pop » et
  apparaissent en grandissant ; pop-ups de points, étincelles dès 128, onde de
  choc dès 2048 et nouvelles couleurs jusqu'à 131072. Les saisies pendant une
  animation sont mises en mémoire.
- **Annuler** (désactivé / 3 par partie / illimité, touche **U** ou Retour
  arrière) - l'utiliser exclut la partie du record et des succès de tuiles.
- **Sauvegarder et reprendre** : la partie en cours est sauvegardée
  automatiquement par taille et par mode ; meilleurs scores et plus grande tuile
  par taille/mode sont dans la section `g2048` de `mem.json`.
- Commandes : flèches/WASD ou **balayage** à la souris/au pavé tactile,
  **R**/**N** = nouvelle partie, **Tab** = réglages. Le record ne compte qu'en
  **4x4 Classique** sans annulation.

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
- **4 variantes** de **400 niveaux** chacune (4 difficultés x 100) : *Classique*
  (les niveaux à graine connus - les niveaux résolus restent cochés), *Sudoku X*
  (les deux diagonales contiennent chaque chiffre une seule fois), *Killer*
  (cages en pointillés avec somme ; 400 niveaux générés à l'avance) et *Mini
  6x6*. Chaque puzzle a une **solution unique** - le niveau 12 de « Difficile »
  est le même puzzle sur chaque PC.
- **Sudoku du jour** : une grille par jour pour tout le monde, identique sur PC
  et dans le navigateur ; la difficulté dépend du jour de la semaine (du lundi
  Facile au samedi Expert), et résoudre chaque jour construit une série.
- **Jusqu'à 3 étoiles par niveau** (résolu · sans erreur ni indice · en plus
  sous le temps cible) et **meilleur temps** dans la sélection ; les **grilles
  commencées** sont sauvegardées automatiquement et reprises la fois suivante.
- **4 modes de jeu** (choisis avant de commencer) avec multiplicateur de
  score : **Classique** (x2,0 - sans aides), **Notes** (x1,5 - + notes au
  crayon et candidats automatiques), **Confort** (x1,0 - + erreurs en rouge,
  conflits et sommes de cage fausses marqués, les entrées correctes se
  verrouillent), **Assistant** (x0,7 - + indice, max. 3). Avec la **limite de
  3 erreurs** activée (option du setup), la troisième erreur met fin à la
  partie.
- Commandes : flèches/WASD = case, **1-9** = chiffre (pavé numérique aussi),
  **0/Suppr/clic droit** = effacer, **U**/**Z** = annuler, **Y** = rétablir,
  **N** = notes, **C** = candidats automatiques, **H** = indice, **M** =
  marqueur de couleur, **R** = recommencer le niveau, **Q** = sélection des
  niveaux. Saisie « chiffre d'abord » (setup, **I**) et **compteur de chiffres
  restants** sous chaque chiffre ; entièrement jouable à la souris. Après la
  fin, **A** affiche la **solution** complète.
- Points = (base de la variante et de la difficulté - temps - erreurs -
  indices) x multiplicateur du mode ; toutes les variantes et le Sudoku du jour
  comptent pour le record.

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
- **Jetons Lama** : le Blackjack joue avec le compte de la **Banque Lama**,
  partagé avec le Poker et le Casino (départ 1000, enregistré durablement dans
  `mem.json`). La mise est débitée dès la distribution ; sous 10 jetons,
  Entrée accorde un **crédit bancaire** qui remonte le compte à 1000.
- **Record** = plus haut niveau de ton **bilan au Blackjack** (1000 plus tout ce
  qui a été gagné et perdu au Blackjack) - les gains à la roulette, à la machine
  à sous ou au poker ne comptent pas ici, les crédits non plus.
- Manipulation par boutons de jetons et touches (**H**it/**S**tand/**D**ouble/
  partager **X**, **1-4** = mise, Retour arrière = annuler la mise, Entrée =
  distribuer) avec animations de cartes ; la carte cachée du croupier se
  retourne désormais vraiment.

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
- Devine le mot caché ; retour coloré (vert/jaune/gris) avec un **comptage
  correct des lettres doubles** et un clavier à l'écran qui se colore (AZERTY en
  français, QWERTZ en allemand, tchèque, slovène et croate, sinon QWERTY).
- **Quatre modes** : *Sans fin* (un mot après l'autre avec 6 essais chacun ;
  chaque mot trouvé rapporte des points, le premier raté termine la partie), *Mot
  du jour* (un mot par jour par langue et longueur - le même sur PC et dans le
  navigateur - avec compte à rebours et série ; un mot du jour commencé est
  sauvegardé), *Dordle* (2 mots à la fois en 7 essais) et *Quordle* (4 mots en 9
  essais, les touches montrent les couleurs de toutes les grilles).
- **Réglages** avant chaque partie : **longueur de 4 à 7 lettres**, **mode
  difficile** (les indices trouvés doivent être réutilisés) et **palette
  daltonien** (orange/bleu) ; les statistiques s'affichent à côté.
- **Vraies listes de mots dans les 14 langues** (dossier `woordlistz/`, A-Z
  uniquement), avec des listes propres à chaque longueur : pour 5 lettres, près de
  **34 000 solutions** et plus de **213 000 mots acceptés**, environ 134 000
  solutions toutes longueurs confondues. Les solutions sont des mots courants,
  sans noms propres, restes d'anglais ni mots choquants ; chaque essai est vérifié
  dans la liste - sinon il est refusé et la ligne tremble brièvement.
- **Statistiques** par langue, longueur et mode : parties, taux de victoire, série
  actuelle et meilleure série et **répartition des essais en histogramme**
  (section `wordle` de `mem.json`). **Partager** (**C**) copie une grille d'emojis
  sans dévoiler la solution.
- Le record ne compte que *Sans fin* en 5 lettres, les autres longueurs ont leurs
  propres records. Succès **Voyant** (2 essais au plus), **Accro aux mots** (7 mots
  du jour d'affilée) et **Génie au carré** (Quordle résolu).

**Poker**
- **3 variantes** sur l'écran de préparation : **Texas Hold'em** contre 1 à 3
  adversaires IA avec bouton de donneur, blinds et quatre tours d'enchères, **5
  Card Draw** (en tête-à-tête contre l'IA, un échange de cartes) et **Vidéo
  Poker** (*Jacks or Better*, en solo contre la table de gains).
- Actions par boutons ou touches : **F** = se coucher, **C** = parole/suivre,
  **R** = relancer, **A** = tapis ; garder/échanger des cartes par clic ou avec
  **1-5**, **Entrée** tire ou distribue la main suivante.
- **Jetons Lama** de la **Banque Lama** commune : au début d'une main, ton compte
  est posé sur la table comme tapis, et ce qui part au pot est débité aussitôt -
  quitter la table en pleine main ne coûte que ta part du pot. À sec (moins que
  la grosse blind de 20, moins de 10 au Vidéo Poker) = crédit bancaire qui
  remonte le compte à 1000.
- **Record** = plus haut niveau de ton **bilan au Poker** (1000 plus tous les
  gains et pertes au poker) ; le succès **Chip leader** ne compte lui aussi que
  le bilan au poker.

**Échecs**
- **Échecs complets** : tous les déplacements, y compris **roque**, **prise en
  passant** et **promotion** (pièce au choix) ; **échec, mat et pat** ainsi que
  nulle par la **règle des 50 coups**, la **triple répétition**, le **matériel
  insuffisant** ou d'un commun accord.
- **Trois modes** : *partie* contre l'IA, *2 joueurs* sur le même ordinateur (le
  plateau peut pivoter après chaque coup) et **problèmes**.
- **IA plus forte et sans saccades** en 6 niveaux de *Débutant* à *Maître* :
  approfondissement itératif, table de transposition, recherche de quiescence,
  livre d'ouvertures et évaluation avec mobilité, structure de pions et sécurité
  du roi. L'IA calcule par petites tranches à chaque image - le jeu ne saccade
  jamais.
- **Réglages** : choix de la couleur, **pendule** (aucune, 1+0, 3+2, 5+0, 10+5)
  et **Chess960** (les 960 positions de départ, le numéro s'affiche au-dessus de
  la liste des coups).
- **Barre latérale** avec pendules, pièces prises, bilan matériel et **liste des
  coups (SAN)** défilable ; glisser-déposer, pièces qui glissent, coordonnées.
  Touches : **U** = annuler, **H** = flèche d'indice, **O** = proposer nulle,
  **X** = abandonner, **F** = retourner le plateau, après la partie **P** =
  **export PGN**.
- **Problèmes** : 200 problèmes en 5 niveaux (mat en 1/2/3, tactique I/II) issus
  de la **base de problèmes libre de Lichess (CC0)**, vérifiés avec le moteur du
  jeu ; dans les problèmes de mat, chaque coup qui mate compte. La progression
  est dans la section `chess` de `mem.json`.
- Annuler et indice rendent une partie « assistée » : le record ne compte que les
  victoires sans aide contre l'IA (par session).

**Tower Defense**
- **Défense de vagues sans fin** sur **4 cartes** (Prairie, Canyon, Croisement,
  Gantelet), chacune avec son propre chemin ; les cartes verrouillées se
  débloquent avec ta meilleure vague, un **boss** arrive toutes les **8 vagues**.
- **3 modes** : Classique (7 tours, le mode principal), Compact (4 tours,
  2 niveaux) et Maximal (**11 tours**, **spécialisation A/B** au niveau maximum,
  ennemis spéciaux, capacités actives **Météore/Nova de givre/Ruée vers l'or**).
- **11 types de tours**, des flèches au laser et à la banque d'or, jusqu'à
  **3 niveaux d'amélioration** chacune, vente remboursée à 70% ; ennemis avec
  armure, régénération, division, camouflage, aura de soin et route aérienne.
- **Économie** : or par élimination, bonus de vague + 5% d'intérêts ; points par
  élimination et par vague. **F** = vitesse x2, **G** = portées, clic droit
  annule.

**Minigolf**
- **360 trous sur 40 parcours** : *Classic* et *Pro* avec 9 trous construits à
  la main chacun, la **Tour** avec 38 parcours de 9 trous générés (342 au total)
  à difficulté croissante, plus *Random* tiré de l'ensemble. Le parcours 7,
  trou 3 est partout identique - rien n'a besoin d'être enregistré.
- **Surfaces et obstacles** : le sable freine, les rampes accélèrent, l'eau coûte
  un coup de pénalité, les pare-chocs renvoient de la vitesse, moulins et blocs
  mobiles demandent du timing. La physique tourne en sous-pas avec frottement
  comme au billard - rien ne saute, rien ne traverse une bande.
- **Commandes** : la souris vise, le clic gauche maintenu charge la puissance
  et le relâchement joue le coup (flèches + espace au choix). **R** annule un
  coup chargé sans jouer. **G** bascule la ligne de visée, **Z** la visée auto,
  **P** le ramassage.
- **Blocage de puissance (maintenir le clic droit)** : fige la barre de charge
  là où elle est - dorée, avec le pourcentage, un cadenas et un anneau qui pulse
  autour de la balle. Tu attends ainsi l'ouverture du moulin avec le coup déjà
  chargé. En relâchant, la charge reprend ; la puissance bloquée survit même au
  coup, et le prochain clic gauche joue exactement cette valeur.
- **Carte de score** à droite avec le par et les coups par trou ; à deux, chacun
  joue le même trou à son tour. Points : 600 par trou, ±300 par coup sous/au-dessus
  du par, **500 de plus pour un trou en un**. Le nombre de coups le plus bas par
  parcours est dans la section `minigolf` de `mem.json`.
- **Le ramassage est désactivable** : par défaut, un trou s'arrête après huit
  coups et est compté au minimum. Si tu préfères jouer jusqu'à ce que la balle
  tombe, mets *Ramassage* sur AUS dans les réglages (ou appuie sur **P**).
- **La visée auto est désactivable** : par défaut, le club se tourne vers le
  trou avant chaque coup. Si tu préfères viser chaque trou toi-même, mets
  *Visée auto* sur NON dans les réglages (ou appuie sur **Z**) - la dernière
  direction choisie reste alors en place, et au départ d'un nouveau trou le
  club pointe neutrement vers le haut.
- **F** réinitialise le trou en cours : coups à 0, balle au départ - même trou,
  même parcours.
- **Suite au lieu de répétition** : à la fin d'une manche, le bouton **Suite** mène
  au parcours suivant (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), donc jamais les
  mêmes neuf trous ; à côté : **Rejouer** (même parcours) et **Réglages**.
  Touches : Entrée = suite, R = rejouer, S = réglages.
- **Replay de la manche** : à la fin, **P** (ou le bouton **Replay**) rejoue
  toute la manche coup par coup. **S** la range dans l'archive (bouton
  **Replays** de la barre latérale).
- **Créer et partager ses propres trous** : l'onglet **MAPS** de l'écran de
  préparation mène à ta collection - **Nouveau** ouvre l'éditeur. Chaque trou
  reçoit un nom et un **id** (minuscules, sans espaces) ; l'id est aussi le nom
  de fichier proposé au partage. Aux sept obstacles classiques s'ajoutent
  **huit nouveaux** : tuyau (téléporte la balle à l'autre bout), glace, zone
  collante, booster, aimant, porte à sens unique, plateau tournant et
  tremplin. La taille du trou se règle librement (60x80 à 160x240), **12
  modèles** servent de point de départ, annuler/rétablir et **Test** sont de
  la partie. **Partager** écrit exactement un trou dans un fichier
  `.lamapgzmap` - par la boîte de dialogue ou directement dans Téléchargements,
  avec ton nom de créateur. **Importer** le relit et bascule automatiquement
  sur `-2` si l'id est pris. Nom, id et créateur passent toujours par un
  **filtre de mots couvrant les 14 langues**. On joue un trou seul via
  **Jouer**, ou toute la collection via le cinquième parcours **Perso**.

**Pinball**
- **Trois tables** : *Classic* (trois bumpers, une série de cibles), *Space*
  (quatre bumpers en losange, deux séries) et *Lama* (plateau ouvert, six cibles
  en arc) ; 3 ou 5 billes par partie, à deux en alternance bille par bille.
- **Tout ce qu'il faut à un flipper** : couloir de lancement avec jauge (tir trop
  faible ? la bille revient et tu recommences), deux flippers, slingshots, séries
  de cibles, quatre couloirs **L-A-M-A**, trou de verrouillage, **multibille avec
  jackpot**, six secondes de **sauvegarde de bille**, secousse et **TILT**.
- **Multiplicateur jusqu'à x5** grâce aux séries abattues et aux couloirs
  complets ; bumpers 100, slingshots 50, cibles 250 - en multibille les bumpers
  paient 2 500 en jackpot.
- Flippers sur les touches gauche/droite assignées (et [Shift] gauche/droite) ou
  à la souris. Le record par table est dans la section `pinball` de `mem.json`.

**Bowling**
- **Dix frames selon les règles officielles**, strikes, spares et lancers bonus
  du dixième frame compris (maximum : 300). La **feuille de score** sous l'en-tête
  montre chaque frame avec X, / et le total courant.
- **Lancer en quatre étapes** : position, angle, effet et puissance. Chaque
  curseur oscille tout seul et se fige avec la touche d'action - ou se règle à la
  main avec gauche/droite, ce qui arrête l'oscillation.
- **Vraie physique des quilles** : dix quilles en cercles avec masse qui se
  renversent entre elles - un strike vient de la physique, pas de la chance. La
  piste est huilée à l'avant, le **hook** ne mord que dans le dernier tiers.
- Vue de piste en perspective avec dalots, flèches et pin deck ; trois difficultés
  (*Facile/Normal/Pro*) changent la vitesse des curseurs et la dispersion. Le
  record par difficulté est dans la section `bowling` de `mem.json`.
- **Replay de la partie** : à la fin, **P** rejoue tous les lancers, **S** les
  enregistre dans l'archive (bouton **Replays** de la barre latérale).

**Crossy Road**
- **Des sauts sans fin** par-dessus des prés (arbres et rochers barrent le
  passage), des routes avec voitures et camions, des rivières avec troncs et
  nénuphars et des **voies ferrées** où un train surgit après feu d'alerte et
  sonnerie - plus loin, de vraies gares avec jusqu'à 5 voies. Le parcours se
  construit rangée par rangée, a toujours un chemin praticable, et la vitesse
  comme le trafic augmentent.
- **Style voxel isométrique** : personnages, véhicules et arbres en blocs ombrés
  (pré-rendus pour chaque taille de case), caméra fluide, squash & stretch à
  chaque saut, éclaboussures, animation d'écrasement, plumes et pièces
  scintillantes ; dès la rangée 50, **cycle jour/nuit** avec phares.
- **L'aigle** : la caméra avance lentement - traîne trop ou recule de plus de
  trois rangées et l'aigle t'emporte (un bord rouge prévient avant). Dériver hors
  de l'écran sur un tronc, c'est aussi perdu.
- **Pièces et personnages** : les pièces ramassées (pièce géante = 5) sont
  sauvegardées et achètent de nouveaux personnages dans l'onglet
  **Personnages** : grenouille, cochon, pingouin, chat, renard, lama, robot,
  fantôme et licorne (25 à 250 pièces) ; le poulet est là dès le départ.
- **Modes** : *Sans fin* (points = rangée la plus lointaine, compte pour le
  record) et *Parcours du jour* (le même pour tous aujourd'hui, aussi dans le
  navigateur, avec son propre record du jour). Commandes : flèches/WASD,
  Espace/Entrée/clic = sauter en avant ; dans le setup **H** = ombres, **N** =
  jour/nuit. Pièces, personnages et record du jour sont dans la section `crossy`
  de `mem.json`.

**Geometry Dash**
- **Plateforme rythmique** : ton personnage fonce tout seul vers la droite - tu
  décides seulement quand sauter ou voler. **Cinq formes** - cube, vaisseau,
  boule, OVNI et onde -, plus des portails de forme, de gravité et de vitesse
  (0,5x à 3x), des **tremplins et orbes** jaunes/roses/bleus, demi-blocs, pics,
  fosses et déclencheurs de couleur.
- **8 niveaux intégrés** de *Facile* à *Démon* (« Lama Inferno ») avec **3 pièces
  secrètes** chacun. Chaque niveau est prouvé faisable : lors de sa création, un
  solveur l'a terminé avec le vrai code du jeu - pièces comprises, et même décalé
  de 1/240 de seconde.
- **Physique précise** : calcul en virgule fixe à pas fixe de 240 Hz ; chaque
  appui agit exactement dans le pas où il a eu lieu - identique à toute cadence
  d'images et au bit près dans le navigateur.
- **Mode entraînement** (**P**) avec points de contrôle automatiques et manuels
  (**Z** pour poser, **X** pour effacer), compteur de tentatives, barre de
  progression, explosions et redémarrage immédiat (**R**). Chaque niveau a sa
  **propre bande-son** - fond, sol et orbes pulsent en rythme (musique coupable
  avec **M**).
- **Étoiles et pièces** : termine un niveau en mode normal pour gagner ses
  étoiles, chaque pièce vaut une étoile de plus ; le record est le **total
  d'étoiles** (65 au maximum). Records par niveau, pièces, tentatives et sauts
  sont dans la section `geodash` de `mem.json`.
- **Éditeur de niveaux** dans l'onglet **NIVEAUX** : toile quadrillée, palette de
  6 groupes (blocs, dangers, tremplins et orbes, portails, vitesse, extras),
  rotation, annuler/rétablir, barre d'aperçu, **test depuis le début ou depuis
  ici** et réglages du niveau (vitesse et forme de départ, style musical, BPM,
  couleurs). La coche **« vérifié »** n'apparaît qu'une fois ton propre niveau
  réussi. **Partager** écrit un fichier `.lamapgzlevel`, **Importer** le relit ;
  les niveaux sont stockés dans `ugc.json` à côté de tes trous de minigolf.

**Battleship**
- **Bataille navale en 10x10** avec porte-avions (5 cases), cuirassé (4),
  croiseur (3), sous-marin (3) et destroyer (2) - le premier à couler toute la
  flotte adverse gagne.
- **Placer la flotte** par glisser-déposer depuis le dock : **R** ou clic droit
  fait pivoter, l'aperçu brille en vert ou en rouge, **X** place tout au hasard,
  **C** vide le plateau ; ton dernier placement est reproposé la manche suivante.
- **Règles dans le setup** (sauvegardées) : *les navires peuvent se toucher*,
  *salve* (autant de tirs par tour que de navires encore à flot) et *tirer encore
  après une touche*.
- **IA à 3 niveaux** : Facile tire au hasard, Moyen traque méthodiquement les
  touches, Difficile calcule une **carte de probabilités** avec parité en damier
  (en moyenne environ 70 / 60 / 45 tirs pour toute une flotte). Ou **2 joueurs**
  sur le même ordinateur - un **écran de passation** cache les deux flottes avant
  chaque tour.
- **Visuels** : balayage radar, vagues animées, obus en cloche, gerbes d'eau,
  explosions avec fumée et cases en feu, révélation « COULÉ ! » et bilan de fin
  de manche avec tirs, touches et précision. Le record compte tes **victoires
  contre l'IA** dans une session.

**Casino**
- **Roulette** (européenne, 37 cases) : toutes les mises classiques en cliquant
  un numéro, un bord ou un coin - **plein** (35:1), cheval, transversale, carré,
  sixain, colonne, douzaine, rouge/noir, pair/impair et manque/passe. Jetons de
  1/5/25/100/500, clic droit pour retirer des jetons ; **Lancer**, **Rejouer**
  (**R**), **Doubler** (**D**) et **Effacer**. La bille tourne en spirale jusqu'à
  la case tirée à l'avance, et les 12 derniers numéros s'affichent en haut.
- **Machine Lama** : 5 rouleaux x 3 rangées, **10 lignes de gain**, **lama =
  wild**, **pièces d'or = scatter** avec 10 tours gratuits à gains doublés, mise
  par ligne 1/2/5/10, **tours automatiques** (10/25), **turbo** et table des
  gains. Le **taux de redistribution est de 96,1 %** - calculé exactement à partir
  des bandes des rouleaux.
- **Banque Lama** : Casino, Blackjack et Poker partagent un compte de **jetons
  Lama** (départ 1000, section `casino` de `mem.json`) ; les anciens soldes sont
  repris automatiquement. Les mises sont débitées aussitôt, chaque jeu tient son
  propre bilan pour son record, et à sec tu reçois un **crédit bancaire** qui
  remonte le compte à 1000.
- Confettis, pluies de pièces, bannières gros gain/méga/jackpot et animations des
  lignes gagnantes ; succès **En plein dans le mille** (plein gagnant à la
  roulette) et **Jackpot Lama** (5 lamas sur une ligne).

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
  affiche le record précédent sous forme de puce. Avec beaucoup de modes et une
  petite résolution, il devient **compact** : Options, Wiki et Retour passent sur
  une seule ligne et la police s'adapte - plus rien ne déborde de l'écran.
- **Look unifié en jeu** : les 46 jeux partagent la palette et la police du
  menu - les HUD, écrans de préparation et superpositions suivent le design
  choisi dans les options (v4.1 / v4 / Classique), tandis que chaque terrain
  garde ses couleurs d'identité. Chaque jeu gère proprement un changement de
  résolution en cours de partie, et les noms des jeux dans le menu s'adaptent à
  la langue (p. ex. « Schach » → « Échecs »).
- **Wiki intégré** (« LamaWiki ») : aide détaillée pour chaque jeu (commandes,
  modes, points, astuces) plus des pages générales - avec **champ de
  recherche**, catégories, articles défilables et puces de touches, dans les
  14 langues. Accessible via le bouton **« Wiki / Aide »** de la barre
  latérale et depuis l'écran de préparation de chaque jeu (ouvre directement sa
  page).
- **Succès & statistiques** : **107 succès** en trois catégories (23 objectifs
  globaux, 37 paliers de points et 47 moments spéciaux comme un échec et mat
  contre l'IA, la tuile 4096, un T-Spin Double, 25 problèmes d'échecs résolus,
  un Killer Sudoku ou le jackpot lama ; dans 2048 et aux échecs, les parties
  avec annulation ou indices ne comptent pas) avec **notification dorée et fanfare**
  au déblocage - même en pleine partie ; les anciens records sont crédités
  automatiquement. Plus un onglet **Statistiques** : temps de jeu total,
  parties, victoires, records, jeu préféré et tableau par jeu trié par temps
  de jeu. Accessible via le bouton **« Succès & statistiques »** de la barre
  latérale.
- **Replays** : le minigolf et le bowling enregistrent chaque manche. À la fin,
  **P** montre la rediffusion et **S** la range dans l'archive - accessible par
  le bouton **Replays** de la barre latérale (un onglet par jeu, pause, saut de
  séquence, vitesse 0,5x à 4x). Désactivable au premier démarrage et dans les
  options.

### Prise en main

- Choisis le jeu avec le bouton du menu de gauche. Ensuite apparaît l'**écran
  de préparation** : choisir **Un joueur** ou **Multijoueur**, aller aux
  **options** ou revenir. Flèches/souris pour choisir, Entrée démarre.
- **ÉCHAP** = pause / reprendre (dans les menus : retour).
- **F11** (ou le bouton « Plein écran oui/non ») = plein écran. L'affichage
  Pygame reste intégré et est agrandi en conservant les proportions (bandes
  noires si le rapport diffère). La fenêtre se redimensionne librement.
- **« Retour au menu »** termine le jeu et enregistre le meilleur score - tout
  comme passer à un autre jeu via la barre latérale.
- **Touches fixes supplémentaires** : en plus des cinq actions réaffectables,
  certains jeux ont leurs propres touches (par ex. réserve **C** et rotation à
  gauche **Z** dans Tetris, annuler **U** dans 2048, aux échecs et au Sudoku).
  Elles ne fonctionnent que si la touche n'est affectée à aucune action dans les
  options et sont indiquées dans l'aide du setup et dans le wiki. Les touches
  maintenues sont bien détectées et relâchées à la pause ou avec Alt-Tab - plus
  rien ne « colle ».
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
- **Apparence** : choisir le **design de l'interface** – **UI v4.2** (par
  défaut : Midnight Glass – dégradé nuit profonde avec des lueurs mouvantes
  en indigo, turquoise et magenta, un fin grain de film, quelques étoiles
  éparses et des panneaux en verre dépoli à liseré lumineux), **UI v4.1**
  (comme UI v4 mais plus vivante – étoiles discrètes plus Saturne et un trou
  noir en arrière-plan de l'écran d'accueil), **UI v4.1.1** (comme v4.1,
  mais un **motif zigzag** en carrelage noir et anthracite à la place du
  ciel étoilé), **UI v4.1.2** (le même motif dans les bleus de la palette –
  bleu d'accent dominant, bleu plus sombre en fond), **UI v4.1.3** (le même
  motif dans l'indigo d'UI v4 sur fond noir), **UI v4.1.4** (dans le
  graphite d'UI v4 sur fond noir), **UI v4** (un look graphite épuré, plat
  et parfaitement calme avec un seul accent indigo), **UI v3** (l'ancienne
  interface classique avec ciel étoilé, aurores et halos lumineux),
  **UI v2** (la toute première refonte de l'interface : dégradé marine, ciel
  étoilé et boutons lumineux, sans aucune animation) ou **UI v1** (le look
  d'avant la refonte : fond sombre uni, boutons plats, aucun effet). Toutes
  les cartes montrent un petit aperçu ; le choix s'applique immédiatement à
  toute l'interface (zone de jeu **et** barre latérale) et est enregistré.

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
settings.py          Charger/enregistrer les réglages (son/vibration/touches/options de jeu avec règles de contrôle) (JSON)
audio.py             Effets sonores procéduraux, boucles musicales + vibration de manette
menu.py              Écrans de langue, de préparation (mode) et d'options (son/commandes)
highscore.py         Charger/enregistrer les meilleurs scores (section dans mem.json)
store.py             Fichier de sauvegarde central mem.json (sections : mem, highscores, stats, achievements + progression des jeux), atomique avec copie .bak
stats.py             Statistiques du joueur (parties, temps de jeu, victoires, records) par jeu
achievements.py      Succès : définitions, logique de déblocage, notification (toast)
progress.py          Écran Succès & statistiques (deux onglets, défilable)
replay.py            Enregistrement et archive des rediffusions (replay.json)
replayview.py        Écran Replay : liste de l'archive et lecture
ugc.py               Contenus personnels (trous de minigolf, niveaux Geometry Dash) : stockage, vérification, export/import (ugc.json)
swear.py             Filtre de mots pour noms et id (lang/swear/*.yml, les 14 langues)
filepick.py          Boîtes de dialogue de fichiers ("Exporter sous ...", "Importer")
prestige.py          Système de prestige de Snake
competitive.py       Paramètres du mode Compétitif de Snake (niveaux, machine à sous, pommes de pari)
ngb.py               Personnalisation visuelle (« mods ») : couleur de tête + grille + menu (mem-ngb.json)
lamabank.py          Banque Lama : compte de jetons commun au Blackjack, au Poker et au Casino (section casino de mem.json)
seedrand.py          Générateur aléatoire aux nombres identiques au bit près en Python et dans le navigateur (modes du jour, nouveaux problèmes)
i18n.py              Moteur de traduction (charge lang/*.json, t("clé"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textes (une clé par texte)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Listes du filtre de mots par langue (regex, .yml)
lamawiki/
  lamawiki.py          Wiki intégré (recherche, catégories, rendu d'articles)
  de.json  en.json  fr.json  es.json  pt.json   Contenu du wiki (une page par jeu + pages générales)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Reconstruit les listes de mots de Wordle (dictionnaires + fréquences)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 lettres), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 lettres), 14 langues
devtools/            Outils de développement (non inclus dans la .exe)
  merge_staging.py           Intègre traductions et pages wiki de devtools/staging/ dans les 14 fichiers de langue
  build_chess_puzzles.py     Construit les 200 problèmes d'échecs depuis la base de problèmes Lichess (CC0)
  build_sudoku_killer.py     Génère les 400 Killer Sudokus à solution unique
  build_crossyroad_models.py Écrit les modèles voxel de Crossy Road pour la version web
  build_geodash_levels.py    Construit les 8 niveaux de Geometry Dash et prouve avec le solveur que chacun est faisable, pièces comprises
  build_geodash_solver.py    Solveur utilisant le vrai code de pas (solutions dans geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Données de niveaux : snake-comp.json, chess-puzzles.json (+ README des sources), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Audit global (saisie, seedrand Python = JS, sauvegarde, fichiers de langue, écrans de préparation) + tous les audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Audits headless par jeu
  newgames_audit.py  blockjump_audit.py
```

La langue choisie est enregistrée dans `mem.json` (dans la section `mem`, à
côté de la section `highscores` du même fichier) et chargée automatiquement au
prochain démarrage.

**Sources et licences :** les 200 problèmes d'échecs proviennent de la
[base de problèmes Lichess](https://database.lichess.org/#puzzles) (licence
**CC0 1.0**, domaine public - merci à lichess.org !) ; les détails sont dans
`games/levels/chess-puzzles.README.md`. Les sources des listes de mots de Wordle
sont indiquées dans `woordlistz/README.md`.

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
Tkinter. Cuarenta y seis juegos con opciones compartidas, controles totalmente
reasignables, récords, efectos de sonido procedurales y, en varios títulos, modo
multijugador. La interfaz es **multilingüe** – **14 idiomas** (alemán / inglés /
francés / español / portugués / polaco / turco / danés / noruego / sueco / finés /
checo / esloveno / croata); el idioma se elige en una **pantalla de bienvenida**
en el primer arranque, que también permite ajustar la **resolución** y el
**sonido** (desactivado por defecto); aparte de los tres idiomas principales,
todos los demás están tras el botón **«Más»**. Todo se puede cambiar en
cualquier momento en las opciones.

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
| **Tetris**   | 1 / 2 jugadores | Reglas Guideline modernas (SRS, reserva, vista previa de 5 piezas, T-Spins): Maratón, Sprint 40, Ultra 2:00, Versus contra la IA (3 niveles) o a dos con líneas basura |
| **Invaders** | 1 jugador       | Space Invaders: vacía las oleadas, protege tus vidas |
| **Asteroids** | 1 / 2 jugadores | Física de inercia, oleadas, OVNIs, power-ups, hiperespacio - solo o duelo cooperativo |
| **Pac-Man**  | 1 jugador       | Clon fiel: 4 IAs de fantasmas, píldoras de poder, túneles, frutas, niveles |
| **Flappy Bird** | 1 jugador    | Vuelo con gravedad entre tuberías, monedas, escudo, día/noche, medallas |
| **Doodle Jump** | 1 jugador    | Salto automático hacia arriba, tipos de plataforma, muelles, hélice, monstruos |
| **2048**     | 1 jugador       | Puzle de números deslizantes de 3x3 a 8x8: Clásico, Contrarreloj e Infinito, deshacer, animaciones fluidas, partidas guardadas |
| **Minesweeper** | 1 jugador    | El clásico con primer clic seguro, chording, smiley y mejores tiempos |
| **Sudoku**      | 1 jugador    | 4 variantes (Clásico, Sudoku X, Killer, Mini 6x6) de 400 niveles cada una, Sudoku del día, hasta 3 estrellas por nivel, 4 modos de asistencia, deshacer, partida guardada |
| **Frogger**     | 1 jugador    | Carretera + río + 5 bahías, mosca de bonus, cocodrilos, límite de tiempo, 3 dificultades |
| **Memory**      | 1 / 2 jugadores | Encuentra parejas en 4x4 hasta 8x6, animación de volteo, solo o duelo |
| **Solitario**   | 1 jugador    | 5 variantes (Klondike, Spider, FreeCell, Pirámide, TriPeaks) con arrastrar y soltar y deshacer |
| **Aim Trainer** | 1 jugador    | Tiro al blanco 3D relajado: el ratón dirige la cámara, 4 modos (precisión/reflejos/móviles/chill), 3 temas incl. un agujero negro |
| **Cuatro en raya** | 1 / 2 jugadores | El clásico con animación de caída: 3 niveles de IA (minimax) o duelo local |
| **Duelo de tanques** | 1 / 2 jugadores | Duelo 2D en arena con disparos con rebote, power-ups, 4 arenas, IA con 3 niveles |
| **Blackjack**    | 1 jugador    | Blackjack de casino con zapato de 4 barajas, doblar/dividir y blackjack 3:2; juega con las fichas Llama del Banco Llama común |
| **Tunnel Racer** | 1 jugador    | Vuelo 3D por un tubo de neón: modo sin fin + 30 niveles, control por teclas o ratón, motion blur |
| **Laberinto 3D** | 1 jugador    | Raycaster en primera persona (estilo Wolfenstein) con 50 niveles con semilla, orbes, minimapa - o vista cenital 2D |
| **Reversi**      | 1 / 2 jugadores | Othello en 8x8: atrapar y voltear fichas, 3 fuerzas de IA (minimax) o un duelo local |
| **Yahtzee**      | 1 / 2 jugadores | Clásico de dados con 13 categorías, bono superior y Yahtzee; carrera por el récord o hotseat a 2 |
| **Wordle**       | 1 jugador    | Adivina palabras de 4 a 7 letras: Infinito, Palabra del día, Dordle y Quordle, modo difícil, paleta para daltónicos, estadísticas con gráfico de barras, compartir resultado, listas reales en 14 idiomas |
| **T-Rex Runner** | 1 jugador    | Carrera infinita por el desierto: salto variable, agacharse, cactus y pterodáctilos, ciclo día/noche, velocidad creciente, 3 dificultades |
| **Damas**        | 1 / 2 jugadores | 3 reglamentos a elegir (alemanas 8×8, internacionales 10×10, checkers), captura obligatoria y dama voladora, 3 fuerzas de IA (minimax) o duelo local |
| **Póker**        | 1 jugador    | 3 variantes a elegir: Texas Hold'em contra la IA, 5 Card Draw y Video Póker; rondas de apuestas, ciegas, fichas Llama del Banco Llama común |
| **Ajedrez**     | 1 / 2 jugadores | Reglas completas, Chess960 y reloj, 6 niveles de IA, 200 problemas de la base de Lichess, deshacer/pista, lista de jugadas, exportación PGN o duelo local |
| **Molino**      | 1 / 2 jugadores | Fases de colocar, mover y volar, molinos y capturas, vuelo desactivable, 3 niveles de IA o duelo local |
| **Simon**       | 1 / 2 jugadores | Juego de memoria Senso: modos Clásico/Speed/Reverse/Mixto + duelo, sonido sí/no/mixto, 4/6/9 casillas, récord por modo |
| **Billar**      | 1 / 2 jugadores | Bola 8, bola 9 y práctica en 2D, vista 3D fija o cámara 3D libre; física suave, ayuda de puntería, IA de 3 niveles |
| **Puzle deslizante** | 1 jugador  | Puzle-15 en 3x3/4x4/5x5: desliza las fichas numeradas al hueco, control con ratón o flechas, puntos por movimientos y tiempo |
| **Mastermind**       | 1 jugador  | Descifra el código de color secreto (3 modos: 4×6, clásico, 5×8), fichas de pista negras/blancas, racha sin fin |
| **Bubble Shooter**   | 1 jugador  | Clon de Puzzle Bobble: dispara colores iguales en grupos de tres, rebotes en las paredes, racimos que caen, 3 dificultades |
| **Hangman**          | 1 jugador  | Adivina la palabra antes de completar la horca; teclado en pantalla, listas de palabras por idioma, 3 modos de longitud, racha sin fin |
| **Block Jump**       | 1 jugador  | Plataformas 3D estilo Minecraft: mundo de bloques texturizados con skin de Minecraft y figura de Steve, escaleras, vallas y bloques-resorte, cámara 1ª/3ª persona, desenfoque, niveles generados |
| **Tower Defense**    | 1 jugador  | Repele oleadas infinitas en 4 mapas: hasta 11 tipos de torres con mejoras, venta y especialización A/B, jefes, 3 modos, habilidades activas |
| **Minigolf**    | 1 / 2 jugadores | 360 hoyos en 40 recorridos (18 hechos a mano, 342 generados): arena, rampas, agua, parachoques, molinos y bloques móviles; tarjeta con par y bonus de hoyo en uno; **editor de hoyos propio** con 15 tipos de objetos, 12 plantillas y compartir como `.lamapgzmap` |
| **Pinball**     | 1 / 2 jugadores | Máquina de pinball con 3 mesas: bumpers, slingshots, dianas, carriles L-A-M-A, multibola con jackpot, salvabolas, empujón y tilt |
| **Bowling**     | 1 / 2 jugadores | 10 frames con la puntuación oficial de strike/spare, física real de bolos, efecto hook y pista en perspectiva, 3 dificultades |
| **Crossy Road** | 1 jugador       | Saltar sin fin por praderas, carreteras, ríos y vías de tren con estilo vóxel isométrico: día/noche, águila, 10 personajes comprables, ruta del día |
| **Geometry Dash** | 1 jugador     | Plataformas rítmico con cubo, nave, bola, OVNI y onda: 8 niveles de Fácil a Demonio con 3 monedas secretas cada uno, modo práctica, banda sonora por nivel; **editor de niveles** con compartir como `.lamapgzlevel` |
| **Battleship**  | 1 / 2 jugadores | Batalla naval 10x10: flota colocada arrastrando, 3 reglas opcionales (contacto, salva, repetir disparo), IA de 3 niveles o duelo local con pantalla de relevo |
| **Casino**      | 1 jugador       | Ruleta europea con todas las apuestas clásicas y Máquina Llama (5 rodillos, 10 líneas, comodín, tiradas gratis); una sola cuenta de fichas Llama con Blackjack y Póker |

**El multijugador (2 jugadores en local)** está disponible en **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Cuatro en raya**, **Duelo de tanques**,
**Reversi**, **Yahtzee**, **Damas**, **Ajedrez**, **Molino**, **Simon (duelo)**,
**Billar**, **Minigolf**, **Pinball**, **Bowling** y **Battleship** (con una
pantalla de relevo que oculta las flotas): 20 juegos en total. El modo se elige
directamente en la pantalla previa (*Un jugador / Multijugador*); Tetris ofrece
además **Versus contra la IA**. La versión web es solo para un jugador.

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
- **Reglas Guideline modernas**: campo de 10x20, piezas de una **bolsa de 7**,
  **sistema de giro SRS** con wall kicks reales (también para la pieza I), giro
  en ambos sentidos, **reserva** (una vez por pieza), **vista previa de 5
  piezas**, pieza fantasma y **lock delay** (0,5 s, como máximo 15 reinicios).
- **Tres modos** en la pantalla previa: *Solo*, *Contra la IA* y *2 jugadores*.
  Solo ofrece en la configuración **Maratón** (nivel inicial 1-15, cuenta para el
  récord), **Sprint 40 líneas** (mejor tiempo) y **Ultra 2 minutos** (mejor
  puntuación); las marcas se guardan en la sección `tetris` de `mem.json`.
- **Puntuación Guideline**: de Single a Tetris, **T-Spins** (completos y mini),
  **Back-to-Back** (x1,5), **combos** y **Perfect Clear**, con avisos en
  pantalla, animación de borrado, estela de hard drop, partículas y efecto de
  subida de nivel.
- **Versus con líneas basura**: las líneas borradas envían basura al rival
  (Tetris = 4, T-Spin Double = 4 …), la basura entrante se anuncia en una barra
  de aviso y se **compensa** con tus propios ataques; ambos campos reciben la
  misma secuencia de piezas. La **IA** tiene 3 niveles y la velocidad sube cada
  40 segundos.
- **Controles**: Izq/Der con **DAS/ARR** propios (ajustables en la
  configuración), Arriba = girar a la derecha, Abajo = soft drop, Acción = hard
  drop; **C**/Mayús = reserva, **Z**/**Y** = girar a la izquierda, **X** = girar
  a la derecha. A dos jugadores, J1 reserva con **Q** y gira a la izquierda con
  **E**, J2 con **Mayús derecha** / **Ctrl derecho**. Tras la partida: **R** =
  otra vez, **S** = configuración.

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

**2048**
- **Pantalla de configuración propia** con tableros de **3x3 a 8x8** y tres
  modos: *Clásico* (meta 2048, después «¿Seguir jugando?»), *Contrarreloj* (3
  minutos, el reloj arranca con el primer movimiento) e *Infinito*.
- **Animaciones fluidas**: las fichas se deslizan, se fusionan con un «pop» y
  aparecen creciendo; puntos emergentes, chispas desde 128, onda expansiva desde
  2048 y nuevos colores hasta 131072. Lo que pulses durante una animación se
  guarda y se ejecuta después.
- **Deshacer** (desactivado / 3 por partida / ilimitado, tecla **U** o Backspace):
  quien lo usa juega sin récord y sin logros de fichas.
- **Guardar y reanudar**: la partida en curso se guarda automáticamente por
  tamaño y modo; mejores puntuaciones y ficha más alta por tamaño/modo están en
  la sección `g2048` de `mem.json`.
- Controles: flechas/WASD o **deslizar** con ratón/touchpad, **R**/**N** = nueva
  partida, **Tab** = configuración. El récord solo cuenta en **4x4 Clásico** sin
  deshacer.

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
- **4 variantes** de **400 niveles** cada una (4 dificultades x 100): *Clásico*
  (los niveles por semilla de siempre; los resueltos siguen marcados), *Sudoku X*
  (ambas diagonales contienen cada dígito una sola vez), *Killer* (jaulas
  discontinuas con suma; 400 niveles generados de antemano) y *Mini 6x6*. Todos
  los puzles tienen **solución única**: el nivel 12 de "Difícil" es el mismo
  puzle en cualquier PC.
- **Sudoku del día**: un puzle diario para todos, idéntico en PC y en el
  navegador; la dificultad depende del día de la semana (de lunes Fácil a sábado
  Experto), y resolverlo cada día crea una racha.
- **Hasta 3 estrellas por nivel** (resuelto · sin errores ni pistas · además por
  debajo del tiempo objetivo) y **mejor tiempo** en la selección; los **sudokus
  empezados** se guardan solos y se reanudan la próxima vez.
- **4 modos de juego** (antes de empezar) con multiplicador: **Clásico** (x2,0 -
  sin ayudas), **Notas** (x1,5 - + notas a lápiz y candidatos automáticos),
  **Confort** (x1,0 - + errores en rojo, conflictos y sumas de jaula erróneas
  marcados, entradas correctas se fijan), **Asistente** (x0,7 - + pista, máx.
  3). Con el **límite de 3 errores** activo (opción del setup) el tercer error
  acaba la partida.
- Controles: flechas/WASD = celda, **1-9** = dígito (también teclado numérico),
  **0/Supr/clic derecho** = borrar, **U**/**Z** = deshacer, **Y** = rehacer,
  **N** = notas, **C** = candidatos automáticos, **H** = pista, **M** = marcador
  de color, **R** = reiniciar nivel, **Q** = selección. Entrada «dígito primero»
  (setup, **I**) y un **contador de dígitos restantes** bajo cada dígito;
  jugable por completo con el ratón. Tras el final, **A** muestra la **solución**
  completa.
- Puntos = (base de variante y dificultad - tiempo - errores - pistas) x
  multiplicador del modo; todas las variantes y el Sudoku del día cuentan para
  el récord.

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
- **Fichas Llama**: el Blackjack juega con la cuenta del **Banco Llama**, que
  comparte con Póker y Casino (inicio 1000, guardada en `mem.json`). La apuesta
  se descuenta al repartir; con menos de 10 fichas, Enter pide un **crédito del
  banco** que devuelve la cuenta a 1000.
- **Récord** = máximo de tu **balance en Blackjack** (1000 más todo lo ganado y
  perdido en Blackjack): las ganancias en la ruleta, la tragaperras o el póker no
  cuentan aquí, y los créditos tampoco.
- Manejo con botones de fichas y teclas (**H**it/**S**tand/**D**ouble/dividir
  **X**, **1-4** = apuesta, Backspace = quitar apuesta, Enter = repartir) con
  animaciones de cartas; la carta tapada del crupier ahora sí se da la vuelta al
  descubrirla.

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
- Adivina la palabra oculta; respuesta de color (verde/amarillo/gris) con
  **conteo correcto de letras repetidas** y un teclado en pantalla que se colorea
  (QWERTZ en alemán, checo, esloveno y croata, AZERTY en francés, QWERTY en el
  resto).
- **Cuatro modos**: *Infinito* (una palabra tras otra con 6 intentos cada una;
  cada palabra resuelta da puntos y la primera sin resolver termina la partida),
  *Palabra del día* (una palabra al día por idioma y longitud, la misma en PC y en
  el navegador, con cuenta atrás y racha; la palabra del día empezada se guarda),
  *Dordle* (2 palabras a la vez en 7 intentos) y *Quordle* (4 palabras en 9
  intentos; las teclas muestran los colores de todos los tableros).
- **Configuración** antes de cada partida: **longitud de 4 a 7 letras**, **modo
  difícil** (hay que reutilizar las pistas encontradas) y **paleta para
  daltónicos** (naranja/azul); al lado se muestran las estadísticas.
- **Listas de palabras reales en los 14 idiomas** (carpeta `woordlistz/`, solo
  A-Z), con listas propias para cada longitud: solo con 5 letras, casi **34.000
  soluciones** y más de **213.000 intentos válidos**, unas 134.000 soluciones
  sumando todas las longitudes. Las soluciones son palabras comunes, sin nombres,
  restos de inglés ni palabras ofensivas; cada intento se comprueba con la lista y
  lo demás se rechaza mientras la fila tiembla un momento.
- **Estadísticas** por idioma, longitud y modo: partidas, porcentaje de victorias,
  racha actual y mejor racha y la **distribución de intentos en un gráfico de
  barras** (sección `wordle` de `mem.json`). **Compartir** (**C**) copia una
  cuadrícula de emojis sin revelar la solución.
- El récord solo cuenta *Infinito* con 5 letras; las demás longitudes tienen sus
  propias marcas. Logros **Clarividente** (2 intentos como máximo), **Hábito de
  palabras** (7 palabras del día seguidas) y **Genio cuádruple** (Quordle
  resuelto).

**Póker**
- **3 variantes** en la pantalla previa: **Texas Hold'em** contra 1-3 rivales de
  IA con botón de repartidor, ciegas y cuatro rondas de apuestas, **5 Card Draw**
  (mano a mano contra la IA, un descarte) y **Video Póker** (*Jacks or Better*,
  en solitario contra la tabla de pagos).
- Acciones con botones o teclas: **F** = retirarse, **C** = pasar/igualar, **R** =
  subir, **A** = all-in; mantener/cambiar cartas con clic o **1-5**, **Enter**
  roba o reparte la siguiente mano.
- **Fichas Llama** del **Banco Llama** común: al empezar una mano tu cuenta está
  en la mesa como pila, y lo que va al bote se descuenta al instante; dejar la
  mesa a mitad de mano solo te cuesta tu parte del bote. Sin fondos (menos que la
  ciega grande de 20, menos de 10 en Video Póker) = crédito del banco que
  devuelve la cuenta a 1000.
- **Récord** = máximo de tu **balance en Póker** (1000 más todas las ganancias y
  pérdidas en póker); el logro **Chip leader** también cuenta solo ese balance.

**Ajedrez**
- **Ajedrez completo**: todos los movimientos, incluidos **enroque**, **captura
  al paso** y **coronación** (pieza a elegir); **jaque, jaque mate y ahogado** y
  tablas por la **regla de los 50 movimientos**, **triple repetición**,
  **material insuficiente** o acuerdo.
- **Tres modos**: *partida* contra la IA, *2 jugadores* en el mismo ordenador (el
  tablero puede girar tras cada jugada) y **problemas**.
- **IA más fuerte y sin tirones** en 6 niveles de *Principiante* a *Maestro*:
  profundización iterativa, tabla de transposición, búsqueda de quiescencia,
  libro de aperturas y una evaluación con movilidad, estructura de peones y
  seguridad del rey. La IA calcula en pequeñas porciones por fotograma: el juego
  nunca da tirones.
- **Configuración**: elección de color, **reloj** (sin reloj, 1+0, 3+2, 5+0,
  10+5) y **Chess960** (las 960 posiciones iniciales, el número aparece sobre la
  lista de jugadas).
- **Barra lateral** con relojes, piezas capturadas, balance de material y una
  **lista de jugadas (SAN)** desplazable; arrastrar y soltar, piezas que se
  deslizan, coordenadas. Teclas: **U** = deshacer, **H** = flecha de pista,
  **O** = ofrecer tablas, **X** = rendirse, **F** = girar el tablero, tras la
  partida **P** = **exportar PGN**.
- **Problemas**: 200 problemas en 5 niveles (mate en 1/2/3, táctica I/II) de la
  **base de problemas libre de Lichess (CC0)**, comprobados con el motor del
  juego; en los problemas de mate vale cualquier jugada que dé mate. El progreso
  está en la sección `chess` de `mem.json`.
- Deshacer y pista marcan la partida como «asistida»: el récord solo cuenta las
  victorias sin ayuda contra la IA (por sesión).

**Tower Defense**
- **Defensa de oleadas sin fin** en **4 mapas** (Pradera, Cañón, Cruce,
  Guantelete), cada uno con su propio camino; los mapas bloqueados se
  desbloquean con tu mejor oleada, un **jefe** llega cada **8 oleadas**.
- **3 modos**: Clásico (7 torres, el modo principal), Compacto (4 torres,
  2 niveles) y Máximo (**11 torres**, **especialización A/B** al nivel máximo,
  enemigos especiales, habilidades activas **Meteoro/Nova de hielo/Fiebre del
  oro**).
- **11 tipos de torres**, de las flechas al láser y el banco de oro, hasta
  **3 niveles de mejora** cada una, la venta devuelve el 70%; enemigos con
  blindaje, regeneración, división, camuflaje, aura curativa y ruta aérea.
- **Economía**: oro por baja, bono de oleada + 5% de intereses; puntos por baja
  y oleada. **F** = velocidad x2, **G** = alcances, clic derecho cancela.

**Minigolf**
- **360 hoyos en 40 recorridos**: *Classic* y *Pro* con 9 hoyos hechos a mano
  cada uno, la **Tour** con 38 recorridos de 9 hoyos generados (342 en total) y
  dificultad creciente, más *Random* sacado de todo el conjunto. El recorrido 7,
  hoyo 3 se ve igual en todas partes; no hay que guardar nada.
- **Superficies y obstáculos**: la arena frena, las rampas aceleran, el agua
  cuesta un golpe de penalización, los parachoques devuelven velocidad y molinos
  y bloques móviles exigen ritmo. La física corre en subpasos con rozamiento como
  en el billar: nada salta ni atraviesa la banda.
- **Controles**: el ratón apunta, mantener el botón izquierdo carga la fuerza y
  soltarlo golpea (también flechas + espacio). **R** cancela un golpe cargado
  sin golpear. **G** cambia la línea de tiro, **Z** la puntería automática,
  **P** la recogida.
- **Bloqueo de fuerza (mantener el botón derecho)**: congela la barra de carga
  donde esté: dorada, con el porcentaje, un candado y un anillo que late
  alrededor de la bola. Así esperas el hueco del molino con el golpe ya cargado.
  Al soltar sigue cargando; la fuerza bloqueada sobrevive incluso al golpe y el
  siguiente clic izquierdo golpea justo con ese valor.
- **Tarjeta de puntuación** a la derecha con el par y los golpes por hoyo; a dos,
  cada jugador juega el mismo hoyo por turnos. Puntos: 600 por hoyo, ±300 por
  golpe bajo/sobre par, **500 extra por un hoyo en uno**. El menor número de
  golpes por recorrido está en la sección `minigolf` de `mem.json`.
- **La recogida se puede desactivar**: por defecto un hoyo termina tras ocho
  golpes y se puntúa al mínimo. Si prefieres seguir hasta embocar, pon *Recoger
  bola* en OFF en la pantalla de ajustes (o pulsa **P**).
- **La puntería automática se puede desactivar**: por defecto el palo apunta al
  hoyo antes de cada golpe. Si prefieres apuntar tú en cada hoyo, pon
  *Auto-puntería* en NO en la pantalla de ajustes (o pulsa **Z**) - así se
  mantiene la última dirección elegida, y en la salida de un hoyo nuevo el palo
  apunta neutro hacia arriba.
- **F** reinicia el hoyo actual: golpes a 0, bola en la salida - mismo hoyo,
  mismo recorrido.
- **Seguir en vez de repetir**: al terminar la ronda, el botón **Siguiente**
  lleva al recorrido siguiente (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), así
  nunca se repiten los mismos nueve hoyos; junto a él: **Otra vez** (mismo
  recorrido) y **Ajustes**. Teclas: Intro = seguir, R = otra vez, S = ajustes.
- **Repetición de la ronda**: al final, **P** (o el botón **Repetición**) vuelve
  a mostrar toda la ronda golpe a golpe. Con **S** pasa al archivo (botón
  **Repeticiones** de la barra lateral).
- **Crear y compartir hoyos propios**: la pestaña **MAPS** de la pantalla de
  preparación lleva a tu colección - **Nuevo** abre el editor. Cada hoyo recibe
  un nombre y un **id** (minúsculas, sin espacios); el id es además el nombre
  de archivo propuesto al compartir. A los siete obstáculos clásicos se suman
  **ocho nuevos**: tubo (lleva la bola al otro extremo), hielo, zona pegajosa,
  impulsor, imán, puerta de un sentido, plato giratorio y trampolín. El tamaño
  del hoyo se ajusta libremente (60x80 a 160x240), **12 plantillas** dan un
  punto de partida, y deshacer/rehacer más **Prueba** vienen incluidos.
  **Compartir** escribe exactamente un hoyo en un archivo `.lamapgzmap` - por
  el diálogo de guardado o directo a la carpeta Descargas, con tu nombre como
  creador. **Importar** lo vuelve a leer y pasa automáticamente a `-2` si el id
  está ocupado. Nombre, id y creador pasan siempre por un **filtro de palabras
  de los 14 idiomas**. Un hoyo se juega solo con **Jugar**, o toda la colección
  con la quinta opción de recorrido **Propios**.

**Pinball**
- **Tres mesas**: *Classic* (tres bumpers, una serie de dianas), *Space* (cuatro
  bumpers en rombo, dos series) y *Lama* (campo abierto, seis dianas en arco);
  3 o 5 bolas por partida, a dos alternando bola a bola.
- **Todo lo que necesita una máquina**: carril de lanzamiento con barra de carga
  (¿demasiado flojo? la bola vuelve y repites), dos flippers, slingshots, series
  de dianas, cuatro carriles **L-A-M-A**, atrapabolas, **multibola con jackpot**,
  seis segundos de **salvabolas**, empujón y **TILT**.
- **Multiplicador hasta x5** con series derribadas y carriles completos; bumpers
  100, slingshots 50, dianas 250 - en multibola los bumpers pagan 2.500 de
  jackpot.
- Los flippers usan las teclas izquierda/derecha asignadas (y [Shift]
  izquierdo/derecho) o el ratón. El récord por mesa está en la sección `pinball`
  de `mem.json`.

**Bowling**
- **Diez frames con las reglas oficiales**, incluidos strikes, spares y los tiros
  de bonificación del décimo frame (máximo: 300). La **tarjeta** bajo la cabecera
  muestra cada frame con X, / y la suma acumulada.
- **Lanzamiento en cuatro pasos**: posición, ángulo, efecto y fuerza. Cada
  control oscila solo y se fija con la tecla de acción, o se ajusta a mano con
  izquierda/derecha, lo que detiene la oscilación.
- **Física real de bolos**: diez bolos como círculos con masa que se derriban
  entre sí; un strike sale de la física, no de la suerte. La pista está aceitada
  delante, así que el **hook** solo agarra en el último tercio.
- Vista de pista en perspectiva con canaletas, flechas y pin deck; tres
  dificultades (*Fácil/Normal/Pro*) cambian la velocidad de los controles y la
  dispersión. El récord por dificultad está en la sección `bowling` de `mem.json`.
- **Repetición de la partida**: al final, **P** muestra otra vez todas las
  tiradas y **S** las guarda en el archivo (botón **Repeticiones**).

**Crossy Road**
- **Saltos sin fin** por praderas (árboles y rocas cortan el paso), carreteras con
  coches y camiones, ríos con troncos y nenúfares y **vías de tren** por las que
  llega un tren tras la luz de aviso y la campana; más adelante esperan
  estaciones con hasta 5 vías. La ruta se crea fila a fila, siempre tiene un
  camino transitable, y la velocidad y el tráfico aumentan.
- **Estilo vóxel isométrico**: personajes, vehículos y árboles hechos de bloques
  sombreados (prerenderizados para cada tamaño de casilla), cámara suave, squash
  & stretch al saltar, salpicaduras, animación de aplastamiento, plumas y
  monedas que brillan; desde la fila 50, **ciclo día/noche** con faros.
- **El águila**: la cámara avanza despacio; si te entretienes demasiado o
  retrocedes más de tres filas, el águila te atrapa (antes avisa un borde rojo).
  Salir de la pantalla a la deriva sobre un tronco también acaba la partida.
- **Monedas y personajes**: las monedas recogidas (moneda gigante = 5) se guardan
  y compran personajes nuevos en la pestaña **Personajes**: rana, cerdo, pingüino,
  gato, zorro, llama, robot, fantasma y unicornio (25 a 250 monedas); el pollo
  está desde el principio.
- **Modos**: *Infinito* (puntos = fila más lejana, cuenta para el récord) y *Ruta
  del día* (hoy igual para todos, también en el navegador, con su propio récord
  del día). Controles: flechas/WASD, Espacio/Enter/clic = saltar hacia delante;
  en la configuración **H** = sombras, **N** = día/noche. Monedas, personajes y
  récord del día se guardan en la sección `crossy` de `mem.json`.

**Geometry Dash**
- **Plataformas rítmico**: tu personaje corre solo hacia la derecha; tú solo
  decides cuándo saltar o volar. **Cinco formas** (cubo, nave, bola, OVNI y onda),
  además de portales de forma, gravedad y velocidad (0,5x a 3x), **plataformas y
  orbes** amarillos/rosas/azules, medios bloques, pinchos, fosos y activadores de
  color.
- **8 niveles incluidos** de *Fácil* a *Demonio* («Lama Inferno») con **3
  monedas secretas** cada uno. Cada nivel es superable de forma demostrada: al
  crearlo, un solucionador lo completó con el código real del juego, monedas
  incluidas e incluso desplazado 1/240 de segundo.
- **Física precisa**: cálculo en coma fija con paso fijo de 240 Hz; cada pulsación
  actúa exactamente en el paso en que ocurrió, igual con cualquier tasa de
  fotogramas e idéntica bit a bit en el navegador.
- **Modo práctica** (**P**) con puntos de control automáticos y propios (**Z**
  pone, **X** borra), contador de intentos, barra de progreso, explosiones y
  reinicio inmediato (**R**). Cada nivel tiene su **propia banda sonora**: fondo,
  suelo y orbes laten al ritmo (la música se apaga con **M**).
- **Estrellas y monedas**: al superar un nivel en modo normal ganas sus
  estrellas, y cada moneda vale una estrella más; el récord es el **total de
  estrellas** (65 como máximo). Mejores marcas por nivel, monedas, intentos y
  saltos se guardan en la sección `geodash` de `mem.json`.
- **Editor de niveles** en la pestaña **NIVELES**: lienzo con cuadrícula, paleta
  de 6 grupos (bloques, peligros, plataformas y orbes, portales, velocidad,
  extras), giro, deshacer/rehacer, barra de vista general, **probar desde el
  inicio o desde aquí** y ajustes del nivel (velocidad y forma inicial, estilo
  musical, BPM, colores). La marca **«verificado»** solo aparece cuando superas
  tu propio nivel. **Compartir** escribe un archivo `.lamapgzlevel` e **Importar**
  lo vuelve a leer; los niveles se guardan en `ugc.json` junto a tus hoyos de
  minigolf.

**Battleship**
- **Batalla naval de 10x10** con portaaviones (5 casillas), acorazado (4),
  crucero (3), submarino (3) y destructor (2): gana quien hunda primero toda la
  flota enemiga.
- **Colocar la flota** arrastrando desde el muelle: **R** o clic derecho gira, la
  vista previa brilla en verde o rojo, **X** lo coloca todo al azar, **C** vacía
  el tablero; tu última colocación se vuelve a proponer.
- **Reglas en la configuración** (se guardan): *los barcos pueden tocarse*,
  *salva* (tantos disparos por turno como barcos propios a flote) y *volver a
  disparar tras un impacto*.
- **IA de 3 niveles**: Fácil dispara al azar, Medio persigue los impactos de forma
  sistemática, Difícil calcula un **mapa de probabilidades** con paridad de
  tablero de ajedrez (de media unos 70 / 60 / 45 disparos para toda una flota).
  O **2 jugadores** en el mismo ordenador: una **pantalla de relevo** oculta ambas
  flotas antes de cada turno.
- **Efectos**: barrido de radar, olas animadas, proyectiles en parábola,
  salpicaduras, explosiones con humo y casillas en llamas, revelación
  «¡HUNDIDO!» y resumen final con disparos, impactos y precisión. El récord
  cuenta tus **victorias contra la IA** en una sesión.

**Casino**
- **Ruleta** (europea, 37 casillas): todas las apuestas clásicas con clic en un
  número, un borde o una esquina: **pleno** (35:1), caballo, transversal, cuadro,
  seisena, columna, docena, rojo/negro, par/impar y falta/pasa. Fichas de
  1/5/25/100/500, clic derecho para retirarlas; **Girar**, **Repetir** (**R**),
  **Doblar** (**D**) y **Borrar**. La bola entra en espiral en la casilla sorteada
  de antemano, y arriba se ven los últimos 12 números.
- **Máquina Llama**: 5 rodillos x 3 filas, **10 líneas de pago**, **llama =
  comodín**, **monedas de oro = scatter** con 10 tiradas gratis y premios dobles,
  apuesta por línea 1/2/5/10, **tirada automática** (10/25), **turbo** y tabla de
  premios. El **retorno al jugador es del 96,1 %**, calculado con exactitud a
  partir de las tiras de los rodillos.
- **Banco Llama**: Casino, Blackjack y Póker comparten una cuenta de **fichas
  Llama** (inicio 1000, sección `casino` de `mem.json`); los saldos antiguos se
  pasan automáticamente. Las apuestas se descuentan al instante, cada juego lleva
  su propio balance para su récord y, si te quedas sin fichas, recibes un
  **crédito del banco** hasta 1000.
- Confeti, lluvia de monedas, carteles de gran premio/mega/jackpot y animaciones
  de líneas ganadoras; logros **Diana** (pleno ganador en la ruleta) y **Jackpot
  Llama** (5 llamas en una línea).

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
  el récord anterior como chip. Con muchos modos y poca resolución se vuelve
  **compacta**: Opciones, Wiki y Volver pasan a una sola fila y la letra se
  adapta, así que nada se sale de la pantalla.
- **Aspecto unificado en el juego**: los 46 juegos comparten la paleta y la
  tipografía del menú - los HUD, pantallas de configuración y superposiciones
  siguen el diseño elegido en las opciones (v4.1 / v4 / Clásico), mientras cada
  campo de juego conserva sus colores de identidad. Todos los juegos gestionan
  limpiamente un cambio de resolución a mitad de partida, y los nombres de los
  juegos en el menú se adaptan al idioma (p. ej. «Schach» → «Ajedrez»).
- **Wiki integrado** ("LamaWiki"): ayuda detallada de cada juego (controles,
  modos, puntos, consejos) más páginas generales - con **buscador**, categorías,
  artículos desplazables y chips de teclas, en los 14 idiomas. Accesible por
  el botón **«Wiki / Ayuda»** de la barra lateral y desde la pantalla previa de
  cada juego (abre directamente su página).
- **Logros y estadísticas**: **107 logros** en tres categorías (23 metas
  globales, 37 hitos de puntuación y 47 momentos especiales como un jaque mate
  a la IA, la ficha de 4096, un T-Spin Double, 25 problemas de ajedrez
  resueltos, un Killer Sudoku o el jackpot de la llama; en 2048 y Ajedrez las
  partidas con deshacer o pistas no cuentan) con **aviso dorado y fanfarria** al desbloquear,
  incluso en plena partida; los récords antiguos se acreditan automáticamente.
  Además, una pestaña de **estadísticas**: tiempo total, partidas, victorias,
  récords, juego favorito y tabla por juego ordenada por tiempo. Accesible por
  el botón **«Logros y estadísticas»** de la barra lateral.
- **Repeticiones**: el minigolf y los bolos graban cada ronda. Al final, **P**
  muestra la repetición y **S** la guarda en el archivo, accesible con el botón
  **Repeticiones** de la barra lateral (una pestaña por juego, pausa, saltos de
  secuencia, velocidad 0,5x a 4x). Se puede desactivar en el primer inicio y en
  las opciones.

### Manejo

- Elige el juego con el botón del menú izquierdo. Después aparece la **pantalla
  previa**: elegir **Un jugador** o **Multijugador**, ir a las **opciones** o
  volver. Flechas/ratón para elegir, Enter empieza.
- **ESC** = pausa / continuar (en menús: volver).
- **F11** (o el botón «Pantalla completa sí/no») = pantalla completa. La pantalla
  de Pygame sigue incrustada y se escala conservando la proporción (bandas negras
  si la proporción difiere). La ventana se puede redimensionar libremente.
- **«Volver al menú»** termina el juego y guarda el récord; lo mismo ocurre al
  cambiar a otro juego desde la barra lateral.
- **Teclas fijas adicionales**: además de las cinco acciones asignables, algunos
  juegos tienen teclas propias (p. ej. reservar **C** y girar a la izquierda
  **Z** en Tetris, deshacer **U** en 2048, Ajedrez y Sudoku). Solo funcionan si
  esa tecla no está asignada a ninguna acción en las opciones, y aparecen en el
  aviso de la configuración y en el wiki. Las teclas mantenidas se detectan bien
  y se sueltan al pausar o con Alt-Tab: ya nada se «atasca».
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
- **Apariencia**: elegir el **diseño de la interfaz** – **UI v4.2**
  (predeterminado: Midnight Glass – degradado de medianoche profundo con
  luces suaves que flotan lentamente en índigo, turquesa y magenta, grano de
  película fino, estrellas dispersas y paneles de vidrio esmerilado con borde
  de luz), **UI v4.1** (como UI v4 pero más viva – estrellas sutiles más
  Saturno y un agujero negro en el fondo de la pantalla de inicio),
  **UI v4.1.1** (como v4.1, pero con un **patrón de zigzag** en mosaico negro
  y antracita en lugar del cielo estrellado), **UI v4.1.2** (el mismo patrón
  en los azules de la paleta: azul de acento como color dominante y un azul
  más oscuro de fondo), **UI v4.1.3** (el mismo patrón en el índigo de UI v4
  sobre negro), **UI v4.1.4** (en el grafito de UI v4 sobre negro), **UI v4**
  (un look grafito limpio, plano y totalmente tranquilo con un solo acento
  índigo), **UI v3** (la interfaz clásica anterior con cielo estrellado,
  auroras y brillos), **UI v2** (el primer rediseño de la interfaz: degradado
  azul marino, cielo estrellado y botones con brillo, sin animaciones) o
  **UI v1** (el aspecto previo al rediseño: fondo oscuro liso, botones
  planos, sin efectos). Todas las tarjetas muestran una pequeña vista previa;
  la elección se aplica al instante a toda la interfaz (área de juego **y**
  barra lateral) y se guarda.

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
settings.py          Cargar/guardar ajustes (sonido/vibración/teclas/opciones de juego con reglas de validación) (JSON)
audio.py             Efectos de sonido procedurales, bucles de música + vibración del mando
menu.py              Pantallas de idioma, previa (modo) y opciones (sonido/controles)
highscore.py         Cargar/guardar récords (sección en mem.json)
store.py             Archivo de guardado central mem.json (secciones: mem, highscores, stats, achievements + progreso de los juegos), atómico con copia .bak
stats.py             Estadísticas del jugador (partidas, tiempo, victorias, récords) por juego
achievements.py      Logros: definiciones, lógica de desbloqueo, aviso (toast)
progress.py          Pantalla de logros y estadísticas (dos pestañas, desplazable)
replay.py            Grabación y archivo de las repeticiones (replay.json)
replayview.py        Pantalla de repeticiones: lista del archivo y reproducción
ugc.py               Contenido propio (hoyos de minigolf, niveles de Geometry Dash): almacenamiento, validación, exportar/importar (ugc.json)
swear.py             Filtro de palabras para nombres e id (lang/swear/*.yml, 14 idiomas)
filepick.py          Diálogos de archivo ("Exportar como ...", "Importar")
prestige.py          Sistema de prestigio de Snake
competitive.py       Parámetros del modo Competitivo de Snake (niveles, tragaperras, manzanas de apuesta)
ngb.py               Personalización visual ("mods"): color de cabeza + rejilla + menú (mem-ngb.json)
lamabank.py          Banco Llama: cuenta de fichas común de Blackjack, Póker y Casino (sección casino de mem.json)
seedrand.py          Generador aleatorio con números idénticos bit a bit en Python y en el navegador (modos diarios, nuevos puzles)
i18n.py              Motor de traducción (carga lang/*.json, t("clave"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textos (una clave por texto)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Listas del filtro de palabras por idioma (regex, .yml)
lamawiki/
  lamawiki.py          Wiki integrado (búsqueda, categorías, renderizador)
  de.json  en.json  fr.json  es.json  pt.json   Contenido del wiki (una página por juego + generales)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Reconstruye las listas de palabras de Wordle (diccionarios + frecuencias)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 letras), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 letras), 14 idiomas
devtools/            Herramientas de desarrollo (no se incluyen en el .exe)
  merge_staging.py           Integra traducciones y páginas del wiki de devtools/staging/ en los 14 archivos de idioma
  build_chess_puzzles.py     Genera los 200 problemas de ajedrez a partir de la base de problemas de Lichess (CC0)
  build_sudoku_killer.py     Genera los 400 Killer Sudokus con solución única
  build_crossyroad_models.py Escribe los modelos vóxel de Crossy Road para la versión web
  build_geodash_levels.py    Construye los 8 niveles de Geometry Dash y demuestra con el solucionador que todos son superables, monedas incluidas
  build_geodash_solver.py    Solucionador con el código de paso real (soluciones en geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Datos de niveles: snake-comp.json, chess-puzzles.json (+ README de fuentes), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Auditoría general (entrada, seedrand Python = JS, guardado, archivos de idioma, pantallas previas) + todos los audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Auditorías headless por juego
  newgames_audit.py  blockjump_audit.py
```

El idioma elegido se guarda en `mem.json` (en la sección `mem`, junto a la
sección `highscores` del mismo archivo) y se carga automáticamente en el
siguiente arranque.

**Fuentes y licencias:** los 200 problemas de ajedrez proceden de la
[base de problemas de Lichess](https://database.lichess.org/#puzzles) (licencia
**CC0 1.0**, dominio público; ¡gracias, lichess.org!); los detalles están en
`games/levels/chess-puzzles.README.md`. Las fuentes de las listas de palabras de
Wordle figuran en `woordlistz/README.md`.

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
Quarenta e seis jogos com opções partilhadas, controlos totalmente reatribuíveis,
recordes, efeitos sonoros procedurais e, em vários títulos, modo multijogador.
A interface é **multilingue** – **14 idiomas** (alemão / inglês / francês /
espanhol / português / polaco / turco / dinamarquês / norueguês / sueco /
finlandês / checo / esloveno / croata); o idioma escolhe-se num **ecrã de
boas-vindas** no primeiro arranque, que também permite definir a **resolução** e
o **som** (desligado por omissão); além dos três idiomas principais, todos os
outros estão atrás do botão **«Mais»**. Tudo pode ser mudado a qualquer momento
nas opções.

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
| **Tetris**   | 1 / 2 jogadores | Regras Guideline modernas (SRS, reserva, pré-visualização de 5 peças, T-Spins): Maratona, Sprint 40, Ultra 2:00, Versus contra a IA (3 níveis) ou a dois com linhas de lixo |
| **Invaders** | 1 jogador       | Space Invaders: limpa as vagas, protege as tuas vidas |
| **Asteroids** | 1 / 2 jogadores | Física de inércia, vagas, OVNIs, power-ups, hiperespaço - a solo ou duelo cooperativo |
| **Pac-Man**  | 1 jogador       | Clone fiel: 4 IAs de fantasmas, pílulas de poder, túneis, frutas, níveis |
| **Flappy Bird** | 1 jogador    | Voo com gravidade entre canos, moedas, escudo, dia/noite, medalhas |
| **Doodle Jump** | 1 jogador    | Salto automático para cima, tipos de plataforma, molas, hélice, monstros |
| **2048**     | 1 jogador       | Puzzle de deslizar números de 3x3 a 8x8: Clássico, Contrarrelógio e Infinito, desfazer, animações fluidas, partidas guardadas |
| **Minesweeper** | 1 jogador    | O clássico com primeiro clique seguro, chording, smiley e melhores tempos |
| **Sudoku**      | 1 jogador    | 4 variantes (Clássico, Sudoku X, Killer, Mini 6x6) com 400 níveis cada, Sudoku do dia, até 3 estrelas por nível, 4 modos de ajuda, desfazer, jogo guardado |
| **Frogger**     | 1 jogador    | Estrada + rio + 5 baías, mosca bónus, crocodilos, limite de tempo, 3 dificuldades |
| **Memory**      | 1 / 2 jogadores | Encontra pares em 4x4 até 8x6, animação de viragem, a solo ou em duelo |
| **Solitário**   | 1 jogador    | 5 variantes (Klondike, Spider, FreeCell, Pirâmide, TriPeaks) com arrastar e largar e anular |
| **Aim Trainer** | 1 jogador    | Tiro ao alvo 3D descontraído: o rato dirige a câmara, 4 modos (precisão/reflexos/móveis/chill), 3 temas incl. um buraco negro |
| **Quatro em linha** | 1 / 2 jogadores | O clássico com animação de queda: 3 níveis de IA (minimax) ou duelo local |
| **Duelo de tanques** | 1 / 2 jogadores | Duelo 2D em arena com tiros com ricochete, power-ups, 4 arenas, IA com 3 níveis |
| **Blackjack**    | 1 jogador    | Blackjack de casino com shoe de 4 baralhos, dobrar/dividir e blackjack 3:2; joga com as fichas Lama do Banco Lama comum |
| **Tunnel Racer** | 1 jogador    | Voo 3D num tubo de néon: modo sem fim + 30 níveis, controlo por teclas ou rato, motion blur |
| **Labirinto 3D** | 1 jogador    | Raycaster na primeira pessoa (estilo Wolfenstein) com 50 níveis com semente, orbes, minimapa - ou vista de cima 2D |
| **Reversi**      | 1 / 2 jogadores | Othello em 8x8: cercar e virar peças, 3 forças de IA (minimax) ou um duelo local |
| **Yahtzee**      | 1 / 2 jogadores | Clássico de dados com 13 categorias, bónus superior e Yahtzee; caça ao recorde ou hotseat a 2 |
| **Wordle**       | 1 jogador    | Adivinha palavras de 4 a 7 letras: Infinito, Palavra do dia, Dordle e Quordle, modo difícil, paleta para daltónicos, estatísticas com gráfico de barras, partilhar resultado, listas reais em 14 idiomas |
| **T-Rex Runner** | 1 jogador    | Corrida infinita pelo deserto: salto variável, agachar, cactos e pterodáctilos, ciclo dia/noite, velocidade crescente, 3 dificuldades |
| **Damas**        | 1 / 2 jogadores | 3 regulamentos à escolha (alemãs 8×8, internacionais 10×10, checkers), captura obrigatória e dama voadora, 3 forças de IA (minimax) ou duelo local |
| **Póquer**       | 1 jogador    | 3 variantes à escolha: Texas Hold'em contra a IA, 5 Card Draw e Video Poker; rondas de apostas, blinds, fichas Lama do Banco Lama comum |
| **Xadrez**      | 1 / 2 jogadores | Regras completas, Chess960 e relógio, 6 níveis de IA, 200 problemas da base do Lichess, desfazer/dica, lista de lances, exportação PGN ou duelo local |
| **Trilha**      | 1 / 2 jogadores | Fases de colocar, mover e voar, moinhos e capturas, voo desligável, 3 níveis de IA ou duelo local |
| **Simon**       | 1 / 2 jogadores | Jogo de memória Senso: modos Clássico/Speed/Reverse/Misto + duelo, som sim/não/misto, 4/6/9 casas, recorde por modo |
| **Bilhar**      | 1 / 2 jogadores | Bola 8, bola 9 e treino em 2D, vista 3D fixa ou câmara 3D livre; física suave, ajuda de mira, IA com 3 níveis |
| **Quebra-cabeça deslizante** | 1 jogador | Jogo do 15 em 3x3/4x4/5x5: deslize as peças numeradas para o vazio, controle por rato ou setas, pontos por jogadas e tempo |
| **Mastermind**       | 1 jogador  | Decifra o código de cor secreto (3 modos: 4×6, clássico, 5×8), pinos de dica pretos/brancos, série sem fim |
| **Bubble Shooter**   | 1 jogador  | Clone do Puzzle Bobble: atira cores iguais em grupos de três, ressaltos nas paredes, grupos que caem, 3 dificuldades |
| **Hangman**          | 1 jogador  | Adivinha a palavra antes de a forca ficar completa; teclado no ecrã, listas de palavras por idioma, 3 modos de tamanho, série sem fim |
| **Block Jump**       | 1 jogador  | Plataforma 3D estilo Minecraft: mundo de blocos texturizados com skin de Minecraft e figura do Steve, escadas, cercas e blocos-mola, câmera 1ª/3ª pessoa, desfoque, níveis gerados |
| **Tower Defense**    | 1 jogador  | Repele ondas infinitas em 4 mapas: até 11 tipos de torres com melhorias, venda e especialização A/B, chefes, 3 modos, habilidades ativas |
| **Minigolf**    | 1 / 2 jogadores | 360 buracos em 40 percursos (18 feitos à mão, 342 gerados): areia, rampas, água, para-choques, moinhos e blocos móveis; cartão com par e bónus de buraco em um; **editor de buracos próprio** com 15 tipos de objetos, 12 modelos e partilha como `.lamapgzmap` |
| **Pinball**     | 1 / 2 jogadores | Máquina de pinball com 3 mesas: bumpers, slingshots, alvos, corredores L-A-M-A, multibola com jackpot, salva-bolas, empurrão e tilt |
| **Bowling**     | 1 / 2 jogadores | 10 frames com a pontuação oficial de strike/spare, física real dos pinos, efeito hook e pista em perspetiva, 3 dificuldades |
| **Crossy Road** | 1 jogador       | Saltar sem fim por relvados, estradas, rios e linhas de comboio em estilo voxel isométrico: dia/noite, águia, 10 personagens para comprar, percurso do dia |
| **Geometry Dash** | 1 jogador     | Plataformas rítmico com cubo, nave, bola, OVNI e onda: 8 níveis de Fácil a Demónio com 3 moedas secretas cada, modo treino, banda sonora por nível; **editor de níveis** com partilha como `.lamapgzlevel` |
| **Battleship**  | 1 / 2 jogadores | Batalha naval 10x10: frota posicionada arrastando, 3 regras opcionais (tocar, salva, voltar a disparar), IA com 3 níveis ou duelo local com ecrã de passagem |
| **Casino**      | 1 jogador       | Roleta europeia com todas as apostas clássicas e Máquina Lama (5 rolos, 10 linhas, wild, rodadas grátis); uma só conta de fichas Lama com o Blackjack e o Póquer |

**O multijogador (2 jogadores em local)** está disponível em **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Quatro em linha**, **Duelo de tanques**,
**Reversi**, **Yahtzee**, **Damas**, **Xadrez**, **Trilha**, **Simon (duelo)**,
**Bilhar**, **Minigolf**, **Pinball**, **Bowling** e **Battleship** (com um ecrã
de passagem que esconde as frotas) - 20 jogos no total. O modo escolhe-se
diretamente no ecrã de preparação (*Um jogador / Multijogador*); o Tetris
oferece ainda **Versus contra a IA**. A versão web é apenas para um jogador.

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
- **Regras Guideline modernas**: campo de 10x20, peças de um **saco de 7**,
  **sistema de rotação SRS** com wall kicks verdadeiros (também para a peça I),
  rotação nos dois sentidos, **reserva** (uma vez por peça), **pré-visualização
  de 5 peças**, peça fantasma e **lock delay** (0,5 s, no máximo 15 reinícios).
- **Três modos** no ecrã de preparação: *Solo*, *Contra a IA* e *2 jogadores*. O
  Solo oferece nas definições **Maratona** (nível inicial 1-15, conta para o
  recorde), **Sprint 40 linhas** (melhor tempo) e **Ultra 2 minutos** (melhor
  pontuação); os melhores valores ficam na secção `tetris` do `mem.json`.
- **Pontuação Guideline**: de Single a Tetris, **T-Spins** (completos e mini),
  **Back-to-Back** (x1,5), **combos** e **Perfect Clear** - com avisos,
  animação de limpeza, rasto de hard drop, partículas e efeito de subida de
  nível.
- **Versus com linhas de lixo**: as linhas limpas enviam lixo ao adversário
  (Tetris = 4, T-Spin Double = 4 …), o lixo que chega é anunciado numa barra de
  aviso e **compensado** pelos teus próprios ataques; os dois campos recebem a
  mesma sequência de peças. A **IA** tem 3 níveis e a velocidade sobe a cada 40
  segundos.
- **Controlos**: Esq/Dir com **DAS/ARR** próprios (ajustáveis nas definições),
  Cima = rodar para a direita, Baixo = soft drop, Ação = hard drop; **C**/Shift =
  reserva, **Z**/**Y** = rodar para a esquerda, **X** = rodar para a direita. A
  dois, o J1 guarda com **Q** e roda para a esquerda com **E**, o J2 com **Shift
  direito** / **Ctrl direito**. Depois do jogo: **R** = de novo, **S** =
  definições.

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

**2048**
- **Ecrã de configuração próprio** com tabuleiros de **3x3 a 8x8** e três modos:
  *Clássico* (objetivo 2048, depois «Continuar a jogar?»), *Contrarrelógio* (3
  minutos, o relógio arranca com o primeiro movimento) e *Infinito*.
- **Animações fluidas**: as peças deslizam, fundem-se com um «pop» e aparecem a
  crescer; pontos a saltar, faíscas a partir de 128, onda de choque a partir de
  2048 e novas cores até 131072. O que premires durante uma animação fica em
  memória.
- **Desfazer** (desligado / 3 por jogo / ilimitado, tecla **U** ou Backspace) -
  quem o usa joga sem recorde e sem conquistas de peças.
- **Guardar e retomar**: a partida em curso é guardada automaticamente por
  tamanho e modo; melhores pontuações e maior peça por tamanho/modo ficam na
  secção `g2048` do `mem.json`.
- Controlos: setas/WASD ou **deslizar** com rato/touchpad, **R**/**N** = novo
  jogo, **Tab** = configuração. O recorde só conta em **4x4 Clássico** sem
  desfazer.

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
- **4 variantes** com **400 níveis** cada (4 dificuldades x 100): *Clássico* (os
  níveis por semente de sempre - os resolvidos continuam assinalados), *Sudoku X*
  (as duas diagonais contêm cada dígito uma só vez), *Killer* (jaulas tracejadas
  com soma; 400 níveis gerados antecipadamente) e *Mini 6x6*. Todos os puzzles
  têm **solução única** - o nível 12 de "Difícil" é o mesmo puzzle em qualquer
  PC.
- **Sudoku do dia**: um puzzle por dia para todos, igual no PC e no navegador; a
  dificuldade depende do dia da semana (de segunda Fácil a sábado Perito), e
  resolver todos os dias cria uma sequência.
- **Até 3 estrelas por nível** (resolvido · sem erros nem dicas · e abaixo do
  tempo-alvo) e **melhor tempo** na escolha de níveis; os **puzzles começados**
  são guardados automaticamente e retomados na vez seguinte.
- **4 modos de jogo** (antes de começar) com multiplicador: **Clássico** (x2,0 -
  sem ajudas), **Notas** (x1,5 - + notas a lápis e candidatos automáticos),
  **Conforto** (x1,0 - + erros a vermelho, conflitos e somas de jaula erradas
  marcados, entradas corretas fixam-se), **Assistente** (x0,7 - + dica, máx. 3).
  Com o **limite de 3 erros** ativo (opção do setup) o terceiro erro acaba a
  partida.
- Controlos: setas/WASD = célula, **1-9** = dígito (também teclado numérico),
  **0/Delete/clique direito** = apagar, **U**/**Z** = desfazer, **Y** = refazer,
  **N** = notas, **C** = candidatos automáticos, **H** = dica, **M** = marcador de
  cor, **R** = recomeçar nível, **Q** = escolha de níveis. Entrada «dígito
  primeiro» (setup, **I**) e um **contador de dígitos restantes** por baixo de
  cada dígito; totalmente jogável com o rato. Depois do fim, **A** mostra a
  **solução** completa.
- Pontos = (base da variante e da dificuldade - tempo - erros - dicas) x
  multiplicador do modo; todas as variantes e o Sudoku do dia contam para o
  recorde.

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
- **Fichas Lama**: o Blackjack joga com a conta do **Banco Lama**, partilhada com
  o Póquer e o Casino (início 1000, guardada no `mem.json`). A aposta é debitada
  ao dar as cartas; com menos de 10 fichas, Enter pede um **crédito do banco**
  que repõe a conta em 1000.
- **Recorde** = máximo do teu **saldo no Blackjack** (1000 mais tudo o que foi
  ganho e perdido no Blackjack) - ganhos na roleta, na slot machine ou no póquer
  não contam aqui, e os créditos também não.
- Manejo com botões de fichas e teclas (**H**it/**S**tand/**D**ouble/dividir
  **X**, **1-4** = aposta, Backspace = limpar aposta, Enter = dar) com animações
  de cartas; a carta tapada do dealer agora vira-se mesmo ao ser revelada.

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
- Adivinha a palavra escondida; resposta colorida (verde/amarelo/cinzento) com
  **contagem correta de letras repetidas** e um teclado no ecrã que se colore
  (QWERTZ em alemão, checo, esloveno e croata, AZERTY em francês, QWERTY nos
  restantes).
- **Quatro modos**: *Infinito* (uma palavra atrás da outra com 6 tentativas cada;
  cada palavra resolvida dá pontos e a primeira falhada termina a partida),
  *Palavra do dia* (uma palavra por dia por idioma e comprimento - a mesma no PC e
  no navegador - com contagem decrescente e sequência; a palavra do dia começada
  fica guardada), *Dordle* (2 palavras ao mesmo tempo em 7 tentativas) e *Quordle*
  (4 palavras em 9 tentativas, as teclas mostram as cores de todos os tabuleiros).
- **Configuração** antes de cada partida: **comprimento de 4 a 7 letras**, **modo
  difícil** (as pistas encontradas têm de ser reaproveitadas) e **paleta para
  daltónicos** (laranja/azul); ao lado aparecem as estatísticas.
- **Listas de palavras a sério nos 14 idiomas** (pasta `woordlistz/`, só A-Z), com
  listas próprias por comprimento: só com 5 letras, quase **34.000 soluções** e
  mais de **213.000 tentativas aceites**, cerca de 134.000 soluções somando todos
  os comprimentos. As soluções são palavras comuns, sem nomes, restos de inglês nem
  palavras ofensivas; cada tentativa é verificada na lista - o resto é recusado e
  a linha abana por instantes.
- **Estatísticas** por idioma, comprimento e modo: jogos, percentagem de vitórias,
  sequência atual e melhor sequência e a **distribuição de tentativas num gráfico
  de barras** (secção `wordle` do `mem.json`). **Partilhar** (**C**) copia uma
  grelha de emojis sem revelar a solução.
- O recorde só conta *Infinito* com 5 letras; os outros comprimentos têm os seus
  próprios melhores valores. Conquistas **Vidente** (2 tentativas no máximo),
  **Hábito de palavras** (7 palavras do dia seguidas) e **Génio quádruplo**
  (Quordle resolvido).

**Póquer**
- **3 variantes** no ecrã de preparação: **Texas Hold'em** contra 1-3 adversários
  da IA com botão de dealer, blinds e quatro rondas de apostas, **5 Card Draw**
  (frente a frente contra a IA, uma troca de cartas) e **Video Poker** (*Jacks or
  Better*, a solo contra a tabela de prémios).
- Ações com botões ou teclas: **F** = desistir, **C** = passar/pagar, **R** =
  aumentar, **A** = all-in; guardar/trocar cartas com clique ou **1-5**, **Enter**
  compra ou dá a mão seguinte.
- **Fichas Lama** do **Banco Lama** comum: no início de uma mão a tua conta fica
  na mesa como pilha, e o que vai para o pote é debitado logo - sair da mesa a meio
  da mão só custa a tua parte do pote. Sem fichas (menos do que o big blind de 20,
  menos de 10 no Video Poker) = crédito do banco que repõe a conta em 1000.
- **Recorde** = máximo do teu **saldo no Póquer** (1000 mais todos os ganhos e
  perdas no póquer); a conquista **Chip leader** também conta só esse saldo.

**Xadrez**
- **Xadrez completo**: todos os lances, incluindo **roque**, **en passant** e
  **promoção** (peça à escolha); **xeque, xeque-mate e afogamento** e empate pela
  **regra dos 50 lances**, **tripla repetição**, **material insuficiente** ou
  acordo.
- **Três modos**: *partida* contra a IA, *2 jogadores* no mesmo computador (o
  tabuleiro pode rodar após cada lance) e **problemas**.
- **IA mais forte e sem soluços** em 6 níveis de *Iniciante* a *Mestre*:
  aprofundamento iterativo, tabela de transposição, pesquisa de quiescência,
  livro de aberturas e uma avaliação com mobilidade, estrutura de peões e
  segurança do rei. A IA calcula em pequenas fatias por imagem - o jogo nunca
  engasga.
- **Configuração**: escolha da cor, **relógio** (sem relógio, 1+0, 3+2, 5+0,
  10+5) e **Chess960** (as 960 posições iniciais, o número aparece por cima da
  lista de lances).
- **Barra lateral** com relógios, peças capturadas, balanço material e uma
  **lista de lances (SAN)** deslocável; arrastar e largar, peças que deslizam,
  coordenadas. Teclas: **U** = desfazer, **H** = seta de dica, **O** = propor
  empate, **X** = desistir, **F** = rodar o tabuleiro, depois da partida **P** =
  **exportar PGN**.
- **Problemas**: 200 problemas em 5 níveis (mate em 1/2/3, tática I/II) da
  **base de problemas livre do Lichess (CC0)**, verificados com o motor do jogo;
  nos problemas de mate vale qualquer lance que dê mate. O progresso fica na
  secção `chess` do `mem.json`.
- Desfazer e dica tornam a partida «assistida»: o recorde só conta vitórias sem
  ajuda contra a IA (por sessão).

**Tower Defense**
- **Defesa de ondas sem fim** em **4 mapas** (Campina, Desfiladeiro, Cruzamento,
  Corredor), cada um com o seu caminho; os mapas bloqueados desbloqueiam-se com
  a tua melhor onda, um **chefe** chega a cada **8 ondas**.
- **3 modos**: Clássico (7 torres, o modo principal), Compacto (4 torres,
  2 níveis) e Máximo (**11 torres**, **especialização A/B** no nível máximo,
  inimigos especiais, habilidades ativas **Meteoro/Nova de gelo/Corrida ao
  ouro**).
- **11 tipos de torres**, das flechas ao laser e ao banco de ouro, até
  **3 níveis de melhoria** cada, a venda devolve 70%; inimigos com blindagem,
  regeneração, divisão, camuflagem, aura de cura e rota aérea.
- **Economia**: ouro por baixa, bónus de onda + 5% de juros; pontos por baixa e
  onda. **F** = velocidade x2, **G** = alcances, botão direito cancela.

**Minigolf**
- **360 buracos em 40 percursos**: *Classic* e *Pro* com 9 buracos feitos à mão
  cada, a **Tour** com 38 percursos de 9 buracos gerados (342 no total) com
  dificuldade crescente, mais *Random* sorteado de tudo. O percurso 7, buraco 3
  fica igual em todo o lado - não é preciso guardar nada.
- **Pisos e obstáculos**: a areia trava, as rampas aceleram, a água custa uma
  tacada de penalização, os para-choques devolvem velocidade e moinhos e blocos
  móveis exigem sentido de tempo. A física corre em subpassos com atrito como no
  bilhar - nada salta nem atravessa a tabela.
- **Comandos**: o rato aponta, manter o botão esquerdo carrega a força e largar
  bate (também setas + espaço). **R** cancela uma tacada carregada sem bater.
  **G** alterna a linha de mira, **Z** a mira automática, **P** o levantamento.
- **Bloqueio de força (manter o botão direito)**: congela a barra de carga onde
  ela está - dourada, com a percentagem, um cadeado e um anel a pulsar à volta
  da bola. Assim esperas pela abertura do moinho com a tacada já carregada. Ao
  largares volta a carregar; a força bloqueada sobrevive à tacada e o clique
  esquerdo seguinte bate exatamente com esse valor.
- **Cartão de pontuação** à direita com o par e as tacadas por buraco; a dois,
  cada um joga o mesmo buraco à vez. Pontos: 600 por buraco, ±300 por tacada
  abaixo/acima do par, **500 extra por um buraco em um**. O menor número de
  tacadas por percurso fica na secção `minigolf` de `mem.json`.
- **O levantamento pode ser desligado**: por omissão um buraco termina ao fim
  de oito tacadas e conta o mínimo. Se preferires jogar até encaçapar, coloca
  *Levantar bola* em OFF nos ajustes (ou carrega em **P**).
- **A mira automática pode ser desligada**: por omissão o taco vira-se para o
  buraco antes de cada tacada. Se preferires apontar tu em cada buraco, coloca
  *Mira automática* em NÃO nos ajustes (ou carrega em **Z**) - a última direção
  escolhida mantém-se, e na saída de um buraco novo o taco aponta neutro para
  cima.
- **F** reinicia o buraco atual: tacadas a 0, bola no tee - mesmo buraco, mesmo
  percurso.
- **Seguir em vez de repetir**: no fim da volta, o botão **Seguinte** leva ao
  percurso seguinte (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), nunca repetindo
  os mesmos nove buracos; ao lado: **Outra vez** (mesmo percurso) e **Ajustes**.
  Teclas: Enter = seguir, R = outra vez, S = ajustes.
- **Replay da ronda**: no fim, **P** (ou o botão **Replay**) mostra a ronda
  inteira, tacada a tacada. Com **S** vai para o arquivo (botão **Replays** na
  barra lateral).
- **Criar e partilhar buracos próprios**: o separador **MAPS** no ecrã de
  preparação leva à tua coleção - **Novo** abre o editor. Cada buraco recebe um
  nome e um **id** (minúsculas, sem espaços); o id é também o nome de ficheiro
  sugerido ao partilhar. Aos sete obstáculos clássicos juntam-se **oito
  novos**: tubo (leva a bola para a outra ponta), gelo, zona pegajosa,
  impulsor, íman, porta de sentido único, prato giratório e trampolim. O
  tamanho do buraco ajusta-se livremente (60x80 a 160x240), **12 modelos** dão
  um ponto de partida, e anular/refazer mais **Teste** fazem parte.
  **Partilhar** escreve exatamente um buraco num ficheiro `.lamapgzmap` - pela
  janela de gravação ou diretamente para a pasta Transferências, com o teu nome
  como criador. **Importar** volta a lê-lo e passa automaticamente para `-2` se
  o id estiver ocupado. Nome, id e criador passam sempre por um **filtro de
  palavras dos 14 idiomas**. Um buraco joga-se sozinho com **Jogar**, ou toda a
  coleção com a quinta escolha de percurso **Próprios**.

**Pinball**
- **Três mesas**: *Classic* (três bumpers, um conjunto de alvos), *Space* (quatro
  bumpers em losango, dois conjuntos) e *Lama* (campo aberto, seis alvos em arco);
  3 ou 5 bolas por jogo, a dois alternando bola a bola.
- **Tudo o que uma máquina precisa**: corredor de lançamento com barra de carga
  (fraco demais? a bola volta e repetes), dois flippers, slingshots, conjuntos de
  alvos, quatro corredores **L-A-M-A**, saucer com bloqueio, **multibola com
  jackpot**, seis segundos de **salva-bolas**, empurrão e **TILT**.
- **Multiplicador até x5** com conjuntos derrubados e corredores completos;
  bumpers 100, slingshots 50, alvos 250 - na multibola os bumpers pagam 2.500 de
  jackpot.
- Os flippers usam as teclas esquerda/direita atribuídas (e [Shift]
  esquerdo/direito) ou o rato. O recorde por mesa fica na secção `pinball` de
  `mem.json`.

**Bowling**
- **Dez frames pelas regras oficiais**, incluindo strikes, spares e os
  lançamentos de bónus do décimo frame (máximo: 300). O **cartão** por baixo do
  cabeçalho mostra cada frame com X, / e a soma corrente.
- **Lançamento em quatro passos**: posição, ângulo, efeito e força. Cada régua
  oscila sozinha e é fixada com a tecla de ação, ou ajustada à mão com
  esquerda/direita, o que para a oscilação.
- **Física real dos pinos**: dez pinos como círculos com massa que se derrubam
  uns aos outros; um strike vem da física e não da sorte. A pista está oleada à
  frente, por isso o **hook** só agarra no último terço.
- Vista da pista em perspetiva com canaletas, setas e pin deck; três dificuldades
  (*Fácil/Normal/Pro*) mudam a velocidade das réguas e a dispersão. O recorde por
  dificuldade fica na secção `bowling` de `mem.json`.
- **Replay da partida**: no fim, **P** mostra todos os lançamentos outra vez e
  **S** guarda-os no arquivo (botão **Replays**).

**Crossy Road**
- **Saltos sem fim** por relvados (árvores e pedras bloqueiam o caminho),
  estradas com carros e camiões, rios com troncos e nenúfares e **linhas de
  comboio** onde um comboio chega a toda a velocidade após luz de aviso e sino -
  mais à frente esperam estações com até 5 linhas. O percurso é criado fila a
  fila, tem sempre um caminho transitável, e a velocidade e o trânsito aumentam.
- **Estilo voxel isométrico**: personagens, veículos e árvores feitos de blocos
  sombreados (pré-renderizados para cada tamanho de casa), câmara suave, squash &
  stretch ao saltar, salpicos, animação de esmagamento, penas e moedas a brilhar;
  a partir da fila 50, **ciclo dia/noite** com faróis.
- **A águia**: a câmara avança devagar - quem se demora demasiado ou recua mais
  de três filas é apanhado pela águia (antes avisa uma borda vermelha). Sair do
  ecrã à deriva num tronco também acaba a partida.
- **Moedas e personagens**: as moedas apanhadas (moeda gigante = 5) ficam
  guardadas e compram novas personagens no separador **Personagens**: rã, porco,
  pinguim, gato, raposa, lama, robô, fantasma e unicórnio (25 a 250 moedas); a
  galinha está lá desde o início.
- **Modos**: *Infinito* (pontos = fila mais distante, conta para o recorde) e
  *Percurso do dia* (hoje igual para todos, também no navegador, com recorde do
  dia próprio). Controlos: setas/WASD, Espaço/Enter/clique = saltar em frente; na
  configuração **H** = sombras, **N** = dia/noite. Moedas, personagens e recorde
  do dia ficam na secção `crossy` do `mem.json`.

**Geometry Dash**
- **Plataformas rítmico**: a tua personagem corre sozinha para a direita - só
  decides quando saltar ou voar. **Cinco formas** - cubo, nave, bola, OVNI e
  onda -, mais portais de forma, gravidade e velocidade (0,5x a 3x),
  **plataformas e orbes** amarelos/rosa/azuis, meios blocos, espinhos, buracos e
  gatilhos de cor.
- **8 níveis incluídos** de *Fácil* a *Demónio* («Lama Inferno») com **3 moedas
  secretas** cada. Cada nível é comprovadamente possível: ao construí-lo, um
  solucionador completou-o com o verdadeiro código do jogo - com todas as moedas
  e até com 1/240 de segundo de desvio.
- **Física precisa**: cálculo em vírgula fixa com passo fixo de 240 Hz; cada
  toque atua exatamente no passo em que aconteceu - igual a qualquer taxa de
  fotogramas e idêntico bit a bit no navegador.
- **Modo treino** (**P**) com pontos de controlo automáticos e próprios (**Z**
  coloca, **X** apaga), contador de tentativas, barra de progresso, explosões e
  recomeço imediato (**R**). Cada nível tem a sua **própria banda sonora** -
  fundo, chão e orbes pulsam ao ritmo (música desligável com **M**).
- **Estrelas e moedas**: quem conclui um nível no modo normal ganha as suas
  estrelas, e cada moeda vale mais uma estrela; o recorde é o **total de
  estrelas** (no máximo 65). Melhores valores por nível, moedas, tentativas e
  saltos ficam na secção `geodash` do `mem.json`.
- **Editor de níveis** no separador **NÍVEIS**: tela com grelha, paleta com 6
  grupos (blocos, perigos, plataformas e orbes, portais, velocidade, extras),
  rotação, desfazer/refazer, barra de visão geral, **testar desde o início ou a
  partir daqui** e definições do nível (velocidade e forma inicial, estilo
  musical, BPM, cores). O visto **«verificado»** só aparece depois de concluíres
  o teu próprio nível. **Partilhar** escreve um ficheiro `.lamapgzlevel` e
  **Importar** volta a lê-lo; os níveis ficam em `ugc.json`, ao lado dos teus
  buracos de minigolfe.

**Battleship**
- **Batalha naval em 10x10** com porta-aviões (5 casas), couraçado (4), cruzador
  (3), submarino (3) e contratorpedeiro (2) - ganha quem afundar primeiro toda a
  frota inimiga.
- **Posicionar a frota** arrastando a partir da doca: **R** ou clique direito
  roda, a pré-visualização brilha a verde ou vermelho, **X** posiciona tudo ao
  acaso, **C** limpa o tabuleiro; a última disposição volta a ser sugerida.
- **Regras na configuração** (guardadas): *os navios podem tocar-se*, *salva*
  (tantos tiros por jogada quantos navios próprios à tona) e *disparar de novo
  após acertar*.
- **IA com 3 níveis**: Fácil dispara ao acaso, Médio persegue os acertos de forma
  sistemática, Difícil calcula um **mapa de probabilidades** com paridade de
  tabuleiro de xadrez (em média cerca de 70 / 60 / 45 tiros para uma frota
  inteira). Ou **2 jogadores** no mesmo computador - um **ecrã de passagem**
  esconde as duas frotas antes de cada jogada.
- **Visual**: varrimento de radar, ondas animadas, granadas em arco, salpicos,
  explosões com fumo e casas a arder, revelação «AFUNDADO!» e resumo final com
  tiros, acertos e pontaria. O recorde conta as tuas **vitórias contra a IA** numa
  sessão.

**Casino**
- **Roleta** (europeia, 37 casas): todas as apostas clássicas com clique num
  número, numa aresta ou num canto - **pleno** (35:1), cavalo, transversal,
  quadrado, sextena, coluna, dúzia, vermelho/preto, par/ímpar e baixo/alto. Fichas
  de 1/5/25/100/500, clique direito retira fichas; **Rodar**, **Repetir** (**R**),
  **Dobrar** (**D**) e **Limpar**. A bola entra em espiral na casa sorteada de
  antemão, e em cima aparecem os últimos 12 números.
- **Máquina Lama**: 5 rolos x 3 filas, **10 linhas de prémio**, **lama = wild**,
  **moedas de ouro = scatter** com 10 rodadas grátis e prémios a dobrar, aposta
  por linha 1/2/5/10, **rodadas automáticas** (10/25), **turbo** e tabela de
  prémios. O **retorno ao jogador é de 96,1 %** - calculado com exatidão a partir
  das fitas dos rolos.
- **Banco Lama**: Casino, Blackjack e Póquer partilham uma conta de **fichas Lama**
  (início 1000, secção `casino` do `mem.json`); os saldos antigos passam
  automaticamente. As apostas são debitadas logo, cada jogo tem o seu próprio
  saldo para o recorde e, sem fichas, recebes um **crédito do banco** até 1000.
- Confetes, chuva de moedas, faixas de grande prémio/mega/jackpot e animações das
  linhas vencedoras; conquistas **Em cheio** (pleno ganho na roleta) e **Jackpot
  Lama** (5 lamas numa linha).

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
  o recorde anterior como chip. Com muitos modos e pouca resolução fica
  **compacto**: Opções, Wiki e Voltar passam para uma só linha e a letra
  ajusta-se - nada sai do ecrã.
- **Visual unificado no jogo**: os 46 jogos partilham a paleta e a tipografia do
  menu - os HUD, ecrãs de preparação e sobreposições seguem o design escolhido
  nas opções (v4.1 / v4 / Clássico), enquanto cada campo de jogo mantém as suas
  cores de identidade. Todos os jogos lidam corretamente com uma mudança de
  resolução a meio da partida, e os nomes dos jogos no menu adaptam-se ao idioma
  (p. ex. «Schach» → «Xadrez»).
- **Wiki integrado** ("LamaWiki"): ajuda detalhada de cada jogo (controlos,
  modos, pontos, dicas) mais páginas gerais - com **campo de pesquisa**,
  categorias, artigos deslocáveis e chips de teclas, nos 14 idiomas.
  Acessível pelo botão **«Wiki / Ajuda»** da barra lateral e a partir do ecrã de
  preparação de cada jogo (abre logo a sua página).
- **Conquistas e estatísticas**: **107 conquistas** em três categorias (23 metas
  globais, 37 marcos de pontuação e 47 momentos especiais como um xeque-mate à
  IA, a peça 4096, um T-Spin Double, 25 problemas de xadrez resolvidos, um
  Killer Sudoku ou o jackpot do lama; no 2048 e no Xadrez, partidas com desfazer
  ou dicas não contam) com **aviso dourado e fanfarra** ao
  desbloquear - mesmo a meio do jogo; os recordes antigos são creditados
  automaticamente. Além disso, um separador de **estatísticas**: tempo total,
  partidas, vitórias, recordes, jogo favorito e tabela por jogo ordenada por
  tempo. Acessível pelo botão **«Conquistas e estatísticas»** da barra
  lateral.
- **Replays**: o minigolfe e o bowling gravam cada ronda. No fim, **P** mostra a
  repetição e **S** guarda-a no arquivo - acessível pelo botão **Replays** na
  barra lateral (um separador por jogo, pausa, saltos de sequência, velocidade
  0,5x a 4x). Pode ser desligado no primeiro arranque e nas opções.

### Utilização

- Escolhe o jogo com o botão no menu à esquerda. Depois aparece o **ecrã de
  preparação**: escolher **Um jogador** ou **Multijogador**, ir às **opções** ou
  voltar. Setas/rato para escolher, Enter começa.
- **ESC** = pausa / continuar (nos menus: voltar).
- **F11** (ou o botão «Ecrã inteiro sim/não») = ecrã inteiro. O ecrã do Pygame
  continua incorporado e é ampliado mantendo a proporção (barras pretas se a
  proporção diferir). A janela pode ser redimensionada livremente.
- **«Voltar ao menu»** termina o jogo e guarda o recorde - tal como mudar para
  outro jogo pela barra lateral.
- **Teclas fixas extra**: além das cinco ações atribuíveis, alguns jogos têm
  teclas próprias (p. ex. reserva **C** e rodar para a esquerda **Z** no Tetris,
  desfazer **U** no 2048, no Xadrez e no Sudoku). Só funcionam se essa tecla não
  estiver atribuída a nenhuma ação nas opções, e aparecem na dica da configuração
  e no wiki. Teclas mantidas premidas são bem detetadas e soltas ao pausar ou com
  Alt-Tab - nada fica «preso».
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
- **Aparência**: escolher o **design da interface** – **UI v4.2** (padrão:
  Midnight Glass – gradiente de meia-noite profundo com luzes suaves a
  flutuar lentamente em índigo, turquesa e magenta, grão de filme fino,
  estrelas esparsas e painéis de vidro fosco com aresta de luz), **UI v4.1**
  (como a UI v4, mas mais viva – estrelas subtis mais Saturno e um buraco
  negro no fundo do ecrã inicial), **UI v4.1.1** (como a v4.1, mas com um
  **padrão ziguezague** em mosaico preto e antracite em vez do céu
  estrelado), **UI v4.1.2** (o mesmo padrão nos azuis da paleta – azul de
  acento dominante e um azul mais escuro no fundo), **UI v4.1.3** (o mesmo
  padrão no índigo da UI v4 sobre preto), **UI v4.1.4** (no grafite da UI v4
  sobre preto), **UI v4** (um visual grafite limpo, plano e totalmente calmo
  com um único acento índigo), **UI v3** (a interface clássica anterior com
  céu estrelado, auroras e brilhos), **UI v2** (a primeira remodelação da
  interface: gradiente azul-marinho, céu estrelado e botões com brilho, sem
  animações) ou **UI v1** (o visual antes da remodelação: fundo escuro liso,
  botões planos, sem efeitos). Todos os cartões mostram uma pequena
  pré-visualização; a escolha aplica-se de imediato a toda a interface (área
  de jogo **e** barra lateral) e é guardada.

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
settings.py          Carregar/guardar definições (som/vibração/teclas/opções de jogo com regras de validação) (JSON)
audio.py             Efeitos sonoros procedurais, loops de música + vibração do comando
menu.py              Ecrãs de idioma, preparação (modo) e opções (som/controlos)
highscore.py         Carregar/guardar recordes (secção em mem.json)
store.py             Ficheiro de gravação central mem.json (secções: mem, highscores, stats, achievements + progresso dos jogos), atómico com cópia .bak
stats.py             Estatísticas do jogador (partidas, tempo, vitórias, recordes) por jogo
achievements.py      Conquistas: definições, lógica de desbloqueio, aviso (toast)
progress.py          Ecrã de conquistas e estatísticas (dois separadores, deslocável)
replay.py            Gravação e arquivo das repetições (replay.json)
replayview.py        Ecrã de replays: lista do arquivo e reprodução
ugc.py               Conteúdo próprio (buracos de minigolfe, níveis de Geometry Dash): armazenamento, validação, exportar/importar (ugc.json)
swear.py             Filtro de palavras para nomes e id (lang/swear/*.yml, 14 idiomas)
filepick.py          Janelas de ficheiros ("Exportar como ...", "Importar")
prestige.py          Sistema de prestígio do Snake
competitive.py       Parâmetros do modo Competitivo do Snake (níveis, slot machine, maçãs de aposta)
ngb.py               Personalização visual ("mods"): cor da cabeça + grelha + menu (mem-ngb.json)
lamabank.py          Banco Lama: conta de fichas comum do Blackjack, Póquer e Casino (secção casino do mem.json)
seedrand.py          Gerador aleatório com números idênticos bit a bit em Python e no navegador (modos diários, novos puzzles)
i18n.py              Motor de tradução (carrega lang/*.json, t("chave"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Textos (uma chave por texto)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Listas do filtro de palavras por idioma (regex, .yml)
lamawiki/
  lamawiki.py          Wiki integrado (pesquisa, categorias, renderizador)
  de.json  en.json  fr.json  es.json  pt.json   Conteúdo do wiki (uma página por jogo + gerais)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Reconstrói as listas de palavras do Wordle (dicionários + frequências)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 letras), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 letras), 14 idiomas
devtools/            Ferramentas de desenvolvimento (não incluídas no .exe)
  merge_staging.py           Integra traduções e páginas do wiki de devtools/staging/ nos 14 ficheiros de idioma
  build_chess_puzzles.py     Gera os 200 problemas de xadrez a partir da base de problemas do Lichess (CC0)
  build_sudoku_killer.py     Gera os 400 Killer Sudokus com solução única
  build_crossyroad_models.py Escreve os modelos voxel do Crossy Road para a versão web
  build_geodash_levels.py    Constrói os 8 níveis do Geometry Dash e prova com o solucionador que cada um é possível, moedas incluídas
  build_geodash_solver.py    Solucionador com o verdadeiro código de passo (soluções em geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Dados de níveis: snake-comp.json, chess-puzzles.json (+ README das fontes), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Auditoria geral (entrada, seedrand Python = JS, gravação, ficheiros de idioma, ecrãs de preparação) + todos os audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Auditorias headless por jogo
  newgames_audit.py  blockjump_audit.py
```

O idioma escolhido é guardado em `mem.json` (na secção `mem`, junto à secção
`highscores` do mesmo ficheiro) e carregado automaticamente no próximo arranque.

**Fontes e licenças:** os 200 problemas de xadrez vêm da
[base de problemas do Lichess](https://database.lichess.org/#puzzles) (licença
**CC0 1.0**, domínio público - obrigado, lichess.org!); os detalhes estão em
`games/levels/chess-puzzles.README.md`. As fontes das listas de palavras do
Wordle estão indicadas em `woordlistz/README.md`.

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

---

<a name="-polski"></a>

## 🇵🇱 Polski

Kolekcja gier na komputer stworzona w Pythonie: **Tkinter** zapewnia okno i menu,
a **Pygame** jest osadzony jako ekran gry wewnątrz okna Tkinter. Czterdzieści sześć
gier ze wspólnymi opcjami, w pełni przypisywalnym sterowaniem, rekordami,
proceduralnymi efektami dźwiękowymi oraz — w części tytułów — trybem wieloosobowym.
Interfejs jest **wielojęzyczny** — **14 języków** (niemiecki / angielski /
francuski / hiszpański / portugalski / polski / turecki / duński / norweski /
szwedzki / fiński / czeski / słoweński / chorwacki); język wybiera się przy
pierwszym uruchomieniu na **ekranie powitalnym**, który pozwala też ustawić
**rozdzielczość** i **dźwięk** (domyślnie wyłączony); poza trzema głównymi językami
wszystkie pozostałe (w tym polski) kryją się za przyciskiem **„Więcej"**. Wszystko
można później w każdej chwili zmienić w opcjach.

### Szybki start

#### Windows

```bat
install-python.bat    :: jednorazowo: instalacja Python 3.13 + .venv + pygame
start.bat             :: uruchom kolekcję gier
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # uruchamia z .venv, w przeciwnym razie systemowy python3
```

`start.bat` / `start.sh` automatycznie używają środowiska wirtualnego `.venv`,
jeśli istnieje, w przeciwnym razie systemowego Pythona. Szczegółowy przewodnik
krok po kroku znajduje się na samym dole w sekcji
**[Przewodnik instalacji](#przewodnik-instalacji)**.

### Gry

| Gra          | Tryby           | Krótki opis |
|--------------|-----------------|-------------|
| **Snake**    | 1 / 2 graczy    | Deluxe Snake z widokiem 2D i 3D, dopalaczem, 6 trybami gry (w tym Competitive), złotymi jabłkami i prestiżem |
| **Pong**     | 1 / 2 graczy    | Klasyk kontra SI lub gracz 2, przełączany tryb ruchu |
| **Air Hockey** | 1 / 2 graczy  | Fizyka 2D z przenoszeniem pędu, sterowanie myszą, SI i power-upy |
| **Kółko i krzyżyk** | 1 / 2 graczy | Gra m,n,k na planszy od 3x3 do 9x9, trzy poziomy SI **lub** lokalnie X kontra O |
| **Breakout** | 1 gracz         | Zbijanie klocków z różnymi rodzajami cegieł, power-upami, combo i wieloma poziomami |
| **Tetris**   | 1 / 2 graczy    | Nowoczesne zasady Guideline (SRS, schowek, podgląd 5 klocków, T-Spiny): Maraton, Sprint 40, Ultra 2:00, pojedynek z SI (3 poziomy) lub we dwoje z liniami śmieci |
| **Invaders** | 1 gracz         | Space Invaders: czyść fale, chroń swoje życia |
| **Asteroids** | 1 / 2 graczy   | Fizyka bezwładności, fale, UFO, power-upy, hiperprzestrzeń — solo lub pojedynek kooperacyjny |
| **Pac-Man**  | 1 gracz         | Wierny klon: 4 SI duchów, kulki mocy, tunel, owoce, poziomy |
| **Flappy Bird** | 1 gracz      | Grawitacyjny lot przez rury, monety, tarcza, dzień/noc, medale |
| **Doodle Jump** | 1 gracz      | Automatyczne skakanie w górę, rodzaje platform, sprężyny, śmigło, potwory |
| **2048**     | 1 gracz         | Układanka z przesuwaniem liczb od 3x3 do 8x8: Klasyczny, Na czas i Bez końca, cofanie, płynne animacje, zapisywane partie |
| **Saper**    | 1 gracz         | Klasyk z bezpiecznym pierwszym kliknięciem, chordingiem, buźką i najlepszymi czasami |
| **Sudoku**   | 1 gracz         | 4 warianty (Klasyczne, X-Sudoku, Killer, Mini 6x6) po 400 poziomów, Sudoku dnia, do 3 gwiazdek na poziom, 4 tryby pomocy, cofanie, zapis gry |
| **Frogger**  | 1 gracz         | Droga + rzeka + 5 przystani, bonusowa mucha, krokodyle, limit czasu, 3 poziomy trudności |
| **Memory**   | 1 / 2 graczy    | Znajdź pary na planszy od 4x4 do 8x6, animacja odwracania, punktacja solo lub pojedynek |
| **Pasjans**  | 1 gracz         | 5 wariantów (Klondike, Spider, FreeCell, Piramida, TriPeaks) z przeciąganiem i cofaniem |
| **Aim Trainer** | 1 gracz      | Wyluzowane strzelanie do celów 3D: mysz steruje kamerą, 4 tryby (precyzja/refleks/ruchome/chill), 3 motywy, w tym czarna dziura |
| **Cztery w rzędzie** | 1 / 2 graczy | Klasyk z animacją opadania: 3 poziomy SI (minimax) lub lokalny pojedynek |
| **Pojedynek czołgów** | 1 / 2 graczy | Pojedynek 2D na arenie z rykoszetującymi pociskami, power-upami, 4 arenami, SI o 3 poziomach |
| **Blackjack** | 1 gracz        | Kasynowy blackjack z butem na 4 talie, double/split i blackjackiem 3:2; gra żetonami Lamy ze wspólnego Banku Lamy |
| **Tunnel Racer** | 1 gracz     | Lot 3D neonowym tunelem: tryb nieskończony + 30 poziomów, sterowanie klawiszami lub myszą, motion blur |
| **Labirynt 3D** | 1 gracz      | Raycaster z perspektywy pierwszej osoby (styl Wolfenstein) z 50 poziomami z ziarna, orbami, minimapą — lub widok 2D z góry |
| **Reversi**  | 1 / 2 graczy    | Othello na 8x8: osaczaj i odwracaj pionki, 3 poziomy SI (minimax) lub lokalny pojedynek |
| **Yahtzee**  | 1 / 2 graczy    | Klasyk w kości z 13 kategoriami, górnym bonusem i Yahtzee; pogoń za rekordem lub hotseat dla 2 graczy |
| **Wordle**   | 1 gracz         | Zgadywanie słów od 4 do 7 liter: Bez końca, Słowo dnia, Dordle i Quordle, tryb trudny, paleta dla daltonistów, statystyki z wykresem, udostępnianie wyniku, prawdziwe listy słów w 14 językach |
| **T-Rex Runner** | 1 gracz     | Nieskończony bieg przez pustynię: zmienny skok, kucanie, kaktusy i pterodaktyle, cykl dzień/noc, rosnące tempo, 3 poziomy trudności |
| **Warcaby**  | 1 / 2 graczy    | 3 zestawy zasad (niemieckie 8×8, międzynarodowe 10×10, checkers), bicie obowiązkowe i latająca damka, 3 poziomy SI (minimax) lub lokalny pojedynek |
| **Poker**    | 1 gracz         | 3 warianty do wyboru: Texas Hold'em przeciw SI, 5 Card Draw i Video Poker; rundy licytacji, blindy, żetony Lamy ze wspólnego Banku Lamy |
| **Szachy**   | 1 / 2 graczy    | Pełne zasady, Chess960 i zegar szachowy, 6 poziomów SI, 200 zadań z bazy Lichess, cofanie/podpowiedź, lista ruchów, eksport PGN lub lokalny pojedynek |
| **Młynek**   | 1 / 2 graczy    | Fazy stawiania/przesuwania/latania, młynki i bicie, opcjonalna zasada latania, 3 poziomy SI lub lokalny pojedynek |
| **Simon**    | 1 / 2 graczy    | Gra pamięciowa Senso: tryby Klasyczny/Szybki/Odwrotny/Mieszany + Pojedynek, dźwięk wył./wł./mieszany, 4/6/9 pól, najlepszy wynik na tryb |
| **Bilard**   | 1 / 2 graczy    | 8-ball, 9-ball i trening w 2D, stały widok 3D lub swobodnie obracana kamera 3D; płynna fizyka, pomoc w celowaniu, 3 poziomy SI |
| **Układanka przesuwana** | 1 gracz | Piętnastka w 3x3/4x4/5x5: wsuwaj ponumerowane kafelki w lukę, sterowanie kliknięciem lub strzałkami, punkty za ruchy i czas |
| **Mastermind** | 1 gracz       | Złam tajny kod kolorów (3 tryby: 4×6, klasyczny, 5×8), czarne/białe kołki podpowiedzi, nieskończona seria jako rekord |
| **Bubble Shooter** | 1 gracz   | Klon Puzzle Bobble: strzelaj pasujące kolory w grupy po trzy, odbicia od ścian, opadające grupy, 3 poziomy trudności |
| **Wisielec** | 1 gracz         | Odgadnij słowo, zanim szubienica będzie gotowa; klawiatura ekranowa, listy słów na język, 3 tryby długości, nieskończona seria |
| **Block Jump** | 1 gracz       | Platformówka 3D w stylu Minecraft: teksturowany świat wokseli z postacią Steve'a, drabinami, płotami i blokami-sprężynami, kamera z 1./3. osoby, motion blur, generowane z ziarna poziomy parkour |
| **Tower Defense** | 1 gracz     | Odpieraj niekończące się fale na 4 mapach: do 11 typów wież z ulepszeniami, sprzedażą i specjalizacją A/B, bossowie, 3 tryby, zdolności aktywne |
| **Minigolf**    | 1 / 2 graczy    | 360 dołków na 40 polach (18 ręcznych, 342 generowane): piasek, rampy, woda, odbijacze, wiatraki i ruchome bloki; karta z par i bonusem hole in one; **własny edytor dołków** z 15 typami obiektów, 12 szablonami i udostępnianiem jako `.lamapgzmap` |
| **Pinball**     | 1 / 2 graczy    | Automat do flipera z 3 stołami: bumpery, slingshoty, cele, tory L-A-M-A, multiball z jackpotem, ratunek kuli, potrącenie i tilt |
| **Bowling**     | 1 / 2 graczy    | 10 frame'ów z oficjalną punktacją strike/spare, prawdziwą fizyką kręgli, hookiem i torem w perspektywie, 3 poziomy trudności |
| **Crossy Road** | 1 gracz       | Niekończące się skoki przez łąki, ulice, rzeki i tory w izometrycznym stylu voxel: dzień/noc, orzeł, 10 postaci do kupienia, trasa dnia |
| **Geometry Dash** | 1 gracz       | Rytmiczna platformówka z kostką, statkiem, kulą, UFO i falą: 8 poziomów od Łatwego do Demona, w każdym 3 sekretne monety, tryb treningu, soundtrack dla każdego poziomu; **edytor poziomów** z udostępnianiem jako `.lamapgzlevel` |
| **Battleship**  | 1 / 2 graczy  | Bitwa morska 10x10: rozstawianie floty przeciąganiem, 3 przełączniki zasad (stykanie, salwa, ponowny strzał), SI z 3 poziomami lub lokalny pojedynek z ekranem przekazania |
| **Casino**      | 1 gracz       | Ruletka europejska ze wszystkimi klasycznymi zakładami i Automat Lamy (5 bębnów, 10 linii, dziki symbol, darmowe spiny); jedno konto żetonów Lamy z Blackjackiem i Pokerem |

**Tryb wieloosobowy (2 graczy lokalnie)** jest dostępny w grach **Snake**, **Pong**,
**Air Hockey**, **Kółko i krzyżyk**, **Tetris (Versus)**, **Asteroids (pojedynek
kooperacyjny)**, **Memory (pojedynek)**, **Cztery w rzędzie**, **Pojedynek czołgów**,
**Reversi**, **Yahtzee**, **Warcaby**, **Szachy**, **Młynek**, **Simon (pojedynek)**,
**Bilard**, **Minigolf**, **Pinball**, **Bowling** i **Battleship** (z ekranem
przekazania, który zasłania floty) — łącznie 20 gier. Tryb wybiera się
bezpośrednio na ekranie przygotowania do gry (*Jeden gracz / Wielu graczy*);
Tetris oferuje dodatkowo **pojedynek z SI**. Wersja przeglądarkowa jest wyłącznie
jednoosobowa.

#### Szczegóły funkcji dla każdej gry

**Snake**
- **NOWOŚĆ — Widok 3D** (klawisz **V** w konfiguracji lub kliknięcie *Widok*):
  plansza jest renderowana jako scena 3D w czasie rzeczywistym — **kamera
  pościgowa** unosi się za wężem, a sterowanie odbywa się **względem kierunku
  patrzenia** (lewo/prawo = obrót, dwa szybkie naciśnięcia = zawrócenie). Z mgłą
  odległości, rozgwieżdżonym niebem, szachownicową podłogą, bandami, obracającymi
  się kryształami jedzenia, cząsteczkami 3D i wstrząsem kamery przy zderzeniu; po
  zakończeniu gry kamera powoli krąży wokół węża. Dopalacz poszerza pole widzenia.
  Dostępne w 3D: *Klasyczny* i *Przeszkody* (ściany są tam zawsze stałe, 3D działa
  tylko dla jednego gracza). Widok jest zapamiętywany w `settings.json`.
- **NOWOŚĆ — Opcje kamery 3D** (w konfiguracji 3D kliknij wiersz *Kamera 3D /
  smooth shake* lub klawisz **K**): osobne menu ze **smooth shake** (łagodniejsza
  kamera, znacznie mniej trzęsienia podczas ruchu/obrotu), regulowanym **polem
  widzenia (FOV)** i **wysokością kamery** oraz przełącznikiem **wstrząs przy
  skręcaniu** (wstrząs ekranu przy skrętach w lewo/prawo wł./wył.). Wszystko
  zapamiętywane w `settings.json`.
- **Dopalacz**: **przytrzymaj** klawisz dopalacza = turbo (podwójna prędkość),
  zużywa wytrzymałość (pasek); po jej wyczerpaniu dopalacz się wyłącza i ładuje
  ponownie. Domyślnie G1 = Spacja/lewy Shift, G2 = Enter/prawy Shift.
- **6 trybów gry** (do wyboru w konfiguracji): *Klasyczny*, *Speed Rush*
  (przyspiesza z każdym jabłkiem), *Przeszkody* (śmiertelne bloki), *Portale*
  (pary teleporterów), *Na czas* (60 sekund, jak najwięcej jabłek) i
  *Competitive* (patrz niżej).
- **NOWOŚĆ — Competitive** (jeden gracz): tryb nieskończony z **awansem poziomów**
  — zaczynasz z dokładnie **jednym** jabłkiem i na początku nie możesz mieć więcej;
  im więcej jabłek zbierzesz łącznie, tym wyższy twój **poziom**, który dokłada na
  pole kolejne jednoczesne jabłko i podnosi mnożnik punktów. **Niebieskie jabłka**
  otwierają **jednorękiego bandytę**: stawką jest twoja długość, wynik bębnów mnoży
  ją lub zmniejsza i na chwilę sprawia, że pojawiają się **dodatkowe jabłka**
  (jackpot przy trzech takich samych symbolach). **Fioletowe jabłka** (hazard)
  stawiają na szali część twojego **rozmiaru** i losowo mnożą tę część, reszta
  pozostaje bezpieczna (nowy rozmiar = rozmiar·(1-p) + rozmiar·p·współczynnik):
  **normalny** stawia stałe 50 % z **x0.5 .. x1.5**, **HARDCORE** jest bardziej
  ryzykowny ze stawką **75-90 %** i **x0.25 .. x2.25**. Twój **rozmiar** wyświetla
  się jako **liczba dziesiętna w lewym górnym rogu** i jest przenoszony dokładnie,
  więc kolejne zakłady bazują na nim. Jest **15 poziomów** (mnożnik do x16, do 16
  jabłek naraz); poziomy znajdują się w `games/levels/snake-comp.json` i można je
  tam rozszerzać bez zmian w kodzie, resztę strojenia zawiera `competitive.py`.
- **NOWOŚĆ — HARDCORE** (przełącznik w konfiguracji Competitive, klawisz **H**):
  każdy **dopalacz pożera długość** twojego węża; tryb oznacza czerwony, świecący
  **napis HARDCORE**. Tylko w Competitive; długość nigdy nie spada poniżej minimum.
  Zapamiętywane w `settings.json`.
- **Złote jabłka** (tymczasowe) dają dużo punktów i natychmiast uzupełniają dopalacz.
- Opcjonalnie: **przechodzenie przez ściany**, jabłka bonusowe, **prestiż** (jeden
  gracz, klawisz **P**).
- **NOWOŚĆ — Personalizacja** (przycisk pędzla w prawym górnym rogu konfiguracji
  lub klawisz **C**): menu czysto wizualne („mody", które *nigdy* nie zmieniają
  rozgrywki) z dwiema zakładkami:
  - **Głowa**: **kolor głowy** węża — 4 gotowe odcienie niebiesko-turkusowe (od
    bardziej niebieskiego do bardziej turkusowego), czerwony, pomarańczowy oraz
    **własny kolor** za pomocą suwaków RGB.
  - **Siatka (drogowskaz)**: nakłada na pole **siatkę współrzędnych** — **numery
    wierszy** (przy lewej i prawej krawędzi) oraz **litery kolumn** (u góry/dołu).
    Dzięki temu na dużych planszach od razu widać, że np. jabłko w polu *8a* leży
    w tym samym wierszu *8* co twoja pozycja *8z*. Kolejność kolorów (5 gotowych +
    dwa własne kolory A/B) ustala motyw kolorystyczny.
  - **Baner**: **włącz/wyłącz** baner mnożnika (np. z fioletowego jabłka) oraz
    ustaw jego **rozmiar** (mniejszy/większy) i **krycie** (bardziej przezroczysty)
    — z podglądem na żywo.
  Wszystko jest zapisywane w `mem-ngb.json`; cała personalizacja wizualna działa
  przez moduł `ngb.py`.
- Wygląd: zaokrąglony wąż z oczami (głowa domyślnie turkusowa), poświata
  dopalacza, cząsteczki.

**Pong**
- Jeden gracz przeciwko SI, tryb wieloosobowy = gracz 2 po prawej. Do 5 punktów.
- **Tryb ruchu przełączany dla każdego zestawu sterowania**: *Ciągły* (naciśnij
  raz -> jedzie dalej, domyślnie) lub *Przytrzymanie* (rusza się tylko, gdy
  trzymasz). Przełączanie: **X** = zestaw sterowania 1, **N** = zestaw sterowania 2
  (zapamiętywane w `settings.json`).
- Fizyka piłki z przyspieszeniem i kątem zależnym od punktu trafienia.

**Air Hockey**
- **Prawdziwa fizyka 2D**: okrągłe bijaki i krążek z przenoszeniem pędu — krążek
  przejmuje prędkość bijaka przy uderzeniu; bandy z odbiciem, lekkie tarcie lodu,
  bramki jako otwory w ścianach bocznych.
- **Sterowanie myszą** w trybie jednoosobowym: bijak podąża za myszą (dowolny
  klawisz wraca do sterowania klawiaturą). Klawiatura: klawisze kierunku w 8
  kierunkach, tryb wieloosobowy = G1 po lewej (WASD), G2 po prawej (IJKL).
- **SI o trzech poziomach** (Łatwy/Średni/Trudny): broni własnej bramki, atakuje
  na swojej połowie i omija krążek, by unikać samobójczych goli.
- **Power-upy** (można wyłączyć): *XL* (większy bijak), *BRAMKA* (bramka
  przeciwnika się kurczy), *>>* (szybszy bijak) — należą do gracza, który ostatni
  dotknął krążka.
- Konfiguracja: poziom trudności, **gole do zwycięstwa** (3/5/7/10), power-upy
  wł./wył. (zapisywane w `settings.json`). Po każdym golu rozgrywa gracz, który go
  stracił.
- Wygląd: świetlny ślad krążka, cząsteczki, pulsujące paszcze bramek, znaczniki
  efektów.

**Kółko i krzyżyk**
- Konfiguracja: poziom trudności (Łatwy/Średni/Trudny) i rozmiar planszy 3x3..9x9;
  długość wygrywająca K = 3 (3x3), 4 (4x4), w innym razie 5.
- **1 gracz** przeciwko SI (Trudny na 3x3 jest nie do pokonania) **lub 2 graczy**
  lokalnie (X kontra O, na zmianę przez kliknięcie). Po zakończeniu gry:
  Enter/kliknięcie = nowa runda, **S** = ustawienia.

**Breakout**
- Rodzaje cegieł: Normalna, **Stalowa** (niezniszczalna), **Bomba** (wybucha),
  **Złota** (dodatkowe punkty).
- Power-upy: laser, kula ognia, lepka, tarcza, moneta i inne; **mnożnik combo**.
- Efekty: cząsteczki, ślady piłki, wstrząs ekranu, wyskakujące punkty, wiele
  wzorów poziomów.
- Konfiguracja: **1/2/3** = poziom trudności, **Lewo/Prawo** = kolor piłki,
  **Góra/Dół** = poziom startowy, **M** = układ. Gra: mysz/strzałki, **Spacja**
  wypuszcza piłkę (wystrzeliwuje laser), **P/Esc** = pauza.

**Tetris**
- **Nowoczesne zasady Guideline**: pole 10x20, klocki z **worka 7**, **system
  obrotu SRS** z prawdziwymi wall kickami (także dla klocka I), obrót w obie
  strony, **schowek** (raz na klocek), **podgląd 5 klocków**, cień klocka i
  **lock delay** (0,5 s, najwyżej 15 resetów).
- **Trzy tryby** na ekranie przygotowania: *Solo*, *Pojedynek z AI* i *2 graczy*.
  Solo oferuje w konfiguracji **Maraton** (poziom startowy 1–15, liczy się do
  rekordu), **Sprint 40 linii** (najlepszy czas) i **Ultra 2 minuty** (najlepszy
  wynik); rekordy trafiają do sekcji `tetris` w `mem.json`.
- **Punktacja Guideline**: od Single do Tetrisa, **T-Spiny** (pełne i mini),
  **Back-to-Back** (x1,5), **combo** i **Perfect Clear** — z napisami na
  ekranie, animacją kasowania linii, smugą hard dropu, cząsteczkami i efektem
  awansu.
- **Pojedynek z liniami śmieci**: skasowane linie wysyłają śmieci do rywala
  (Tetris = 4, T-Spin Double = 4 …), nadchodzące śmieci zapowiada pasek
  ostrzeżenia, a własne ataki je **odejmują**; oba pola dostają tę samą kolejność
  klocków. **SI** ma 3 poziomy, a tempo rośnie co 40 sekund.
- **Sterowanie**: Lewo/Prawo z własnym **DAS/ARR** (ustawianym w konfiguracji),
  Góra = obrót w prawo, Dół = soft drop, Akcja = hard drop; **C**/Shift =
  schowek, **Z**/**Y** = obrót w lewo, **X** = obrót w prawo. We dwoje gracz 1
  odkłada klocek **Q** i obraca w lewo **E**, gracz 2 **prawym Shiftem** /
  **prawym Ctrl**. Po grze: **R** = jeszcze raz, **S** = konfiguracja.

**Asteroids**
- **Fizyka bezwładności**: góra = ciąg w kierunku patrzenia, lewo/prawo = obrót,
  statek nadal dryfuje (lekkie tłumienie); wszystko przechodzi przez krawędzie
  ekranu. Klasyczny **wektorowy wygląd** z płomieniem silnika i rozgwieżdżonym
  niebem; każda skała ma własny losowy kształt wielokąta.
- Skały rozpadają się na dwie mniejsze (3 rozmiary, **20/50/100 punktów**), **fale**
  o rosnącej liczbie i zapowiedź w banerze.
- **UFO** (można wyłączyć): okresowo przecina ekran i celuje w statki (błąd
  celowania zależny od poziomu trudności) — 200 punktów za zestrzelenie.
- **Power-upy** (można wyłączyć), wypadające ze zniszczonych skał: **S** = tarcza
  (6 s nietykalności), **T** = potrójny strzał, **R** = szybki ogień.
- **Hiperprzestrzeń** (klawisz Dół): awaryjny skok w losowe miejsce z 4 s
  odnowienia — i 12 % ryzyka, że rozbijesz się po przybyciu.
- 3 życia, bezpieczne odradzanie z miganiem nietykalności, **dodatkowe życie co
  5000 punktów**; cząsteczki wybuchu i wstrząs kamery.
- **Pojedynek kooperacyjny** (tryb wieloosobowy): oba statki lecą jednocześnie z
  osobnymi życiami i punktami — wygrywa ten, kto zdobędzie więcej punktów.
- Konfiguracja: poziom trudności, UFO wł./wył., power-upy wł./wył. (zapisywane w
  `settings.json`).

**Pac-Man**
- **Klasyczny labirynt 28x31** w neonowym stylu z kropkami, 4 kulkami mocy,
  bocznymi tunelami warp i domkiem duchów pośrodku.
- **Cztery duchy z oryginalnymi zachowaniami** (SI docelowego pola): *Blinky* goni
  wprost, *Pinky* zastawia zasadzkę (4 pola przed graczem), *Inky* korzysta z
  wektora przez Blinky'ego, *Clyde* odsuwa się z bliska.
- **Fazy scatter/chase** na przemian (duchy zawracają przy każdej zmianie); **kulka
  mocy** zmienia duchy na niebieskie i jadalne (łańcuch 200/400/800/1600), potem
  oczy wracają do domku.
- Domek duchów ze **stopniowym wypuszczaniem**, **owoce** jako bonus (na poziom),
  **3 życia**, **dodatkowe życie przy 10 000**, system poziomów (coraz szybszy),
  animacja śmierci, ekrany READY/GAME OVER.
- Konfiguracja: **poziom trudności** (Normalny/Trudny/Ekstremalny) — prędkość
  duchów i czas przestrachu.
- Sterowanie: **strzałki lub WASD**. Enter = nowa gra, S = konfiguracja.

**Flappy Bird**
- **Fizyka grawitacji**: Spacja / Góra / W / **kliknięcie myszą** sprawia, że ptak
  trzepocze; przechyla się w zależności od tempa wznoszenia/opadania.
- Nieskończone **pary rur** z przerwą (+1 za rurę); w przerwach pojawiają się
  **monety** (bonus) i power-up **tarcza** (przeżywa jedno zderzenie).
- **Motywy dzień/noc** zmieniają się wraz z wynikiem; dryfujące chmury (paralaksa),
  przewijające się podłoże.
- Poziom trudności (Łatwy/Normalny/Trudny): rozmiar przerwy, prędkość, odstęp
  między rurami — przerwa nieco się zwęża wraz ze wzrostem wyniku.
- **Medale** (brązowy/srebrny/złoty/platynowy) po zakończeniu gry, animacja
  rozbicia z wstrząsem kamery, rekord.

**Doodle Jump**
- Doodler **skacze automatycznie** przy lądowaniu; sterujesz tylko lewo/prawo (z
  bezwładnością), krawędzie się zawijają, a kamera przewija się w górę w miarę
  wznoszenia.
- **Rodzaje platform**: zielona (normalna), niebieska (ruchoma), brązowa (pęka),
  biała (znika). **Sprężyny** dają superodbicie, a **czapka ze śmigłem** unosi cię
  na chwilę w górę (i czyni nietykalnym).
- **Potwory**: kontakt jest śmiertelny — ale możesz je **zestrzelić** klawiszem
  Góra / Spacja (dodatkowe punkty).
- Punkty = osiągnięta wysokość; trudność rośnie z wysokością. Rekord.
- Sterowanie: lewo/prawo = ruch, Góra / Spacja = strzał.

**2048** — strzałki/WASD przesuwają wszystkie kafelki; równe liczby się łączą.

**2048**
- **Własny ekran konfiguracji** z planszami od **3x3 do 8x8** i trzema trybami:
  *Klasyczny* (cel 2048, potem „Grać dalej?"), *Na czas* (3 minuty, zegar rusza
  z pierwszym ruchem) i *Bez końca*.
- **Płynne animacje**: kafelki się przesuwają, łączą z efektem „pop" i rosną przy
  pojawieniu; wyskakujące punkty, iskry od 128, fala uderzeniowa od 2048 i nowe
  kolory aż do 131072. Wejścia podczas animacji są buforowane.
- **Cofanie** (wyłączone / 3 na grę / bez limitu, klawisz **U** lub Backspace) —
  kto z niego korzysta, gra bez rekordu i bez osiągnięć za kafelki.
- **Zapis i wznawianie**: trwająca partia zapisuje się automatycznie dla każdego
  rozmiaru i trybu; najlepsze wyniki i największy kafelek dla rozmiaru/trybu są w
  sekcji `g2048` pliku `mem.json`.
- Sterowanie: strzałki/WASD lub **przeciągnięcie** myszą/touchpadem,
  **R**/**N** = nowa gra, **Tab** = konfiguracja. Rekord liczy się tylko w
  **4x4 Klasycznym** bez cofania.

**Saper**
- Trzy poziomy: **Początkujący** (9x9, 10 min), **Zaawansowany** (16x16, 40),
  **Ekspert** (30x16, 99) — **najlepszy czas na każdym poziomie** jest zapisywany
  i pokazywany w konfiguracji.
- **Pierwsze kliknięcie jest zawsze bezpieczne** (miny rozmieszczane są dopiero
  potem, obszar 3x3 wokół kliknięcia pozostaje wolny).
- **Lewy przycisk** = odkryj, **prawy przycisk** = flaga (opcjonalnie z cyklem
  znaku zapytania), **F** = flaga pod kursorem, **R** = nowa gra.
- **Chording**: kliknięcie na spełnioną liczbę odkrywa pozostałych sąsiadów.
- Klasyczny interfejs: licznik min, **klikalna buźka** (zdziwiona/w okularach
  przeciwsłonecznych/martwa), stoper; błędne flagi są na końcu przekreślane,
  konfetti przy zwycięstwie.
- Punkty = wartość bazowa poziomu minus sekundy.

**Sudoku**
- **4 warianty** po **400 poziomów** (4 poziomy trudności x 100): *Klasyczne*
  (znane poziomy z ziarna — rozwiązane pozostają odhaczone), *X-Sudoku* (obie
  przekątne zawierają każdą cyfrę dokładnie raz), *Killer* (przerywane klatki
  z sumą; 400 poziomów wygenerowanych wcześniej) i *Mini 6x6*. Każda łamigłówka
  ma **jednoznaczne rozwiązanie** — poziom 12 z „Trudnego" to ta sama łamigłówka
  na każdym komputerze.
- **Sudoku dnia**: jedna łamigłówka dziennie dla wszystkich, taka sama na
  komputerze i w przeglądarce; trudność zależy od dnia tygodnia (od
  poniedziałku Łatwy do soboty Ekspert), a codzienne rozwiązywanie buduje serię.
- **Do 3 gwiazdek na poziom** (rozwiązany · bez błędów i podpowiedzi · dodatkowo
  poniżej czasu docelowego) i **najlepszy czas** w wyborze poziomów; **rozpoczęte
  łamigłówki** zapisują się automatycznie i są wznawiane przy następnym starcie.
- **4 tryby gry** (wybierane przed startem) z mnożnikiem punktów: **Klasyczny**
  (x2,0 — bez pomocy), **Notatki** (x1,5 — + ołówkowe notatki i automatyczne
  kandydaty), **Komfort** (x1,0 — + błędne cyfry na czerwono, oznaczone
  konflikty i błędne sumy klatek, poprawne wpisy się blokują), **Asystent**
  (x0,7 — + podpowiedź, maks. 3). Przy włączonym **limicie 3 błędów** (opcja w
  konfiguracji) trzeci błąd kończy grę.
- Sterowanie: strzałki/WASD = komórka, **1-9** = cyfra (również klawiatura
  numeryczna), **0/Delete/prawy przycisk** = wymaż, **U**/**Z** = cofnij,
  **Y** = ponów, **N** = notatki, **C** = automatyczne kandydaty, **H** =
  podpowiedź, **M** = kolorowy znacznik, **R** = zacznij poziom od nowa, **Q** =
  wybór poziomu. Wpisywanie „najpierw cyfra" (konfiguracja, **I**) i **licznik
  brakujących cyfr** pod każdą cyfrą; w pełni grywalne myszą. Po zakończeniu gry
  **A** pokazuje pełne **rozwiązanie**.
- Punkty = (baza wariantu i poziomu trudności − czas − błędy − podpowiedzi) x
  mnożnik trybu; wszystkie warianty i Sudoku dnia liczą się do rekordu.

**Frogger**
- 5 pasów ruchu (samochody/ciężarówki) i 5 torów rzeki (kłody, żółwie, które
  **nurkują** na wyższych poziomach); u góry 5 przystani — zapełnienie wszystkich =
  następny poziom, wszystko przyspiesza.
- Dodatki: **bonusowa mucha** (+200) w pustych przystaniach, **krokodyle** zajmują
  przystanie na wyższych poziomach, **pasek limitu czasu** dla każdej żaby,
  dodatkowe życie przy 10 000.
- 3 poziomy trudności (prędkość, gęstość ruchu, czas); punkty za każdy nowy rząd,
  przystań = 50 + bonus czasowy, ukończony poziom = +1000.

**Memory**
- Rozmiary planszy **4x4, 6x6, 8x6**; motywy to kombinacje kształt-kolor rysowane
  w całości prymitywami; **animacja odwracania**, niepasujące pary odwracają się z
  powrotem automatycznie.
- **Solo**: baza − 15 za ruch − 2 za sekundę (min. 100). **Pojedynek** (lokalny):
  na zmianę, trafienie = kolejna kolejka, wygrywa gracz z największą liczbą par.

**Pasjans**
- **5 wariantów** na ekranie przygotowania: Klondike (opcja dobierania 1/3), Spider
  (1/2/4 kolory), FreeCell (limit superprzeniesień), Piramida (pary do 13, 2
  rozdania ponowne) i TriPeaks (łańcuch ±1 z mnożnikiem combo).
- **Przeciąganie i upuszczanie** lub klik-klik, **prawy przycisk** = na fundament,
  **U** = nieograniczone cofanie, **R** = nowe rozdanie, Spacja = talia.
- Karty są renderowane bez plików graficznych (`games/cards.py`); wszystkie
  warianty dzielą jedną listę rekordów z formułami zależnymi od wariantu.

**Aim Trainer**
- **Prawdziwe programowe 3D** (jak tryb 3D w Snake'u): stały celownik na środku
  ekranu, **bezpośrednie sterowanie myszą 1:1 jak w strzelance** (przechwycenie
  kursora: kursor zostaje uwięziony w oknie, Esc go zwalnia; regulowana czułość,
  nieograniczony yaw, pitch ±60°). Lewe kliknięcie strzela dokładnie przez środek,
  z błyskiem wylotowym, smugą i cząsteczkami trafienia.
- **4 tryby**: precyzja (60 s, 3 kule, bonus za celność), refleks (30 pojedynczych
  celów, statystyki czasu reakcji), ruchome cele (trasy + mnożnik combo do x4) i
  chill (nieskończony, bez kar, **E** kończy sesję).
- **3 motywy** (w konfiguracji, zapisywane): **kosmos** ze sferą gwiazd, **czarną
  dziurą ze świecącym pierścieniem** i planetą (domyślny), neonowa arena z siatką
  podłogi i słońcem synthwave oraz kryta strzelnica.
- Czułość można zmieniać też w trakcie gry klawiszami **+/-**; do tego **regulowany
  motion blur** (0-80%) dla dodatkowo wyluzowanego wyglądu — oba są zapisywane.

**Cztery w rzędzie**
- Plansza 7x6 z **animacją opadania krążka**, podglądem po najechaniu i pulsującą
  linią zwycięstwa; mysz, strzałki lub bezpośredni wybór **1-7**.
- **3 poziomy SI** (minimax z przeszukiwaniem alfa-beta): Łatwy celowo przeocza
  zagrożenia, Średni niezawodnie blokuje, Trudny planuje głęboko do przodu — lub
  **2 graczy** lokalnie na tym samym urządzeniu.
- Gracz rozpoczynający zmienia się co rundę; rekord liczy twoje **zwycięstwa nad
  SI** w jednej sesji.

**Pojedynek czołgów**
- Pojedynek 2D na arenie: **pociski odbijają się raz od ścian** (rykoszet) —
  trafiaj zza rogów (albo samego siebie!). Pierwszy do 5 rund z odliczaniem.
- **4 areny** (Otwarta, Krzyż, Kolumny, Labirynt) lub losowa rotacja; **power-upy**:
  szybki ogień, tarcza, potrójny strzał.
- **SI o 3 poziomach** — trudna prowadzi swoje strzały i celowo odbija je od ścian
  — lub **2 graczy** na jednej klawiaturze (G1 WASD+Spacja, G2 strzałki+Enter).

**Blackjack**
- Prawdziwe zasady kasyna: **but na 4 talie**, krupier staje na 17, **blackjack
  płaci 3:2**, podejrzenie krupiera przy asie/10; **podwojenie** i **jeden podział**
  (rozdzielone asy dostają po jednej karcie).
- **Żetony Lamy**: Blackjack gra kontem **Banku Lamy**, które dzieli z Pokerem i
  Casino (start 1000, trwale zapisane w `mem.json`). Stawka jest pobierana od razu
  przy rozdaniu; poniżej 10 żetonów Enter bierze **kredyt bankowy**, który
  uzupełnia konto do 1000.
- **Rekord** = najwyższy stan twojego **bilansu w Blackjacku** (1000 plus
  wszystko, co wygrano i przegrano w Blackjacku) — wygrane w ruletce, na
  automacie czy w pokerze się tu nie liczą, kredyty też nie.
- Obsługa przyciskami żetonów i klawiszami (**H**it/**S**tand/**D**ouble/podział
  **X**, **1-4** = zakład, Backspace = cofnij zakład, Enter = rozdanie) z
  animacjami kart; zakryta karta krupiera przy odsłonięciu naprawdę się teraz
  odwraca.

**Tunnel Racer**
- **Lot 3D neonowym tunelem** (renderer programowy jak w Aim Trainerze): belki,
  bloki i **pierścieniowe bramki do przewleczenia**, monety na idealnej linii.
- **Dwa tryby**: nieskończony (prędkość rośnie do limitu, rekord) i **30 poziomów
  z ziarna** z metą, bonusem czasowym i odhaczonym postępem.
- **Sterowanie klawiszami** (domyślne) lub **bezpośrednie sterowanie myszą**
  (przechwycenie kursora, klawisz **C**); do tego regulowany **motion blur**
  (klawisz **B**, 0-80%) — wszystko jest zapisywane.

**Labirynt 3D**
- **Raycaster z perspektywy pierwszej osoby w stylu Wolfenstein** (DDA, mgła
  odległości, sprity) z mouselookiem + WASD, **minimapą** (klawisz **M**) i
  zielonym, pulsującym wyjściem — lub klasyczny **widok 2D z góry** (klawisz **V**
  w konfiguracji).
- **50 poziomów z ziarna**, które wciąż rosną; wyjście zawsze leży w punkcie
  najdalszym od startu, **orby** po drodze dają punkty bonusowe.
- Punktacja: 500 za poziom + 100 za orb + bonus czasowy; rozwiązane poziomy są
  odhaczane, a suma z sesji staje się rekordem.

**Reversi**
- **Othello na 8x8**: stawiaj pionki, które osaczają rzędy przeciwnika, i odwracaj
  wszystko, co zostało zamknięte; ruchy nielegalne są zablokowane, a kolejka bez
  legalnego ruchu jest **pomijana automatycznie**.
- **Jeden gracz przeciwko SI** (3 poziomy: negamax z alfa-beta, wagi pozycyjne +
  mobilność) **lub lokalny pojedynek**, Czarne kontra Białe.
- Legalne pola są podświetlane; grasz **myszą** lub ramką wyboru (strzałki +
  Spacja/Enter). Każde zwycięstwo nad SI liczy jeden punkt do rekordu.

**Yahtzee**
- **Klasyk w kości**: 5 kości, do 3 rzutów na turę, **zatrzymuj** kości
  pojedynczo, potem zapisz jedną z **13 kategorii** (z podglądem na żywo możliwego
  wyniku).
- Pełna karta wyników: górna sekcja z **bonusem 63 punktów (+35)**, trójka/czwórka,
  full, mały/duży strit, **Yahtzee (50)** i Szansa.
- **Jeden gracz jako pogoń za rekordem** o najwyższą sumę lub **hotseat dla 2
  graczy** z dwiema kartami obok siebie; grasz myszą lub klawiszami (Spacja, 1-5,
  strzałki, Enter).

**Wordle**
- Odgadnij ukryte słowo; kolorowa odpowiedź (zielony/żółty/szary) z poprawnym
  **liczeniem powtórzonych liter** i klawiaturą ekranową, która się koloruje
  (QWERTZ dla niemieckiego, czeskiego, słoweńskiego i chorwackiego, AZERTY dla
  francuskiego, w pozostałych QWERTY).
- **Cztery tryby**: *Bez końca* (słowo za słowem, po 6 prób; każde odgadnięte
  słowo daje punkty, pierwsze nieodgadnięte kończy grę), *Słowo dnia* (jedno słowo
  dziennie dla każdego języka i długości — takie samo na komputerze i w
  przeglądarce — z odliczaniem i serią; rozpoczęte słowo dnia jest zapisywane),
  *Dordle* (2 słowa naraz w 7 próbach) i *Quordle* (4 słowa w 9 próbach, klawisze
  pokazują kolory wszystkich plansz).
- **Konfiguracja** przed każdą grą: **długość słowa od 4 do 7**, **tryb trudny**
  (znalezione podpowiedzi trzeba wykorzystywać dalej) i **paleta dla daltonistów**
  (pomarańczowy/niebieski); obok widać statystyki.
- **Prawdziwe listy słów w 14 językach** (katalog `woordlistz/`, tylko A-Z), dla
  każdej długości osobne: przy 5 literach blisko **34 000 haseł** i ponad
  **213 000 dozwolonych słów**, łącznie ok. 134 000 haseł we wszystkich
  długościach. Hasła to popularne słowa bez imion, angielskich wtrąceń i słów
  obraźliwych; każda próba jest sprawdzana z listą — reszta jest odrzucana, a
  wiersz chwilę się trzęsie.
- **Statystyki** dla języka, długości i trybu: gry, procent wygranych, bieżąca i
  najlepsza seria oraz **rozkład prób na wykresie słupkowym** (sekcja `wordle` w
  `mem.json`). **Udostępnij** (**C**) kopiuje siatkę emoji bez zdradzania hasła.
- Rekord liczy tylko *Bez końca* z 5 literami; pozostałe długości mają własne
  najlepsze wyniki. Osiągnięcia **Jasnowidz** (najwyżej 2 próby), **Słowny nawyk**
  (7 słów dnia z rzędu) i **Poczwórny geniusz** (rozwiązane Quordle).

**Poker**
- **3 warianty** na ekranie przygotowania: **Texas Hold'em** przeciw 1–3
  przeciwnikom SI z przyciskiem krupiera, blindami i czterema rundami licytacji,
  **5 Card Draw** (jeden na jednego z SI, jedna wymiana kart) i **Video Poker**
  (*Jacks or Better*, solo przeciw tabeli wypłat).
- Akcje przyciskami lub klawiszami: **F** = pas, **C** = czekaj/sprawdź, **R** =
  podbij, **A** = all-in; zatrzymywanie/wymiana kart kliknięciem lub **1-5**,
  **Enter** dobiera albo rozdaje kolejną rękę.
- **Żetony Lamy** ze wspólnego **Banku Lamy**: na początku rozdania twoje konto
  leży na stole jako stos, a to, co trafia do puli, jest od razu pobierane —
  wyjście od stołu w trakcie rozdania kosztuje tylko twój udział w puli.
  Bankructwo (mniej niż big blind 20, w Video Pokerze mniej niż 10) = kredyt
  bankowy do 1000.
- **Rekord** = najwyższy stan **bilansu w Pokerze** (1000 plus wszystkie wygrane
  i przegrane w pokerze); osiągnięcie **Chip leader** też liczy tylko ten bilans.

**Szachy**
- **Pełne szachy**: wszystkie ruchy bierek, w tym **roszada**, **en passant** i
  **promocja pionka** (do wyboru bierka); **szach, mat i pat** oraz remisy z
  **reguły 50 posunięć**, **trzykrotnego powtórzenia pozycji**,
  **niewystarczającego materiału** lub za zgodą.
- **Trzy tryby**: *partia* z SI, *2 graczy* przy jednym komputerze (plansza może
  się obracać po każdym ruchu) i **zadania**.
- **Silniejsza, płynna SI** w 6 poziomach od *Początkującego* do *Mistrza*:
  iteracyjne pogłębianie, tablica transpozycji, przeszukiwanie spokojne, księga
  otwarć i ocena uwzględniająca mobilność, strukturę pionków i bezpieczeństwo
  króla. SI liczy w małych porcjach na klatkę — gra nigdy się nie zacina.
- **Konfiguracja**: wybór koloru, **zegar szachowy** (bez, 1+0, 3+2, 5+0, 10+5)
  i **Chess960** (wszystkie 960 ustawień początkowych, numer widać nad listą
  ruchów).
- **Panel boczny** z zegarami, zbitymi bierkami, bilansem materiału i przewijaną
  **listą ruchów (SAN)**; przeciąganie, płynnie sunące bierki, współrzędne.
  Klawisze: **U** = cofnij, **H** = strzałka podpowiedzi, **O** = propozycja
  remisu, **X** = poddanie, **F** = obrót planszy, po partii **P** = **eksport
  PGN**.
- **Zadania**: 200 zadań w 5 etapach (mat w 1/2/3, taktyka I/II) z **wolnej bazy
  zadań Lichess (CC0)**, sprawdzonych własnym silnikiem; w zadaniach matowych
  liczy się każdy ruch dający mata. Postęp jest w sekcji `chess` pliku
  `mem.json`.
- Cofanie i podpowiedź czynią partię „wspomaganą": do rekordu liczą się tylko
  zwycięstwa nad SI bez pomocy (w ramach sesji).

**Młynek**
- **Młynek** ze wszystkimi trzema fazami: **stawianie** (po 9 pionków),
  **przesuwanie** wzdłuż linii i **latanie** po zejściu do 3 pionków (można
  wyłączyć).
- Ukończony **młynek** usuwa pionek przeciwnika (najlepiej spoza młynka);
  przegrywasz, gdy spadniesz poniżej 3 pionków lub nie możesz się ruszyć.
- **3 poziomy SI** (minimax z alfa-beta, ocena zależna od fazy) lub **lokalny
  pojedynek**; z podpowiedziami ruchów, podświetlaniem młynków i licznikiem pionków.

**Simon**
- **Gra pamięciowa Senso**: podświetlona sekwencja rośnie z każdą rundą i trzeba ją
  dokładnie powtórzyć.
- **Tryby**: *Klasyczny*, *Szybki* (przyspiesza), *Odwrotny* (od tyłu), *Mieszany*
  (tryb zmienia się co rundę) oraz dwuosobowy **Pojedynek** (na zmianę dokładać i
  powtarzać).
- **Dźwięk** *wył. / wł. / mieszany* (mieszany trenuje pamięć wzrokową ORAZ
  słuchową), **4/6/9 pól** jako poziom trudności; **najlepszy wynik na tryb** jest
  zapisywany. Grasz myszą lub klawiszami numerycznymi 1-9.

**Bilard**
- **8-ball**, **9-ball** i tryb **treningowy** bez zasad, przeciwko SI (z pomocą w
  celowaniu) lub **we dwoje lokalnie**.
- **Trzy dowolnie wybieralne widoki**: klasyczny **2D z góry**, stała **skośna
  perspektywa 3D** z cieniowanymi bilami oraz **swobodnie obracana kamera 3D**
  (prawy przycisk myszy). Cały ruch opiera się na krokach czasowych i jest
  **płynnie wytłumiany** (tarcie, podkroki zapobiegające przenikaniu).
- **Uderzanie**: przytrzymaj lewy przycisk myszy, aby naładować siłę, puść, aby
  uderzyć; pomagają linia celowania i miernik siły. **Bila w ręku** po faulu. Widok
  (V) jest zapamiętywany w `settings.json`; wygrane partie liczą się do rekordu.

**Układanka przesuwana**
- Piętnastka w trzech rozmiarach: **3×3** (łatwa), **4×4** (klasyczna) i **5×5**
  (trudna); wsuwaj ponumerowane kafelki w wolną lukę.
- Zawsze rozwiązywalna (mieszana wieloma losowymi ruchami). Sterowanie przez
  **kliknięcie** kafelka w wierszu/kolumnie luki (przesuwa się cała linia) lub
  **strzałkami**.
- Punkty = wartość bazowa na rozmiar minus ruchy i czas; po rozwiązaniu od razu
  startuje nowa plansza.

**Mastermind**
- Złam ukryty **kod kolorów**; po każdej próbie dostajesz **czarne** kołki (dobry
  kolor + pozycja) i **białe** kołki (dobry kolor, zła pozycja).
- **3 tryby**: Łatwy (4 kołki / 6 kolorów / 12 rzędów), Klasyczny (4/6/10) i Trudny
  (5 kołków / 8 kolorów); kolory mogą się powtarzać.
- Grasz paletą kolorów (kliknięcie lub klawisze **1–8**), OK/Enter sprawdza rząd.
  **Nieskończona seria** jak w Wordle: każdy złamany kod daje punkty.

**Bubble Shooter**
- **Puzzle Bobble** na plastrze miodu: celuj myszą, strzelaj kulami w górę, **trzy
  lub więcej w jednym kolorze** rozbija grupę.
- Kule, które tracą połączenie z sufitem, **spadają** (bonus); strzały **odbijają
  się od ścian**, z podglądem następnej kuli.
- **3 tryby** (4/5/6 kolorów, część z opadającymi rzędami); koniec gry na czerwonej
  linii.

**Wisielec**
- Odgadnij słowo **litera po literze**; każdy błąd rysuje część wisielca, przegrana
  po **6 błędach**.
- **Listy słów na język** (tylko A–Z), **3 tryby długości** (krótkie / mieszane /
  długie); pisz lub klikaj klawiaturę ekranową.
- **Nieskończona seria**: każde odgadnięte słowo daje punkty (więcej pozostałych
  żyć + dłuższe słowo = więcej).

**Block Jump**
- **Platformówka 3D w stylu Minecraft** (programowe 3D jak tryb 3D w Snake'u):
  skacz po unoszącym się **świecie wokseli** z bloków aż do świecącego celu.
- **Skin z Minecrafta**: wszystkie bloki mają prawdziwe **tekstury pikselowe**
  (trawa, ziemia, kamień, deski, diament, śluz, drewno); poziom szczegółów
  zależy od odległości (**T** = wysokie/niskie/wyłączone).
- Do tego: animowana postać **Steve'a** (kamera z 3. osoby), **ręka** w widoku
  z 1. osoby, **promień sygnalizatora** przy celu, obracające się **sztabki
  złota** zamiast monet, kwadratowe **słońce**, **pikselowe chmury** i HUD
  z **sercami**.
- Rodzaje bloków: bloki stałe (trawa/ziemia/kamień/drewno), **drabiny**
  (wspinaczka), **płoty** (przeskakiwanie), **bloki-sprężyny** (wyrzut) i
  **monety**.
- Kamera **domyślnie z pierwszej osoby jak w Minecrafcie**, **V** przełącza na
  kamerę pościgową; **rozglądanie myszą** z przechwyceniem kursora, regulowany
  **motion blur** (**B**) i czułość (**+/-**).
- **Generowane z ziarna poziomy parkour** stają się coraz trudniejsze; cel = punkty
  + bonus czasowy, monety +50, upadek kosztuje życie (start z 3). Sterowanie:
  WASD/strzałki, **Spacja** = skok.

**Tower Defense**
- **Niekończąca się obrona przed falami** na **4 mapach** (Łąka, Kanion,
  Skrzyżowanie, Szpaler), każda z własną ścieżką; zablokowane mapy odblokowuje
  najlepsza fala, co **8 fal** nadchodzi **boss**.
- **3 tryby**: Klasyczny (7 wież, tryb główny), Kompaktowy (4 wieże, 2 poziomy)
  i Maksymalny (**11 wież**, **specjalizacja A/B** na najwyższym poziomie,
  specjalni wrogowie, zdolności aktywne **Meteor/Nova mrozu/Gorączka złota**).
- **11 typów wież**, od łuczniczej po laser i bank złota, każda z maks.
  **3 poziomami ulepszeń**, sprzedaż zwraca 70%; wrogowie z pancerzem,
  regeneracją, podziałem, kamuflażem, aurą leczenia i trasą powietrzną.
- **Ekonomia**: złoto za zestrzelenia, bonus za falę + 5% odsetek; punkty za
  zestrzelenia i fale. **F** = tempo x2, **G** = zasięgi, prawy przycisk
  anuluje.

**Minigolf**
- **360 dołków na 40 polach**: *Classic* i *Pro* po 9 ręcznie zbudowanych
  dołków, **Tour** z 38 polami po 9 generowanych dołków (łącznie 342) o rosnącej
  trudności, do tego *Random* z całości. Pole 7, dołek 3 wygląda wszędzie tak
  samo - nic nie trzeba zapisywać.
- **Nawierzchnie i przeszkody**: piasek hamuje, rampy przyspieszają, woda kosztuje
  uderzenie karne, gumowe odbijacze oddają prędkość, a wiatraki i ruchome bloki
  wymagają wyczucia czasu. Fizyka liczy się w podkrokach z tarciem jak w bilardzie
  - nic nie skacze i nie przenika przez bandę.
- **Sterowanie**: mysz celuje, przytrzymanie lewego przycisku ładuje siłę, a
  puszczenie uderza (można też strzałkami + spacją). **R** anuluje naładowane
  uderzenie bez uderzania. **G** przełącza linię celowania, **Z**
  auto-celowanie, **P** podnoszenie.
- **Blokada siły (przytrzymany prawy przycisk)**: zatrzymuje pasek ładowania
  dokładnie tam, gdzie jest - złoty, z procentami, kłódką i pulsującym
  pierścieniem wokół piłki. Tak czekasz na przerwę w wiatraku z naładowanym
  uderzeniem. Puszczenie ładuje dalej; zablokowana siła przetrwa nawet
  uderzenie, a kolejne lewe kliknięcie uderza dokładnie tą wartością.
- **Karta wyników** po prawej z par i uderzeniami na dołek; we dwoje każdy gra ten
  sam dołek po kolei. Punkty: 600 za dołek, ±300 za uderzenie poniżej/powyżej par,
  **500 dodatkowo za hole in one**. Najmniejsza liczba uderzeń na każdym polu
  zapisuje się w sekcji `minigolf` pliku `mem.json`.
- **Podnoszenie można wyłączyć**: domyślnie dołek kończy się po ośmiu
  uderzeniach i liczy się na minimum. Kto woli grać aż do wbicia, ustawia
  *Podnoszenie* na WYŁ w ustawieniach (albo naciska **P**).
- **Auto-celowanie można wyłączyć**: domyślnie kij przed każdym uderzeniem sam
  obraca się w stronę dołka. Kto woli celować sam na każdym dołku, ustawia
  *Auto-celowanie* na WYŁ. w ustawieniach (albo naciska **Z**) - wtedy zostaje
  ostatnio wybrany kierunek, a na tee nowego dołka kij wskazuje neutralnie w
  górę.
- **F** resetuje bieżący dołek: uderzenia na 0, piłka na tee - ten sam dołek, ten
  sam kurs.
- **Dalej zamiast powtórki**: na koniec rundy przycisk **Dalej** prowadzi do
  kolejnego kursu (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), więc te same
  dziewięć dołków się nie powtarza; obok: **Jeszcze raz** (ten sam kurs) i
  **Ustawienia**. Klawisze: Enter = dalej, R = jeszcze raz, S = ustawienia.
- **Powtórka rundy**: na końcu **P** (albo przycisk **Powtórka**) pokazuje całą
  rundę uderzenie po uderzeniu. Klawisz **S** przenosi ją do archiwum (przycisk
  **Powtórki** na pasku bocznym).
- **Budowanie i udostępnianie własnych dołków**: zakładka **MAPS** na ekranie
  przygotowania prowadzi do własnej kolekcji - **Nowy** otwiera edytor. Każdy
  dołek dostaje nazwę i **id** (małe litery, bez spacji); id jest zarazem
  proponowaną nazwą pliku przy udostępnianiu. Do siedmiu klasycznych przeszkód
  dochodzi **osiem nowych**: rura (przenosi piłkę na drugi koniec), lód,
  lepkie pole, przyspieszacz, magnes, bramka jednokierunkowa, obrotnica i
  skocznia. Rozmiar dołka ustawia się dowolnie (60x80 do 160x240), **12
  szablonów** daje punkt wyjścia, a cofanie/ponawianie oraz **Test** są w
  komplecie. **Udostępnij** zapisuje dokładnie jeden dołek jako plik
  `.lamapgzmap` - przez okno zapisu albo prosto do folderu Pobrane, z twoją
  nazwą jako twórcy. **Importuj** wczytuje go z powrotem i przy zajętym id
  automatycznie przechodzi na `-2`. Nazwa, id i twórca zawsze przechodzą przez
  **filtr słów obejmujący wszystkie 14 języków**. Dołek gra się pojedynczo
  przez **Graj**, a całą kolekcję przez piąty wybór trasy **Własne**.

**Pinball**
- **Trzy stoły**: *Classic* (trzy bumpery, jedna seria celów), *Space* (cztery
  bumpery w romb, dwie serie) i *Lama* (otwarte pole, sześć celów w łuku); 3 lub 5
  kul na partię, we dwoje na zmianę kula po kuli.
- **Wszystko, czego potrzebuje fliper**: wyrzutnia z paskiem ładowania (za słabo?
  kula wraca i można powtórzyć), dwa flipery, slingshoty, serie celów, cztery tory
  **L-A-M-A**, łapacz z blokadą kuli, **multiball z jackpotem**, sześć sekund
  **ratunku kuli**, potrącanie i **TILT**.
- **Mnożnik do x5** dzięki zbitym seriom i pełnym torom; bumpery 100, slingshoty
  50, cele 250 - w multiballu bumpery płacą 2500 jako jackpot.
- Flipery działają na przypisanych klawiszach lewo/prawo (oraz lewy/prawy
  [Shift]) albo myszą. Rekord każdego stołu leży w sekcji `pinball` pliku
  `mem.json`.

**Bowling**
- **Dziesięć frame'ów według oficjalnych zasad** wraz ze strike'ami, spare'ami i
  rzutami dodatkowymi w dziesiątym frame (maksimum: 300). **Karta wyników** pod
  nagłówkiem pokazuje każdy frame z X, / i sumą bieżącą.
- **Rzut w czterech krokach**: pozycja, kąt, rotacja i siła. Każdy suwak waha się
  sam i zatrzymuje go klawisz akcji - albo ustawia się go ręcznie strzałkami
  lewo/prawo, co przerywa wahanie.
- **Prawdziwa fizyka kręgli**: dziesięć kręgli to koła z masą, które przewracają
  się nawzajem; strike wynika z fizyki, a nie z przypadku. Tor jest z przodu
  naoliwiony, więc **hook** łapie dopiero w ostatniej trzeciej części.
- Widok toru w perspektywie z rynnami, strzałkami i deską kręgli; trzy poziomy
  trudności (*Łatwy/Normalny/Pro*) zmieniają tempo suwaków i rozrzut. Rekord dla
  każdego poziomu leży w sekcji `bowling` pliku `mem.json`.
- **Powtórka partii**: na końcu **P** pokazuje wszystkie rzuty jeszcze raz, a
  **S** zapisuje je w archiwum (przycisk **Powtórki**).

**Crossy Road**
- **Niekończące się skoki** przez łąki (drzewa i kamienie blokują drogę), ulice z
  samochodami i ciężarówkami, rzeki z pniami i liśćmi lilii oraz **tory**, po
  których po sygnale świetlnym i dzwonku pędzi pociąg — dalej czekają całe stacje
  z maksymalnie 5 torami. Trasa powstaje rząd po rzędzie, zawsze ma przejście, a
  tempo i ruch rosną.
- **Izometryczny styl voxel**: postacie, pojazdy i drzewa z cieniowanych bloków
  (wstępnie renderowane dla każdego rozmiaru pola), płynna kamera, squash &
  stretch przy skokach, plusk wody, animacja zgniecenia, pióra i błyszczące
  monety; od rzędu 50 **zmiana dnia i nocy** z reflektorami.
- **Orzeł**: kamera powoli sunie do przodu — kto za długo zwleka albo cofnie się
  o więcej niż trzy rzędy, tego porywa orzeł (wcześniej ostrzega czerwona
  ramka). Zniesienie na pniu poza ekran też kończy grę.
- **Monety i postacie**: zebrane monety (wielka moneta = 5) zostają zapisane i
  kupują nowe postacie w zakładce **Postacie**: żaba, świnia, pingwin, kot, lis,
  lama, robot, duch i jednorożec (25–250 monet); kurczak jest od początku.
- **Tryby**: *Bez końca* (punkty = najdalszy rząd, liczy się do rekordu) i
  *Trasa dnia* (dziś taka sama dla wszystkich, także w przeglądarce, z własnym
  rekordem dnia). Sterowanie: strzałki/WASD, Spacja/Enter/klik = skok do przodu;
  w konfiguracji **H** = cienie, **N** = dzień/noc. Monety, postacie i rekord
  dnia są w sekcji `crossy` pliku `mem.json`.

**Geometry Dash**
- **Rytmiczna platformówka**: postać sama pędzi w prawo - ty decydujesz tylko,
  kiedy skoczyć lub lecieć. **Pięć form** — kostka, statek, kula, UFO i fala — do
  tego portale formy, grawitacji i prędkości (0,5x do 3x), żółte/różowe/niebieskie
  **wyrzutnie i kule**, półbloki, kolce, doły i wyzwalacze kolorów.
- **8 wbudowanych poziomów** od *Łatwego* do *Demona* („Lama Inferno"), w każdym
  **3 sekretne monety**. Udowodniono, że każdy poziom da się przejść: przy budowie
  solver ukończył go prawdziwym kodem gry — razem ze wszystkimi monetami, nawet
  przesunięty o 1/240 sekundy.
- **Precyzyjna fizyka**: obliczenia stałoprzecinkowe ze stałym krokiem 240 Hz;
  każde wciśnięcie działa dokładnie w kroku, w którym nastąpiło — tak samo przy
  każdej liczbie klatek i bit w bit tak samo w przeglądarce.
- **Tryb treningu** (**P**) z automatycznymi i własnymi punktami kontrolnymi
  (**Z** stawia, **X** usuwa), licznikiem prób, paskiem postępu, eksplozjami i
  natychmiastowym restartem (**R**). Każdy poziom ma **własny soundtrack** — tło,
  podłoże i kule pulsują w rytm (muzykę wyłącza **M**).
- **Gwiazdki i monety**: kto ukończy poziom w trybie normalnym, dostaje jego
  gwiazdki, a każda moneta jest warta kolejną; rekord to **suma gwiazdek**
  (najwyżej 65). Najlepsze wyniki poziomów, monety, próby i skoki są w sekcji
  `geodash` pliku `mem.json`.
- **Edytor poziomów** w zakładce **POZIOMY**: płótno z siatką, paleta z 6 grupami
  (bloki, zagrożenia, wyrzutnie i kule, portale, prędkość, dodatki), obracanie,
  cofnij/ponów, pasek podglądu, **test od startu lub od tego miejsca** i
  ustawienia poziomu (prędkość i forma startowa, styl muzyki, BPM, kolory).
  Znaczek **„zweryfikowany"** pojawia się dopiero, gdy sam przejdziesz swój
  poziom. **Udostępnij** zapisuje plik `.lamapgzlevel`, **Import** wczytuje go z
  powrotem; poziomy trafiają do `ugc.json` obok własnych dołków do minigolfa.

**Battleship**
- **Bitwa morska 10x10** z lotniskowcem (5 pól), pancernikiem (4), krążownikiem
  (3), okrętem podwodnym (3) i niszczycielem (2) — wygrywa ten, kto pierwszy
  zatopi całą wrogą flotę.
- **Rozstawianie floty** przeciąganiem z doku: **R** lub prawy przycisk obraca,
  podgląd świeci na zielono lub czerwono, **X** rozstawia losowo, **C** czyści
  planszę; ostatnie ustawienie jest proponowane ponownie.
- **Zasady w konfiguracji** (zapisywane): *statki mogą się stykać*, *salwa* (tyle
  strzałów na turę, ile twoich statków jest na wodzie) i *kolejny strzał po
  trafieniu*.
- **SI z 3 poziomami**: Łatwy strzela losowo, Średni systematycznie dobija
  trafienia, Trudny liczy **mapę prawdopodobieństwa** z parzystością szachownicy
  (średnio ok. 70 / 60 / 45 strzałów na całą flotę). Albo **2 graczy** przy
  jednym komputerze — **ekran przekazania** zasłania obie floty przed każdą turą.
- **Oprawa**: omiatanie radaru, animowane fale, pociski lecące łukiem, plusk,
  eksplozje z dymem i płonące pola, odsłonięcie „ZATOPIONY!" oraz podsumowanie
  rundy ze strzałami, trafieniami i celnością. Rekord liczy **zwycięstwa nad SI**
  w jednej sesji.

**Casino**
- **Ruletka** (europejska, 37 pól): wszystkie klasyczne zakłady kliknięciem w
  liczbę, krawędź lub róg — **plein** (35:1), cheval, transversale, carré,
  sixain, kolumna, tuzin, czerwone/czarne, parzyste/nieparzyste i
  manque/passe. Żetony 1/5/25/100/500, prawy przycisk zdejmuje żetony;
  **Zakręć**, **Powtórz** (**R**), **Podwój** (**D**) i **Wyczyść**. Kulka wpada
  spiralą do pola wylosowanego wcześniej, a u góry widać ostatnie 12 liczb.
- **Automat Lamy**: 5 bębnów x 3 rzędy, **10 linii wygrywających**, **lama =
  dziki symbol**, **złote monety = scatter** z 10 darmowymi spinami i
  podwojonymi wygranymi, stawka na linię 1/2/5/10, **auto-spin** (10/25),
  **turbo** i tabela wypłat. **Wskaźnik zwrotu wynosi 96,1%** — policzony
  dokładnie z pasów bębnów.
- **Bank Lamy**: Casino, Blackjack i Poker dzielą jedno konto **żetonów Lamy**
  (start 1000, sekcja `casino` w `mem.json`); stare stany żetonów przechodzą
  automatycznie. Stawki są pobierane od razu, każda gra prowadzi własny bilans
  do rekordu, a przy bankructwie dostajesz **kredyt bankowy** do 1000.
- Konfetti, deszcz monet, banery big/mega/jackpot i animacje linii
  wygrywających; osiągnięcia **W dziesiątkę** (wygrany plein w ruletce) i
  **Jackpot Lamy** (5 lam na jednej linii).

Rekordy są przechowywane w sekcji `highscores` pliku `mem.json` (obok kodu) —
razem z językiem (sekcja `mem`).

### Interfejs

Cały interfejs jest rysowany od zera (czysty Tkinter + Pygame, bez dodatkowych
pakietów) i dopracowany w stylu nowoczesnego launchera gier:

- **Pasek boczny z listą gier**: każdy wiersz ma własny **mini-piktogram** w
  kolorze akcentu danej gry, pokazuje aktualny **rekord (★)** i reaguje płynnie
  animowanymi efektami najechania. Bieżąca gra pozostaje wyróżniona kolorem; przy
  małych oknach lista **przewija się** kółkiem myszy.
- **Karta stanu** w lewym dolnym rogu z **diodą stanu** (szara = menu, zielona = w
  toku, złota = pauza, czerwona = koniec gry) i **licznikiem FPS na żywo**.
- **Ekran bezczynności** ze światłami zorzy, paralaksowym polem gwiazd ze
  spadającymi gwiazdami, unoszącym się logo z iskrami na orbicie, **klikalną siatką
  gier** tuż pod logo (wszystkie gry z efektem najechania w swoim kolorze akcentu)
  i **przewijanym paskiem rekordów**.
- **Efekty wszędzie**: miękkie przejścia ekranów, iskry przy potwierdzaniu w menu,
  **deszcz konfetti przy nowym rekordzie** i prawdziwe **rozmycie** za nakładką
  pauzy.
- **Ekran przygotowania** każdej gry pojawia się w jej kolorze akcentu i pokazuje
  poprzedni rekord jako chip. Przy wielu trybach i małej rozdzielczości staje się
  **kompaktowy**: Opcje, Wiki i Powrót trafiają do jednego rzędu, a czcionka się
  dopasowuje — nic już nie wychodzi poza ekran.
- **Jednolity wygląd w grze**: wszystkie 46 gier korzysta z tej samej palety motywu
  i czcionki co menu — HUD-y, ekrany konfiguracji i nakładki podążają za wyglądem
  wybranym w opcjach (v4.1 / v4 / Klasyczny), a każde pole gry zachowuje swoje
  kolory tożsamości. Każda gra poprawnie obsługuje zmianę rozdzielczości w trakcie
  gry, a nazwy w menu zależą od języka (np. „Schach" → „Szachy").
- **Wbudowana wiki** („LamaWiki"): szczegółowa pomoc do każdej gry (sterowanie,
  tryby, punktacja, wskazówki) plus strony ogólne — z **polem wyszukiwania**,
  kategoriami, przewijanymi artykułami i chipami klawiszy, we wszystkich 14
  językach. Dostępna przez przycisk **„Wiki / Pomoc"** na pasku bocznym i z ekranu
  przygotowania każdej gry (otwiera bezpośrednio jej stronę).
- **Osiągnięcia i statystyki**: **107 osiągnięć** w trzech kategoriach (23 cele
  ogólne, 37 progów punktowych i 47 wyjątkowych momentów, jak mat SI, kafelek
  4096, T-Spin Double, 25 rozwiązanych zadań szachowych, Killer Sudoku czy
  lamowy jackpot; w 2048 i szachach partie z cofaniem lub podpowiedziami się
  nie liczą) ze **złotym powiadomieniem i fanfarą** przy odblokowaniu -
  nawet w trakcie gry; stare rekordy są zaliczane automatycznie. Do tego
  zakładka **statystyk**: łączny czas gry, rozgrywki, zwycięstwa, rekordy,
  ulubiona gra i tabela gier posortowana według czasu. Dostępne przez przycisk
  **„Osiągnięcia i statystyki"** na pasku bocznym.
- **Powtórki**: minigolf i bowling nagrywają każdą rundę. Na końcu **P**
  pokazuje powtórkę, a **S** zapisuje ją w archiwum - dostępnym pod przyciskiem
  **Powtórki** na pasku bocznym (zakładka na grę, pauza, skoki między
  sekwencjami, tempo 0,5x do 4x). Można to wyłączyć przy pierwszym starcie i w
  opcjach.

### Obsługa

- Wybierz grę przyciskiem w menu po lewej. Następnie pojawia się **ekran
  przygotowania**: wybierz **Jeden gracz** lub **Wielu graczy**, przejdź do
  **opcji** albo wróć. Strzałki/mysz do wyboru, Enter uruchamia.
- **ESC** = pauza / wznowienie (w menu: wstecz).
- **F11** (lub przycisk „Pełny ekran wł./wył.") = przełącz pełny ekran. Wyświetlacz
  Pygame pozostaje osadzony i jest skalowany w górę z zachowaniem proporcji (czarne
  pasy przy innym stosunku boków). Okno można dowolnie skalować.
- **„Powrót do menu"** kończy grę i zapisuje rekord — tak samo jak przejście do
  innej gry z paska bocznego.
- **Stałe klawisze dodatkowe**: oprócz pięciu przypisywalnych akcji niektóre gry
  mają własne klawisze (np. schowek **C** i obrót w lewo **Z** w Tetrisie,
  cofnięcie **U** w 2048, szachach i Sudoku). Działają tylko wtedy, gdy klawisz
  nie jest przypisany do żadnej akcji w opcjach, a wymieniają je podpowiedź w
  konfiguracji i wiki. Przytrzymane klawisze są poprawnie rozpoznawane i
  zwalniane przy pauzie lub Alt-Tab — nic się nie „zacina".
- **„Wyjście"** zamyka Pygame i Tkinter w czysty sposób.

### Opcje, sterowanie i dźwięk

Ekran opcji otwiera się przyciskiem **„Opcje / Sterowanie"** (po lewej) lub z
ekranu przygotowania. Jest podzielony na **trzy zakładki** (**Ogólne / Sterowanie /
Wygląd**; przełączasz kliknięciem lub klawiszem Tab):

- **Ogólne**: **dźwięk** wł./wył., **głośność** i **haptyka** (wibracje gamepada,
  działają tylko z podłączonym kontrolerem) oraz **automatyczna rozdzielczość**,
  **rozdzielczość**, **FPS** i **język** — każde przełączane strzałkami Lewo/Prawo.
- **Sterowanie**: **szablony** (*WASD + strzałki*, *WASD + IJKL*, *strzałki +
  WASD*) oraz **każdy pojedynczy klawisz** gracza 1 i gracza 2 do przypisania na
  nowo: wybierz wiersz, naciśnij Enter, naciśnij żądany klawisz (Esc anuluje).
- **Wygląd**: wybierz **projekt interfejsu** — **UI v4.2** (domyślny: Midnight Glass
  — głęboki północny gradient z powoli dryfującymi, miękkimi światłami w indygo,
  turkusie i magencie, delikatnym ziarnem filmu, rzadkimi gwiazdami i panelami jak
  matowe szkło ze świetlistą krawędzią), **UI v4.1** (jak UI v4, ale żywszy —
  subtelne gwiazdy oraz Saturn i czarna dziura w tle ekranu startowego),
  **UI v4.1.1** (jak v4.1, ale zamiast rozgwieżdżonego nieba kafelkowy
  **wzór zygzak** w czerni i antracycie), **UI v4.1.2** (ten sam wzór w niebieskich
  barwach palety — niebieski akcent jako kolor dominujący, ciemniejszy niebieski
  jako tło), **UI v4.1.3** (ten sam wzór w indygo z UI v4 na czerni), **UI v4.1.4**
  (w grafitowym tonie UI v4 na czerni), **UI v4** (całkowicie spokojny, płaski
  wygląd grafitowy z jednym akcentem indygo), **UI v3** (poprzedni klasyczny
  interfejs z rozgwieżdżonym niebem, światłami zorzy i efektami poświaty), **UI v2**
  (pierwsza przebudowa interfejsu: granatowy gradient, rozgwieżdżone niebo i
  świecące przyciski, bez animacji) lub **UI v1** (wygląd sprzed przebudowy
  interfejsu: jednolite ciemne tło, płaskie przyciski, bez efektów). Wszystkie karty
  pokazują mały podgląd; wybór działa natychmiast na cały interfejs (obszar gry
  **i** pasek boczny) i jest zapisywany.

Ustawienia są trwale zapisywane w `settings.json`. W trybie **jednoosobowym** oba
przypisania sterują tą samą postacią (domyślnie: WASD *i* strzałki), w
**wieloosobowym** po jednym na gracza. Wszystkie gry mają **efekty dźwiękowe**
(generowane proceduralnie, bez potrzeby dodatkowych plików), które można globalnie
wyciszyć.

### Struktura projektu

```
install-python.bat  Instalacja na Windows: Python 3.13 + .venv + pygame
start.bat            Skrypt uruchamiający (Windows)
start.sh             Skrypt uruchamiający (Linux / macOS / Git Bash)
pyinstall.bat        Budowanie EXE (Windows): pakuje wszystko do builds\PyGameZ.exe
main.py              Interfejs Tkinter, osadzenie Pygame, centralna pętla gry
game_base.py         Klasa bazowa gry (update/draw/handle_event) + InputEvent + funkcje pomocnicze
settings.py          Wczytywanie/zapis ustawień (dźwięk/haptyka/klawisze/opcje gier z regułami sprawdzania) (JSON)
audio.py             Proceduralne efekty dźwiękowe, pętle muzyczne + wibracje gamepada
menu.py              Ekran języka, przygotowania (tryb) i opcji (dźwięk/sterowanie)
highscore.py         Wczytywanie/zapis rekordów (sekcja w mem.json)
store.py             Centralny plik zapisu mem.json (sekcje: mem, highscores, stats, achievements + postępy gier), atomowo z kopią .bak
stats.py             Statystyki gracza (rozgrywki, czas, zwycięstwa, rekordy) na grę
achievements.py      Osiągnięcia: definicje, logika odblokowań, powiadomienie (toast)
progress.py          Ekran osiągnięć i statystyk (dwie zakładki, przewijany)
replay.py            Nagrywanie i archiwum powtórek (replay.json)
replayview.py        Ekran powtórek: lista archiwum i odtwarzanie
ugc.py               Własne treści (dołki do minigolfa, poziomy Geometry Dash): przechowywanie, sprawdzanie, eksport/import (ugc.json)
swear.py             Filtr słów dla nazw i id (lang/swear/*.yml, wszystkie 14 języków)
filepick.py          Okna plików ("Eksportuj jako ...", "Importuj")
prestige.py          System prestiżu dla Snake
competitive.py       Strojenie trybu Competitive w Snake (poziomy, slot machine, jabłka zakładów)
ngb.py               Personalizacja wizualna („mody"): kolor głowy + siatka współrzędnych + menu (mem-ngb.json)
lamabank.py          Bank Lamy: wspólne konto żetonów Blackjacka, Pokera i Casino (sekcja casino w mem.json)
seedrand.py          Generator losowy z identycznymi co do bitu liczbami w Pythonie i przeglądarce (tryby dnia, nowe łamigłówki)
i18n.py              Silnik tłumaczeń (wczytuje lang/*.json, t("klucz"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Teksty językowe (jeden klucz zastępczy na tekst)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Listy filtra słów dla każdego języka (regex, .yml)
lamawiki/
  lamawiki.py          Wbudowana wiki (wyszukiwanie, kategorie, renderer artykułów)
  de.json  en.json  fr.json  es.json  pt.json   Treść wiki (jedna strona na grę + strony ogólne)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Buduje listy słów Wordle na nowo (słowniki + listy częstości)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 liter), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 liter), 14 języków
devtools/            Narzędzia deweloperskie (nie trafiają do .exe)
  merge_staging.py           Wgrywa tłumaczenia i strony wiki z devtools/staging/ do wszystkich 14 plików językowych
  build_chess_puzzles.py     Buduje 200 zadań szachowych z bazy zadań Lichess (CC0)
  build_sudoku_killer.py     Generuje 400 Killer Sudoku z jednoznacznym rozwiązaniem
  build_crossyroad_models.py Zapisuje modele voxel Crossy Road dla wersji przeglądarkowej
  build_geodash_levels.py    Buduje 8 poziomów Geometry Dash i solverem dowodzi, że każdy da się przejść razem z monetami
  build_geodash_solver.py    Solver korzystający z prawdziwego kodu kroku (rozwiązania w geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Dane poziomów: snake-comp.json, chess-puzzles.json (+ README ze źródłami), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Audyt całościowy (wejście, seedrand Python = JS, zapis, pliki językowe, ekrany przygotowania) + wszystkie audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Audyty headless dla każdej gry
  newgames_audit.py  blockjump_audit.py
```

Wybrany język jest zapisywany w `mem.json` (w sekcji `mem`, obok sekcji
`highscores` w tym samym pliku) i automatycznie wczytywany przy następnym
uruchomieniu.

**Źródła i licencje:** 200 zadań szachowych pochodzi z
[bazy zadań Lichess](https://database.lichess.org/#puzzles) (licencja
**CC0 1.0**, domena publiczna — dziękujemy, lichess.org!); szczegóły są w
`games/levels/chess-puzzles.README.md`. Źródła list słów do Wordle podaje
`woordlistz/README.md`.

### Uwagi dotyczące platform

Wyświetlacz działa **poza ekranem** (off-screen): pygame używa atrapy sterownika
wideo (`SDL_VIDEODRIVER=dummy`), więc renderuje do powierzchni, a każda klatka jest
rysowana jako obraz w widżecie Tkinter. **Nie ma natywnego okna SDL**, które
mogłoby walczyć z Tkinter o rozmiar/pozycję. Dzięki temu okno zachowuje się wszędzie
tak samo i stabilnie:

- **Windows**: proces jest dodatkowo oznaczony jako DPI-aware, aby obraz pozostawał
  ostry na skalowanych ekranach (125/150/200 %) i się nie „trząsł".
- **Linux/X11 i Wayland**: działa bez przypadków szczególnych (bez `SDL_WINDOWID`).
- **macOS**: również działa (wcześniej osadzone okno w ogóle się tu nie
  wyświetlało).

---

### Przewodnik instalacji

Wymagania: **Python 3.9+** (zalecany 3.12 lub 3.13) i **pygame ≥ 2.6**.

#### Windows (zalecane: automatycznie)

1. Otwórz folder projektu i kliknij dwukrotnie **`install-python.bat`**. Skrypt
   - sprawdza, czy jest **Python 3.13**, a jeśli nie, instaluje go przez **winget**
     (`winget install Python.Python.3.13`),
   - tworzy środowisko wirtualne **`.venv`**,
   - instaluje **pygame** z `requirements.txt`.
2. Następnie uruchom kolekcję za pomocą **`start.bat`** (dwukrotne kliknięcie).

> Uwaga: jeśli skrypt zgłosi „jeszcze niedostępne w tym oknie", Python właśnie
> został zainstalowany — po prostu otwórz **nowy terminal/okno** i uruchom
> `install-python.bat` ponownie. Jeśli **winget** nie jest dostępny, zainstaluj
> Pythona 3.13 ręcznie z <https://www.python.org/downloads/> i zaznacz
> **„Add python.exe to PATH"**.

#### Windows / Linux / macOS (ręcznie)

```bash
# 1. Sprawdź Pythona (3.9+)
python --version

# 2. Utwórz i aktywuj środowisko wirtualne
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Zainstaluj zależności
pip install -r requirements.txt
#   lub:  pip install "pygame>=2.6" (lub pygame-ce)
#                                    pip install pygame-ce
# 4. Uruchom
python main.py
```

#### Linux / macOS z start.sh

```bash
# Przygotuj Python + venv jak wyżej (kroki 2 i 3), następnie:
chmod +x start.sh      # jednorazowo, jeśli nie jest jeszcze wykonywalny
./start.sh
```

W systemie Linux w razie potrzeby zainstaluj Pythona menedżerem pakietów, np.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); w macOS np.
`brew install python`.

#### Użycie innej wersji Pythona

`install-python.bat` domyślnie instaluje Pythona 3.13. Jeśli wolisz 3.12 (lub inną
wersję), zmień w pliku wiersz `set "PYVER=3.13"` na żądaną wersję, a identyfikator
winget odpowiednio (`Python.Python.3.12`).

#### Budowanie samodzielnego pliku EXE (Windows)

```bat
pyinstall.bat         :: buduje builds\PyGameZ.exe (wszystko w jednym pliku)
```

`pyinstall.bat` używa `.venv` (i tworzy je w razie potrzeby), automatycznie
instaluje **PyInstaller** i pakuje kompletną grę — Pythona, pygame, wszystkie gry,
języki, wiki i loga — w **jeden plik `PyGameZ.exe`** w folderze **`builds\`**. Plik
działa na każdym komputerze z Windows bez zainstalowanego Pythona i można go
dowolnie kopiować. Ustawienia i rekordy (`settings.json`, `mem.json`,
`mem-ngb.json`) są tworzone obok pliku .exe podczas gry.

#### Rozwiązywanie problemów

- **`pygame` nie znaleziony** → czy venv jest aktywowany? Powtórz krok 3
  (`pip install -r requirements.txt`).
- **`python` nie jest rozpoznawany (Windows)** → Python został zainstalowany bez
  „Add to PATH"; zainstaluj ponownie i zaznacz to pole albo użyj `py` zamiast
  `python`.
- **Brak dźwięku** → sprawdź „Dźwięk" w opcjach; haptyka działa tylko z kontrolerem.
- **Okno/osadzenie w Linuksie** → zobacz *Uwagi dotyczące platform*
  (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ powrót na górę / back to top</a></b></div>

---

<a name="-turkce"></a>

## 🇹🇷 Türkçe

Python ile hazırlanmış bir masaüstü oyun koleksiyonu: pencereyi ve menüyü
**Tkinter** sağlar, **Pygame** ise oyun ekranı olarak Tkinter penceresinin içine
gömülüdür. Ortak seçenekler, tamamen yeniden atanabilen kontroller, yüksek
skorlar, prosedürel ses efektleri ve bazı oyunlarda çok oyunculu mod içeren kırk
altı oyun. Arayüz **çok dillidir** – **14 dil** (Almanca / İngilizce /
Fransızca / İspanyolca / Portekizce / Lehçe / Türkçe / Danca / Norveççe /
İsveççe / Fince / Çekçe / Slovence / Hırvatça); dil, ilk açılışta bir
**karşılama ekranında** seçilir; bu ekran ayrıca **çözünürlüğü** ve **sesi**
(öntanımlı olarak kapalı) ayarlamanıza da olanak tanır; üç ana dilin dışında
kalan diğer tüm diller (İspanyolca, Portekizce ve —Türkçe dahil— dokuz ek dil)
**«Daha fazla»** düğmesinin arkasında durur. Her şey daha sonra seçeneklerden
istediğiniz zaman değiştirilebilir.

### Hızlı başlangıç

#### Windows

```bat
install-python.bat    :: tek seferlik: Python 3.13 + .venv + pygame kurar
start.bat             :: oyun koleksiyonunu başlatır
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # .venv ile başlar, yoksa sistem python3'ü ile
```

`start.bat` / `start.sh`, varsa `.venv` sanal ortamını otomatik olarak kullanır,
yoksa sistemdeki Python'ı. Ayrıntılı, adım adım bir kılavuz en altta
**[Kurulum Kılavuzu](#kurulum-kılavuzu)** başlığı altında yer alır.

### Oyunlar

| Oyun         | Modlar          | Kısa açıklama |
|--------------|-----------------|---------------|
| **Snake**    | 1 / 2 oyuncu    | 2D ve 3D görünüm, hızlanma (boost), 6 oyun modu (Rekabetçi dahil), altın elmalar ve prestij içeren lüks Snake |
| **Pong**     | 1 / 2 oyuncu    | Yapay zekâya ya da 2. oyuncuya karşı klasik oyun, değiştirilebilir hareket modu |
| **Air Hockey** | 1 / 2 oyuncu  | Momentum aktarımlı 2D fizik, fare kontrolü, yapay zekâ ve güçlendirmeler |
| **Tic-Tac-Toe** | 1 / 2 oyuncu | 3x3'ten 9x9'a m,n,k oyunu, üç yapay zekâ seviyesi **veya** yerel X'e karşı O |
| **Breakout** | 1 oyuncu        | Tuğla türleri, güçlendirmeler, kombolar ve pek çok bölüm içeren tuğla kırma oyunu |
| **Tetris**   | 1 / 2 oyuncu    | Modern Guideline kuralları (SRS, saklama, 5 parça önizleme, T-Spin): Maraton, Sprint 40, Ultra 2:00, yapay zekâya karşı Versus (3 seviye) ya da çöp satırlı iki kişilik düello |
| **Invaders** | 1 oyuncu        | Space Invaders: dalgaları temizle, canlarını koru |
| **Asteroids** | 1 / 2 oyuncu   | Atalet fiziği, dalgalar, UFO'lar, güçlendirmeler, hiperuzay - tek başına ya da ortaklaşa düello |
| **Pac-Man**  | 1 oyuncu        | Aslına sadık klon: 4 hayalet yapay zekâsı, güç hapları, tünel, meyveler, bölümler |
| **Flappy Bird** | 1 oyuncu     | Borular arasında yer çekimiyle uçuş, madeni paralar, kalkan, gündüz/gece, madalyalar |
| **Doodle Jump** | 1 oyuncu     | Yukarı doğru otomatik zıplama, platform türleri, yaylar, pervane, canavarlar |
| **2048**     | 1 oyuncu        | 3x3'ten 8x8'e sayı kaydırma bulmacası: Klasik, Zamana karşı ve Sonsuz, geri alma, akıcı animasyonlar, kaydedilen oyunlar |
| **Minesweeper** | 1 oyuncu     | Güvenli ilk tıklama, chording, gülen yüz ve en iyi süreler içeren klasik oyun |
| **Sudoku**      | 1 oyuncu     | Her biri 400 bölümlü 4 tür (Klasik, X-Sudoku, Killer, Mini 6x6), Günün Sudokusu, bölüm başına 3 yıldıza kadar, 4 yardım modu, geri alma, kayıt |
| **Frogger**     | 1 oyuncu     | Yol + nehir + 5 yuva, bonus sinek, timsahlar, süre sınırı, 3 zorluk |
| **Memory**      | 1 / 2 oyuncu | 4x4'ten 8x6'ya kadar eş bul, çevirme animasyonu, tekli puanlama ya da düello |
| **Solitaire**   | 1 oyuncu     | Sürükle-bırak ve geri al özellikli 5 çeşit (Klondike, Spider, FreeCell, Piramit, TriPeaks) |
| **Aim Trainer** | 1 oyuncu     | Sakin 3D hedef atışı: fare kamerayı yönlendirir, 4 mod (isabet/refleks/hareketli/sakin), kara delik dahil 3 tema |
| **Dört Taş**    | 1 / 2 oyuncu | Düşen pul animasyonlu klasik oyun: 3 yapay zekâ seviyesi (minimax) ya da yerel düello |
| **Tank Düellosu** | 1 / 2 oyuncu | Sekmeli atışlar, güçlendirmeler, 4 arena ve 3 seviyeli yapay zekâ içeren 2D arena düellosu |
| **Blackjack**    | 1 oyuncu    | 4 desteli shoe, ikiye katlama/bölme ve 3:2 blackjack içeren kumarhane blackjack'i; ortak Lama Bankası'nın Lama çipleriyle oynanır |
| **Tunnel Racer** | 1 oyuncu    | 3D neon tüp uçuşu: sonsuz mod + 30 bölüm, tuş ya da fare ile kontrol, hareket bulanıklığı (motion blur) |
| **3D Labirent**  | 1 oyuncu    | Birinci şahıs raycaster (Wolfenstein tarzı), 50 tohumlu bölüm, orblar, mini harita - ya da 2D kuşbakışı görünüm |
| **Reversi**      | 1 / 2 oyuncu | 8x8 Othello: taşları kıstırıp çevir, 3 yapay zekâ seviyesi (minimax) ya da yerel düello |
| **Kniffel (Yahtzee)** | 1 / 2 oyuncu | 13 kategori, üst bonus ve Yahtzee içeren klasik zar oyunu; rekor avı ya da 2 oyunculu hotseat |
| **Wordle**       | 1 oyuncu    | 4 ile 7 harfli kelime tahmini: Sonsuz, Günün kelimesi, Dordle ve Quordle, zor mod, renk körü paleti, çubuk grafikli istatistik, sonucu paylaşma, 14 dilde gerçek kelime listeleri |
| **T-Rex Runner** | 1 oyuncu    | Sonsuz çöl koşusu: değişken zıplama, eğilme, kaktüsler ve pterodaktiller, gündüz/gece döngüsü, artan hız, 3 zorluk |
| **Dama**         | 1 / 2 oyuncu | 3 kural seti (Alman 8×8, Uluslararası 10×10, Checkers), zorunlu yeme ve uçan dama, 3 yapay zekâ seviyesi (minimax) ya da yerel düello |
| **Poker**        | 1 oyuncu    | Seçilebilir 3 çeşit: yapay zekâya karşı Texas Hold'em, 5 Card Draw ve Video Poker; bahis turları, blind'lar, ortak Lama Bankası'nın Lama çipleri |
| **Satranç**      | 1 / 2 oyuncu | Tüm kurallar, Chess960 ve satranç saati, 6 yapay zekâ seviyesi, Lichess veritabanından 200 bulmaca, geri alma/ipucu, hamle listesi, PGN dışa aktarma ya da yerel düello |
| **Dokuz Taş**    | 1 / 2 oyuncu | Dizme/hareket/uçma aşamaları, değirmenler ve alışlar, isteğe bağlı uçma kuralı, 3 yapay zekâ seviyesi ya da yerel düello |
| **Simon**        | 1 / 2 oyuncu | Senso hafıza oyunu: Klasik/Hız/Ters/Karışık modlar + Düello, ses kapalı/açık/karışık, 4/6/9 tuş, mod başına en iyi skor |
| **Bilardo**      | 1 / 2 oyuncu | 2D'de 8-top, 9-top ve antrenman, sabit 3D görünüm ya da serbest dönen 3D kamera; yumuşak fizik, nişan yardımı, 3 yapay zekâ seviyesi |
| **Kaydırmalı Bulmaca** | 1 oyuncu | 3x3/4x4/5x5 boyutlarında 15 bulmacası: numaralı karoları boşluğa kaydır, tıklama ya da ok tuşu kontrolü, hamle ve süreye göre puan |
| **Mastermind**     | 1 oyuncu  | Gizli renk kodunu çöz (3 mod: 4×6, klasik, 5×8), siyah/beyaz geri bildirim pimleri, sonsuz seri rekoru |
| **Bubble Shooter** | 1 oyuncu  | Puzzle Bobble klonu: eşleşen renkleri üçlü gruplar halinde ateşle, duvar sekmeleri, düşen kümeler, 3 zorluk |
| **Adam Asmaca**    | 1 oyuncu  | Darağacı tamamlanmadan kelimeyi tahmin et; ekran klavyesi, dile göre kelime listeleri, 3 uzunluk modu, sonsuz seri |
| **Block Jump**  | 1 oyuncu     | Minecraft tarzı 3D platform oyunu: dokulu voksel dünyası, Steve figürü, merdivenler, çitler ve balçık blokları, birinci/üçüncü şahıs kamera, hareket bulanıklığı, tohumla üretilen parkur bölümleri |
| **Tower Defense** | 1 oyuncu   | 4 haritada sonsuz dalgaları püskürt: geliştirme, satış ve A/B uzmanlaşmalı 11 kule tipine kadar, bosslar, 3 mod, aktif yetenekler |
| **Minigolf**    | 1 / 2 oyuncu | 40 parkurda 360 delik (18 elle tasarlanmış, 342 üretilmiş): kum, rampalar, su, tamponlar, yel değirmenleri ve gezen bloklar; par ve hole-in-one bonuslu skor kartı; 15 nesne türü, 12 şablon ve `.lamapgzmap` olarak paylaşımla **kendi delik düzenleyicisi** |
| **Pinball**     | 1 / 2 oyuncu | 3 masalı pinball makinesi: bumperlar, slingshotlar, hedefler, L-A-M-A şeritleri, jackpotlu multiball, top koruma, sarsma ve tilt |
| **Bowling**     | 1 / 2 oyuncu | Resmî strike/spare puanlamasıyla 10 frame, gerçek labut fiziği, hook efekti ve perspektifli pist, 3 zorluk |
| **Crossy Road** | 1 oyuncu     | İzometrik voksel görünümde çayırlar, yollar, nehirler ve raylar üzerinden sonsuz zıplama: gece/gündüz, kartal, satın alınabilen 10 karakter, günün parkuru |
| **Geometry Dash** | 1 oyuncu    | Küp, gemi, top, UFO ve dalgalı ritim platform oyunu: Kolay'dan İblis'e her birinde 3 gizli altın olan 8 bölüm, antrenman modu, bölüm başına müzik; `.lamapgzlevel` olarak paylaşımlı **bölüm editörü** |
| **Battleship**  | 1 / 2 oyuncu | 10x10 deniz savaşı: sürükle-bırak filo yerleşimi, 3 kural anahtarı (temas, salvo, tekrar ateş), 3 seviyeli yapay zekâ ya da devir ekranlı yerel düello |
| **Casino**      | 1 oyuncu     | Tüm klasik bahislerle Avrupa ruleti ve Lama Slotu (5 makara, 10 hat, joker, bedava dönüşler); Blackjack ve Poker ile tek bir Lama çipi hesabı |

**Çok oyunculu (2 oyuncu yerel)** şu oyunlarda mevcuttur: **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (ortaklaşa
düello)**, **Memory (düello)**, **Dört Taş**, **Tank Düellosu**, **Reversi**,
**Kniffel**, **Dama**, **Satranç**, **Dokuz Taş**, **Simon (düello)**,
**Bilardo**, **Minigolf**, **Pinball**, **Bowling** ve **Battleship** (filoları
gizleyen bir devir ekranıyla) - toplam 20 oyun. Mod, doğrudan oyun öncesi
ekranında (*Tek oyunculu / Çok oyunculu*) seçilir; Tetris ayrıca **yapay zekâya
karşı Versus** sunar. Web sürümü yalnızca tek oyunculudur.

#### Oyun başına özellik ayrıntıları

**Snake**
- **YENİ - 3D görünüm** (setup'ta **V** tuşu ya da *Görünüm* tıklaması): tahta
  gerçek zamanlı bir 3D sahne olarak işlenir - yılanın arkasında bir **takip
  kamerası** süzülür ve yönlendirme **bakış yönüne göre** yapılır (sol/sağ = dön,
  iki hızlı basış = U dönüşü). Mesafe sisi, yıldızlı gökyüzü, satranç tahtası
  zemin, kenar duvarları, dönen yem kristalleri, 3D parçacıklar ve çarpışmada
  kamera sarsıntısıyla; oyun bittikten sonra kamera yılanın etrafında yavaşça
  döner. Hızlanma görüş alanını genişletir. 3D'de mevcut: *Klasik* ve *Engeller*
  (orada duvarlar her zaman katıdır, 3D yalnızca tek oyunculudur). Görünüm
  `settings.json` içinde hatırlanır.
- **YENİ - 3D kamera seçenekleri** (3D setup'ta *3D kamera / smooth shake*
  satırına tıklayın ya da **K** tuşu): **smooth shake** (daha yumuşak kamera,
  hareket/dönme sırasında çok daha az titreme), ayarlanabilir **görüş alanı
  (FOV)** ve **kamera yüksekliği** ile bir de **dönerken sarsıntı** anahtarı
  (sol/sağ dönüşlerde ekran sarsıntısı açık/kapalı) içeren özel bir menü. Hepsi
  `settings.json` içinde hatırlanır.
- **Hızlanma**: hızlanma tuşunu **basılı tutmak** = turbo (iki kat hız),
  dayanıklılık harcar (çubuk); tükendiğinde hızlanma kapanır ve yeniden dolar.
  Öntanımlı O1 = Boşluk/Sol Shift, O2 = Enter/Sağ Shift.
- **6 oyun modu** (setup'ta seçilebilir): *Klasik*, *Speed Rush* (her elmayla
  hızlanır), *Engeller* (ölümcül bloklar), *Portallar* (ışınlanma çiftleri),
  *Zamana Karşı* (60 saniye, olabildiğince çok elma) ve *Rekabetçi* (aşağıya
  bakın).
- **YENİ - Rekabetçi** (tek oyunculu): **seviye tırmanışı** olan sonsuz mod - tam
  olarak **bir** elmayla başlarsınız ve başta daha fazlasını alamazsınız;
  topladığınız toplam elma arttıkça **seviyeniz** yükselir; bu da alana sürekli
  bir elma daha ekler ve skor çarpanını artırır. **Mavi elmalar** bir **slot
  makinesi** açar: bahsiniz uzunluğunuzdur, makaraların sonucu bunu çarpar ya da
  küçültür ve kısa süreliğine **ekstra elmalar** oluşturur (üç aynı sembolde
  jackpot). **Mor elmalar** (kumar) **boyutunuzun** bir kısmını ortaya koyar ve o
  kısmı rastgele çarpar, geri kalan güvende kalır (yeni boyut = boyut·(1-p) +
  boyut·p·çarpan): **normal**'de sabit %50 bahis, **x0.5 .. x1.5** ile;
  **HARDCORE**'da ise **%75-90** bahis ve **x0.25 .. x2.25** ile daha risklidir.
  **Boyutunuz** **sol üstte ondalık sayı olarak** gösterilir ve tam olarak
  taşınır, böylece sonraki bahisler onun üzerine kurulur. **15 seviye** vardır
  (çarpan x16'ya kadar, aynı anda 16 elmaya kadar); seviyeler
  `games/levels/snake-comp.json` içinde tutulur ve koda dokunmadan orada
  genişletilebilir, geri kalan ince ayar `competitive.py` içinde yer alır.
- **YENİ - HARDCORE** (Rekabetçi setup'ında anahtar, **H** tuşu): her
  **hızlanma yılanınızın uzunluğunu yer**; kırmızı parlayan bir **HARDCORE
  yazısı** modu belirtir. Yalnızca Rekabetçi'de; uzunluk asla minimumun altına
  düşmez. `settings.json` içinde hatırlanır.
- **Altın elmalar** (geçici) bol puan verir ve hızlanmayı anında doldurur.
- İsteğe bağlı **duvarlardan geçme**, bonus elmalar, **prestij** (tek oyunculu,
  **P** tuşu).
- **YENİ - Kişiselleştir** (setup'ın en sağ üstündeki fırça düğmesi ya da **C**
  tuşu): oyunu *asla* değiştirmeyen ("mods") yalnızca görsel bir menü, iki
  sekmeli:
  - **Baş**: yılanın **baş rengi** - 4 mavi-turkuaz hazır ayar (daha maviden
    daha turkuaza), kırmızı, turuncu ve RGB kaydırıcılarıyla bir **özel renk**.
  - **Izgara (yön levhası)**: alanın üzerine bir **koordinat ızgarası** bindirir
    - **satır numaraları** (sol ve sağ kenarlarda) ve **sütun harfleri**
    (üst/alt). Böylece büyük tahtalarda örneğin *8a*'daki elmanın, kendi
    konumunuz *8z* ile aynı *8* satırında olduğunu anında görürsünüz. Renk dizisi
    (5 hazır ayar + iki özel renk A/B) renk temasını belirler.
  - **Afiş**: çarpan afişini (örneğin mor elmadan gelen) **açar/kapatır** ve
    **boyutunu** (daha küçük/daha büyük) ile **saydamlığını** (daha şeffaf)
    ayarlar - canlı önizlemeyle.
  Her şey `mem-ngb.json` içinde saklanır; tüm görsel kişiselleştirme `ngb.py`
  modülü üzerinden çalışır.
- Görsel: gözleri olan yuvarlak hatlı yılan (baş öntanımlı olarak turkuaz),
  hızlanma parıltısı, parçacıklar.

**Pong**
- Tek oyunculu yapay zekâya karşı, çok oyunculu = sağdaki 2. oyuncu. İlk 5 puana
  kadar.
- **Kontrol setine göre değiştirilebilir hareket modu**: *Sürekli* (bir kez bas
  -> hareket etmeye devam eder, öntanımlı) ya da *Basılı tut* (yalnızca basılıyken
  hareket eder). Değiştirme: **X** = kontrol seti 1, **N** = kontrol seti 2
  (`settings.json` içinde hatırlanır).
- Vuruş noktasına göre ivme ve açı içeren top fiziği.

**Air Hockey**
- **Gerçek 2D fizik**: momentum aktarımlı yuvarlak vurucular ve puck - puck
  vuruşta vurucunun hızını alır; geri tepmeli duvarlar, hafif buz sürtünmesi, yan
  duvarlardaki açıklıklar biçiminde kaleler.
- Tek oyuncuda **fare kontrolü**: vurucu fareyi takip eder (herhangi bir tuş
  klavyeye geri döndürür). Klavye: 8 yönde yön tuşları, çok oyunculu = O1 solda
  (WASD), O2 sağda (IJKL).
- **Üç seviyeli yapay zekâ** (Kolay/Orta/Zor): kendi kalesini savunur, kendi
  yarısında saldırır ve kendi kalesine gol atmamak için puck'ın etrafından döner.
- **Güçlendirmeler** (kapatılabilir): *XL* (daha büyük vurucu), *TOR* (rakibin
  kalesi küçülür), *>>* (daha hızlı vurucu) - bunlar puck'a en son dokunan
  oyuncuya aittir.
- Setup: zorluk, **kazanmak için gol sayısı** (3/5/7/10), güçlendirmeler
  açık/kapalı (`settings.json` içinde kaydedilir). Her golden sonra golü yiyen
  oyuncu servis atar.
- Görsel: puck ışık izi, parçacıklar, nabız gibi atan kale ağızları, efekt
  rozetleri.

**Tic-Tac-Toe**
- Setup: zorluk (Kolay/Orta/Zor) ve tahta boyutu 3x3..9x9; kazanma uzunluğu
  K = 3 (3x3), 4 (4x4), aksi hâlde 5.
- **1 oyuncu** yapay zekâya karşı (3x3'te Zor yenilmez) **ya da 2 oyuncu** yerel
  (X'e karşı O, tıklayarak sırayla). Oyun bittiğinde: Enter/tıklama = yeni tur,
  **S** = ayarlar.

**Breakout**
- Tuğla türleri: Normal, **Çelik** (yok edilemez), **Bomba** (patlar), **Altın**
  (ekstra puan).
- Güçlendirmeler: lazer, ateş topu, yapışkan, kalkan, madeni para ve daha
  fazlası; **kombo çarpanı**.
- Efektler: parçacıklar, top izleri, ekran sarsıntısı, puan baloncukları, çok
  sayıda bölüm deseni.
- Setup: **1/2/3** = zorluk, **Sol/Sağ** = top rengi, **Yukarı/Aşağı** =
  başlangıç bölümü, **M** = düzen. Oyun: fare/oklar, **Boşluk** topu fırlatır
  (lazer ateşler), **P/Esc** = duraklat.

**Tetris**
- **Modern Guideline kuralları**: 10x20 alan, **7'li torbadan** gelen parçalar,
  gerçek wall kick'li **SRS dönüş sistemi** (I parçası dahil), iki yöne dönüş,
  **saklama** (parça başına bir kez), **5 parça önizleme**, gölge parça ve **lock
  delay** (0,5 sn, en fazla 15 sıfırlama).
- Oyun öncesi ekranında **üç mod**: *Tek Kişilik*, *YZ'ye Karşı* ve *2 Oyuncu*.
  Tek Kişilik, ayarlarda **Maraton** (başlangıç seviyesi 1-15, rekora sayılır),
  **Sprint 40 satır** (en iyi süre) ve **Ultra 2 dakika** (en iyi skor) sunar; en
  iyiler `mem.json` dosyasının `tetris` bölümünde durur.
- **Guideline puanlaması**: Single'dan Tetris'e, **T-Spin'ler** (tam ve mini),
  **Back-to-Back** (x1,5), **combo'lar** ve **Perfect Clear** - ekran yazıları,
  satır silme animasyonu, hard drop izi, parçacıklar ve seviye atlama efektiyle.
- **Çöp satırlı Versus**: silinen satırlar rakibe çöp gönderir (Tetris = 4,
  T-Spin Double = 4 …), gelen çöp bir uyarı çubuğunda duyurulur ve kendi
  saldırılarınla **mahsup edilir**; iki alan da aynı parça sırasını alır. **Yapay
  zekâ** 3 seviyelidir ve hız her 40 saniyede artar.
- **Kontroller**: kendi **DAS/ARR** değerleriyle Sol/Sağ (ayarlardan
  değiştirilebilir), Yukarı = sağa çevir, Aşağı = soft drop, Aksiyon = hard drop;
  **C**/Shift = saklama, **Z**/**Y** = sola çevir, **X** = sağa çevir. İki
  kişilikte 1. oyuncu **Q** ile saklar ve **E** ile sola çevirir, 2. oyuncu
  **sağ Shift** / **sağ Ctrl** ile. Oyun bitince: **R** = tekrar, **S** =
  ayarlar.

**Invaders** – iki mod (oyun öncesi ekranda seçilebilir):
- **Klasik**: klasik uzaylı bloğu; ardından setup ekranında seçilebilir:
  **Hareket** (yalnızca sol/sağ *ya da* WASD ile serbest) ve **Nişan alma** (her
  zaman yukarı *ya da* **fareye** doğru – o zaman imlecin olduğu yere ateş
  edersiniz). Yok edilen uzaylılar bazen güçlendirme düşürür.
- **Arena (serbest)**: her yönde serbest hareket, düşmanlar her kenardan akın
  eder; hareket yönünde nişan alırsınız, silahı **1–4** ile değiştirirsiniz.
Ortak: her 4. bölümde **boss** olan seviye sistemi, dört silah (blaster, saçma
atış, seri ateş, lazer), güçlendirmeler (ekstra can, kalkan, silah yükseltmesi),
patlama efektleri, yüksek skor.

**Asteroids**
- **Atalet fiziği**: yukarı = bakış yönünde itiş, sol/sağ = dön, gemi kaymaya
  devam eder (hafif sönümleme); her şey ekran kenarlarında sarar. İtiş alevi ve
  yıldızlı gökyüzüyle klasik **vektör görünümü**; her kayanın kendine ait
  rastgele bir çokgen şekli vardır.
- Kayalar iki daha küçük parçaya bölünür (3 boyut, **20/50/100 puan**), giderek
  artan sayıda **dalgalar** ve afiş duyurusu.
- **UFO** (kapatılabilir): ekranı düzenli olarak geçer ve gemileri hedef alır
  (nişan hatası zorluğa bağlıdır) - düşürmek 200 puan.
- **Güçlendirmeler** (kapatılabilir), yok edilen kayalardan düşer: **S** = kalkan
  (6 sn dokunulmaz), **T** = üçlü atış, **R** = seri ateş.
- **Hiperuzay** (aşağı tuşu): 4 sn bekleme süresiyle rastgele bir konuma acil
  sıçrama - ve varışta parçalanma riski %12.
- 3 can, dokunulmazlık yanıp sönmesiyle güvenli yeniden doğuş, **her 5000 puanda
  ekstra can**; patlama parçacıkları ve kamera sarsıntısı.
- **Ortaklaşa düello** (çok oyunculu): iki gemi aynı anda ayrı can ve puanlarla
  uçar - çok puan yapan kazanır.
- Setup: zorluk, UFO'lar açık/kapalı, güçlendirmeler açık/kapalı
  (`settings.json` içinde kaydedilir).

**Pac-Man**
- Yem topakları, 4 güç hapı, yandaki tünel geçitleri ve ortada bir hayalet evi
  olan neon görünümlü **klasik 28x31 labirent**.
- **Orijinal davranışlara sahip dört hayalet** (hedef kare yapay zekâsı):
  *Blinky* doğrudan kovalar, *Pinky* pusu kurar (4 kare önde), *Inky* Blinky
  üzerinden bir vektör kullanır, *Clyde* yaklaşınca geri çekilir.
- **Dağılma/kovalama aşamaları** dönüşümlü ilerler (hayaletler her geçişte yön
  değiştirir); bir **güç hapı** hayaletleri maviye çevirir ve yenilebilir kılar
  (zincir 200/400/800/1600), ardından gözleri eve döner.
- **Kademeli salıverme** içeren hayalet evi, **meyve** bonusları (bölüm başına),
  **3 can**, **10.000'de ekstra can**, seviye sistemi (hızlanır), ölüm
  animasyonu, READY/GAME OVER ekranları.
- Setup: **zorluk** (Normal/Zor/Ekstrem) – hayalet hızı ve korku süresi.
- Kontroller: **oklar ya da WASD**.  Enter = yeni, S = setup.

**Flappy Bird**
- **Yer çekimi fiziği**: Boşluk / Yukarı / W / **fare tıklaması** kuşu kanat
  çırptırır; kuş, yükseliş/düşüş hızına göre eğilir.
- Aralıklı sonsuz **boru çiftleri** (boru başına +1); aralıklarda **madeni
  paralar** (bonus) ve bir **kalkan** güçlendirmesi (bir çarpmayı atlatır)
  belirir.
- **Gündüz/gece temaları** skorla değişir; sürüklenen bulutlar (parallax), kayan
  zemin.
- Zorluk (Kolay/Normal/Zor): aralık boyutu, hız, boru aralığı – skor yükseldikçe
  aralık biraz daralır.
- Oyun bittiğinde **madalyalar** (bronz/gümüş/altın/platin), kamera sarsıntılı
  çarpma animasyonu, yüksek skor.

**Doodle Jump**
- Doodler inişte **otomatik zıplar**; yalnızca sol/sağ yönlendirirsiniz
  (ataletle), kenarlar birbirine sarar ve yükseldikçe kamera yukarı kayar.
- **Platform türleri**: yeşil (normal), mavi (hareketli), kahverengi (kırılır),
  beyaz (kaybolur). **Yaylar** süper bir sıçrama verir, **pervaneli şapka** sizi
  kısa süre yukarı taşır (ve dokunulmaz kılar).
- **Canavarlar**: temas ölümcüldür – ama onları Yukarı / Boşluk ile
  **vurabilirsiniz** (bonus puan).
- Puan = ulaşılan yükseklik; zorluk yükseklikle artar. Yüksek skor.
- Kontroller: sol/sağ = hareket, Yukarı / Boşluk = ateş.

**2048**
- **3x3'ten 8x8'e** tahta boyutları ve üç modla **kendi ayar ekranı**: *Klasik*
  (hedef 2048, ardından "Oynamaya devam?"), *Zamana karşı* (3 dakika, saat ilk
  hamleyle başlar) ve *Sonsuz*.
- **Akıcı animasyonlar**: taşlar kayar, "pop" efektiyle birleşir ve büyüyerek
  belirir; puan göstergeleri, 128'den itibaren kıvılcımlar, 2048'den itibaren şok
  dalgası ve 131072'ye kadar yeni renkler. Animasyon sırasında girilen hamleler
  sıraya alınır.
- **Geri alma** (kapalı / oyun başına 3 / sınırsız, **U** tuşu ya da Backspace) -
  kullanan, rekorsuz ve taş başarımları olmadan oynar.
- **Kaydet ve devam et**: süren oyun, boyut ve moda göre otomatik kaydedilir; en
  iyi skorlar ve boyut/mod başına en büyük taş `mem.json` dosyasının `g2048`
  bölümündedir.
- Kontroller: oklar/WASD ya da fare/touchpad ile **kaydırma**, **R**/**N** = yeni
  oyun, **Tab** = ayarlar. Rekor yalnızca geri almasız **4x4 Klasik**'te
  sayılır.

**Minesweeper**
- Üç seviye: **Başlangıç** (9x9, 10 mayın), **Orta** (16x16, 40), **Uzman**
  (30x16, 99) - **seviye başına en iyi süre** kaydedilir ve setup'ta gösterilir.
- **İlk tıklama her zaman güvenlidir** (mayınlar sonrasında yerleştirilir,
  tıklamanın etrafındaki 3x3 alan boş kalır).
- **Sol tıklama** = aç, **sağ tıklama** = bayrak (isteğe bağlı soru işareti
  döngüsüyle), **F** = imlecin altındaki kareye bayrak, **R** = yeni oyun.
- **Chording**: tamamlanmış bir sayıya tıklamak kalan komşuları açar.
- Klasik HUD: mayın sayacı, **tıklanabilir gülen yüz** (şaşkın/güneş
  gözlüklü/ölü), kronometre; yanlış bayraklar sonda üzeri çizilir, zaferde
  konfeti.
- Puan = seviyenin taban değeri eksi saniyeler.

**Sudoku**
- Her biri **400 bölümlü 4 tür** (4 zorluk x 100): *Klasik* (bilinen tohumlu
  bölümler - çözülenler işaretli kalır), *X-Sudoku* (iki köşegen de her rakamı
  tam bir kez içerir), *Killer* (toplamlı kesik çizgili kafesler; önceden
  üretilmiş 400 bölüm) ve *Mini 6x6*. Her bulmacanın **tek çözümü** vardır -
  "Zor"un 12. bölümü her bilgisayarda aynı bulmacadır.
- **Günün Sudokusu**: herkes için günde bir bulmaca, bilgisayarda ve tarayıcıda
  aynı; zorluk haftanın gününe bağlıdır (pazartesi Kolay'dan cumartesi Uzman'a),
  her gün çözen bir seri oluşturur.
- **Bölüm başına 3 yıldıza kadar** (çözüldü · hatasız ve ipucusuz · ayrıca hedef
  sürenin altında) ve bölüm seçiminde **en iyi süre**; **yarım kalan bulmacalar**
  otomatik kaydedilir ve bir sonraki açılışta devam eder.
- Skor çarpanlı **4 oyun modu** (başlamadan önce seçilir): **Klasik** (x2.0 -
  yardımsız), **Notlar** (x1.5 - + kalem notları ve otomatik adaylar), **Konfor**
  (x1.0 - + yanlış rakamlar kırmızı, çakışmalar ve hatalı kafes toplamları
  işaretli, doğru girişler kilitlenir), **Asistan** (x0.7 - + ipucu tuşu, en
  fazla 3). **3 hata sınırı** etkinken (setup seçeneği) üçüncü hata oyunu
  bitirir.
- Kontroller: oklar/WASD = hücre, **1-9** = rakam (numpad da), **0/Delete/sağ
  tıklama** = sil, **U**/**Z** = geri al, **Y** = yinele, **N** = notlar, **C** =
  otomatik adaylar, **H** = ipucu, **M** = renk işaretçisi, **R** = bölümü yeniden
  başlat, **Q** = bölüm seçimi. "Önce rakam" girişi (setup, **I**) ve her rakamın
  altında bir **kalan rakam sayacı**; tamamen fareyle oynanabilir. Oyun bittikten
  sonra **A** tam **çözümü** gösterir.
- Puan = (tür ve zorluk tabanı - süre - hatalar - ipuçları) x mod çarpanı; tüm
  türler ve Günün Sudokusu rekora sayılır.

**Frogger**
- 5 trafik şeridi (arabalar/kamyonlar) ve 5 nehir şeridi (kütükler, üst
  bölümlerde **dalan** kaplumbağalar); üstte 5 yuva - hepsini doldur = sonraki
  bölüm, her şey hızlanır.
- Ekstralar: boş yuvalarda **bonus sinek** (+200), üst bölümlerde **timsahlar**
  yuvaları işgal eder, kurbağa başına **süre sınırı çubuğu**, 10.000'de ekstra
  can.
- 3 zorluk (hız, trafik yoğunluğu, süre); her yeni sıra için puan, yuva = 50 +
  süre bonusu, bölüm tamamlama = +1000.

**Memory**
- Tahta boyutları **4x4, 6x6, 8x6**; desenler tamamen ilkel şekillerle çizilen
  şekil-renk kombinasyonlarıdır; **çevirme animasyonu**, eşleşmeyen çiftler
  otomatik geri kapanır.
- **Tekli**: taban - hamle başına 15 - saniye başına 2 (en az 100). **Düello**
  (yerel): sırayla, eşleşme = tekrar oynarsın, en çok çifti bulan kazanır.

**Solitaire**
- Oyun öncesi ekranda **5 çeşit**: Klondike (1/3 çekme seçeneği), Spider (1/2/4
  takım), FreeCell (süperhamle sınırı), Piramit (13'lük çiftler, 2 yeniden
  dağıtım) ve TriPeaks (kombo çarpanlı ±1 zinciri).
- **Sürükle-bırak** ya da tıkla-tıkla, **sağ tıklama** = temele, **U** = sınırsız
  geri alma, **R** = yeni dağıtım, Boşluk = deste.
- Kartlar görüntü dosyası olmadan işlenir (`games/cards.py`); tüm çeşitler çeşide
  özel formüllerle tek bir yüksek skor listesini paylaşır.

**Aim Trainer**
- **Gerçek yazılım 3D'si** (Snake'in 3D modu gibi): ekranın merkezinde sabit bir
  nişangâh, **bir nişancı oyunu gibi doğrudan 1:1 fare bakışı** (imleç yakalama:
  imleç pencerenin içinde tutulur, Esc onu serbest bırakır; ayarlanabilir
  hassasiyet, sınırsız yaw, ±60° pitch). Sol tıklama tam merkezden ateş eder;
  namlu alevi, iz mermisi ve isabet parçacıklarıyla.
- **4 mod**: isabet (60 sn, 3 küre, doğruluk bonusu), refleks (30 tekil hedef,
  tepki süresi istatistikleri), hareketli hedefler (rotalar + x4'e kadar kombo
  çarpanı) ve sakin (sonsuz, cezasız, **E** oturumu bitirir).
- **3 tema** (setup'ta, kaydedilir): yıldız küresi, **parlayan halkalı bir kara
  delik** ve bir gezegen içeren **uzay** (öntanımlı), zemin ızgarası ve synthwave
  güneşi olan neon arena ve kapalı bir atış poligonu.
- Hassasiyet oyunun ortasında da **+/-** ile değiştirilebilir; ekstra sakin bir
  görsel için ayrıca **ayarlanabilir hareket bulanıklığı** (0-80%) - ikisi de
  kaydedilir.

**Dört Taş**
- **Düşen pul animasyonu**, üzerine gelince önizleme ve nabız gibi atan kazanan
  çizgi içeren 7x6 tahta; fare, ok tuşları ya da doğrudan seçim **1-7**.
- **3 yapay zekâ seviyesi** (alfa-beta aramalı minimax): Kolay tehditleri kasten
  kaçırır, Orta güvenilir şekilde bloklar, Zor derinlemesine plan yapar - ya da
  aynı cihazda yerel **2 oyuncu**.
- Başlayan oyuncu her turda değişir; yüksek skor bir oturumdaki **yapay zekâya
  karşı galibiyetlerinizi** sayar.

**Tank Düellosu**
- 2D arena düellosu: **atışlar duvarlardan bir kez seker** (rikoşet) - köşelerden
  vurur (ya da kendinizi!). Geri sayımla ilk 5 tur.
- **4 arena** (Açık, Haç, Sütunlar, Labirent) ya da rastgele döngü;
  **güçlendirmeler**: seri ateş, kalkan, üçlü atış.
- **3 seviyeli yapay zekâ** - zor olanı atışlarında hedefin önünü tutar ve
  mermileri bilerek duvarlardan sektirir - ya da tek klavyede **2 oyuncu** (O1
  WASD+Boşluk, O2 oklar+Enter).

**Blackjack**
- Gerçek kumarhane kuralları: **4 desteli shoe**, kurpiyer 17'de durur,
  **blackjack 3:2 öder**, as/10'da kurpiyer peek'i; **ikiye katlama** ve **bir
  kez bölme** (bölünen aslar her biri bir kart alır).
- **Lama çipleri**: Blackjack, Poker ve Casino ile paylaşılan **Lama Bankası**
  hesabıyla oynanır (başlangıç 1000, kalıcı olarak `mem.json` içinde). Bahis kart
  dağıtılırken hemen düşülür; 10 çipin altında Enter, hesabı 1000'e tamamlayan bir
  **banka kredisi** alır.
- **Rekor** = kendi **Blackjack bakiyenin** en yüksek seviyesi (1000 artı
  Blackjack'te kazanılan ve kaybedilen her şey) - rulette, slotta ya da pokerde
  kazanılanlar burada sayılmaz, krediler de sayılmaz.
- Çip düğmeleri ve tuşlarla oynanır (**H**it/**S**tand/**D**ouble/bölme **X**,
  **1-4** = bahis, Backspace = bahsi temizle, Enter = dağıt); kart
  animasyonlarıyla - kurpiyerin kapalı kartı açılırken artık gerçekten döner.

**Tunnel Racer**
- **3D neon tüp uçuşu** (Aim Trainer gibi yazılım işleyici): çubuklar, bloklar ve
  **içinden geçilecek halka kapılar**, ideal çizgi üzerinde madeni paralar.
- **İki mod**: sonsuz (hız bir üst sınıra kadar yükselir, yüksek skor) ve bitiş
  çizgisi, süre bonusu ve işaretlenen ilerleme içeren **30 tohumlu bölüm**.
- **Tuşla kontrol** (öntanımlı) ya da **doğrudan fareyle kontrol** (imleç
  yakalama, **C** tuşu); ayrıca ayarlanabilir **hareket bulanıklığı** (**B**
  tuşu, 0-80%) - her şey kaydedilir.

**3D Labirent**
- Mouselook + WASD ile **Wolfenstein tarzı birinci şahıs raycaster** (DDA, mesafe
  sisi, sprite'lar), bir **mini harita** (**M** tuşu) ve yeşil, nabız gibi atan
  bir çıkış - ya da klasik bir **2D kuşbakışı görünüm** (setup'ta **V** tuşu).
- Sürekli büyüyen **50 tohumlu bölüm**; çıkış her zaman başlangıca en uzak
  noktada bulunur, yol boyunca **orblar** bonus puan verir.
- Puanlama: bölüm başına 500 + orb başına 100 + süre bonusu; çözülen bölümler
  işaretlenir ve oturum toplamı yüksek skor olur.

**Reversi**
- **8x8 Othello**: rakibin sıralarını kıstıran taşlar koyun ve kapanan her şeyi
  çevirin; geçersiz hamleler engellenir ve geçerli hamlesi olmayan bir el
  **otomatik olarak pas geçilir**.
- **Yapay zekâya karşı tek oyuncu** (3 seviye: alfa-beta ile negamax, konumsal
  ağırlıklandırma + hareketlilik) **ya da yerel düello**, Siyah'a karşı Beyaz.
- Geçerli kareler vurgulanır; **fare** ya da seçim çerçevesiyle (oklar +
  Boşluk/Enter) oynayın. Yapay zekâya karşı her galibiyet yüksek skora bir puan
  sayar.

**Kniffel (Yahtzee)**
- **Klasik zar oyunu**: 5 zar, tur başına en fazla 3 atış, zarları tek tek
  **tut**, ardından **13 kategoriden** birini işle (olası puanın canlı
  önizlemesiyle).
- Tam puan çizelgesi: **63 puan bonuslu (+35)** üst bölüm, üç benzer/dört benzer,
  full house, küçük/büyük kent, **Yahtzee (50)** ve Şans.
- **En yüksek toplamı hedefleyen rekor avı olarak tek oyunculu** ya da yan yana
  iki çizelgeyle **2 oyunculu hotseat**; fareyle ya da tuşlarla oynayın (Boşluk,
  1-5, oklar, Enter).

**Wordle**
- Gizli kelimeyi tahmin edin; doğru **tekrar eden harf sayımıyla** renkli geri
  bildirim (yeşil/sarı/gri) ve renklenen bir ekran klavyesi (Almanca, Çekçe,
  Slovence ve Hırvatça için QWERTZ, Fransızca için AZERTY, diğerlerinde QWERTY).
- **Dört mod**: *Sonsuz* (her biri 6 denemeli art arda kelimeler; çözülen her
  kelime puan kazandırır, çözülemeyen ilk kelime oyunu bitirir), *Günün kelimesi*
  (her dil ve uzunluk için günde bir kelime - bilgisayarda ve tarayıcıda aynı -
  geri sayım ve seriyle; başlanmış günlük kelime kaydedilir), *Dordle* (7
  denemede aynı anda 2 kelime) ve *Quordle* (9 denemede 4 kelime, tuşlar tüm
  tahtaların renklerini gösterir).
- Her oyundan önce **ayarlar**: **4 ile 7 arası kelime uzunluğu**, **zor mod**
  (bulunan ipuçları yeniden kullanılmalı) ve **renk körü paleti** (turuncu/mavi);
  yanında istatistikler görünür.
- **14 dilde gerçek kelime listeleri** (`woordlistz/` klasörü, yalnızca A-Z), her
  uzunluk için ayrı: yalnızca 5 harfte yaklaşık **34.000 çözüm** ve **213.000'i
  aşkın geçerli kelime**, dört uzunluğun toplamında yaklaşık 134.000 çözüm.
  Çözümler; özel isim, İngilizce kalıntı ve saldırgan sözcük içermeyen yaygın
  kelimelerdir; her tahmin listeyle karşılaştırılır - listede yoksa reddedilir ve
  satır kısaca titrer.
- Dil, uzunluk ve mod başına **istatistik**: oyunlar, kazanma oranı, güncel ve en
  iyi seri ile **çubuk grafik olarak deneme dağılımı** (`mem.json` içindeki
  `wordle` bölümü). **Paylaş** (**C**), çözümü ele vermeden bir emoji ızgarasını
  panoya kopyalar.
- Rekora yalnızca 5 harfli *Sonsuz* sayılır; diğer uzunlukların kendi en iyileri
  vardır. **Kâhin** (en fazla 2 deneme), **Kelime alışkanlığı** (art arda 7 günün
  kelimesi) ve **Dörtlü dahi** (Quordle çözüldü) başarımları.

**Poker**
- Oyun öncesi ekranında **3 çeşit**: krupiye düğmesi, blind'lar ve dört bahis
  turuyla 1-3 yapay zekâ rakibe karşı **Texas Hold'em**, **5 Card Draw** (yapay
  zekâyla bire bir, bir kez kart değiştirme) ve **Video Poker** (*Jacks or
  Better*, ödeme tablosuna karşı tek başına).
- Düğmeler ya da tuşlarla eylemler: **F** = pas (fold), **C** = kontrol/gör,
  **R** = artır, **A** = all-in; kartları tıklayarak ya da **1-5** ile tut/değiştir,
  **Enter** kart çeker ya da sonraki eli dağıtır.
- Ortak **Lama Bankası**'nın **Lama çipleri**: bir elin başında hesabın masada
  yığın olarak durur ve pota giden her şey hemen düşülür - eli yarıda bırakıp
  masadan kalkmak yalnızca pottaki payına mal olur. İflas (20'lik büyük blind'ın,
  Video Poker'de 10'un altı) = hesabı 1000'e tamamlayan banka kredisi.
- **Rekor** = **Poker bakiyenin** en yüksek seviyesi (1000 artı pokerdeki tüm
  kazanç ve kayıplar); **Chip lideri** başarımı da yalnızca bu bakiyeyi sayar.

**Satranç**
- **Eksiksiz satranç**: **rok**, **geçerken alma** ve **piyon terfisi** (taş
  seçilebilir) dahil tüm taş hamleleri; **şah, şah mat ve pat** ile **elli hamle
  kuralı**, **üçlü tekrar**, **yetersiz materyal** ya da anlaşma yoluyla
  beraberlikler.
- **Üç mod**: yapay zekâya karşı *oyun*, aynı bilgisayarda *2 oyuncu* (tahta
  istenirse her hamleden sonra döner) ve **bulmacalar**.
- *Acemi*'den *Usta*'ya 6 seviyede **daha güçlü, takılmayan yapay zekâ**:
  yinelemeli derinleştirme, transpozisyon tablosu, sükûnet araması, açılış
  kitabı ve hareketlilik, piyon yapısı ile şah güvenliğini hesaba katan bir
  değerlendirme. Yapay zekâ kare başına küçük parçalar hâlinde hesaplar - oyun
  asla takılmaz.
- **Ayarlar**: renk seçimi, **satranç saati** (yok, 1+0, 3+2, 5+0, 10+5) ve
  **Chess960** (960 başlangıç dizilişinin tamamı, numarası hamle listesinin
  üstünde yazar).
- Saatler, alınan taşlar, materyal dengesi ve kaydırılabilir **hamle listesi
  (SAN)** içeren **kenar paneli**; sürükle-bırak, kayan taşlar, koordinatlar.
  Tuşlar: **U** = geri al, **H** = ipucu oku, **O** = beraberlik teklif et, **X** =
  terk et, **F** = tahtayı çevir, oyundan sonra **P** = **PGN dışa aktarma**.
- **Bulmacalar**: 5 aşamada 200 bulmaca (1/2/3 hamlede mat, taktik I/II),
  **Lichess'in serbest bulmaca veritabanından (CC0)** alınmış ve oyunun kendi
  motoruyla doğrulanmış; mat bulmacalarında mat eden her hamle geçerlidir.
  İlerleme `mem.json` dosyasının `chess` bölümündedir.
- Geri alma ve ipucu oyunu "yardımlı" yapar: rekor yalnızca yapay zekâya karşı
  yardımsız galibiyetleri sayar (oturum başına).

**Dokuz Taş**
- Üç aşamalı **değirmenler**: **dizme** (her biri 9 taş), çizgiler boyunca
  **hareket** ve yalnızca 3 taş kalınca **uçma** (kapatılabilir).
- Tamamlanan bir **değirmen** rakibin bir taşını kaldırır (tercihen bir
  değirmenin dışındakini); 3 taşın altına düşünce ya da hareket edemez hâle
  gelince kaybedersiniz.
- **3 yapay zekâ seviyesi** (alfa-beta ile minimax, aşamaya duyarlı
  değerlendirme) ya da **yerel düello**; hamle ipuçları, değirmen vurgusu ve taş
  sayacıyla.

**Simon**
- **Senso hafıza oyunu**: yanan dizi her turda büyür ve tam olarak
  tekrarlanmalıdır.
- **Modlar**: *Klasik*, *Hız* (hızlanır), *Ters* (geriye doğru), *Karışık* (mod
  her turda değişir) ve iki oyunculu **Düello** (sırayla ekleyip tekrarlayın).
- **Ses** *kapalı / açık / karışık* (karışık, görsel VE işitsel hafızanızı
  çalıştırır), zorluk olarak **4/6/9 tuş**; **mod başına en iyi skor** kaydedilir.
  Fareyle ya da 1-9 sayı tuşlarıyla oynayın.

**Bilardo**
- **8-top**, **9-top** ve kuralsız bir **antrenman** modu, yapay zekâya karşı
  (nişan yardımıyla) ya da **yerel iki oyuncu**.
- **Serbestçe seçilebilen üç görünüm**: klasik **2D kuşbakışı**, gölgeli toplarla
  sabit bir **3D açılı perspektif** ve **serbest dönen 3D kamera** (sağ fare
  düğmesi). Tüm hareket zaman adımı temellidir ve **yumuşakça sönümlenir**
  (sürtünme, tünellemeyi önlemek için alt adımlar).
- **Vuruş**: gücü doldurmak için sol fare düğmesini basılı tutun, vurmak için
  bırakın; bir nişan çizgisi ve güç göstergesi yardımcı olur. Fauldan sonra
  **elde top**. Görünüm (V) `settings.json` içinde hatırlanır; kazanılan frameler
  yüksek skora sayılır.

**Kaydırmalı Bulmaca**
- Üç boyutta 15 bulmacası: **3×3** (kolay), **4×4** (klasik) ve **5×5** (zor);
  numaralı karoları boş boşluğa kaydırın.
- Her zaman çözülebilir (çok sayıda rastgele hamleyle karıştırılır). Boşluğun
  satırındaki/sütunundaki bir karoya **tıklayarak** (tüm sıra kayar) ya da **ok
  tuşlarıyla** kontrol edin.
- Puan = boyut başına bir taban değer eksi hamleler ve süre; çözünce hemen yeni
  bir tahta başlar.

**Mastermind**
- Gizli **renk kodunu** çözün; her tahminden sonra **siyah** pimler (doğru renk +
  konum) ve **beyaz** pimler (doğru renk, yanlış konum) alırsınız.
- **3 mod**: Kolay (4 pim / 6 renk / 12 sıra), Klasik (4/6/10) ve Zor (5 pim / 8
  renk); aynı rengin tekrarı serbesttir.
- Renk paletiyle oynanır (tıklama ya da **1–8** tuşları), OK/Enter sırayı
  değerlendirir. Wordle gibi **sonsuz seri**: çözülen her kod puan kazandırır.

**Bubble Shooter**
- Bal peteği ızgarasında **Puzzle Bobble**: fareyle nişan alın, baloncukları
  yukarı ateşleyin, **aynı renkten üç ya da daha fazlası** grubu patlatır.
- Tavana bağlantısını kaybeden baloncuklar **düşer** (bonus); atışlar
  **duvarlardan seker**, sonraki baloncuğun önizlemesiyle.
- **3 mod** (4/5/6 renk, bazıları alçalan sıralarla); kırmızı çizgide oyun biter.

**Adam Asmaca**
- Kelimeyi **harf harf** tahmin edin; her hata adam asmacanın bir parçasını
  çizer, **6 hatadan** sonra kaybedersiniz.
- **Dile göre kelime listeleri** (yalnızca A–Z), **3 uzunluk modu** (kısa /
  karışık / uzun); ekran klavyesiyle yazın ya da tıklayın.
- **Sonsuz seri**: tahmin edilen her kelime puan kazandırır (daha çok kalan can +
  daha uzun kelime = daha çok).

**Block Jump**
- **Minecraft tarzı 3D platform oyunu** (Snake'in 3D modu gibi yazılım 3D'si):
  parlayan hedefe ulaşmak için havada süzülen bir **voksel dünyasındaki** bloklar
  arasında zıplayın.
- **Minecraft görünümü**: tüm bloklarda gerçek **piksel dokular** var (çim,
  toprak, taş, tahta, elmas, balçık, odun); ayrıntı düzeyi mesafeye göre
  değişir (**T** = yüksek/düşük/kapalı).
- Ayrıca: yürüyüş animasyonlu **Steve** figürü (üçüncü şahıs kamera), birinci
  şahısta **el**, hedefte **işaret ışını**, para yerine dönen **altın
  külçeleri**, kare **güneş**, **piksel bulutlar** ve **kalpli** bir HUD.
- Blok türleri: katı bloklar (çim/toprak/taş/tahta), **merdivenler** (tırman),
  **çitler** (üzerinden atla), **yay blokları** (fırlatır) ve **madeni paralar**.
- Kamera **öntanımlı olarak Minecraft gibi birinci şahıs**, **V** takip
  kamerasına geçer; imleç yakalamalı **fare bakışı**, ayarlanabilir **hareket
  bulanıklığı** (**B**) ve hassasiyet (**+/-**).
- **Tohumla üretilen parkur bölümleri** zorlaşır; hedef = puan + süre bonusu,
  madeni paralar +50, düşmek bir cana mal olur (3 ile başlar). Kontroller:
  WASD/oklar, zıplamak için **Boşluk**.

**Tower Defense**
- **4 haritada** (Çayır, Kanyon, Kavşak, Dar Geçit) **sonsuz dalga savunması**;
  her haritanın kendi yolu var, kilitli haritalar en iyi dalganla açılır, her
  **8 dalgada** bir **boss** gelir.
- **3 mod**: Klasik (7 kule, ana mod), Kompakt (4 kule, 2 seviye) ve Maksimum
  (**11 kule**, en yüksek seviyede **A/B uzmanlaşması**, özel düşmanlar, aktif
  yetenekler **Meteor/Buz novası/Altına hücum**).
- Oktan lazere ve altın bankasına **11 kule tipi**, her biri en fazla
  **3 geliştirme seviyesi**, satış %70 iade eder; zırhlı, yenilenen, bölünen,
  gizlenen, şifa auralı ve hava rotalı düşmanlar.
- **Ekonomi**: yok etme başına altın, dalga bonusu + %5 faiz; yok etme ve dalga
  başına puan. **F** = 2x hız, **G** = menziller, sağ tık iptal eder.

**Minigolf**
- **40 parkurda 360 delik**: *Classic* ve *Pro* dokuzar elle tasarlanmış delikle,
  **Tour** giderek zorlaşan dokuzar üretilmiş delikten oluşan 38 parkurla
  (toplam 342) ve tüm havuzdan çeken *Random*. Parkur 7, delik 3 her yerde aynı
  görünür - bunun için hiçbir şey kaydedilmez.
- **Zeminler ve engeller**: kum yavaşlatır, rampalar hızlandırır, su bir ceza
  vuruşuna mal olur, lastik tamponlar hız geri verir, yel değirmenleri ve gezen
  bloklar zamanlama ister. Fizik, bilardodaki gibi sürtünmeli alt adımlarla çalışır
  - hiçbir şey zıplamaz ya da bandın içinden geçmez.
- **Kontroller**: fare nişan alır, sol tuşu basılı tutmak gücü doldurur,
  bırakmak vurur (oklar + boşluk da olur). **R** yüklenmiş vuruşu vurmadan
  iptal eder. **G** nişan çizgisini, **Z** otomatik nişanı, **P** topu almayı
  açıp kapatır.
- **Güç kilidi (sağ tuşu basılı tut)**: doldurma çubuğunu olduğu yerde
  dondurur - altın rengi, yüzde, kilit simgesi ve topun çevresinde nabız gibi
  atan bir halka. Vuruş hazırken değirmendeki boşluğu böyle beklersin.
  Bıraktığında yeniden dolar; kilitli güç vuruştan sonra da kalır ve bir sonraki
  sol tık tam o değerle vurur.
- **Skor kartı** sağda, par ve delik başına vuruşlarla; iki kişide herkes aynı
  deliği sırayla oynar. Puanlar: delik başına 600, parın altındaki/üstündeki her
  vuruş için ±300, **hole in one için 500 ek puan**. Her parkurun en düşük vuruş
  sayısı `mem.json` içindeki `minigolf` bölümündedir.
- **Topu alma kapatılabilir**: öntanımlı olarak bir delik sekiz vuruştan sonra
  biter ve en düşük değerle sayılır. Deliğe girene kadar oynamayı yeğleyenler
  ayar ekranında *Topu alma* seçeneğini KAPALI yapar (ya da **P** tuşuna basar).
- **Otomatik nişan kapatılabilir**: öntanımlı olarak sopa her vuruştan önce
  kendiliğinden deliğe döner. Her deliği kendisi nişanlamayı yeğleyenler ayar
  ekranında *Otomatik nişan* seçeneğini KAPALI yapar (ya da **Z** tuşuna basar)
  - o zaman son seçilen yön korunur ve yeni bir deliğin başında sopa nötr
  olarak yukarıyı gösterir.
- **F** o anki deliği sıfırlar: vuruşlar 0, top başlangıçta - aynı delik, aynı
  saha.
- **Tekrar yerine devam**: tur bitince **Devam** düğmesi bir sonraki sahaya götürür
  (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), böylece aynı dokuz delik
  tekrarlanmaz; yanında **Tekrar** (aynı saha) ve **Ayarlar**. Tuşlar: Enter =
  devam, R = tekrar, S = ayarlar.
- **Turun tekrarı**: turun sonunda **P** (ya da **Tekrar** düğmesi) tüm turu
  vuruş vuruş yeniden gösterir. **S** onu arşive koyar (kenar çubuğundaki
  **Tekrarlar** düğmesi).
- **Kendi deliklerini kur ve paylaş**: hazırlık ekranındaki **MAPS** sekmesi
  kendi koleksiyonuna götürür - **Yeni** düzenleyiciyi açar. Her delik bir ad
  ve bir **id** alır (küçük harf, boşluksuz); id aynı zamanda paylaşırken
  önerilen dosya adıdır. Yedi klasik engelin yanına **sekiz yenisi** gelir:
  boru (topu diğer uca taşır), buz, yapışkan alan, hızlandırıcı, mıknatıs, tek
  yön kapısı, döner tabla ve atlama rampası. Delik boyutu serbestçe ayarlanır
  (60x80 ile 160x240 arası), **12 şablon** bir başlangıç noktası verir, geri
  al/yinele ve **Test** de vardır. **Paylaş** tam olarak bir deliği
  `.lamapgzmap` dosyası olarak yazar - kaydetme penceresiyle ya da doğrudan
  İndirilenler klasörüne, yapımcı adınla birlikte. **İçe aktar** onu geri okur
  ve id doluysa kendiliğinden `-2`ye geçer. Ad, id ve yapımcı adı her zaman
  **14 dilin tamamını kapsayan bir kelime filtresinden** geçer. Bir delik
  **Oyna** ile tek başına, tüm koleksiyon ise beşinci parkur seçeneği
  **Kendi** ile oynanır.

**Pinball**
- **Üç masa**: *Classic* (üç bumper, bir hedef dizisi), *Space* (romb dizilmiş
  dört bumper, iki dizi) ve *Lama* (açık alan, yay biçiminde altı hedef); oyun
  başına 3 ya da 5 top, iki kişide top top sırayla.
- **Bir flipperin ihtiyacı olan her şey**: doldurma göstergeli fırlatma kanalı
  (çok zayıf mı? top geri döner ve tekrar denersin), iki flipper, slingshotlar,
  hedef dizileri, dört **L-A-M-A** şeridi, top kilitleyen tutucu, **jackpotlu
  multiball**, altı saniyelik **top koruma**, sarsma ve **TILT**.
- **x5'e kadar çarpan**: düşürülen diziler ve tamamlanan şeritlerle yükselir;
  bumperlar 100, slingshotlar 50, hedefler 250 - multiball sırasında bumperlar
  2.500 jackpot öder.
- Flipperler atanmış sol/sağ tuşlarla (ayrıca sol/sağ [Shift]) ya da fareyle
  çalışır. Her masanın rekoru `mem.json` içindeki `pinball` bölümündedir.

**Bowling**
- **Resmî kurallara göre on frame**, onuncu frame'in strike, spare ve bonus
  atışları dahil (en fazla: 300). Başlığın altındaki **skor kartı** her frame'i X,
  / ve güncel toplamla gösterir.
- **Dört adımda atış**: konum, açı, efekt ve güç. Her ayar kendiliğinden salınır
  ve eylem tuşuyla sabitlenir - ya da sol/sağ ile elle ayarlanır, o zaman salınım
  durur.
- **Gerçek labut fiziği**: on labut, birbirini deviren gerçek kütleli dairelerdir;
  strike şansın değil fiziğin sonucudur. Pist önde yağlıdır, bu yüzden **hook**
  ancak son üçte birde tutar.
- Oluklar, nişan okları ve labut alanıyla perspektifli pist görünümü; üç zorluk
  (*Kolay/Normal/Pro*) ayarların salınım hızını ve saçılmayı değiştirir. Her
  zorluğun rekoru `mem.json` içindeki `bowling` bölümündedir.
- **Partinin tekrarı**: sonunda **P** tüm atışları yeniden gösterir, **S** onları
  arşive kaydeder (**Tekrarlar** düğmesi).

**Crossy Road**
- Çayırlar (ağaçlar ve kayalar yolu keser), araba ve kamyonlu yollar, kütük ve
  nilüferli nehirler ve uyarı ışığı ile zilden sonra bir trenin geldiği **raylar**
  üzerinden **sonsuz zıplama** - ilerleyince 5 raya kadar koca istasyonlar
  bekler. Parkur satır satır oluşur, her zaman geçilebilir bir yol vardır; hız ve
  trafik giderek artar.
- **İzometrik voksel görünüm**: gölgeli bloklardan karakterler, araçlar ve
  ağaçlar (her kare boyutu için önceden çizilir), yumuşak kamera, zıplarken
  squash & stretch, su sıçraması, ezilme animasyonu, tüyler ve parıldayan
  jetonlar; 50. satırdan itibaren farlarla **gece/gündüz döngüsü**.
- **Kartal**: kamera yavaşça ilerler - çok oyalanan ya da üç satırdan fazla geri
  giden kartala yakalanır (önce kırmızı bir kenar uyarır). Bir kütükle ekranın
  dışına sürüklenmek de oyunu bitirir.
- **Jetonlar ve karakterler**: toplanan jetonlar (dev jeton = 5) kaydedilir ve
  **Karakterler** sekmesinde yeni karakterler satın alır: kurbağa, domuz,
  penguen, kedi, tilki, lama, robot, hayalet ve tek boynuzlu at (25-250 jeton);
  tavuk en baştan beri senindir.
- **Modlar**: *Sonsuz* (puan = en uzak satır, rekora sayılır) ve *Günün parkuru*
  (bugün herkes için aynı, tarayıcıda da, kendi günlük rekoruyla). Kontroller:
  oklar/WASD, Boşluk/Enter/tıklama = ileri zıpla; ayarlarda **H** = gölgeler,
  **N** = gece/gündüz. Jetonlar, karakterler ve günlük rekor `mem.json`
  dosyasının `crossy` bölümündedir.

**Geometry Dash**
- **Ritim platform oyunu**: karakterin kendiliğinden sağa doğru hızla ilerler -
  sen yalnızca ne zaman zıplayacağına ya da uçacağına karar verirsin. **Beş
  biçim** - küp, gemi, top, UFO ve dalga -, ayrıca biçim, yerçekimi ve hız
  portalları (0,5x ile 3x arası), sarı/pembe/mavi **pedler ve küreler**, yarım
  bloklar, dikenler, çukurlar ve renk tetikleyicileri.
- *Kolay*'dan *İblis*'e (“Lama Inferno”) her birinde **3 gizli altın** bulunan
  **8 hazır bölüm**. Her bölümün geçilebildiği kanıtlanmıştır: yapım sırasında bir
  çözücü onu gerçek oyun koduyla bitirdi - tüm altınlarla ve saniyenin 1/240'ı
  kadar kaydırıldığında bile.
- **Hassas fizik**: sabit 240 Hz adımlı sabit noktalı hesaplama; her basış tam
  olarak gerçekleştiği adımda etki eder - her kare hızında aynı, tarayıcıda bit bit
  aynı.
- Otomatik ve kendi kontrol noktalarıyla (**Z** koyar, **X** siler) **antrenman
  modu** (**P**), deneme sayacı, ilerleme çubuğu, patlamalar ve anında yeniden
  başlama (**R**). Her bölümün **kendi müziği** vardır - arka plan, zemin ve
  küreler ritimle atar (müzik **M** ile kapatılır).
- **Yıldızlar ve altınlar**: bir bölümü normal modda bitiren yıldızlarını alır, her
  altın bir yıldız daha değerindedir; rekor **toplam yıldız** sayısıdır (en fazla
  65). Bölüm başına en iyiler, altınlar, denemeler ve zıplamalar `mem.json`
  dosyasının `geodash` bölümünde durur.
- **BÖLÜMLER** sekmesinde **bölüm editörü**: ızgaralı tuval, 6 gruplu palet
  (bloklar, tehlikeler, pedler ve küreler, portallar, hız, ekstralar), döndürme,
  geri al/yinele, genel bakış şeridi, **baştan ya da buradan test** ve bölüm
  ayarları (başlangıç hızı ve biçimi, müzik tarzı, BPM, renkler). **“Doğrulandı”**
  işareti ancak kendi bölümünü geçtiğinde gelir. **Paylaş** bir `.lamapgzlevel`
  dosyası yazar, **İçe aktar** onu geri okur; bölümler kendi minigolf
  parkurlarının yanında `ugc.json` içinde saklanır.

**Battleship**
- Uçak gemisi (5 kare), zırhlı (4), kruvazör (3), denizaltı (3) ve muhrip (2) ile
  **10x10 deniz savaşı** - düşman filosunu ilk batıran kazanır.
- **Filoyu yerleştirme**: rıhtımdan sürükle-bırak; **R** ya da sağ tıklama
  döndürür, önizleme yeşil ya da kırmızı yanar, **X** her şeyi rastgele yerleştirir,
  **C** tahtayı boşaltır; son dizilimin bir sonraki turda yeniden önerilir.
- **Ayarlardaki kurallar** (kaydedilir): *gemiler birbirine değebilir*, *salvo*
  (tur başına yüzen gemi sayın kadar atış) ve *isabetten sonra tekrar ateş et*.
- **3 seviyeli yapay zekâ**: Kolay rastgele ateş eder, Orta isabetlerin peşine
  sistemli biçimde düşer, Zor dama tahtası paritesiyle bir **olasılık haritası**
  hesaplar (bütün bir filo için ortalama yaklaşık 70 / 60 / 45 atış). Ya da aynı
  bilgisayarda **2 oyuncu** - bir **devir ekranı** her turdan önce iki filoyu da
  gizler.
- **Görseller**: radar taraması, animasyonlu dalgalar, kavis çizen mermiler, su
  sıçramaları, dumanlı patlamalar ve yanan kareler, "BATTI!" gösterimi ve atış,
  isabet ve isabet oranını gösteren tur sonu özeti. Rekor, bir oturumdaki **yapay
  zekâya karşı galibiyetleri** sayar.

**Casino**
- **Rulet** (Avrupa, 37 cep): bir sayıya, kenara ya da köşeye tıklayarak tüm
  klasik bahisler - **plein** (35:1), cheval, transversale, carré, sixain, kolon,
  düzine, kırmızı/siyah, tek/çift ve manque/passe. 1/5/25/100/500'lük çipler, sağ
  tıklama çip kaldırır; **Çevir**, **Tekrarla** (**R**), **İkiye katla** (**D**)
  ve **Temizle**. Top önceden çekilen cebe spiral çizerek düşer, üstte son 12
  sayı görünür.
- **Lama Slotu**: 5 makara x 3 sıra, **10 kazanç hattı**, **lama = joker**,
  **altın paralar = scatter** ile iki kat kazançlı 10 bedava dönüş, hat başına
  1/2/5/10 bahis, **otomatik dönüş** (10/25), **turbo** ve ödeme tablosu.
  **Geri ödeme oranı %96,1'dir** - makara şeritlerinden tam olarak hesaplanmıştır.
- **Lama Bankası**: Casino, Blackjack ve Poker tek bir **Lama çipi** hesabını
  paylaşır (başlangıç 1000, `mem.json` içindeki `casino` bölümü); eski çip
  bakiyeleri otomatik aktarılır. Bahisler hemen düşülür, her oyun rekoru için
  kendi bakiyesini tutar, iflasta 1000'e tamamlayan bir **banka kredisi** gelir.
- Konfeti, para yağmuru, büyük/mega/jackpot afişleri ve kazanç hattı
  animasyonları; **Tam isabet** (rulette kazanan plein) ve **Lama Jackpotu** (tek
  hatta 5 lama) başarımları.

Yüksek skorlar `mem.json` dosyasının `highscores` bölümünde (kodun yanında) - dil
ile birlikte (`mem` bölümü) saklanır.

### Arayüz

Tüm arayüz sıfırdan çizilmiştir (saf Tkinter + Pygame, ek paket yok) ve modern
bir oyun başlatıcısı gibi tasarlanmıştır:

- **Kenar çubuğu oyun listesi**: her satırın oyunun vurgu renginde kendi **mini
  piktogramı** vardır, güncel **yüksek skoru (★)** gösterir ve yumuşak animasyonlu
  hover efektleriyle tepki verir. Çalışan oyun renkli olarak işaretli kalır;
  küçük pencerelerde liste fare tekerleğiyle **kaydırılır**.
- Sol altta **durum kartı**, bir **durum LED'i** (gri = menü, yeşil = çalışıyor,
  altın = duraklatıldı, kırmızı = oyun bitti) ve **canlı FPS göstergesi** ile.
- Aurora ışıkları, kayan yıldızlar dahil parallax bir yıldız alanı, yörüngede
  kıvılcımlar olan süzülen bir logo, logonun hemen altında **tıklanabilir bir
  oyun ızgarası** (tüm oyunlar kendi vurgu renginde hover efektiyle) ve bir
  **yüksek skor bandı** içeren **bekleme ekranı**.
- **Her yerde efektler**: yumuşak ekran geçişleri, menüde bir seçimi onaylarken
  kıvılcımlar, **yeni bir yüksek skorda konfeti yağmuru** ve duraklatma
  katmanının arkasında gerçek bir **bulanıklık**.
- Her oyunun **oyun öncesi ekranı** o oyunun vurgu renginde belirir ve önceki
  rekoru bir çip olarak gösterir. Çok mod ve düşük çözünürlükte **kompakt** hâle
  gelir: Seçenekler, Wiki ve Geri tek satıra geçer, yazı boyutu uyum sağlar -
  artık hiçbir şey ekrandan taşmaz.
- **Birleşik oyun içi görünüm**: 46 oyunun tamamı menünün tema paletini ve yazı
  tipini paylaşır - HUD'lar, setup ekranları ve katmanlar seçeneklerde belirlenen
  tasarımı (v4.1 / v4 / Klasik) izlerken her oyun alanı kendi kimlik renklerini
  korur. Artık her oyun, oyun ortasındaki çözünürlük değişikliklerini düzgün
  şekilde ele alır ve menüdeki oyun adları dile duyarlıdır (örneğin "Schach" →
  "Satranç" / « Échecs »).
- **Oyun içi wiki** ("LamaWiki"): her oyun için ayrıntılı yardım (kontroller,
  modlar, puanlama, ipuçları) ve genel sayfalar - **arama kutusu**, kategoriler,
  kaydırılabilir makaleler ve tuş çipleriyle, 14 dilde. Kenar çubuğundaki
  **"Wiki / Yardım"** düğmesiyle ve her oyunun oyun öncesi ekranından erişilebilir
  (doğrudan o oyunun sayfasını açar).
- **Başarımlar ve istatistikler**: üç kategoride **107 başarım** (23 genel
  hedef, 37 puan hedefi ve yapay zekâyı mat etmek, 4096 taşı, T-Spin Double, 25
  çözülmüş satranç bulmacası, Killer Sudoku ya da lama jackpotu gibi 47 özel an;
  2048 ve satrançta geri alma veya ipucu kullanılan oyunlar sayılmaz); açıldığında **altın bildirim ve fanfar** - oyunun
  ortasında bile; eski rekorlar otomatik sayılır. Ayrıca bir **istatistik**
  sekmesi: toplam süre, oyunlar, galibiyetler, rekorlar, favori oyun ve süreye
  göre sıralı oyun tablosu. Kenar çubuğundaki **"Başarımlar ve
  istatistikler"** düğmesinden ulaşılır.
- **Tekrarlar**: minigolf ve bowling her turu kaydeder. Tur sonunda **P**
  tekrarı gösterir, **S** onu arşive koyar - kenar çubuğundaki **Tekrarlar**
  düğmesinden ulaşılır (oyun başına bir sekme, duraklatma, sekans atlama, hız
  0,5x - 4x). İlk açılışta ve seçeneklerde kapatılabilir.

### Kullanım

- Soldaki menüde düğmeyle bir oyun seçin. Ardından bir **oyun öncesi ekranı**
  belirir: **Tek oyunculu** ya da **Çok oyunculu** seçin, **seçeneklere** gidin
  ya da geri dönün. Seçmek için oklar/fare, başlatmak için Enter.
- **ESC** = duraklat / devam et (menülerde: geri).
- **F11** (ya da "Tam ekran aç/kapa" düğmesi) = tam ekranı açar/kapatır. Pygame
  ekranı gömülü kalır ve en boy oranını koruyarak büyütülür (oran farklıysa siyah
  şeritler). Pencere serbestçe yeniden boyutlandırılabilir.
- **"Menüye dön"** oyunu bitirir ve yüksek skoru kaydeder - kenar çubuğundan
  başka bir oyuna geçmek de aynısını yapar.
- **Sabit ek tuşlar**: atanabilen beş eylemin yanında bazı oyunların kendi
  tuşları vardır (ör. Tetris'te saklama **C** ve sola çevirme **Z**, 2048,
  satranç ve Sudoku'da geri alma **U**). Bunlar yalnızca o tuş seçeneklerde
  hiçbir eyleme atanmamışsa çalışır; ayar ipucunda ve wiki'de listelenir. Basılı
  tutulan tuşlar doğru algılanır ve duraklatmada ya da Alt-Tab ile bırakılır -
  artık hiçbir tuş "takılı" kalmaz.
- **"Çıkış"** Pygame ve Tkinter'ı düzgün biçimde kapatır.

### Seçenekler, kontroller ve ses

Seçenekler ekranı, (soldaki) **"Seçenekler / Kontroller"** düğmesiyle ya da oyun
öncesi ekranından açılır. **Üç sekmeye** ayrılmıştır (**Genel / Kontroller /
Görünüm**; tıklayarak ya da Tab tuşuyla geçiş yapılır):

- **Genel**: **ses** açık/kapalı, **ses düzeyi** ve **titreşim** (gamepad
  titreşimi, yalnızca bağlı bir kumandayla etkilidir) ile **otomatik çözünürlük**,
  **çözünürlük**, **FPS** ve **dil** – her biri Sol/Sağ ile değiştirilir.
- **Kontroller**: **hazır ayarlar** (*WASD + Oklar*, *WASD + IJKL*, *Oklar +
  WASD*) ve 1. ve 2. oyuncu için **her bir tuşu yeniden atama**: bir satır seçin,
  Enter'a basın, istediğiniz tuşa basın (Esc iptal eder).
- **Görünüm**: **arayüz tasarımını** seçin – **UI v4.2** (öntanımlı: Midnight
  Glass – yavaşça süzülen çivit, turkuaz ve macenta ışıklarla derin gece mavisi
  geçiş, ince film greni, seyrek yıldızlar ve ışık kenarlı buzlu cam paneller),
  **UI v4.1** (UI v4 gibi ama daha canlı – ince yıldızların yanı sıra başlangıç
  ekranının arka planında Satürn ve bir kara delik), **UI v4.1.1** (v4.1 gibi,
  ancak yıldızlı gökyüzü yerine siyah ve antrasit renkli döşenmiş bir
  **zikzak deseni**), **UI v4.1.2** (aynı desen paletin mavileriyle – baskın
  renk vurgu mavisi, zemin daha koyu mavi), **UI v4.1.3** (aynı desen, siyah
  üzerine UI v4 çivit rengi), **UI v4.1.4** (siyah üzerine UI v4 grafit tonu),
  **UI v4** (tek bir çivit vurgusuyla tamamen sakin, düz grafit görünüm),
  **UI v3** (yıldızlı gökyüzü, aurora ışıkları ve parıltı efektleriyle önceki
  klasik arayüz), **UI v2** (ilk arayüz yenilemesi: lacivert geçiş, yıldızlı
  gökyüzü ve parlayan düğmeler, hiç animasyon yok) ya da **UI v1** (arayüz
  yenilemesinden önceki görünüm: düz koyu arka plan, düz düğmeler, efekt yok).
  Tüm kartlar küçük bir önizleme gösterir; seçim anında tüm arayüze (oyun alanı
  **ve** kenar çubuğu) uygulanır ve kaydedilir.

Ayarlar kalıcı olarak `settings.json` içinde saklanır. **Tek oyuncuda** her iki
atama da aynı karakteri kontrol eder (öntanımlı: WASD *ve* oklar), **çok
oyuncuda** her biri birini. Tüm oyunlarda küresel olarak sessize alınabilen **ses
efektleri** vardır (prosedürel olarak üretilir, ek dosya gerekmez).

### Proje yapısı

```
install-python.bat  Windows kurulumu: Python 3.13 + .venv + pygame
start.bat           Başlatma betiği (Windows)
start.sh            Başlatma betiği (Linux / macOS / Git Bash)
pyinstall.bat       EXE derlemesi (Windows): her şeyi builds\PyGameZ.exe içine paketler
main.py             Tkinter arayüzü, Pygame gömme, merkezî oyun döngüsü
game_base.py        Oyun temel sınıfı (update/draw/handle_event) + InputEvent + yardımcılar
settings.py          Ayarları yükler/kaydeder (ses/titreşim/tuş atamaları/doğrulama kurallı oyun seçenekleri) (JSON)
audio.py             Prosedürel ses efektleri, müzik döngüleri + gamepad titreşimi
menu.py             Dil, oyun öncesi (mod) ve seçenekler ekranı (ses/kontroller)
highscore.py        Yüksek skorları yükle/kaydet (mem.json içindeki bölüm)
store.py             Merkezi kayıt dosyası mem.json (bölümler: mem, highscores, stats, achievements + oyun ilerlemeleri), .bak yedekli atomik yazım
stats.py             Oyuncu istatistikleri (oyun, süre, galibiyet, rekor) oyun başına
achievements.py      Başarımlar: tanımlar, açma mantığı, bildirim (toast)
progress.py          Başarımlar ve istatistikler ekranı (iki sekme, kaydırılabilir)
replay.py            Tekrarların kaydı ve arşivi (replay.json)
replayview.py        Tekrar ekranı: arşiv listesi ve oynatma
ugc.py               Kendi içerikler (minigolf parkurları, Geometry Dash seviyeleri): depolama, doğrulama, dışa/içe aktarma (ugc.json)
swear.py             Adlar ve id'ler için kelime filtresi (lang/swear/*.yml, 14 dil)
filepick.py          Dosya pencereleri ("Farklı dışa aktar ...", "İçe aktar")
prestige.py         Snake için prestij sistemi
competitive.py      Snake'in Rekabetçi modu için ince ayar (seviyeler, slot makinesi, kumar elmaları)
ngb.py              Görsel kişiselleştirme ("mods"): baş rengi + koordinat ızgarası + menü (mem-ngb.json)
lamabank.py          Lama Bankası: Blackjack, Poker ve Casino'nun ortak çip hesabı (mem.json içindeki casino bölümü)
seedrand.py          Python'da ve tarayıcıda bit bit aynı sayılar üreten rastgele üreteci (günlük modlar, yeni bulmacalar)
i18n.py             Çeviri motoru (lang/*.json yükler, t("anahtar"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Dil metinleri (metin başına bir yer tutucu anahtar)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Her dil için kelime filtresi listeleri (regex, .yml)
lamawiki/
  lamawiki.py          Oyun içi wiki (arama, kategoriler, makale işleyici)
  de.json  en.json  fr.json  es.json  pt.json   Wiki içeriği (oyun başına bir sayfa + genel sayfalar)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Wordle kelime listelerini yeniden üretir (sözlükler + sıklık listeleri)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 harf), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 harf), 14 dil
devtools/            Geliştirici araçları (.exe'ye paketlenmez)
  merge_staging.py           devtools/staging/ içindeki çevirileri ve wiki sayfalarını 14 dil dosyasına işler
  build_chess_puzzles.py     200 satranç bulmacasını Lichess bulmaca veritabanından (CC0) oluşturur
  build_sudoku_killer.py     Tek çözümlü 400 Killer Sudoku üretir
  build_crossyroad_models.py Crossy Road voksel modellerini web sürümü için yazar
  build_geodash_levels.py    8 Geometry Dash bölümünü oluşturur ve her birinin altınlarla birlikte geçilebildiğini çözücüyle kanıtlar
  build_geodash_solver.py    Gerçek adım koduyla çalışan çözücü (çözümler geodash_proofs.json içinde)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Seviye verileri: snake-comp.json, chess-puzzles.json (+ kaynak README), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Genel denetim (giriş, seedrand Python = JS, kayıt, dil dosyaları, oyun öncesi ekranları) + tüm audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Oyun başına headless denetimler
  newgames_audit.py  blockjump_audit.py
```

Seçilen dil `mem.json` içinde (aynı dosyadaki `highscores` bölümünün yanındaki
`mem` bölümünde) saklanır ve bir sonraki açılışta otomatik olarak yüklenir.

**Kaynaklar ve lisanslar:** 200 satranç bulmacası
[Lichess bulmaca veritabanından](https://database.lichess.org/#puzzles) gelir
(lisans **CC0 1.0**, kamu malı - teşekkürler, lichess.org!); ayrıntılar
`games/levels/chess-puzzles.README.md` dosyasındadır. Wordle kelime listelerinin
kaynakları `woordlistz/README.md` dosyasında belirtilir.

### Platform notları

Görüntü **ekran dışında** çalışır: pygame, dummy video sürücüsünü
(`SDL_VIDEODRIVER=dummy`) kullanır, yani bir yüzeye işler ve her kare bir Tkinter
parçacığına görüntü olarak çizilir. Boyut/konum için Tkinter ile çekişebilecek
**yerel bir SDL penceresi yoktur**. Sonuç olarak pencere her yerde aynı ve
kararlı şekilde davranır:

- **Windows**: süreç ayrıca DPI-aware yapılır; böylece görüntü ölçeklenmiş
  ekranlarda (%125/150/200) net kalır ve "titremez".
- **Linux/X11 ve Wayland**: özel durumlar olmadan çalışır (`SDL_WINDOWID` yok).
- **macOS**: burada da çalışır (önceden gömülü pencere burada hiç görünmüyordu).

---

### Kurulum Kılavuzu

Gereksinim: **Python 3.9+** (önerilen 3.12 ya da 3.13) ve **pygame ≥ 2.6**.

#### Windows (önerilen: otomatik)

1. Proje klasörünü açın ve **`install-python.bat`** dosyasına çift tıklayın.
   Betik
   - **Python 3.13**'ün olup olmadığını kontrol eder, yoksa **winget** ile kurar
     (`winget install Python.Python.3.13`),
   - **`.venv`** sanal ortamını oluşturur,
   - `requirements.txt` üzerinden **pygame**'i kurar.
2. Ardından koleksiyonu **`start.bat`** ile başlatın (çift tıklama).

> Not: betik "bu pencerede henüz kullanılamıyor" derse, Python az önce
> kurulmuştur – yalnızca **yeni bir terminal/pencere** açın ve
> `install-python.bat` dosyasını tekrar çalıştırın. **winget** yoksa Python
> 3.13'ü <https://www.python.org/downloads/> adresinden elle kurun ve
> **"Add python.exe to PATH"** kutucuğunu işaretleyin.

#### Windows / Linux / macOS (elle)

```bash
# 1. Python'ı kontrol et (3.9+)
python --version

# 2. Sanal ortam oluştur ve etkinleştir
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Bağımlılıkları kur
pip install -r requirements.txt
#   ya da:  pip install "pygame>=2.6" (veya pygame-ce)
#                                     pip install pygame-ce
# 4. Başlat
python main.py
```

#### start.sh ile Linux / macOS

```bash
# Python + venv'i yukarıdaki gibi kur (adım 2 ve 3), sonra:
chmod +x start.sh      # bir kez, henüz çalıştırılabilir değilse
./start.sh
```

Linux'ta gerekirse Python'ı paket yöneticisiyle kurun, örneğin
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); macOS'ta
örneğin `brew install python`.

#### Farklı bir Python sürümü kullanma

`install-python.bat` öntanımlı olarak Python 3.13 kurar. 3.12'yi (ya da başka bir
sürümü) tercih ederseniz, dosyadaki `set "PYVER=3.13"` satırını istediğiniz
sürüme ve winget kimliğini buna göre (`Python.Python.3.12`) değiştirin.

#### Bağımsız bir EXE oluşturma (Windows)

```bat
pyinstall.bat         :: builds\PyGameZ.exe oluşturur (her şey tek dosyada)
```

`pyinstall.bat`, `.venv`'i kullanır (gerekirse oluşturur), **PyInstaller**'ı
otomatik olarak kurar ve oyunun tamamını - Python, pygame, tüm oyunlar, diller,
wiki ve logolar - **`builds\`** klasöründe **tek bir `PyGameZ.exe`** içine
paketler. Dosya, Python kurulu olmayan herhangi bir Windows bilgisayarında
çalışır ve serbestçe kopyalanabilir. Ayarlar ve yüksek skorlar (`settings.json`,
`mem.json`, `mem-ngb.json`) oyun sırasında .exe'nin yanında oluşturulur.

#### Sorun giderme

- **`pygame` bulunamadı** → venv etkin mi? 3. adımı tekrarlayın
  (`pip install -r requirements.txt`).
- **`python` tanınmıyor (Windows)** → Python "Add to PATH" olmadan kurulmuş;
  yeniden kurup kutucuğu işaretleyin ya da `python` yerine `py` kullanın.
- **Ses yok** → seçeneklerde "Ses"i kontrol edin; titreşim yalnızca bir kumandayla
  çalışır.
- **Linux'ta pencere/gömme** → bkz. *Platform notları* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ başa dön / back to top</a></b></div>

---

<a name="-dansk"></a>

## 🇩🇰 Dansk

En skrivebords-spilsamling i Python: **Tkinter** leverer vinduet og menuen,
**Pygame** indlejres som spildisplay inde i Tkinter-vinduet. Seksogfyrre spil
med fælles indstillinger, frit omdefinerbare kontroller, highscores, proceduralt
genererede lydeffekter og – for nogle titler – en multiplayer-tilstand.
Grænsefladen er **flersproget** og fås på **14 sprog**; sproget vælges på en
**velkomstskærm** ved første start, hvor man også kan indstille **opløsning** og
**lyd** (slået fra som standard). Ud over kernesprogene gemmer de øvrige sprog –
heriblandt dansk – sig bag knappen **»Flere sprog«**. Alt kan til enhver tid
ændres senere i indstillingerne.

### Hurtig start

#### Windows

```bat
install-python.bat    :: én gang: opsæt Python 3.13 + .venv + pygame
start.bat             :: start spilsamlingen
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # starter med .venv, ellers systemets python3
```

`start.bat` / `start.sh` bruger automatisk det virtuelle miljø `.venv`, hvis det
findes, ellers systemets Python. En udførlig trin-for-trin-vejledning findes
nederst under **[Installationsguide](#installationsguide)**.

### Spillene

| Spil         | Tilstande       | Kort beskrivelse |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 spillere  | Deluxe-Snake med 2D- og 3D-visning, boost, 6 spiltilstande (inkl. Competitive), gyldne æbler og prestige |
| **Pong**     | 1 / 2 spillere  | Klassikeren mod AI eller spiller 2, omskiftelig bevægelsestilstand |
| **Air Hockey** | 1 / 2 spillere | 2D-fysik med impulsoverførsel, musestyring, AI og power-ups |
| **Tic-Tac-Toe** | 1 / 2 spillere | m,n,k-spil på 3x3 til 9x9, tre AI-styrker **eller** lokal X mod O |
| **Breakout** | 1 spiller       | Brick breaker med stentyper, power-ups, combos og mange baner |
| **Tetris**   | 1 / 2 spillere  | Moderne Guideline-regler (SRS, reserve, 5-brikkers forhåndsvisning, T-Spins): Maraton, Sprint 40, Ultra 2:00, Versus mod AI'en (3 styrker) eller for to med affaldsrækker |
| **Invaders** | 1 spiller       | Space Invaders: ryd bølgerne, beskyt dine liv |
| **Asteroids** | 1 / 2 spillere | Inertifysik, bølger, UFO'er, power-ups, hyperrum - solo eller co-op-duel |
| **Pac-Man**  | 1 spiller       | Tro klon: 4 spøgelses-AI'er, power-piller, tunnel, frugt, baner |
| **Flappy Bird** | 1 spiller    | Tyngdekraftsflugt gennem rør, mønter, skjold, dag/nat, medaljer |
| **Doodle Jump** | 1 spiller    | Auto-hop opad, platformstyper, fjedre, propel, monstre |
| **2048**     | 1 spiller       | Tal-skydespil fra 3x3 til 8x8: Klassisk, Tidsangreb og Uendelig, fortryd, flydende animationer, gemte partier |
| **Minesweeper** | 1 spiller    | Klassikeren med sikkert første klik, chording, smiley og bedste tider |
| **Sudoku**      | 1 spiller    | 4 varianter (Klassisk, X-sudoku, Killer, Mini 6x6) med hver 400 baner, dagens sudoku, op til 3 stjerner pr. bane, 4 hjælpetilstande, fortryd, gemt spil |
| **Frogger**     | 1 spiller    | Vej + flod + 5 bugter, bonusflue, krokodiller, tidsgrænse, 3 sværhedsgrader |
| **Memory**      | 1 / 2 spillere | Find par på 4x4 op til 8x6, vendeanimation, solo-scoring eller duel |
| **Kabale**      | 1 spiller    | 5 varianter (Klondike, Spider, FreeCell, Pyramide, TriPeaks) med træk og slip og fortryd |
| **Aim Trainer** | 1 spiller    | Afslappet 3D-målskydning: musen styrer kameraet, 4 tilstande (præcision/refleks/bevægelig/chill), 3 temaer inkl. et sort hul |
| **Fire på stribe** | 1 / 2 spillere | Klassikeren med falde-animation: 3 AI-styrker (minimax) eller lokal duel |
| **Tankduel**    | 1 / 2 spillere | 2D-arenaduel med rikochetskud, power-ups, 4 arenaer, AI med 3 styrker |
| **Blackjack**   | 1 spiller    | Casino-blackjack med 4-decks sko, double/split og 3:2-blackjack; spilles med lama-chips fra den fælles Lama-bank |
| **Tunnel Racer** | 1 spiller   | 3D-neon-rørflugt: endeløs tilstand + 30 baner, tast- eller musestyring, motion blur |
| **3D-labyrint** | 1 spiller    | Førstepersons-raycaster (Wolfenstein-stil) med 50 seed-genererede baner, orbs, minikort - eller 2D-fugleperspektiv |
| **Reversi**     | 1 / 2 spillere | Othello på 8x8: indfang og vend brikker, 3 AI-styrker (minimax) eller lokal duel |
| **Yatzy**       | 1 / 2 spillere | Terningklassiker med 13 kategorier, øvre bonus og Yatzy; highscore-jagt eller 2-spiller-hotseat |
| **Wordle**      | 1 spiller    | Gæt ord på 4 til 7 bogstaver: Uendelig, Dagens ord, Dordle og Quordle, svær tilstand, farveblind-palet, statistik med søjlediagram, del resultatet, rigtige ordlister på 14 sprog |
| **T-Rex Runner** | 1 spiller   | Endeløst ørkenløb: variabelt hop, dukke, kaktusser og pterodaktyler, dag/nat-cyklus, stigende tempo, 3 sværhedsgrader |
| **Dam**         | 1 / 2 spillere | 3 regelsæt (tysk 8×8, international 10×10, checkers), slagtvang og flyvende dam, 3 AI-styrker (minimax) eller lokal duel |
| **Poker**       | 1 spiller    | 3 valgbare varianter: Texas Hold'em mod AI, 5 Card Draw og Video Poker; budrunder, blinds, lama-chips fra den fælles Lama-bank |
| **Skak**        | 1 / 2 spillere | Fulde regler, Chess960 og skakur, 6 AI-styrker, 200 opgaver fra Lichess-databasen, fortryd/hint, trækliste, PGN-eksport eller lokal duel |
| **Mølle**       | 1 / 2 spillere | Sætte-/flytte-/springfaser, møller og slag, valgfri flyveregel, 3 AI-styrker eller lokal duel |
| **Simon**       | 1 / 2 spillere | Senso-huskespil: Klassisk/Speed/Reverse/Blandet-tilstande + duel, lyd fra/til/blandet, 4/6/9 felter, bedste pr. tilstand |
| **Billard**     | 1 / 2 spillere | 8-ball, 9-ball og øvelsestilstand i 2D, fast 3D-visning eller frit roterbart 3D-kamera; blød fysik, sigtehjælp, 3 AI-styrker |
| **Skydepuslespil** | 1 spiller | 15-spillet i 3x3/4x4/5x5: skub de nummererede brikker ind i hullet, klik- eller pilestyring, point efter træk og tid |
| **Mastermind**  | 1 spiller    | Knæk den hemmelige farvekode (3 tilstande: 4×6, klassisk, 5×8), sorte/hvide feedbackpinde, endeløs streak-highscore |
| **Bubble Shooter** | 1 spiller | Puzzle Bobble-klon: skyd ens farver i grupper af tre, vægafspring, faldende klynger, 3 sværhedsgrader |
| **Galgemand**   | 1 spiller    | Gæt ordet, før galgen er færdig; skærmtastatur, ordlister pr. sprog, 3 længdetilstande, endeløs streak |
| **Block Jump**  | 1 spiller    | 3D-platformspil i Minecraft-stil: tekstureret voxelverden med Steve-figur, stiger, hegn og slimblokke, første-/tredjepersonskamera, motion blur, seed-genererede parkourbaner |
| **Tower Defense** | 1 spiller   | Slå endeløse bølger tilbage på 4 kort: op til 11 tårntyper med opgraderinger, salg & A/B-specialisering, bosser, 3 tilstande, aktive evner |
| **Minigolf**    | 1 / 2 spillere | 360 baner på 40 forløb (18 håndbyggede, 342 genererede): sand, ramper, vand, bumpere, vindmøller & vandrende klodser; scorekort med par og hole-in-one-bonus; **egen huleditor** med 15 objekttyper, 12 skabeloner og deling som `.lamapgzmap` |
| **Pinball**     | 1 / 2 spillere | Flippermaskine med 3 borde: bumpere, slingshots, mål, L-A-M-A-baner, multiball med jackpot, kugleredning, puf & tilt |
| **Bowling**     | 1 / 2 spillere | 10 frames med officiel strike/spare-optælling, ægte keglefysik, hook-skrue og bane i perspektiv, 3 sværhedsgrader |
| **Crossy Road** | 1 spiller     | Endeløse hop over enge, veje, floder og togskinner i isometrisk voxel-look: dag/nat, ørn, 10 figurer at købe, dagens rute |
| **Geometry Dash** | 1 spiller   | Rytme-platformspil med terning, skib, bold, UFO og bølge: 8 baner fra Let til Dæmon med 3 hemmelige mønter hver, træningstilstand, soundtrack pr. bane; **baneeditor** med deling som `.lamapgzlevel` |
| **Battleship**  | 1 / 2 spillere | Søslag på 10x10: flåden placeres med træk og slip, 3 regelkontakter (berøring, salve, skyd igen), AI med 3 styrker eller lokal duel med overdragelsesskærm |
| **Casino**      | 1 spiller     | Europæisk roulette med alle klassiske indsatser og Lama-slot (5 hjul, 10 gevinstlinjer, wild, gratisspil); én lama-chip-konto sammen med Blackjack og Poker |

**Multiplayer (2 spillere lokalt)** findes til **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (co-op-duel)**,
**Memory (duel)**, **Fire på stribe**, **Tankduel**, **Reversi**, **Yatzy**,
**Dam**, **Skak**, **Mølle**, **Simon (duel)**, **Billard**, **Minigolf**,
**Pinball**, **Bowling** og **Battleship** (med en overdragelsesskærm, der skjuler
flåderne) - i alt 20 spil. Tilstanden vælges direkte på forspils-skærmen
(*Enkeltspiller / Multiplayer*); Tetris har desuden **Versus mod AI'en**.
Webversionen er kun til én spiller.

#### Funktionsdetaljer pr. spil

**Snake**
- **NYT - 3D-visning** (tast **V** i setup eller klik på *Visning*): brættet
  gengives som en 3D-scene i realtid - et **forfølgerkamera** svæver bag slangen,
  og styringen er **relativ til synsretningen** (venstre/højre = drej, to hurtige
  tryk = helomvending). Med afstandståge, en stjernehimmel, skakbrætgulv, bander,
  roterende madkrystaller, 3D-partikler og kamerarystelse ved sammenstød; efter
  game over kredser kameraet langsomt om slangen. Boost udvider synsfeltet.
  Tilgængeligt i 3D: *Klassisk* og *Forhindringer* (murene er altid faste dér,
  3D er kun for én spiller). Visningen huskes i `settings.json`.
- **NYT - 3D-kameraindstillinger** (klik på rækken *3D-kamera / smooth shake* i
  3D-setup, eller tast **K**): en dedikeret menu med **smooth shake** (et blidere
  kamera, langt mindre rysten ved bevægelse/drejning), justerbart **synsfelt
  (FOV)** og **kamerahøjde** samt en kontakt **rysten ved drejning**
  (skærmrystelse ved venstre/højre-drejninger til/fra). Alt huskes i
  `settings.json`.
- **Boost**: **hold** boost-tasten nede = turbo (dobbelt tempo), forbruger
  udholdenhed (bjælke); når den er tom, slår boostet fra og lades op igen.
  Standard S1 = Mellemrum/venstre Shift, S2 = Enter/højre Shift.
- **6 spiltilstande** (vælges i setup): *Klassisk*, *Speed Rush* (bliver
  hurtigere for hvert æble), *Forhindringer* (dødbringende klodser), *Portaler*
  (teleporterpar), *Tidsangreb* (60 sekunder, så mange æbler som muligt) og
  *Competitive* (se nedenfor).
- **NYT - Competitive** (én spiller): endeløs tilstand med en **niveaustigning** -
  du starter med præcis **ét** æble og kan ikke få flere i begyndelsen; jo flere
  æbler du samler i alt, jo højere bliver dit **niveau**, som løbende lægger endnu
  et samtidigt æble på banen og hæver pointmultiplikatoren. **Blå æbler** åbner en
  **enarmet tyveknægt**: din længde er indsatsen, hjulresultatet mangedobler eller
  formindsker den og får kortvarigt **ekstra æbler** til at dukke op (jackpot ved
  tre ens symboler). **Lilla æbler** (gambling) sætter en del af din **størrelse**
  på spil og mangedobler den del tilfældigt, resten forbliver sikker (ny størrelse
  = størrelse·(1-p) + størrelse·p·faktor): **normal** satser faste 50 % med
  **x0.5 .. x1.5**, **HARDCORE** er mere risikabel med en indsats på **75-90 %** og
  **x0.25 .. x2.25**. Din **størrelse** vises som et **decimaltal øverst til
  venstre** og videreføres nøjagtigt, så efterfølgende væddemål bygger videre på
  den. Der er **15 niveauer** (multiplikator op til x16, op til 16 æbler ad
  gangen); niveauerne ligger i `games/levels/snake-comp.json` og kan udvides dér
  uden at røre koden, resten af finindstillingen ligger i `competitive.py`.
- **NYT - HARDCORE** (kontakt i Competitive-setup, tast **H**): hvert **boost æder
  af din slanges længde**; et rødt lysende **HARDCORE-skilt** markerer tilstanden.
  Kun i Competitive; længden falder aldrig under minimum. Huskes i `settings.json`.
- **Gyldne æbler** (midlertidige) giver masser af point og fylder boostet op med
  det samme.
- Valgfrit: **vægge man kan passere** (wrap-around), bonusæbler, **prestige** (én
  spiller, tast **P**).
- **NYT - Tilpas** (penselknap øverst til højre i setup, eller tast **C**): en rent
  visuel menu ("mods", der *aldrig* ændrer gameplayet) med to faner:
  - **Hoved**: slangens **hovedfarve** - 4 blå-turkise forudindstillinger (fra
    mere blå til mere turkis), rød, orange og en **egen farve** via RGB-skydere.
  - **Gitter (vejviser)**: lægger et **koordinatgitter** over banen - **rækkenumre**
    (i venstre og højre kant) og **kolonnebogstaver** (top/bund). På store brætter
    ser man dermed straks, at f.eks. æblet ved *8a* ligger i samme række *8* som ens
    egen position *8z*. Farvesekvensen (5 forudindstillinger + to egne farver A/B)
    fastsætter farvetemaet.
  - **Banner**: slå multiplikatorbanneret (f.eks. fra det lilla æble) **til/fra** og
    juster dets **størrelse** (mindre/større) og **uigennemsigtighed** (mere
    gennemsigtig) - med en live-forhåndsvisning.
  Alt gemmes i `mem-ngb.json`; al visuel tilpasning kører gennem modulet `ngb.py`.
- Udseende: afrundet slange med øjne (hovedet er turkis som standard), boost-glød,
  partikler.

**Pong**
- Én spiller mod AI, multiplayer = spiller 2 til højre. Først til 5 point.
- **Bevægelsestilstand kan skiftes pr. kontrolsæt**: *Kontinuerlig* (tryk én gang
  -> bliver ved med at bevæge sig, standard) eller *Hold* (bevæger sig kun, mens
  der holdes). Skift: **X** = kontrolsæt 1, **N** = kontrolsæt 2 (huskes i
  `settings.json`).
- Boldfysik med acceleration og vinkel afhængigt af rammepunktet.

**Air Hockey**
- **Ægte 2D-fysik**: runde køller og puck med impulsoverførsel - pucken overtager
  køllens hastighed ved anslag; bander med restitution, let isfriktion, mål som
  åbninger i sidevæggene.
- **Musestyring** i énspiller: køllen følger musen (enhver tast skifter tilbage
  til tastatur). Tastatur: retningstaster i 8 retninger, multiplayer = S1 venstre
  (WASD), S2 højre (IJKL).
- **AI med tre styrker** (Let/Middel/Svær): forsvarer sit eget mål, angriber i sin
  egen halvdel og manøvrerer uden om pucken for at undgå selvmål.
- **Power-ups** (kan slås fra): *XL* (større kølle), *MÅL* (modstanderens mål
  skrumper), *>>* (hurtigere kølle) - de tilhører den spiller, der sidst rørte
  pucken.
- Setup: sværhedsgrad, **mål for at vinde** (3/5/7/10), power-ups til/fra (gemmes i
  `settings.json`). Efter hvert mål har den, der lukkede målet ind, afslag.
- Udseende: puckens lysspor, partikler, pulserende målmunde, effektmærker.

**Tic-Tac-Toe**
- Setup: sværhedsgrad (Let/Middel/Svær) og brætstørrelse 3x3..9x9; vinderlængde
  K = 3 (3x3), 4 (4x4), ellers 5.
- **1 spiller** mod AI (Svær på 3x3 er uovervindelig) **eller 2 spillere** lokalt
  (X mod O, skiftevis ved klik). Ved game over: Enter/klik = ny runde,
  **S** = indstillinger.

**Breakout**
- Stentyper: Normal, **Stål** (uødelæggelig), **Bombe** (eksploderer), **Guld**
  (ekstra point).
- Power-ups: laser, ildkugle, klæbrig, skjold, mønt m.m.; **combo-multiplikator**.
- Effekter: partikler, boldspor, skærmrystelse, point-popups, mange banemønstre.
- Setup: **1/2/3** = sværhedsgrad, **Venstre/Højre** = boldfarve, **Op/Ned** =
  startniveau, **M** = opbygning. Spil: mus/piletaster, **Mellemrum** lancerer
  bolden (affyrer laser), **P/Esc** = pause.

**Tetris**
- **Moderne Guideline-regler**: 10x20-felt, brikker fra en **7-pose**,
  **SRS-rotationssystemet** med ægte wall kicks (også for I-brikken), drejning i
  begge retninger, **reserve** (én gang pr. brik), **forhåndsvisning af 5
  brikker**, skyggebrik og **lock delay** (0,5 s, højst 15 nulstillinger).
- **Tre tilstande** på forspils-skærmen: *Solo*, *Mod AI* og *2 spillere*. Solo
  tilbyder i opsætningen **Maraton** (startniveau 1-15, tæller til highscoren),
  **Sprint 40 rækker** (bedste tid) og **Ultra 2 minutter** (bedste score);
  rekorderne ligger i afsnittet `tetris` i `mem.json`.
- **Guideline-pointgivning**: single til Tetris, **T-Spins** (fulde og mini),
  **Back-to-Back** (x1,5), **combos** og **Perfect Clear** - med tekster på
  skærmen, rydningsanimation, hard-drop-spor, partikler og level-up-effekt.
- **Versus med affaldsrækker**: ryddede rækker sender affald til modstanderen
  (Tetris = 4, T-Spin Double = 4 …), indkommende affald varsles i en advarselsbjælke
  og **modregnes** af dine egne angreb; begge felter får den samme brikrækkefølge.
  **AI'en** findes i 3 styrker, og tempoet stiger hvert 40. sekund.
- **Styring**: Venstre/Højre med eget **DAS/ARR** (kan indstilles i
  opsætningen), Op = drej til højre, Ned = soft drop, Handling = hard drop;
  **C**/Shift = reserve, **Z**/**Y** = drej til venstre, **X** = drej til højre.
  For to lægger spiller 1 i reserve med **Q** og drejer til venstre med **E**,
  spiller 2 med **højre Shift** / **højre Ctrl**. Efter spillet: **R** = igen,
  **S** = opsætning.

**Invaders** – to tilstande (vælges på forspils-skærmen):
- **Klassisk**: den klassiske alienblok; derefter valgbart på setup-skærmen:
  **Bevægelse** (kun venstre/højre *eller* frit med WASD) og **Sigte** (altid
  opad *eller* mod **musen** – så skyder du derhen, hvor markøren er). Ødelagte
  aliens taber nogle gange power-ups.
- **Arena (fri)**: fri bevægelse i alle retninger, fjender strømmer ind fra alle
  kanter; man sigter i bevægelsesretningen, skift våben med **1–4**.
Fælles: niveausystem med en **boss** i hvert 4. niveau, fire våben (blaster,
spredeskud, hurtigskud, laser), power-ups (ekstra liv, skjold, våbenopgradering),
eksplosionseffekter, highscore.

**Asteroids**
- **Inertifysik**: op = fremdrift i synsretningen, venstre/højre = drej, skibet
  driver videre (let dæmpning); alt fortsætter over på den modsatte skærmkant.
  Klassisk **vektorlook** med udstødningsflamme og en stjernehimmel; hver klump
  har sin egen tilfældige polygonform.
- Klumper splintres i to mindre (3 størrelser, **20/50/100 point**), **bølger**
  med voksende antal og en banner-annoncering.
- **UFO** (kan slås fra): krydser skærmen med jævne mellemrum og sigter på skibene
  (sigtefejl afhænger af sværhedsgraden) - 200 point for at skyde den ned.
- **Power-ups** (kan slås fra), tabes af ødelagte klumper: **S**kjold (6 s
  usårlig), **T** = trippelskud, **R** = hurtigskud.
- **Hyperrum** (Ned-tast): nødspring til en tilfældig position med 4 s nedkøling -
  og 12 % risiko for at sprænges ved ankomst.
- 3 liv, sikker genopstandelse med usårligheds-blinken, **ekstra liv for hver
  5000 point**; eksplosionspartikler og kamerarystelse.
- **Co-op-duel** (multiplayer): begge skibe flyver samtidig med separate liv og
  point - den, der scorer mest, vinder.
- Setup: sværhedsgrad, UFO'er til/fra, power-ups til/fra (gemmes i `settings.json`).

**Pac-Man**
- **Klassisk 28x31-labyrint** i neonlook med piller, 4 power-piller,
  sidetunnel-warps og et spøgelseshus i midten.
- **Fire spøgelser med de originale adfærdsmønstre** (målfelt-AI): *Blinky* jager
  direkte, *Pinky* lægger baghold (4 felter foran), *Inky* bruger en vektor gennem
  Blinky, *Clyde* trækker sig, når han er tæt på.
- **Scatter/chase-faser** skifter på skift (spøgelser vender om ved hvert skift);
  en **power-pille** gør spøgelserne blå og spiselige (kæde 200/400/800/1600),
  hvorefter deres øjne vender tilbage til huset.
- Spøgelseshus med **forskudt frigivelse**, **frugt**-bonusser (pr. niveau),
  **3 liv**, **ekstra liv ved 10.000**, niveausystem (bliver hurtigere),
  dødsanimation, READY/GAME OVER-skærme.
- Setup: **sværhedsgrad** (Normal/Svær/Ekstrem) – spøgelsestempo og frightened-tid.
- Styring: **piletaster eller WASD**.  Enter = ny, S = setup.

**Flappy Bird**
- **Tyngdekraftsfysik**: Mellemrum / Op / W / **museklik** får fuglen til at
  baske; den hælder efter stige-/faldetempoet.
- Endeløse **rørpar** med et hul (+1 pr. rør); **mønter** (bonus) og et
  **skjold**-power-up (overlever ét sammenstød) dukker op i hullerne.
- **Dag/nat-temaer** skifter med pointtallet; drivende skyer (parallakse),
  rullende jord.
- Sværhedsgrad (Let/Normal/Svær): hulstørrelse, tempo, rørafstand – hullet bliver
  en smule snævrere, efterhånden som pointtallet stiger.
- **Medaljer** (bronze/sølv/guld/platin) ved game over, sammenstøds-animation med
  kamerarystelse, highscore.

**Doodle Jump**
- Doodleren **hopper automatisk** ved landing; du styrer kun venstre/højre (med
  inerti), kanterne går rundt (wrap-around), og kameraet ruller op, mens du stiger.
- **Platformstyper**: grøn (normal), blå (bevægelig), brun (går i stykker), hvid
  (forsvinder). **Fjedre** giver et superhop, **propelhatten** bærer dig kortvarigt
  opad (og gør dig usårlig).
- **Monstre**: berøring er dødelig – men du kan **skyde** dem med Op / Mellemrum
  (bonuspoint).
- Point = nået højde; sværhedsgraden stiger med højden. Highscore.
- Styring: venstre/højre = bevæg, Op / Mellemrum = skyd.

**2048**
- **Egen opsætningsskærm** med brætstørrelser fra **3x3 til 8x8** og tre
  tilstande: *Klassisk* (mål 2048, derefter »Spil videre?«), *Tidsangreb* (3
  minutter, uret starter ved første træk) og *Uendelig*.
- **Flydende animationer**: brikker glider, smelter sammen med et »pop« og vokser
  frem; point-popups, gnister fra 128, en trykbølge fra 2048 og nye farver helt
  op til 131072. Input under en animation bliver gemt og udført bagefter.
- **Fortryd** (fra / 3 pr. spil / ubegrænset, tast **U** eller Backspace) - bruger
  du det, spiller du uden highscore og uden brik-præstationer.
- **Gem og fortsæt**: det igangværende parti gemmes automatisk pr. størrelse og
  tilstand; bedste score og største brik pr. størrelse/tilstand ligger i afsnittet
  `g2048` i `mem.json`.
- Styring: piletaster/WASD eller **swipe** med mus/touchpad, **R**/**N** = nyt
  spil, **Tab** = opsætning. Highscoren tæller kun i **4x4 Klassisk** uden
  fortryd.

**Minesweeper**
- Tre niveauer: **Begynder** (9x9, 10 miner), **Øvet** (16x16, 40), **Ekspert**
  (30x16, 99) - den **bedste tid pr. niveau** gemmes og vises i setup.
- Det **første klik er altid sikkert** (minerne placeres først bagefter, 3x3-området
  omkring klikket forbliver frit).
- **Venstreklik** = afdæk, **højreklik** = flag (valgfrit med spørgsmålstegns-cyklus),
  **F** = flag under markøren, **R** = nyt spil.
- **Chording**: klik på et opfyldt tal afdækker de resterende naboer.
- Klassisk HUD: minetæller, **klikbar smiley** (forbløffet/solbriller/død), timer;
  forkerte flag streges ud til sidst, konfetti ved sejr.
- Point = niveauets grundværdi minus sekunder.

**Sudoku**
- **4 varianter** med hver **400 baner** (4 sværhedsgrader x 100): *Klassisk*
  (de kendte seed-baner - løste baner forbliver afkrydset), *X-sudoku* (begge
  diagonaler indeholder hvert ciffer præcis én gang), *Killer* (stiplede bure med
  sum; 400 baner genereret på forhånd) og *Mini 6x6*. Alle puslespil har en
  **entydig løsning** - bane 12 af "Svær" er det samme puslespil på enhver PC.
- **Dagens sudoku**: én opgave om dagen til alle, ens på PC og i browseren;
  sværhedsgraden afhænger af ugedagen (fra mandag Let til lørdag Ekspert), og løser
  du hver dag, bygger du en stime op.
- **Op til 3 stjerner pr. bane** (løst · uden fejl og tips · desuden under
  måltiden) og **bedste tid** i baneudvalget; **påbegyndte opgaver** gemmes
  automatisk og fortsættes næste gang.
- **4 spiltilstande** (vælges før start) med en pointmultiplikator: **Klassisk**
  (x2.0 - ingen hjælp), **Noter** (x1.5 - + blyantsnoter og automatiske
  kandidater), **Komfort** (x1.0 - + forkerte cifre røde, konflikter og forkerte
  bursummer markeret, korrekte indtastninger låser fast), **Assistent** (x0.7 - +
  tip-tast, maks. 3). Med **3-fejls-grænsen** slået til (setup-mulighed) slutter
  spillet ved den tredje fejl.
- Styring: piletaster/WASD = celle, **1-9** = ciffer (også numpad),
  **0/Delete/højreklik** = slet, **U**/**Z** = fortryd, **Y** = gentag, **N** =
  noter, **C** = udfyld kandidater automatisk, **H** = tip, **M** = farvemarkør,
  **R** = genstart bane, **Q** = baneudvalg. Indtastning »ciffer først« (setup,
  **I**) og en **tæller for manglende cifre** under hvert ciffer; fuldt spilbart
  med musen. Når spillet er slut, viser **A** den fulde **løsning**.
- Point = (grundværdi for variant og sværhedsgrad - tid - fejl - tips) x
  tilstandens multiplikator; alle varianter og dagens sudoku tæller til
  highscoren.

**Frogger**
- 5 trafikbaner (biler/lastbiler) og 5 flodbaner (træstammer, skildpadder der
  **dykker** på højere niveauer); 5 hjemmebugter øverst - fyld alle = næste niveau,
  alt bliver hurtigere.
- Ekstra: **bonusflue** (+200) i tomme bugter, **krokodiller** optager bugter på
  højere niveauer, **tidsgrænse-bjælke** pr. frø, ekstra liv ved 10.000.
- 3 sværhedsgrader (tempo, trafiktæthed, tid); point pr. ny række, bugt = 50 +
  tidsbonus, bane fuldført = +1000.

**Memory**
- Brætstørrelser **4x4, 6x6, 8x6**; motiverne er form-farve-kombinationer tegnet
  udelukkende med primitiver; **vendeanimation**, uens par vender automatisk
  tilbage.
- **Solo**: basis - 15 pr. træk - 2 pr. sekund (min. 100). **Duel** (lokal):
  skiftevis, et par giver endnu et træk, flest par vinder.

**Kabale**
- **5 varianter** på forspils-skærmen: Klondike (træk 1/3 som mulighed), Spider
  (1/2/4 kulører), FreeCell (supermove-grænse), Pyramide (13-par, 2 omdelinger) og
  TriPeaks (±1-kæde med combo-multiplikator).
- **Træk og slip** eller klik-klik, **højreklik** = til fundamentet, **U** =
  ubegrænset fortryd, **R** = ny giv, Mellemrum = talon.
- Kortene gengives uden billedfiler (`games/cards.py`); alle varianter deler én
  highscore-liste med variantspecifikke formler.

**Aim Trainer**
- **Ægte software-3D** (som Snakes 3D-tilstand): fast sigtekorn i skærmens midte,
  **direkte 1:1-musesigte som i en shooter** (pointer-capture: markøren fanges inde
  i vinduet, Esc frigiver den; justerbar følsomhed, ubegrænset yaw, pitch ±60°).
  Venstreklik skyder præcist gennem midten, med mundingsglimt, tracer og
  træfferpartikler.
- **4 tilstande**: præcision (60 s, 3 kugler, nøjagtighedsbonus), refleks (30
  enkeltmål, reaktionstids-statistik), bevægelige mål (baner + combo-multiplikator
  op til x4) og chill (endeløs, ingen straf, **E** afslutter sessionen).
- **3 temaer** (i setup, gemmes): **rummet** med en stjernekugle, et **sort hul med
  en lysende ring** og en planet (standard), en neonarena med gulvgitter og
  synthwave-sol, og en indendørs skydebane.
- Følsomheden kan også ændres midt i spillet med **+/-**; dertil en **justerbar
  motion blur** (0-80 %) for ekstra chill-look - begge gemmes.

**Fire på stribe**
- 7x6-bræt med en **falde-animation**, hover-forhåndsvisning og en pulserende
  vinderlinje; mus, piletaster eller direkte valg **1-7**.
- **3 AI-styrker** (minimax med alfa-beta-søgning): Let overser bevidst trusler,
  Middel blokerer pålideligt, Svær planlægger dybt frem - eller **2 spillere**
  lokalt på samme enhed.
- Startspilleren skifter hver runde; highscoren tæller dine **sejre mod AI'en** i
  én session.

**Tankduel**
- 2D-arenaduel: **skud rikochetterer én gang på væggene** (rikochet) - ram rundt om
  hjørner (eller dig selv!). Først til 5 runder med en nedtælling.
- **4 arenaer** (Åben, Kryds, Søjler, Labyrint) eller tilfældig rotation;
  **power-ups**: hurtigskud, skjold, trippelskud.
- **AI med 3 styrker** - den svære fører sine skud og banker dem bevidst af væggene
  - eller **2 spillere** på ét tastatur (S1 WASD+Mellemrum, S2 piletaster+Enter).

**Blackjack**
- Ægte casinoregler: **4-decks sko**, dealeren står på 17, **blackjack betaler
  3:2**, dealer-peek ved es/10; **double down** og **ét split** (splittede esser
  får ét kort hver).
- **Lama-chips**: Blackjack spiller med kontoen i **Lama-banken**, som deles med
  Poker og Casino (start 1000, gemt permanent i `mem.json`). Indsatsen trækkes,
  så snart der gives; under 10 chips tager Enter et **banklån**, der fylder
  kontoen op til 1000.
- **Highscore** = højeste stand af din egen **Blackjack-balance** (1000 plus alt,
  hvad der er vundet og tabt i Blackjack) - gevinster i roulette, på slotten eller
  i poker tæller ikke her, og det gør lån heller ikke.
- Spilles via chipknapper og taster (**H**it/**S**tand/**D**ouble/split **X**,
  **1-4** = indsats, Backspace = nulstil indsats, Enter = giv) med
  kortanimationer; dealerens hulkort vender nu også rigtigt rundt, når det
  afsløres.

**Tunnel Racer**
- **3D-neon-rørflugt** (softwarerenderer som Aim Traineren): bjælker, klodser og
  **ringporte at træde igennem**, mønter på ideallinjen.
- **To tilstande**: endeløs (tempoet stiger til et loft, highscore) og **30
  seed-genererede baner** med en målstreg, tidsbonus og afkrydset fremskridt.
- **Taststyring** (standard) eller **direkte musestyring** (pointer-capture, tast
  **C**); dertil justerbar **motion blur** (tast **B**, 0-80 %) - alt gemmes.

**3D-labyrint**
- **Førstepersons-raycaster i Wolfenstein-stil** (DDA, afstandståge, sprites) med
  mouselook + WASD, et **minikort** (tast **M**) og en grøn pulserende udgang -
  eller et klassisk **2D-fugleperspektiv** (tast **V** i setup).
- **50 seed-genererede baner**, der bliver ved med at vokse; udgangen ligger altid
  i det punkt, der er længst fra starten, **orbs** undervejs giver bonuspoint.
- Point: 500 pr. bane + 100 pr. orb + tidsbonus; løste baner krydses af, og
  sessionens sum bliver til highscoren.

**Reversi**
- **Othello på 8x8**: læg brikker, der indfanger modstanderens rækker, og vend alt
  det indelukkede; ulovlige træk er spærret, og en tur uden lovligt træk **passes
  automatisk**.
- **Én spiller mod AI'en** (3 styrker: negamax med alfa-beta, positionsvægtning +
  mobilitet) **eller en lokal duel**, Sort mod Hvid.
- Lovlige felter fremhæves; spil med **musen** eller markeringsrammen (piletaster +
  Mellemrum/Enter). Hver sejr mod AI'en tæller ét point til highscoren.

**Yatzy**
- **Terningklassiker**: 5 terninger, op til 3 kast pr. tur, **hold** terninger
  enkeltvis, bogfør derefter en af de **13 kategorier** (med en live-forhåndsvisning
  af de mulige point).
- Fuldt scoreark: øverste sektion med **63-points-bonus (+35)**, tre/fire ens,
  fuldt hus, lille/stor straight, **Yatzy (50)** og Chance.
- **Én spiller som highscore-jagt** efter den højeste slutsum, eller
  **2-spiller-hotseat** med to ark side om side; spil med mus eller taster
  (Mellemrum, 1-5, piletaster, Enter).

**Wordle**
- Gæt det skjulte ord; farvet feedback (grøn/gul/grå) med korrekt **optælling af
  dobbeltbogstaver** og et skærmtastatur, der farves (QWERTZ til tysk, tjekkisk,
  slovensk og kroatisk, AZERTY til fransk, ellers QWERTY).
- **Fire tilstande**: *Uendelig* (det ene ord efter det andet med 6 forsøg hver;
  hvert løst ord giver point, det første uløste afslutter spillet), *Dagens ord*
  (ét ord om dagen pr. sprog og længde - ens på PC og i browseren - med nedtælling
  og stime; et påbegyndt dagens ord gemmes), *Dordle* (2 ord på én gang på 7
  forsøg) og *Quordle* (4 ord på 9 forsøg, tasterne viser farverne fra alle
  plader).
- **Opsætning** før hvert spil: **ordlængde 4 til 7**, **svær tilstand** (fundne
  spor skal bruges igen) og **farveblind-palet** (orange/blå); ved siden af står
  statistikken.
- **Rigtige ordlister på alle 14 sprog** (mappen `woordlistz/`, kun A-Z), med egne
  lister for hver længde: alene ved 5 bogstaver knap **34.000 løsninger** og over
  **213.000 tilladte gæt**, omkring 134.000 løsninger på tværs af alle fire
  længder. Løsningerne er almindelige ord uden navne, engelske rester og stødende
  ord; hvert gæt tjekkes mod listen - alt andet afvises, og rækken ryster kort.
- **Statistik** pr. sprog, længde og tilstand: spil, vinderprocent, aktuel og
  bedste stime og **fordelingen af forsøg som søjlediagram** (afsnittet `wordle`
  i `mem.json`). **Del** (**C**) kopierer et emoji-gitter uden at afsløre ordet.
- Highscoren tæller kun *Uendelig* med 5 bogstaver; de andre længder har egne
  bedste resultater. Præstationerne **Synsk** (højst 2 forsøg), **Ordvane** (7
  dagens ord i træk) og **Firdobbelt geni** (Quordle løst).

**Poker**
- **3 varianter** på forspils-skærmen: **Texas Hold'em** mod 1-3 AI-modstandere
  med dealerknap, blinds og fire budrunder, **5 Card Draw** (heads-up mod AI'en,
  én bytterunde) og **Video Poker** (*Jacks or Better*, solo mod
  gevinsttabellen).
- Handlinger via knapper eller taster: **F** = fold, **C** = check/call, **R** =
  raise, **A** = all-in; hold/byt kort med klik eller **1-5**, **Enter** trækker
  eller giver næste hånd.
- **Lama-chips** fra den fælles **Lama-bank**: ved starten af en hånd ligger din
  konto på bordet som stak, og det, der går i potten, trækkes med det samme -
  forlader du bordet midt i en hånd, mister du kun din andel af potten. Pengene
  opbrugt (under big blind på 20, under 10 i Video Poker) = banklån op til 1000.
- **Highscore** = højeste stand af din **Poker-balance** (1000 plus alle
  gevinster og tab i poker); præstationen **Chipleader** tæller også kun
  pokerbalancen.

**Skak**
- **Fuldstændigt skak**: alle brikkers træk inkl. **rokade**, **en passant** og
  **bondeforvandling** (vælg brikken); **skak, skakmat og pat** samt remis ved
  **50-træks-reglen**, **trefoldig stillingsgentagelse**, **utilstrækkeligt
  materiale** eller aftale.
- **Tre tilstande**: *parti* mod AI'en, *2 spillere* ved samme computer
  (brættet kan dreje efter hvert træk) og **opgaver**.
- **Stærkere AI uden hak** i 6 niveauer fra *Begynder* til *Mester*: iterativ
  uddybning, transpositionstabel, quiescence-søgning, åbningsbog og en vurdering
  med mobilitet, bondestruktur og kongesikkerhed. AI'en regner i små bidder pr.
  billede - spillet hakker aldrig.
- **Opsætning**: farvevalg, **skakur** (intet, 1+0, 3+2, 5+0, 10+5) og
  **Chess960** (alle 960 startopstillinger, nummeret står over træklisten).
- **Sidepanel** med ure, slåede brikker, materialebalance og en rullebar
  **trækliste (SAN)**; træk og slip, glidende brikker, koordinater. Taster: **U**
  = fortryd, **H** = hintpil, **O** = tilbyd remis, **X** = opgiv, **F** = vend
  brættet, efter partiet **P** = **PGN-eksport**.
- **Opgaver**: 200 opgaver i 5 trin (mat i 1/2/3, taktik I/II) fra **Lichess'
  frie opgavedatabase (CC0)**, kontrolleret med spillets egen motor; i
  matopgaver tæller ethvert træk, der sætter mat. Fremskridtet ligger i afsnittet
  `chess` i `mem.json`.
- Fortryd og hint gør et parti »assisteret«: highscoren tæller kun sejre uden
  hjælp mod AI'en (pr. session).

**Mølle**
- **Mølle** med alle tre faser: **at sætte** (9 brikker hver), **at flytte** langs
  linjerne og **at flyve**, når man er nede på 3 brikker (kan slås fra).
- En færdig **mølle** fjerner en modstanderbrik (helst en uden for en mølle); man
  taber, når man er reduceret til under 3 brikker eller ikke kan trække.
- **3 AI-styrker** (minimax med alfa-beta, fase-bevidst evaluering) eller en
  **lokal duel**; med trækhint, mølle-fremhævning og en briktæller.

**Simon**
- **Senso-huskespil**: den oplyste sekvens vokser hver runde og skal gentages
  præcist.
- **Tilstande**: *Klassisk*, *Speed* (bliver hurtigere), *Reverse* (baglæns),
  *Blandet* (tilstanden roterer hver runde) og en to-spiller-**Duel** (skiftes til
  at tilføje og gentage).
- **Lyd** *fra / til / blandet* (blandet træner både din visuelle OG auditive
  hukommelse), **4/6/9 felter** som sværhedsgrad; den **bedste score pr. tilstand**
  gemmes. Spil med musen eller taltasterne 1-9.

**Billard**
- **8-ball**, **9-ball** og en regelfri **øvelses**tilstand, mod AI'en (med
  sigtehjælp) eller **to spillere lokalt**.
- **Tre frit valgbare visninger**: klassisk **2D-fugleperspektiv**, et fast
  **3D-skråperspektiv** med skyggelagte kugler og et **frit roterbart 3D-kamera**
  (højre museknap). Al bevægelse er tidsskridt-baseret og **blødt dæmpet**
  (friktion, delskridt for at undgå gennemtunnelering).
- **At støde**: hold venstre museknap for at lade kraften op, slip for at støde; en
  sigtelinje og en kraftmåler hjælper. **Bold i hånden** efter en fejl. Visningen
  (V) huskes i `settings.json`; vundne frames tæller med til highscoren.

**Skydepuslespil**
- 15-spillet i tre størrelser: **3×3** (let), **4×4** (klassisk) og **5×5** (svær);
  skub de nummererede brikker ind i det frie hul.
- Altid løsbart (blandet med mange tilfældige træk). Styr ved at **klikke** på en
  brik i hullets række/kolonne (hele linjen glider) eller med **piletasterne**.
- Point = en grundværdi pr. størrelse minus træk og tid; når det er løst, starter
  et nyt bræt med det samme.

**Mastermind**
- Knæk den skjulte **farvekode**; efter hvert gæt får du **sorte** pinde (rigtig
  farve + position) og **hvide** pinde (rigtig farve, forkert position).
- **3 tilstande**: Let (4 pinde / 6 farver / 12 rækker), Klassisk (4/6/10) og Svær
  (5 pinde / 8 farver); gentagne farver er tilladt.
- Spil via farvepaletten (klik eller taster **1–8**), OK/Enter tjekker rækken.
  **Endeløs streak** som i Wordle: hver knækket kode giver point.

**Bubble Shooter**
- **Puzzle Bobble** på et bikubegitter: sigt med musen, skyd bobler opad, **tre
  eller flere af samme farve** springer gruppen.
- Bobler, der mister forbindelsen til loftet, **falder** (bonus); skud **afspringer
  på væggene**, med en forhåndsvisning af næste boble.
- **3 tilstande** (4/5/6 farver, nogle med nedrykkende rækker); game over ved den
  røde linje.

**Galgemand**
- Gæt ordet **bogstav for bogstav**; hver fejl tegner en del af galgemanden, tabt
  efter **6 fejl**.
- **Ordlister pr. sprog** (kun A–Z), **3 længdetilstande** (kort / blandet / lang);
  skriv eller klik på skærmtastaturet.
- **Endeløs streak**: hvert gættet ord giver point (flere resterende liv + længere
  ord = flere).

**Block Jump**
- **3D-platformspil i Minecraft-stil** (software-3D som Snakes 3D-tilstand): hop hen
  over en svævende **voxelverden** af klodser til det glødende mål.
- **Minecraft-skin**: alle blokke har ægte **pixeltexturer** (græs, jord, sten,
  planker, diamant, slim, træ); detaljegraden følger afstanden
  (**T** = høj/lav/fra).
- Desuden: **Steve**-figur med ganganimation (tredjepersonskamera), **hånd** i
  førsteperson, **beacon-stråle** ved målet, roterende **guldbarrer** som
  mønter, firkantet **sol**, **pixelskyer** og et HUD med **hjerter**.
- Klodstyper: faste klodser (græs/jord/sten/træ), **stiger** (klatre), **hegn**
  (springe over), **fjederblokke** (kaste op) og **mønter**.
- Kamera **førsteperson som Minecraft som standard**, **V** skifter til et
  forfølgerkamera; **mouse look** med pointer-capture, justerbar **motion blur**
  (**B**) og følsomhed (**+/-**).
- **Seed-genererede parkourbaner** bliver sværere; mål = point + tidsbonus, mønter
  +50, et fald koster et liv (start med 3). Styring: WASD/piletaster, **Mellemrum**
  for at hoppe.

**Tower Defense**
- **Endeløst bølgeforsvar** på **4 kort** (Eng, Kløft, Korsvej, Spidsrod), hvert
  med sin egen sti; låste kort låses op med din bedste bølge, en **boss**
  ankommer hver **8. bølge**.
- **3 tilstande**: Klassisk (7 tårne, hovedtilstanden), Kompakt (4 tårne,
  2 niveauer) og Maksimal (**11 tårne**, **A/B-specialisering** på højeste
  niveau, specialfjender, aktive evner **Meteor/Frostnova/Guldfeber**).
- **11 tårntyper** fra pil til laser og guldbank, hver med op til
  **3 opgraderingsniveauer**, salg refunderer 70%; fjender med panser,
  regeneration, deling, camouflage, healingaura og luftrute.
- **Økonomi**: guld pr. nedskydning, bølgebonus + 5% renter; point pr.
  nedskydning og bølge. **F** = 2x tempo, **G** = rækkevidder, højreklik
  annullerer.

**Minigolf**
- **360 baner på 40 forløb**: *Classic* og *Pro* med 9 håndbyggede baner hver,
  **Touren** med 38 forløb à 9 genererede baner (342 i alt) og stigende
  sværhedsgrad, dertil *Random* fra det hele. Forløb 7, bane 3 ser ens ud alle
  steder - der skal ikke gemmes noget.
- **Underlag og forhindringer**: sand bremser, ramper accelererer, vand koster et
  strafslag, gummibumpere giver fart tilbage, og vindmøller og vandrende klodser
  kræver timing. Fysikken kører i deltrin med friktion som i billard - intet
  hakker, og intet smutter gennem banden.
- **Styring**: musen sigter, venstre museknap holdt nede lader kraften, og et
  slip slår (pile + mellemrum virker også). **R** afbryder et ladet slag uden
  at slå. **G** slår sigtelinjen til og fra, **Z** auto-sigtet, **P** Saml op.
- **Kraftlås (hold højre museknap)**: fryser ladebjælken præcis der, hvor den
  er - gylden, med procenten, en hængelås og en pulserende ring om bolden. Så
  venter du på hullet i møllen med slaget færdigladet. Slipper du, lades der
  videre; en låst kraft overlever endda slaget, og næste venstreklik slår med
  præcis den værdi.
- **Scorekort** til højre med par og slag pr. bane; til to spiller begge den samme
  bane efter tur. Point: 600 pr. bane, ±300 pr. slag under/over par, **500 ekstra
  for hole in one**. Det laveste antal slag pr. forløb ligger i afsnittet
  `minigolf` i `mem.json`.
- **Saml op kan slås fra**: som standard slutter en bane efter otte slag og
  tæller minimum. Vil du hellere spille videre, til bolden er i hul, så sæt
  *Saml op* på FRA i opsætningen (eller tryk **P**).
- **Auto-sigte kan slås fra**: som standard drejer køllen af sig selv mod
  hullet før hvert slag. Vil du hellere sigte selv på hver bane, så sæt
  *Auto-sigte* på FRA i opsætningen (eller tryk **Z**) - så bliver den senest
  valgte retning stående, og på en ny banes tee peger køllen neutralt opad.
- **F** nulstiller det igangværende hul: slag på 0, bolden på teestedet - samme
  hul, samme bane.
- **Videre i stedet for gentagelse**: når runden er slut, fører knappen **Videre**
  til den næste bane (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), så de samme ni
  huller aldrig gentages; ved siden af: **Igen** (samme bane) og **Opsætning**.
  Taster: Enter = videre, R = igen, S = opsætning.
- **Replay af runden**: til sidst viser **P** (eller knappen **Replay**) hele
  runden slag for slag. Med **S** ryger den i arkivet (sidebjælkeknappen
  **Replays**).
- **Byg og del dine egne huller**: fanen **MAPS** på opsætningsskærmen fører
  til din egen samling - **Ny** åbner huleditoren. Hvert hul får et navn og et
  **id** (små bogstaver, ingen mellemrum); id'et er samtidig det filnavn, der
  foreslås ved deling. Til de syv klassiske forhindringer kommer **otte nye**:
  rør (flytter bolden til den anden ende), is, klæbefelt, booster, magnet,
  ensrettet port, drejeskive og springrampe. Hullets størrelse kan indstilles
  frit (60x80 til 160x240), **12 skabeloner** giver et udgangspunkt, og
  fortryd/gendan samt **Test** er med. **Del** skriver præcis ét hul som en
  `.lamapgzmap`-fil - via gemme-dialogen eller direkte i Overførsler-mappen,
  med dit navn som skaber. **Importér** læser den ind igen og skifter
  automatisk til `-2`, hvis id'et er optaget. Navn, id og skaber løber altid
  gennem et **ordfilter på tværs af alle 14 sprog**. Et hul spilles alene via
  **Spil**, eller hele samlingen via det femte banevalg **Egne**.

**Pinball**
- **Tre borde**: *Classic* (tre bumpere, én målrække), *Space* (fire bumpere i
  rombe, to rækker) og *Lama* (åbent felt, seks mål i en bue); 3 eller 5 kugler
  pr. spil, til to skiftevis kugle for kugle.
- **Alt hvad en flipper skal have**: udskyderbane med ladebjælke (for svagt? så
  ruller kuglen tilbage, og du må prøve igen), to flippere, slingshots, målrækker,
  fire **L-A-M-A**-baner, hul med kuglelås, **multiball med jackpot**, seks
  sekunders **kugleredning**, puf og **TILT**.
- **Multiplikator op til x5** via ryddede rækker og komplette baner; bumpere 100,
  slingshots 50, mål 250 - under multiball betaler bumperne 2.500 i jackpot.
- Flipperne kører på de tildelte venstre/højre-taster (samt venstre/højre [Shift])
  eller musen. Rekorden pr. bord ligger i afsnittet `pinball` i `mem.json`.

**Bowling**
- **Ti frames efter de officielle regler** inklusive strikes, spares og tiende
  frames bonuskast (maksimum: 300). **Scorekortet** under overskriften viser hver
  frame med X, / og løbende sum.
- **Kast i fire trin**: position, vinkel, skrue og kraft. Hver skyder svinger af
  sig selv og låses med handlingstasten - eller indstilles i hånden med
  venstre/højre, hvilket standser svingningen.
- **Ægte keglefysik**: ti kegler som cirkler med masse, der vælter hinanden; en
  strike kommer af fysik og ikke af held. Banen er olieret forrest, så **hooken**
  først griber i den sidste tredjedel.
- Baneview i perspektiv med render, sigtepile og keglefelt; tre sværhedsgrader
  (*Let/Normal/Pro*) ændrer skydernes tempo og spredningen. Rekorden pr.
  sværhedsgrad ligger i afsnittet `bowling` i `mem.json`.
- **Replay af partiet**: til sidst viser **P** alle kast igen, og **S** gemmer
  dem i arkivet (knappen **Replays**).

**Crossy Road**
- **Endeløse hop** over enge (træer og sten spærrer vejen), veje med biler og
  lastbiler, floder med træstammer og åkander og **togskinner**, hvor et tog
  suser forbi efter advarselslys og klokke - længere fremme venter hele
  stationer med op til 5 spor. Ruten bygges række for række, har altid en
  farbar vej, og både tempo og trafik stiger.
- **Isometrisk voxel-look**: figurer, køretøjer og træer af skyggelagte klodser
  (forudrenderet til hver feltstørrelse), et blødt følgende kamera, squash &
  stretch ved hop, vandplask, en fladtrykningsanimation, fjer og funklende
  mønter; fra række 50 **dag/nat-skift** med forlygter.
- **Ørnen**: kameraet kryber fremad - nøler du for længe eller går mere end tre
  rækker tilbage, tager ørnen dig (en rød kant advarer først). At drive ud af
  billedet på en træstamme er også slut.
- **Mønter og figurer**: indsamlede mønter (kæmpemønt = 5) gemmes og køber nye
  figurer under fanen **Figurer**: frø, gris, pingvin, kat, ræv, lama, robot,
  spøgelse og enhjørning (25 til 250 mønter); kyllingen er med fra start.
- **Tilstande**: *Uendelig* (point = længste række, tæller til highscoren) og
  *Dagens rute* (ens for alle i dag, også i browseren, med egen dagsrekord).
  Styring: piletaster/WASD, mellemrum/Enter/klik = hop frem; i opsætningen **H** =
  skygger, **N** = dag/nat. Mønter, figurer og dagsrekord ligger i afsnittet
  `crossy` i `mem.json`.

**Geometry Dash**
- **Rytme-platformspil**: figuren suser selv mod højre - du bestemmer kun, hvornår
  der hoppes eller flyves. **Fem former** - terning, skib, bold, UFO og bølge -
  plus form-, tyngdekraft- og fartportaler (0,5x til 3x), gule/lyserøde/blå
  **pads og kugler**, halvblokke, pigge, gruber og farvetriggere.
- **8 indbyggede baner** fra *Let* til *Dæmon* (»Lama Inferno«) med **3
  hemmelige mønter** hver. Hver bane kan beviseligt klares: da den blev bygget,
  løste en løsningsalgoritme den med spillets rigtige kode - inklusive alle mønter
  og endda forskudt med 1/240 sekund.
- **Præcis fysik**: fastkommaberegning med fast 240 Hz-skridt; hvert tryk virker
  præcis i det skridt, hvor det skete - ens ved enhver billedhastighed og
  bit-identisk i browseren.
- **Træningstilstand** (**P**) med automatiske og egne checkpoints (**Z** sætter,
  **X** sletter), forsøgstæller, fremskridtsbjælke, eksplosioner og øjeblikkelig
  genstart (**R**). Hver bane har sit **eget soundtrack** - baggrund, underlag og
  kugler pulserer i takt (musikken slås fra med **M**).
- **Stjerner og mønter**: klarer du en bane i normal tilstand, får du dens
  stjerner, og hver mønt er en ekstra stjerne værd; highscoren er det **samlede
  antal stjerner** (højst 65). Bedste resultater pr. bane, mønter, forsøg og hop
  ligger i afsnittet `geodash` i `mem.json`.
- **Baneeditor** under fanen **BANER**: lærred med gitter, palet med 6 grupper
  (blokke, farer, pads og kugler, portaler, fart, ekstra), drejning,
  fortryd/gentag, oversigtsbjælke, **test fra start eller herfra** og
  baneindstillinger (startfart, startform, musikstil, BPM, farver). Fluebenet
  **»verificeret«** kommer først, når du selv har klaret din bane. **Del**
  skriver en `.lamapgzlevel`-fil, **Import** læser den ind igen; banerne gemmes i
  `ugc.json` ved siden af dine egne minigolfbaner.

**Battleship**
- **Søslag på 10x10** med hangarskib (5 felter), slagskib (4), krydser (3),
  ubåd (3) og destroyer (2) - den, der først sænker hele fjendens flåde, vinder.
- **Placér flåden** med træk og slip fra dokken: **R** eller højreklik drejer,
  forhåndsvisningen lyser grønt eller rødt, **X** placerer alt tilfældigt, **C**
  rydder brættet; din sidste opstilling foreslås igen.
- **Regler i opsætningen** (gemmes): *skibe må røre hinanden*, *salve* (lige så
  mange skud pr. tur, som du har skibe flydende) og *skyd igen efter et træf*.
- **AI med 3 styrker**: Let skyder tilfældigt, Mellem forfølger træffere
  systematisk, Svær beregner et **sandsynlighedskort** med skakbrætsparitet (i
  snit ca. 70 / 60 / 45 skud for en hel flåde). Eller **2 spillere** ved samme
  computer - en **overdragelsesskærm** skjuler begge flåder før hver tur.
- **Grafik**: radar-sweep, animerede bølger, granater i bue, plask, eksplosioner
  med røg og brændende felter, en »SÆNKET!«-afsløring og en rundeoversigt med
  skud, træffere og træfprocent. Highscoren tæller dine **sejre mod AI'en** i en
  session.

**Casino**
- **Roulette** (europæisk, 37 felter): alle klassiske indsatser ved klik på et
  tal, en kant eller et hjørne - **plein** (35:1), cheval, transversale, carré,
  sixain, kolonne, dusin, rød/sort, lige/ulige og manque/passe. Chipværdier
  1/5/25/100/500, højreklik fjerner chips; **Drej**, **Gentag** (**R**),
  **Fordobl** (**D**) og **Ryd**. Kuglen løber i spiral ned i det felt, der er
  trukket på forhånd, og øverst står de sidste 12 tal.
- **Lama-slot**: 5 hjul x 3 rækker, **10 gevinstlinjer**, **lama = wild**,
  **guldmønter = scatter** med 10 gratisspil og dobbelte gevinster, indsats pr.
  linje 1/2/5/10, **auto-spin** (10/25), **turbo** og gevinsttabel.
  **Tilbagebetalingsprocenten er 96,1 %** - beregnet nøjagtigt ud fra hjulstrimlerne.
- **Lama-banken**: Casino, Blackjack og Poker deler én konto med **lama-chips**
  (start 1000, afsnittet `casino` i `mem.json`); gamle chipsaldi overføres
  automatisk. Indsatser trækkes med det samme, hvert spil fører sin egen balance
  til highscoren, og går du fallit, får du et **banklån** op til 1000.
- Konfetti, mønteregn, big/mega/jackpot-bannere og gevinstlinje-animationer;
  præstationerne **Plet** (vundet plein i roulette) og **Lama-jackpot** (5 lamaer
  på én linje).

Highscores gemmes i afsnittet `highscores` i `mem.json` (ved siden af koden) –
sammen med sproget (afsnittet `mem`).

### Grænsefladen

Hele grænsefladen er tegnet fra bunden (ren Tkinter + Pygame, ingen ekstra pakker)
og stylet som en moderne spil-launcher:

- **Sidebjælke med spilliste**: hver række har sit eget **mini-piktogram** i
  spillets accentfarve, viser den aktuelle **highscore (★)** og reagerer med blødt
  animerede hover-effekter. Det kørende spil forbliver fremhævet; i små vinduer
  **ruller** listen med musehjulet.
- **Statuskort** nederst til venstre med en **tilstands-LED** (grå = menu, grøn =
  kører, guld = pause, rød = game over) og en **live-FPS-visning**.
- **Startskærm** med aurora-lys, et parallakse-stjernefelt inkl. stjerneskud, et
  svævende logo med kredsende gnister, et **klikbart spilgitter** lige under logoet
  (alle spil med en hover-effekt i deres accentfarve) og en **highscore-ticker**.
- **Effekter overalt**: bløde skærmovergange, gnister når man bekræfter et
  menupunkt, **konfettiregn ved en ny highscore** og en ægte **sløring** bag
  pause-overlayet.
- Hvert spils **forspils-skærm** vises i det pågældende spils accentfarve og viser
  den tidligere rekord som en chip. Ved mange tilstande og lav opløsning bliver den
  **kompakt**: Indstillinger, Wiki og Tilbage rykker op på én række, og skriften
  tilpasser sig - intet løber længere ud af billedet.
- **Ensartet look i spillet**: alle 46 spil deler menuens temapalet og skrifttype -
  HUD'er, setup-skærme og overlays følger det design, der er valgt i indstillingerne
  (v4.1 / v4 / Klassisk), mens hvert spillefelt beholder sine identitetsfarver.
  Hvert spil håndterer nu en opløsningsændring midt i spillet rent, og menunavnene
  er sprogafhængige (f.eks. »Schach« → »Skak«).
- **Indbygget wiki** ("LamaWiki"): detaljeret hjælp til hvert spil (styring,
  tilstande, point, tips) plus generelle sider - med et **søgefelt**, kategorier,
  rulbare artikler og tastekap-chips, på alle 14 sprog. Nås via sidebjælkeknappen
  **»Wiki / Hjælp«** og fra hvert spils forspils-skærm (åbner det pågældende spils
  side direkte).
- **Præstationer & statistik**: **107 præstationer** i tre kategorier (23 mål på
  tværs af samlingen, 37 point-milepæle og 47 særlige øjeblikke som skakmat mod
  AI'en, 4096-brikken, en T-Spin Double, 25 løste skakopgaver, en Killer Sudoku
  eller lama-jackpotten; i 2048 og skak tæller partier med fortryd eller hint
  ikke) med **gylden notifikation og fanfare**
  ved oplåsning - selv midt i spillet; gamle rekorder godskrives automatisk.
  Dertil en **statistik**-fane: samlet spilletid, partier, sejre, rekorder,
  yndlingsspil og en tabel pr. spil sorteret efter spilletid. Nås via knappen
  **"Præstationer & statistik"** i sidepanelet.
- **Replays**: minigolf og bowling optager hver runde. Til sidst viser **P**
  gengivelsen, og **S** lægger den i arkivet - tilgængeligt via sidebjælkens
  knap **Replays** (en fane pr. spil, pause, spring mellem sekvenser, tempo
  0,5x til 4x). Kan slås fra ved første start og i indstillingerne.

### Betjening

- Vælg et spil via knappen i menuen til venstre. Derefter dukker en
  **forspils-skærm** op: vælg **Enkeltspiller** eller **Multiplayer**, gå til
  **indstillingerne** eller tilbage. Piletaster/mus for at vælge, Enter for at
  starte.
- **ESC** = pause / fortsæt (i menuer: tilbage).
- **F11** (eller knappen »Fuldskærm til/fra«) = slå fuldskærm til/fra.
  Pygame-displayet forbliver indlejret og skaleres op med bevaret
  højde-bredde-forhold (sorte bjælker, når forholdet afviger). Vinduet kan frit
  ændre størrelse.
- **»Tilbage til menu«** afslutter spillet og gemmer highscoren - det samme gør
  et skift til et andet spil via sidebjælken.
- **Faste ekstrataster**: ud over de fem handlinger, der kan bindes, har nogle
  spil deres egne taster (f.eks. reserve **C** og drej til venstre **Z** i
  Tetris, fortryd **U** i 2048, skak og sudoku). De virker kun, hvis tasten ikke
  er bundet til en handling i indstillingerne, og står i opsætningshintet og i
  wikien. Holdte taster registreres rent og slippes ved pause eller Alt-Tab -
  intet »hænger« længere.
- **»Afslut«** lukker Pygame og Tkinter rent.

### Indstillinger, styring og lyd

Indstillingsskærmen åbnes via knappen **»Indstillinger / Styring«** (til venstre)
eller fra forspils-skærmen. Den er organiseret i **tre faner** (**Generelt /
Styring / Udseende**; skift ved at klikke eller med Tab-tasten):

- **Generelt**: **lyd** til/fra, **lydstyrke** og **haptik** (gamepad-vibration,
  kun effektiv med en tilsluttet controller) samt **auto-opløsning**,
  **opløsning**, **FPS** og **sprog** – hver skiftes med Venstre/Højre.
- **Styring**: **skabeloner** (*WASD + Piletaster*, *WASD + IJKL*, *Piletaster +
  WASD*) og **omdefiner hver enkelt tast** for spiller 1 og spiller 2: vælg en
  række, tryk Enter, tryk på den ønskede tast (Esc annullerer).
- **Udseende**: vælg **UI-designet** – **UI v4.2** (standard: Midnight Glass – en
  dyb midnatsgradient med langsomt drivende bløde lys i indigo, turkis og magenta,
  fint filmkorn, spredte stjerner og paneler som mat glas med en lys kant),
  **UI v4.1** (som UI v4, men mere levende – diskrete stjerner plus Saturn og et
  sort hul i startskærmens baggrund), **UI v4.1.1** (som v4.1, men et fliselagt
  **zigzagmønster** i sort og antracit i stedet for stjernehimlen), **UI v4.1.2**
  (samme mønster i palettens blå toner – accentblå som dominerende farve, en
  mørkere blå som bund), **UI v4.1.3** (samme mønster i UI v4's indigo på sort),
  **UI v4.1.4** (i UI v4's grafittone på sort), **UI v4** (et fuldstændig roligt,
  fladt grafit-look med en enkelt indigo-accent), **UI v3** (den tidligere
  klassiske UI med stjernehimmel, aurora-lys og glød-effekter), **UI v2** (den
  allerførste UI-fornyelse: marineblå gradient, stjernehimmel og glødende knapper,
  helt uden animationer) eller **UI v1** (looket før UI-fornyelsen: ensfarvet mørk
  baggrund, flade knapper, ingen effekter). Alle kort viser en lille
  forhåndsvisning; valget slår straks igennem på hele grænsefladen (spilområde
  **og** sidebjælke) og gemmes.

Indstillingerne gemmes permanent i `settings.json`. I **enkeltspiller** styrer
begge tildelinger den samme figur (standard: WASD *og* piletaster), i
**multiplayer** én hver. Alle spil har **lydeffekter** (proceduralt genereret,
ingen ekstra filer nødvendige), som kan slås fra globalt.

### Projektstruktur

```
install-python.bat  Windows-opsætning: Python 3.13 + .venv + pygame
start.bat            Startscript (Windows)
start.sh             Startscript (Linux / macOS / Git Bash)
pyinstall.bat        EXE-build (Windows): pakker alt ind i builds\PyGameZ.exe
main.py              Tkinter-UI, Pygame-indlejring, central spil-loop
game_base.py         Spil-basisklasse (update/draw/handle_event) + InputEvent + hjælpere
settings.py          Indlæser/gemmer indstillinger (lyd/haptik/tastebindinger/spilindstillinger med kontrolregler) (JSON)
audio.py             Procedurale lydeffekter, musikløkker + gamepad-rumble
menu.py              Sprog-, forspils- (tilstand) og indstillingsskærm (lyd/styring)
highscore.py         Indlæs/gem highscores (afsnit i mem.json)
store.py             Central gemmefil mem.json (afsnit: mem, highscores, stats, achievements + spilfremskridt), atomisk med .bak-kopi
stats.py             Spillerstatistik (partier, spilletid, sejre, rekorder) pr. spil
achievements.py      Præstationer: definitioner, oplåsning, notifikation (toast)
progress.py          Skærm for præstationer & statistik (to faner, rulbar)
replay.py            Optagelse og arkiv over replays (replay.json)
replayview.py        Replay-skærm: arkivliste og afspilning
ugc.py               Eget indhold (minigolfbaner, Geometry Dash-baner): lagring, kontrol, eksport/import (ugc.json)
swear.py             Ordfilter til navne og id'er (lang/swear/*.yml, alle 14 sprog)
filepick.py          Fildialoger ("Eksportér som ...", "Importér")
prestige.py          Prestige-system til Snake
competitive.py       Finindstilling til Snakes Competitive-tilstand (niveauer, enarmet tyveknægt, gamble-æbler)
ngb.py               Visuel tilpasning ("mods"): hovedfarve + koordinatgitter + menu (mem-ngb.json)
lamabank.py          Lama-banken: fælles chipkonto for Blackjack, Poker og Casino (afsnittet casino i mem.json)
seedrand.py          Tilfældighedsgenerator med bit-identiske tal i Python og browseren (dagens tilstande, nye opgaver)
i18n.py              Oversættelsesmotor (indlæser lang/*.json, t("nøgle"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Sprogstrenge (én pladsholdernøgle pr. tekst)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Ordfilter-lister pr. sprog (regex, .yml)
lamawiki/
  lamawiki.py          Indbygget wiki (søgning, kategorier, artikel-renderer)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-indhold (én side pr. spil + generelle sider)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Bygger Wordles ordlister igen (ordbøger + frekvenslister)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 bogstaver), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 bogstaver), 14 sprog
devtools/            Udviklerværktøjer (pakkes ikke med i .exe'en)
  merge_staging.py           Indsætter oversættelser og wikisider fra devtools/staging/ i alle 14 sprogfiler
  build_chess_puzzles.py     Bygger de 200 skakopgaver ud fra Lichess' opgavedatabase (CC0)
  build_sudoku_killer.py     Genererer de 400 entydigt løsbare Killer Sudokuer
  build_crossyroad_models.py Skriver voxel-modellerne til Crossy Road til webversionen
  build_geodash_levels.py    Bygger de 8 Geometry Dash-baner og beviser med løsningsalgoritmen, at hver kan klares inkl. mønter
  build_geodash_solver.py    Løsningsalgoritme med spillets rigtige skridt-kode (løsninger i geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Banedata: snake-comp.json, chess-puzzles.json (+ kilde-README), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Samlet audit (input, seedrand Python = JS, gemning, sprogfiler, forspils-skærme) + alle audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless-audits pr. spil
  newgames_audit.py  blockjump_audit.py
```

Det valgte sprog gemmes i `mem.json` (i afsnittet `mem`, ved siden af afsnittet
`highscores` i den samme fil) og indlæses automatisk ved næste start.

**Kilder og licenser:** de 200 skakopgaver stammer fra
[Lichess' opgavedatabase](https://database.lichess.org/#puzzles) (licens
**CC0 1.0**, public domain - tak til lichess.org!); detaljer står i
`games/levels/chess-puzzles.README.md`. Kilderne til Wordles ordlister står i
`woordlistz/README.md`.

### Platformsnoter

Displayet kører **off-screen**: pygame bruger dummy-videodriveren
(`SDL_VIDEODRIVER=dummy`), så det renderer ind i en surface, og hvert frame tegnes
som et billede ind i et Tkinter-widget. Der er **intet nativt SDL-vindue**, der
kunne kæmpe med Tkinter om størrelse/position. Som følge heraf opfører vinduet sig
ens og stabilt overalt:

- **Windows**: processen gøres desuden DPI-aware, så displayet forbliver skarpt på
  skalerede skærme (125/150/200 %) og ikke "ryster".
- **Linux/X11 og Wayland**: fungerer uden specialtilfælde (ingen `SDL_WINDOWID`).
- **macOS**: fungerer også (tidligere blev det indlejrede vindue slet ikke vist her).

---

### Installationsguide

Krav: **Python 3.9+** (anbefalet 3.12 eller 3.13) og **pygame ≥ 2.6**.

#### Windows (anbefalet: automatisk)

1. Åbn projektmappen og dobbeltklik på **`install-python.bat`**. Scriptet
   - tjekker, om **Python 3.13** er til stede, og installerer det ellers via
     **winget** (`winget install Python.Python.3.13`),
   - opretter det virtuelle miljø **`.venv`**,
   - installerer **pygame** fra `requirements.txt`.
2. Start derefter samlingen med **`start.bat`** (dobbeltklik).

> Bemærk: hvis scriptet melder "endnu ikke tilgængelig i dette vindue", blev Python
> netop installeret – åbn blot **et nyt terminal-/vindue** og kør
> `install-python.bat` igen. Hvis **winget** ikke er tilgængelig, så installér
> Python 3.13 manuelt fra <https://www.python.org/downloads/> og sæt flueben ved
> **"Add python.exe to PATH"**.

#### Windows / Linux / macOS (manuelt)

```bash
# 1. Tjek Python (3.9+)
python --version

# 2. Opret og aktivér et virtuelt miljø
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Installér afhængigheder
pip install -r requirements.txt
#   eller:  pip install "pygame>=2.6" (eller pygame-ce)
#                                   pip install pygame-ce
# 4. Start
python main.py
```

#### Linux / macOS med start.sh

```bash
# Opsæt Python + venv som ovenfor (trin 2 og 3), derefter:
chmod +x start.sh      # én gang, hvis ikke allerede eksekverbar
./start.sh
```

På Linux installeres Python om nødvendigt via pakkehåndteringen, f.eks.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); på macOS
f.eks. `brew install python`.

#### Brug en anden Python-version

`install-python.bat` opsætter som standard Python 3.13. Foretrækker du 3.12 (eller
en anden version), så ændr linjen `set "PYVER=3.13"` i filen til den ønskede
version og winget-id'et tilsvarende (`Python.Python.3.12`).

#### Byg en selvstændig EXE (Windows)

```bat
pyinstall.bat         :: bygger builds\PyGameZ.exe (alt i én fil)
```

`pyinstall.bat` bruger `.venv` (og opretter det om nødvendigt), installerer
automatisk **PyInstaller** og pakker det komplette spil - Python, pygame, alle
spil, sprog, wiki og logoer - ind i **en enkelt `PyGameZ.exe`** i mappen
**`builds\`**. Filen kører på en hvilken som helst Windows-PC uden installeret
Python og kan kopieres frit. Indstillinger og highscores (`settings.json`,
`mem.json`, `mem-ngb.json`) oprettes ved siden af .exe'en, mens du spiller.

#### Fejlfinding

- **`pygame` ikke fundet** → er venv aktiveret? Gentag trin 3
  (`pip install -r requirements.txt`).
- **`python` genkendes ikke (Windows)** → Python blev installeret uden "Add to
  PATH"; geninstallér og sæt flueben, eller brug `py` i stedet for `python`.
- **Ingen lyd** → tjek "Lyd" i indstillingerne; haptik virker kun med en controller.
- **Vindue/indlejring på Linux** → se *Platformsnoter* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ til toppen / back to top</a></b></div>

---

<a name="-norsk"></a>

## 🇳🇴 Norsk

En samling skrivebordsspill i Python: **Tkinter** står for vinduet og menyen,
**Pygame** er bygget inn som spillvisning inne i Tkinter-vinduet. Førtiseks
spill med felles innstillinger, fritt omdefinerbar styring, rekorder,
prosedyregenererte lydeffekter og – for enkelte titler – flerspillermodus.
Grensesnittet er **flerspråklig** – **14 språk** (tysk / engelsk / fransk /
spansk / portugisisk / polsk / tyrkisk / dansk / norsk / svensk / finsk /
tsjekkisk / slovensk / kroatisk); språket velges på en **velkomstskjerm** ved
første oppstart, der du også kan stille inn **oppløsning** og **lyd** (av som
standard); bortsett fra de tre hovedspråkene ligger alle de andre (spansk,
portugisisk og de ni ekstra – norsk blant dem) bak knappen **«Flere språk»**.
Alt kan endres senere i innstillingene.

### Hurtigstart

#### Windows

```bat
install-python.bat    :: én gang: sett opp Python 3.13 + .venv + pygame
start.bat             :: start spillsamlingen
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # starter med .venv, ellers system-python3
```

`start.bat` / `start.sh` bruker automatisk det virtuelle miljøet `.venv` hvis
det finnes, ellers system-Python. En detaljert trinn-for-trinn-guide finnes helt
nederst under **[Installasjonsveiledning](#installasjonsveiledning)**.

### Spillene

| Spill        | Moduser         | Kort beskrivelse |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 spillere  | Deluxe-Snake med 2D- og 3D-visning, boost, 6 spillmoduser (inkl. Competitive), gyllne epler og prestige |
| **Pong**     | 1 / 2 spillere  | Klassikeren mot KI eller spiller 2, omstillbar bevegelsesmodus |
| **Air Hockey** | 1 / 2 spillere | 2D-fysikk med impulsoverføring, musestyring, KI og power-ups |
| **Tic-Tac-Toe** | 1 / 2 spillere | m,n,k-spill på 3x3 til 9x9, tre KI-nivåer **eller** lokalt X mot O |
| **Breakout** | 1 spiller       | Murbrekker med steintyper, power-ups, comboer og mange nivåer |
| **Tetris**   | 1 / 2 spillere  | Moderne Guideline-regler (SRS, reserve, forhåndsvisning av 5 brikker, T-Spins): Maraton, Sprint 40, Ultra 2:00, Versus mot KI (3 nivåer) eller for to med søppelrader |
| **Invaders** | 1 spiller       | Space Invaders: tøm bølgene, beskytt livene dine |
| **Asteroids** | 1 / 2 spillere | Treghetsfysikk, bølger, UFO-er, power-ups, hyperrom – solo eller co-op-duell |
| **Pac-Man**  | 1 spiller       | Tro klone: 4 spøkelses-KI-er, kraftpiller, tunnel, frukt, nivåer |
| **Flappy Bird** | 1 spiller    | Tyngdekraftsflukt gjennom rør, mynter, skjold, dag/natt, medaljer |
| **Doodle Jump** | 1 spiller    | Automatisk hopp oppover, plattformtyper, fjærer, propell, monstre |
| **2048**     | 1 spiller       | Tallskyvespill fra 3x3 til 8x8: Klassisk, Tidsangrep og Uendelig, angre, flytende animasjoner, lagrede partier |
| **Minesweeper** | 1 spiller    | Klassikeren med trygt førsteklikk, chording, smilefjes og bestetider |
| **Sudoku**      | 1 spiller    | 4 varianter (Klassisk, X-sudoku, Killer, Mini 6x6) med 400 nivåer hver, dagens sudoku, opptil 3 stjerner per nivå, 4 hjelpemoduser, angre, lagret spill |
| **Frogger**     | 1 spiller    | Vei + elv + 5 bukter, bonusflue, krokodiller, tidsgrense, 3 vanskelighetsgrader |
| **Memory**      | 1 / 2 spillere | Finn par på 4x4 opp til 8x6, snuanimasjon, solopoeng eller duell |
| **Kabal**       | 1 spiller    | 5 varianter (Klondike, Spider, FreeCell, Pyramide, TriPeaks) med dra og slipp og angre |
| **Aim Trainer** | 1 spiller    | Avslappet 3D-blinkskyting: musa styrer kameraet, 4 moduser (presisjon/refleks/bevegelig/chill), 3 temaer inkl. et svart hull |
| **Fire på rad** | 1 / 2 spillere | Klassikeren med fallanimasjon: 3 KI-nivåer (minimax) eller lokal duell |
| **Tankduell**    | 1 / 2 spillere | 2D-arenaduell med rikosjettskudd, power-ups, 4 arenaer, KI med 3 nivåer |
| **Blackjack**    | 1 spiller    | Casino-blackjack med 4-kortstokks sko, doble/dele og 3:2-blackjack; spilles med lama-sjetonger fra den felles Lama-banken |
| **Tunnel Racer** | 1 spiller    | 3D-neonrørflukt: endeløs modus + 30 nivåer, tast- eller musestyring, motion blur |
| **3D-labyrint**  | 1 spiller    | Førstepersons raycaster (Wolfenstein-stil) med 50 seed-genererte nivåer, orber, minikart – eller 2D-ovenfravisning |
| **Reversi**      | 1 / 2 spillere | Othello på 8x8: fang og snu brikker, 3 KI-nivåer (minimax) eller lokal duell |
| **Yatzy**        | 1 / 2 spillere | Terningklassiker med 13 kategorier, øvre bonus og Yatzy; rekordjakt eller 2-spiller-hotseat |
| **Wordle**       | 1 spiller    | Gjett ord med 4 til 7 bokstaver: Uendelig, Dagens ord, Dordle og Quordle, vanskelig modus, fargeblind-palett, statistikk med søylediagram, del resultatet, ekte ordlister på 14 språk |
| **T-Rex Runner** | 1 spiller    | Endeløst ørkenløp: variabelt hopp, dukking, kaktus og pterodaktyler, dag/natt-syklus, økende fart, 3 vanskelighetsgrader |
| **Dam**          | 1 / 2 spillere | 3 regelsett (tysk 8×8, internasjonal 10×10, checkers), slåtvang og flygende dame, 3 KI-nivåer (minimax) eller lokal duell |
| **Poker**        | 1 spiller    | 3 valgbare varianter: Texas Hold'em mot KI, 5 Card Draw og Video Poker; innsatsrunder, blinds, lama-sjetonger fra den felles Lama-banken |
| **Sjakk**        | 1 / 2 spillere | Fullstendige regler, Chess960 og sjakkur, 6 KI-nivåer, 200 oppgaver fra Lichess-databasen, angre/hint, trekkliste, PGN-eksport eller lokal duell |
| **Mølle**        | 1 / 2 spillere | Legge-/flytte-/flygefaser, møller og slåing, valgfri flygeregel, 3 KI-nivåer eller lokal duell |
| **Simon**        | 1 / 2 spillere | Senso-huskespill: modusene Klassisk/Speed/Reverse/Blandet + Duell, lyd av/på/blandet, 4/6/9 felt, best per modus |
| **Biljard**      | 1 / 2 spillere | 8-ball, 9-ball og øvingsmodus i 2D, fast 3D-visning eller fritt roterbart 3D-kamera; myk fysikk, siktehjelp, 3 KI-nivåer |
| **Skyvepuslespill** | 1 spiller | 15-puslespill i 3x3/4x4/5x5: skyv de nummererte brikkene inn i åpningen, klikk- eller pilstyring, poeng etter trekk og tid |
| **Mastermind**     | 1 spiller  | Knekk den hemmelige fargekoden (3 moduser: 4×6, klassisk, 5×8), svarte/hvite tilbakemeldingspinner, endeløs streak-rekord |
| **Bubble Shooter** | 1 spiller  | Puzzle Bobble-klone: skyt like farger i grupper på tre, veggsprett, fallende klynger, 3 vanskelighetsgrader |
| **Hangman**        | 1 spiller  | Gjett ordet før galgen er ferdig; skjermtastatur, ordlister per språk, 3 lengdemoduser, endeløs streak |
| **Block Jump**  | 1 spiller       | 3D-plattformspill i Minecraft-stil: teksturert voxelverden med Steve-figur, stiger, gjerder og slimblokker, første-/tredjepersonskamera, motion blur, seed-genererte parkour-nivåer |
| **Tower Defense** | 1 spiller      | Slå tilbake endeløse bølger på 4 kart: opptil 11 tårntyper med oppgraderinger, salg & A/B-spesialisering, bosser, 3 moduser, aktive evner |
| **Minigolf**    | 1 / 2 spillere | 360 baner på 40 løyper (18 håndlagde, 342 genererte): sand, ramper, vann, støtfangere, vindmøller & vandrende klosser; scorekort med par og hole-in-one-bonus; **egen hullredigerer** med 15 objekttyper, 12 maler og deling som `.lamapgzmap` |
| **Pinball**     | 1 / 2 spillere | Flipperautomat med 3 bord: bumpere, slingshots, mål, L-A-M-A-baner, multiball med jackpot, kuleredning, dytt & tilt |
| **Bowling**     | 1 / 2 spillere | 10 frames med offisiell strike/spare-telling, ekte kjeglefysikk, hook-skru og bane i perspektiv, 3 vanskelighetsgrader |
| **Crossy Road** | 1 spiller     | Endeløse hopp over gress, veier, elver og togspor i isometrisk voxel-stil: dag/natt, ørn, 10 figurer å kjøpe, dagens rute |
| **Geometry Dash** | 1 spiller   | Rytme-plattformspill med kube, skip, ball, UFO og bølge: 8 baner fra Lett til Demon med 3 hemmelige mynter hver, treningsmodus, lydspor per bane; **baneeditor** med deling som `.lamapgzlevel` |
| **Battleship**  | 1 / 2 spillere | Sjøslag på 10x10: flåten plasseres med dra og slipp, 3 regelbrytere (berøring, salve, skyt igjen), KI med 3 nivåer eller lokal duell med overleveringsskjerm |
| **Casino**      | 1 spiller     | Europeisk rulett med alle klassiske innsatser og Lama-automat (5 hjul, 10 gevinstlinjer, wild, gratisspinn); én konto med lama-sjetonger sammen med Blackjack og Poker |

**Flerspiller (2 spillere lokalt)** er tilgjengelig for **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids
(co-op-duell)**, **Memory (duell)**, **Fire på rad**, **Tankduell**, **Reversi**,
**Yatzy**, **Dam**, **Sjakk**, **Mølle**, **Simon (duell)**, **Biljard**,
**Minigolf**, **Pinball**, **Bowling** og **Battleship** (med en
overleveringsskjerm som skjuler flåtene) – til sammen 20 spill. Modusen velges
rett i forspillskjermen (*Énspiller / Flerspiller*); Tetris har i tillegg
**Versus mot KI**. Nettversjonen er kun for én spiller.

#### Detaljer per spill

**Snake**
- **NYTT - 3D-visning** (tast **V** i oppsettet eller klikk på *Visning*): brettet
  gjengis som en sanntids 3D-scene – et **forfølgerkamera** svever bak slangen,
  og styringen skjer **relativt til blikkretningen** (venstre/høyre = svinge, to
  raske trykk = helomvending). Med avstandståke, stjernehimmel, sjakkbrettgulv,
  kantvegger, roterende matkrystaller, 3D-partikler og kamerarystelser ved krasj;
  etter game over sirkler kameraet sakte rundt slangen. Boost utvider synsfeltet.
  Tilgjengelig i 3D: *Klassisk* og *Hindringer* (veggene er alltid faste der,
  3D finnes bare i énspiller). Visningen huskes i `settings.json`.
- **NYTT - 3D-kameraalternativer** (i 3D-oppsettet, klikk raden *3D-kamera /
  smooth shake*, eller tast **K**): en egen meny med **smooth shake** (mykere
  kamera, langt mindre risting ved bevegelse/sving), justerbart **synsfelt (FOV)**
  og **kamerahøyde**, pluss en bryter for **risting ved sving** (skjermrist ved
  venstre/høyre-svinger på/av). Alt huskes i `settings.json`.
- **Boost**: **hold inne** boost-tasten = turbo (dobbel fart), tapper utholdenhet
  (linje); når den er tom, slår boosten seg av og lades opp igjen. Standard S1 =
  mellomrom/venstre Shift, S2 = Enter/høyre Shift.
- **6 spillmoduser** (valgbare i oppsettet): *Klassisk*, *Speed Rush* (raskere for
  hvert eple), *Hindringer* (dødelige blokker), *Portaler* (teleportørpar),
  *Time Attack* (60 sekunder, så mange epler som mulig) og *Competitive* (se under).
- **NYTT - Competitive** (énspiller): endeløs modus med **nivåstigning** – du
  starter med nøyaktig **ett** eple og kan ikke få flere i begynnelsen; jo flere
  epler du samler totalt, jo høyere blir **nivået**, som stadig legger enda et
  samtidig eple på brettet og øker poengmultiplikatoren.
  **Blå epler** åpner en **enarmet banditt**: innsatsen er lengden din,
  valseresultatet multipliserer eller krymper den og får **ekstra epler** til å
  dukke opp en kort stund (jackpot ved tre like symboler). **Lilla epler**
  (gambling) setter en del av **størrelsen** din på spill og multipliserer den
  delen tilfeldig, resten er trygg (ny størrelse = størrelse·(1-p) +
  størrelse·p·faktor): **normal** satser faste 50 % med **x0.5 .. x1.5**,
  **HARDCORE** er mer risikabelt med **75-90 %** innsats og **x0.25 .. x2.25**.
  **Størrelsen** vises som et **desimaltall øverst til venstre** og føres videre
  nøyaktig, slik at videre veddemål bygger på den. Det finnes **15 nivåer**
  (multiplikator opptil x16, opptil 16 epler samtidig); nivåene ligger i
  `games/levels/snake-comp.json` og kan utvides der uten å røre koden, resten av
  finjusteringen ligger i `competitive.py`.
- **NYTT - HARDCORE** (bryter i Competitive-oppsettet, tast **H**): hver **boost
  spiser av slangens lengde**; en rødglødende **HARDCORE-tekst** markerer modusen.
  Kun i Competitive; lengden faller aldri under minimum. Huskes i `settings.json`.
- **Gyllne epler** (midlertidige) gir mange poeng og fyller boosten umiddelbart.
- Valgfritt: **vegger du kan gå gjennom** (wrap-around), bonusepler, **prestige**
  (énspiller, tast **P**).
- **NYTT - Tilpass** (penselknapp helt øverst til høyre i oppsettet, eller tast
  **C**): en rent visuell meny («mods» som *aldri* endrer spillet) med to faner:
  - **Hode**: slangens **hodefarge** – 4 blågrønne forhåndsinnstillinger (fra mer
    blått til mer turkis), rødt, oransje og en **egen farge** via RGB-glidere.
  - **Rutenett (veiviser)**: legger et **koordinatrutenett** over brettet –
    **radnumre** (ved venstre og høyre kant) og **kolonnebokstaver** (topp/bunn).
    På store brett ser du dermed med en gang at f.eks. eplet på *8a* står i samme
    rad *8* som din egen posisjon *8z*. Fargesekvensen (5 forhåndsinnstillinger +
    to egne farger A/B) bestemmer fargetemaet.
  - **Banner**: slå multiplikatorbanneret (f.eks. fra det lilla eplet) **på/av** og
    juster **størrelse** (mindre/større) og **gjennomsiktighet** (mer transparent)
    – med sanntids forhåndsvisning.
  Alt lagres i `mem-ngb.json`; all visuell tilpasning går gjennom modulen `ngb.py`.
- Utseende: avrundet slange med øyne (hodet er turkis som standard), boost-glød,
  partikler.

**Pong**
- Énspiller mot KI, flerspiller = spiller 2 til høyre. Først til 5 poeng.
- **Bevegelsesmodus som kan byttes per tastesett**: *Kontinuerlig* (trykk én gang
  -> fortsetter å bevege seg, standard) eller *Hold* (beveger seg bare mens tasten
  holdes). Bytt: **X** = tastesett 1, **N** = tastesett 2 (huskes i `settings.json`).
- Ballfysikk med akselerasjon og vinkel avhengig av treffpunktet.

**Air Hockey**
- **Ekte 2D-fysikk**: runde køller og puck med impulsoverføring – pucken overtar
  køllas fart ved treff; vegger med restitusjon, lett isfriksjon, mål som
  åpninger i sideveggene.
- **Musestyring** i énspiller: køllen følger musa (enhver tast bytter tilbake til
  tastatur). Tastatur: retningstaster i 8 retninger, flerspiller = S1 venstre
  (WASD), S2 høyre (IJKL).
- **KI med tre nivåer** (Lett/Middels/Vanskelig): forsvarer sitt eget mål,
  angriper i sin egen halvdel og svinger rundt pucken for å unngå selvmål.
- **Power-ups** (kan slås av): *XL* (større kølle), *MÅL* (motstanderens mål
  krymper), *>>* (raskere kølle) – de tilhører spilleren som sist rørte pucken.
- Oppsett: vanskelighetsgrad, **mål for å vinne** (3/5/7/10), power-ups på/av
  (lagret i `settings.json`). Etter hvert mål server den som slapp inn målet.
- Utseende: lysspor etter pucken, partikler, pulserende målmunner, effektmerker.

**Tic-Tac-Toe**
- Oppsett: vanskelighetsgrad (Lett/Middels/Vanskelig) og brettstørrelse 3x3..9x9;
  vinnerlengde K = 3 (3x3), 4 (4x4), ellers 5.
- **1 spiller** mot KI (Vanskelig på 3x3 er uslåelig) **eller 2 spillere** lokalt
  (X mot O, annenhver tur med klikk). Ved game over: Enter/klikk = ny runde,
  **S** = innstillinger.

**Breakout**
- Steintyper: Normal, **Stål** (uknuselig), **Bombe** (eksploderer), **Gull**
  (ekstrapoeng).
- Power-ups: laser, ildkule, klebrig, skjold, mynt med mer; **combo-multiplikator**.
- Effekter: partikler, ballspor, skjermrist, poeng-popups, mange nivåmønstre.
- Oppsett: **1/2/3** = vanskelighetsgrad, **Venstre/Høyre** = ballfarge,
  **Opp/Ned** = startnivå, **M** = oppbygging. Spill: mus/piler, **mellomrom**
  skyter ut ballen (fyrer laser), **P/Esc** = pause.

**Tetris**
- **Moderne Guideline-regler**: 10x20-felt, brikker fra en **7-pose**,
  **SRS-rotasjonssystemet** med ekte wall kicks (også for I-brikken), rotasjon i
  begge retninger, **reserve** (én gang per brikke), **forhåndsvisning av 5
  brikker**, skyggebrikke og **lock delay** (0,5 s, maks. 15 nullstillinger).
- **Tre moduser** på forspillskjermen: *Solo*, *Mot AI* og *2 spillere*. Solo
  tilbyr i oppsettet **Maraton** (startnivå 1–15, teller for rekorden), **Sprint
  40 rader** (beste tid) og **Ultra 2 minutter** (beste poengsum); rekordene
  ligger i seksjonen `tetris` i `mem.json`.
- **Guideline-poengberegning**: single til Tetris, **T-Spins** (fulle og mini),
  **Back-to-Back** (x1,5), **combos** og **Perfect Clear** – med tekster på
  skjermen, rydde-animasjon, hard-drop-spor, partikler og nivå-opp-effekt.
- **Versus med søppelrader**: ryddede rader sender søppel til motstanderen
  (Tetris = 4, T-Spin Double = 4 …), innkommende søppel varsles i en varselstolpe
  og **motregnes** av dine egne angrep; begge feltene får samme brikkerekkefølge.
  **KI-en** finnes i 3 nivåer, og tempoet øker hvert 40. sekund.
- **Styring**: Venstre/Høyre med egen **DAS/ARR** (kan stilles inn i oppsettet),
  Opp = roter mot høyre, Ned = soft drop, Handling = hard drop; **C**/Shift =
  reserve, **Z**/**Y** = roter mot venstre, **X** = roter mot høyre. For to legger
  spiller 1 i reserve med **Q** og roterer mot venstre med **E**, spiller 2 med
  **høyre Shift** / **høyre Ctrl**. Etter spillet: **R** = igjen, **S** =
  oppsett.

**Invaders** – to moduser (valgbare i forspillskjermen):
- **Klassisk**: den klassiske alienblokken; deretter valgbart i oppsettskjermen:
  **Bevegelse** (bare venstre/høyre *eller* fritt med WASD) og **Sikting** (alltid
  oppover *eller* mot **musa** – da skyter du dit markøren er). Ødelagte aliener
  slipper av og til power-ups.
- **Arena (fri)**: fri bevegelse i alle retninger, fiendene strømmer inn fra alle
  kanter; du sikter i bevegelsesretningen, bytt våpen med **1–4**.
Felles: nivåsystem med **boss** hvert 4. nivå, fire våpen (blaster,
spredningsskudd, hurtigild, laser), power-ups (ekstraliv, skjold,
våpenoppgradering), eksplosjonseffekter, rekord.

**Asteroids**
- **Treghetsfysikk**: opp = skyv i pekeretningen, venstre/høyre = rotere, skipet
  driver videre (lett demping); alt går rundt skjermkantene. Klassisk
  **vektorutseende** med skyvflamme og stjernehimmel; hver stein har sin egen
  tilfeldige polygonform.
- Steiner splittes i to mindre (3 størrelser, **20/50/100 poeng**), **bølger** med
  økende antall og bannerinnblending.
- **UFO** (kan slås av): krysser skjermen med jevne mellomrom og sikter på skipene
  (siktefeil avhenger av vanskelighetsgrad) – 200 poeng for å skyte den ned.
- **Power-ups** (kan slås av), slippes av ødelagte steiner: **S**kjold (6 s
  usårbar), **T** = trippelskudd, **R** = hurtigild.
- **Hyperrom** (Ned-tast): nødhopp til en tilfeldig posisjon med 4 s nedkjøling –
  og 12 % risiko for å knuses ved ankomst.
- 3 liv, trygg gjenoppstandelse med usårbarhetsblinking, **ekstraliv for hver
  5000 poeng**; eksplosjonspartikler og kamerarystelser.
- **Co-op-duell** (flerspiller): begge skip flyr samtidig med separate liv og
  poeng – den med flest poeng vinner.
- Oppsett: vanskelighetsgrad, UFO-er på/av, power-ups på/av (lagret i `settings.json`).

**Pac-Man**
- **Klassisk 28x31-labyrint** i neonlook med prikker, 4 kraftpiller, sidetunnelvarp
  og spøkelseshus i midten.
- **Fire spøkelser med de originale oppførslene** (mål-rute-KI): *Blinky* jager
  direkte, *Pinky* legger bakhold (4 ruter foran), *Inky* bruker en vektor gjennom
  Blinky, *Clyde* trekker seg unna når han er nær.
- **Scatter/chase-faser** veksler (spøkelsene snur ved hver veksling); en
  **kraftpille** gjør spøkelsene blå og spiselige (kjede 200/400/800/1600),
  deretter vender øynene tilbake til huset.
- Spøkelseshus med **trinnvis frigivelse**, **frukt**-bonuser (per nivå),
  **3 liv**, **ekstraliv ved 10,000**, nivåsystem (blir raskere), dødsanimasjon,
  READY/GAME OVER-skjermer.
- Oppsett: **vanskelighetsgrad** (Normal/Vanskelig/Ekstrem) – spøkelsesfart og
  redselstid.
- Styring: **piler eller WASD**.  Enter = ny, S = oppsett.

**Flappy Bird**
- **Tyngdekraftsfysikk**: mellomrom / Opp / W / **museklikk** får fuglen til å
  flakse; den heller etter stige-/fallfart.
- Endeløse **rørpar** med en åpning (+1 per rør); **mynter** (bonus) og et
  **skjold**-power-up (overlever ett treff) dukker opp i åpningene.
- **Dag/natt-temaer** skifter med poengsummen; drivende skyer (parallakse),
  rullende bakke.
- Vanskelighetsgrad (Lett/Normal/Vanskelig): åpningsstørrelse, fart, røravstand –
  åpningen blir litt trangere når poengsummen stiger.
- **Medaljer** (bronse/sølv/gull/platina) ved game over, krasjanimasjon med
  kamerarystelser, rekord.

**Doodle Jump**
- Doodleren **hopper automatisk** ved landing; du styrer bare venstre/høyre (med
  treghet), kantene går rundt (wrap-around), og kameraet ruller oppover mens du
  klatrer.
- **Plattformtyper**: grønn (normal), blå (bevegelig), brun (knuses), hvit
  (forsvinner). **Fjærer** gir et superhopp, **propellhatten** bærer deg oppover
  en kort stund (og gjør deg usårbar).
- **Monstre**: berøring er dødelig – men du kan **skyte** dem med Opp / mellomrom
  (bonuspoeng).
- Poeng = høyde oppnådd; vanskelighetsgraden stiger med høyden. Rekord.
- Styring: venstre/høyre = bevege, Opp / mellomrom = skyte.

**2048**
- **Egen oppsettskjerm** med brettstørrelser fra **3x3 til 8x8** og tre moduser:
  *Klassisk* (mål 2048, deretter «Spille videre?»), *Tidsangrep* (3 minutter,
  klokka starter ved første trekk) og *Uendelig*.
- **Flytende animasjoner**: brikker glir, smelter sammen med et «pop» og vokser
  fram; poeng-popups, gnister fra 128, en trykkbølge fra 2048 og nye farger helt
  opp til 131072. Input under en animasjon lagres og utføres rett etter.
- **Angre** (av / 3 per spill / ubegrenset, tast **U** eller Backspace) – bruker
  du det, spiller du uten rekord og uten brikke-prestasjoner.
- **Lagre og fortsette**: partiet som pågår, lagres automatisk per størrelse og
  modus; beste poengsum og største brikke per størrelse/modus ligger i seksjonen
  `g2048` i `mem.json`.
- Styring: piler/WASD eller **sveip** med mus/touchpad, **R**/**N** = nytt spill,
  **Tab** = oppsett. Rekorden teller bare i **4x4 Klassisk** uten angre.

**Minesweeper**
- Tre nivåer: **Nybegynner** (9x9, 10 miner), **Viderekommen** (16x16, 40),
  **Ekspert** (30x16, 99) – **bestetiden per nivå** lagres og vises i oppsettet.
- **Det første klikket er alltid trygt** (minene plasseres etterpå, 3x3-området
  rundt klikket forblir fritt).
- **Venstreklikk** = avdekke, **høyreklikk** = flagg (valgfritt med
  spørsmålstegn-syklus), **F** = flagg under markøren, **R** = ny.
- **Chording**: klikk på et oppfylt tall avdekker de resterende naboene.
- Klassisk HUD: mineteller, **klikkbart smilefjes** (overrasket/solbriller/død),
  tidtaker; feilflagg strekes over på slutten, konfetti ved seier.
- Poeng = nivåets grunnverdi minus sekunder.

**Sudoku**
- **4 varianter** med **400 nivåer** hver (4 vanskelighetsgrader x 100):
  *Klassisk* (de kjente seed-nivåene – løste nivåer forblir avhuket), *X-sudoku*
  (begge diagonalene inneholder hvert siffer nøyaktig én gang), *Killer*
  (stiplede bur med sum; 400 nivåer generert på forhånd) og *Mini 6x6*. Alle
  puslespill har **én entydig løsning** – nivå 12 av «Vanskelig» er det samme
  puslespillet på hver PC.
- **Dagens sudoku**: én oppgave om dagen for alle, lik på PC og i nettleseren;
  vanskelighetsgraden avhenger av ukedagen (fra mandag Lett til lørdag Ekspert),
  og løser du hver dag, bygger du opp en rekke.
- **Opptil 3 stjerner per nivå** (løst · uten feil og hint · i tillegg under
  måltiden) og **beste tid** i nivåvalget; **påbegynte oppgaver** lagres
  automatisk og fortsettes neste gang.
- **4 spillmoduser** (velges før start) med poengmultiplikator: **Klassisk**
  (x2.0 – ingen hjelp), **Notater** (x1.5 – + blyantnotater og automatiske
  kandidater), **Komfort** (x1.0 – + gale sifre i rødt, konflikter og feil
  bursummer markert, riktige oppføringer låses), **Assistent** (x0.7 – +
  hinttast, maks. 3). Med **3-feil-grensen** aktivert (oppsettvalg) avslutter den
  tredje feilen partiet.
- Styring: piler/WASD = celle, **1-9** = siffer (også talltastatur),
  **0/Delete/høyreklikk** = viske ut, **U**/**Z** = angre, **Y** = gjør om, **N** =
  notater, **C** = fyll inn kandidater automatisk, **H** = hint, **M** =
  fargemarkør, **R** = start nivå på nytt, **Q** = nivåvalg. Inntasting «siffer
  først» (oppsett, **I**) og en **teller for gjenværende sifre** under hvert
  siffer; fullt spillbart med musa. Etter at spillet er over, viser **A** hele
  **løsningen**.
- Poeng = (basis for variant og vanskelighetsgrad - tid - feil - hint) x
  modusmultiplikator; alle varianter og dagens sudoku teller for rekorden.

**Frogger**
- 5 trafikkfelt (biler/lastebiler) og 5 elvefelt (tømmerstokker, skilpadder som
  **dykker** på høyere nivåer); 5 hjembukter øverst – fyll alle = neste nivå, alt
  går raskere.
- Ekstra: **bonusflue** (+200) i tomme bukter, **krokodiller** okkuperer bukter på
  høyere nivåer, **tidsgrenselinje** per frosk, ekstraliv ved 10,000.
- 3 vanskelighetsgrader (fart, trafikktetthet, tid); poeng per ny rad, bukt = 50 +
  tidsbonus, fullført nivå = +1000.

**Memory**
- Brettstørrelser **4x4, 6x6, 8x6**; motiver er form-farge-kombinasjoner tegnet
  helt med primitiver; **snuanimasjon**, par som ikke matcher, snur seg tilbake
  automatisk.
- **Solo**: basis - 15 per trekk - 2 per sekund (min. 100). **Duell** (lokal):
  annenhver tur, et treff gir en ny tur, flest par vinner.

**Kabal**
- **5 varianter** på forspillskjermen: Klondike (valg for å trekke 1/3), Spider
  (1/2/4 farger), FreeCell (grense for supertrekk), Pyramide (13-par, 2
  omdelinger) og TriPeaks (±1-kjede med combo-multiplikator).
- **Dra og slipp** eller klikk-klikk, **høyreklikk** = til grunnbunken,
  **U** = ubegrenset angre, **R** = ny gi, mellomrom = bunke.
- Kortene gjengis uten bildefiler (`games/cards.py`); alle variantene deler én
  rekordliste med variantspesifikke formler.

**Aim Trainer**
- **Ekte software-3D** (som Snakes 3D-modus): fast sikte i skjermsenteret,
  **direkte 1:1-musesikt som i et skytespill** (pekerfangst: markøren fanges inne
  i vinduet, Esc slipper den fri; justerbar følsomhet, ubegrenset yaw, pitch
  ±60°). Venstreklikk skyter nøyaktig gjennom senteret, med munningsflamme,
  sporlys og treffpartikler.
- **4 moduser**: presisjon (60 s, 3 kuler, nøyaktighetsbonus), refleks (30
  enkeltmål, reaksjonstidsstatistikk), bevegelige mål (baner + combo-multiplikator
  opptil x4) og chill (endeløs, ingen straff, **E** avslutter økten).
- **3 temaer** (i oppsettet, lagret): **verdensrom** med stjernekule, et **svart
  hull med en glødende ring** og en planet (standard), en neonarena med
  gulvrutenett og synthwave-sol, og en innendørs skytebane.
- Følsomheten kan endres midt i spillet med **+/-**; pluss en **justerbar motion
  blur** (0-80 %) for ekstra chill-visuals – begge lagres.

**Fire på rad**
- 7x6-brett med **fallanimasjon**, forhåndsvisning ved hover og en pulserende
  vinnerlinje; mus, piltaster eller direkte valg **1-7**.
- **3 KI-nivåer** (minimax med alfa-beta-søk): Lett bommer bevisst på trusler,
  Middels blokkerer pålitelig, Vanskelig planlegger dypt fremover – eller
  **2 spillere** lokalt på samme enhet.
- Startspilleren veksler hver runde; rekorden teller **seirene mot KI-en** i én økt.

**Tankduell**
- 2D-arenaduell: **skudd spretter av vegger én gang** (rikosjett) – treff rundt
  hjørner (eller deg selv!). Først til 5 runder med nedtelling.
- **4 arenaer** (Åpen, Kryss, Søyler, Labyrint) eller tilfeldig rotasjon;
  **power-ups**: hurtigild, skjold, trippelskudd.
- **KI med 3 nivåer** – den vanskelige leder skuddene sine og spretter dem av
  veggene med vilje – eller **2 spillere** på ett tastatur (S1 WASD+mellomrom,
  S2 piler+Enter).

**Blackjack**
- Ekte casinoregler: **4-kortstokks sko**, giveren står på 17, **blackjack betaler
  3:2**, giveren titter ved ess/10; **doble** og **én deling** (delte ess får ett
  kort hver).
- **Lama-sjetonger**: Blackjack spiller med kontoen i **Lama-banken**, som deles
  med Poker og Casino (start 1000, lagret permanent i `mem.json`). Innsatsen
  trekkes så snart kortene gis; under 10 sjetonger tar Enter et **banklån** som
  fyller kontoen opp til 1000.
- **Rekord** = høyeste stand av din egen **Blackjack-balanse** (1000 pluss alt som
  er vunnet og tapt i Blackjack) – gevinster i rulett, på automaten eller i poker
  teller ikke her, og det gjør heller ikke lån.
- Spilles via sjetongknapper og taster (**H**it/**S**tand/**D**oble/dele **X**,
  **1-4** = innsats, Backspace = nullstill innsats, Enter = gi) med
  kortanimasjoner; giverens skjulte kort snur seg nå virkelig når det avsløres.

**Tunnel Racer**
- **3D-neonrørflukt** (software-renderer som Aim Trainer): stenger, blokker og
  **ringporter å tre gjennom**, mynter på ideallinjen.
- **To moduser**: endeløs (farten stiger til et tak, rekord) og **30 seed-genererte
  nivåer** med mållinje, tidsbonus og avhaket fremgang.
- **Taststyring** (standard) eller **direkte musestyring** (pekerfangst, tast
  **C**); pluss justerbar **motion blur** (tast **B**, 0-80 %) – alt lagres.

**3D-labyrint**
- **Førstepersons raycaster i Wolfenstein-stil** (DDA, avstandståke, sprites) med
  mouselook + WASD, et **minikart** (tast **M**) og en grønn pulserende utgang –
  eller en klassisk **2D-ovenfravisning** (tast **V** i oppsettet).
- **50 seed-genererte nivåer** som stadig vokser; utgangen ligger alltid i punktet
  lengst fra starten, **orber** langs veien gir bonuspoeng.
- Poeng: 500 per nivå + 100 per orb + tidsbonus; løste nivåer hakes av, og øktens
  totalsum blir rekorden.

**Reversi**
- **Othello på 8x8**: legg brikker som fanger motstanderens rader, og snu alt som
  er innesluttet; ulovlige trekk er sperret, og en tur uten lovlig trekk **passes
  automatisk**.
- **Énspiller mot KI-en** (3 nivåer: negamax med alfa-beta, posisjonsvekting +
  mobilitet) **eller en lokal duell**, Svart mot Hvit.
- Lovlige ruter er uthevet; spill med **musa** eller utvalgsrammen (piler +
  mellomrom/Enter). Hver seier mot KI-en teller ett poeng mot rekorden.

**Yatzy**
- **Terningklassiker**: 5 terninger, opptil 3 kast per tur, **hold** terninger
  enkeltvis, marker deretter en av de **13 kategoriene** (med sanntids
  forhåndsvisning av mulige poeng).
- Fullstendig poengark: øvre del med **63-poengs bonus (+35)**, tre like/fire like,
  hus, liten/stor straight, **Yatzy (50)** og Sjanse.
- **Énspiller som rekordjakt** på høyest mulig totalsum, eller **2-spiller-hotseat**
  med to ark side om side; spill med mus eller taster (mellomrom, 1-5, piler, Enter).

**Wordle**
- Gjett det skjulte ordet; farget tilbakemelding (grønn/gul/grå) med korrekt
  **telling av doble bokstaver** og et skjermtastatur som farges (QWERTZ for tysk,
  tsjekkisk, slovensk og kroatisk, AZERTY for fransk, ellers QWERTY).
- **Fire moduser**: *Uendelig* (det ene ordet etter det andre med 6 forsøk hver;
  hvert løste ord gir poeng, det første uløste avslutter spillet), *Dagens ord*
  (ett ord om dagen per språk og lengde – likt på PC og i nettleseren – med
  nedtelling og rekke; et påbegynt dagens ord lagres), *Dordle* (2 ord samtidig
  på 7 forsøk) og *Quordle* (4 ord på 9 forsøk, tastene viser fargene fra alle
  brettene).
- **Oppsett** før hvert spill: **ordlengde 4 til 7**, **vanskelig modus** (hint du
  har funnet, må brukes videre) og **fargeblind-palett** (oransje/blå); ved siden
  av står statistikken.
- **Ekte ordlister på alle 14 språk** (mappen `woordlistz/`, kun A-Z), med egne
  lister for hver lengde: bare ved 5 bokstaver nesten **34 000 løsninger** og over
  **213 000 tillatte gjett**, rundt 134 000 løsninger i alle fire lengdene til
  sammen. Løsningene er vanlige ord uten navn, engelske rester og støtende ord;
  hvert gjett sjekkes mot listen – alt annet avvises, og raden rister kort.
- **Statistikk** per språk, lengde og modus: spill, vinnerprosent, nåværende og
  beste rekke og **fordelingen av forsøk som søylediagram** (seksjonen `wordle` i
  `mem.json`). **Del** (**C**) kopierer et emoji-rutenett uten å avsløre ordet.
- Rekorden teller bare *Uendelig* med 5 bokstaver; de andre lengdene har egne beste
  resultater. Prestasjonene **Synsk** (høyst 2 forsøk), **Ordvane** (7 dagens ord
  på rad) og **Firedobbelt geni** (Quordle løst).

**Poker**
- **3 varianter** på forspillskjermen: **Texas Hold'em** mot 1–3 KI-motstandere
  med dealerknapp, blinds og fire innsatsrunder, **5 Card Draw** (heads-up mot
  KI-en, én byttedel) og **Video Poker** (*Jacks or Better*, solo mot
  gevinsttabellen).
- Handlinger via knapper eller taster: **F** = fold, **C** = check/call, **R** =
  raise, **A** = all-in; hold/bytt kort med klikk eller **1-5**, **Enter** trekker
  eller gir neste hånd.
- **Lama-sjetonger** fra den felles **Lama-banken**: ved starten av en hånd ligger
  kontoen din på bordet som stabel, og det som går i potten, trekkes med en gang –
  forlater du bordet midt i en hånd, taper du bare din andel av potten. Blakk
  (under big blind på 20, under 10 i Video Poker) = banklån opp til 1000.
- **Rekord** = høyeste stand av **Poker-balansen** din (1000 pluss alle gevinster
  og tap i poker); prestasjonen **Chipleader** teller også bare pokerbalansen.

**Sjakk**
- **Fullstendig sjakk**: alle brikketrekk inkludert **rokade**, **en passant** og
  **bondeforfremmelse** (velg brikke); **sjakk, sjakkmatt og patt** pluss remis
  ved **femtitrekksregelen**, **trefoldig stillingsgjentakelse**,
  **utilstrekkelig materiale** eller avtale.
- **Tre moduser**: *parti* mot KI-en, *2 spillere* ved samme datamaskin (brettet
  kan snu seg etter hvert trekk) og **oppgaver**.
- **Sterkere KI uten hakking** i 6 nivåer fra *Nybegynner* til *Mester*: iterativ
  fordypning, transposisjonstabell, ro-søk, åpningsbok og en evaluering med
  mobilitet, bondestruktur og kongesikkerhet. KI-en regner i små biter per bilde –
  spillet hakker aldri.
- **Oppsett**: fargevalg, **sjakkur** (ingen, 1+0, 3+2, 5+0, 10+5) og **Chess960**
  (alle 960 startstillinger, nummeret står over trekklisten).
- **Sidepanel** med klokker, slåtte brikker, materiellbalanse og en rullbar
  **trekkliste (SAN)**; dra og slipp, glidende brikker, koordinater. Taster: **U** =
  angre, **H** = hintpil, **O** = tilby remis, **X** = gi opp, **F** = snu
  brettet, etter partiet **P** = **PGN-eksport**.
- **Oppgaver**: 200 oppgaver i 5 trinn (matt i 1/2/3, taktikk I/II) fra
  **Lichess' frie oppgavedatabase (CC0)**, kontrollert med spillets egen motor; i
  matt-oppgaver teller ethvert trekk som setter matt. Fremgangen ligger i
  seksjonen `chess` i `mem.json`.
- Angre og hint gjør et parti «assistert»: rekorden teller bare seire uten hjelp
  mot KI-en (per økt).

**Mølle**
- **Mølle** med alle tre faser: **legging** (9 brikker hver), **flytting** langs
  linjene og **flyging** når man er nede i 3 brikker (kan slås av).
- En fullført **mølle** fjerner en av motstanderens brikker (helst en utenfor en
  mølle); du taper når du er redusert til under 3 brikker eller ikke kan flytte.
- **3 KI-nivåer** (minimax med alfa-beta, faseavhengig evaluering) eller en **lokal
  duell**; med trekkhint, mølleutheving og en brikketeller.

**Simon**
- **Senso-huskespill**: den opplyste sekvensen vokser hver runde og må gjentas
  nøyaktig.
- **Moduser**: *Klassisk*, *Speed* (blir raskere), *Reverse* (baklengs), *Blandet*
  (modusen roterer hver runde) og en to-spiller-**Duell** (bytt på å legge til og
  gjenta).
- **Lyd** *av / på / blandet* (blandet trener både visuell OG auditiv hukommelse),
  **4/6/9 felt** som vanskelighetsgrad; **beste poengsum per modus** lagres. Spill
  med musa eller talltastene 1-9.

**Biljard**
- **8-ball**, **9-ball** og en regelfri **øvingsmodus**, mot KI-en (med siktehjelp)
  eller **to spillere lokalt**.
- **Tre fritt valgbare visninger**: klassisk **2D-ovenfra**, et fast
  **3D-skråperspektiv** med skyggelagte kuler og et **fritt roterbart 3D-kamera**
  (høyre musetast). All bevegelse er tidssteg-basert og **mykt dempet** (friksjon,
  deltrinn for å unngå gjennomtunnelering).
- **Støt**: hold inne venstre musetast for å lade kraft, slipp for å støte; en
  siktelinje og kraftmåler hjelper. **Ball i hånd** etter en feil. Visningen (V)
  huskes i `settings.json`; vunne frames teller mot rekorden.

**Skyvepuslespill**
- 15-puslespill i tre størrelser: **3×3** (lett), **4×4** (klassisk) og **5×5**
  (vanskelig); skyv de nummererte brikkene inn i den frie åpningen.
- Alltid løsbart (stokket med mange tilfeldige trekk). Styring ved å **klikke** på
  en brikke i åpningens rad/kolonne (hele linjen sklir) eller med **piltastene**.
- Poeng = en grunnverdi per størrelse minus trekk og tid; når du løser det, starter
  et nytt brett med en gang.

**Mastermind**
- Knekk den skjulte **fargekoden**; etter hvert gjett får du **svarte** pinner
  (riktig farge + posisjon) og **hvite** pinner (riktig farge, feil plass).
- **3 moduser**: Lett (4 pinner / 6 farger / 12 rader), Klassisk (4/6/10) og
  Vanskelig (5 pinner / 8 farger); dubletter av farger er tillatt.
- Spill via fargepaletten (klikk eller taster **1–8**), OK/Enter sjekker raden.
  **Endeløs streak** som i Wordle: hver knekt kode gir poeng.

**Bubble Shooter**
- **Puzzle Bobble** på et bikubenett: sikt med musa, skyt bobler oppover, **tre
  eller flere av en farge** får gruppen til å sprekke.
- Bobler som mister forbindelsen til taket, **faller** (bonus); skudd **spretter av
  veggene**, med forhåndsvisning av neste boble.
- **3 moduser** (4/5/6 farger, noen med nedadgående rader); game over ved den røde
  linjen.

**Hangman**
- Gjett ordet **bokstav for bokstav**; hver feil tegner en del av galgemannen,
  tapt etter **6 feil**.
- **Ordlister per språk** (kun A–Z), **3 lengdemoduser** (kort / blandet / lang);
  skriv eller klikk på skjermtastaturet.
- **Endeløs streak**: hvert gjettet ord gir poeng (flere gjenværende liv + lengre
  ord = mer).

**Block Jump**
- **3D-plattformspill i Minecraft-stil** (software-3D som Snakes 3D-modus): hopp
  over en svevende **voxelverden** av blokker til det glødende målet.
- **Minecraft-skin**: alle blokker har ekte **pikseltexturer** (gress, jord,
  stein, planker, diamant, slim, tre); detaljnivået følger avstanden
  (**T** = høy/lav/av).
- I tillegg: **Steve**-figur med ganganimasjon (tredjepersonskamera), **hånd**
  i førsteperson, **beacon-stråle** ved målet, roterende **gullbarrer** som
  mynter, firkantet **sol**, **pikselskyer** og et HUD med **hjerter**.
- Blokktyper: faste blokker (gress/jord/stein/tre), **stiger** (klatre), **gjerder**
  (hopp over), **hoppeblokker** (kaster deg opp) og **mynter**.
- Kamera **førsteperson som i Minecraft som standard**, **V** bytter til et
  forfølgerkamera; **mouselook** med pekerfangst, justerbar **motion blur** (**B**)
  og følsomhet (**+/-**).
- **Seed-genererte parkour-nivåer** blir vanskeligere; mål = poeng + tidsbonus,
  mynter +50, et fall koster et liv (start med 3). Styring: WASD/piler, **mellomrom**
  for å hoppe.

**Tower Defense**
- **Endeløst bølgeforsvar** på **4 kart** (Eng, Kløft, Veikryss, Spissrotgang),
  hvert med sin egen sti; låste kart låses opp med din beste bølge, en **boss**
  ankommer hver **8. bølge**.
- **3 moduser**: Klassisk (7 tårn, hovedmodusen), Kompakt (4 tårn, 2 nivåer) og
  Maksimal (**11 tårn**, **A/B-spesialisering** på høyeste nivå, spesialfiender,
  aktive evner **Meteor/Frostnova/Gullrush**).
- **11 tårntyper** fra pil til laser og gullbank, hvert med opptil
  **3 oppgraderingsnivåer**, salg refunderer 70%; fiender med panser,
  regenerering, deling, kamuflasje, healingaura og luftrute.
- **Økonomi**: gull per nedskyting, bølgebonus + 5% renter; poeng per nedskyting
  og bølge. **F** = 2x tempo, **G** = rekkevidder, høyreklikk avbryter.

**Minigolf**
- **360 baner på 40 løyper**: *Classic* og *Pro* med 9 håndlagde baner hver,
  **Touren** med 38 løyper à 9 genererte baner (342 totalt) og stigende
  vanskelighet, i tillegg *Random* fra alt sammen. Løype 7, bane 3 ser lik ut
  overalt - ingenting må lagres.
- **Underlag og hindringer**: sand bremser, ramper akselererer, vann koster et
  straffeslag, gummistøtfangere gir fart tilbake, og vindmøller og vandrende
  klosser krever timing. Fysikken går i deltrinn med friksjon som i biljard -
  ingenting hakker, og ingenting sklir gjennom vantet.
- **Styring**: musa sikter, venstre museknapp holdt nede lader kraften, og et
  slipp slår (piler + mellomrom går også). **R** avbryter et ladet slag uten å
  slå. **G** slår siktelinjen av og på, **Z** auto-siktet, **P** Plukk opp.
- **Kraftlås (hold høyre museknapp)**: fryser ladebjelken nøyaktig der den er -
  gyllen, med prosenten, en hengelås og en pulserende ring rundt ballen. Slik
  venter du på åpningen i mølla med slaget ferdig ladet. Slipper du, lades det
  videre; en låst kraft overlever til og med slaget, og neste venstreklikk slår
  med nøyaktig den verdien.
- **Scorekort** til høyre med par og slag per bane; for to spillere spiller begge
  den samme banen etter tur. Poeng: 600 per bane, ±300 per slag under/over par,
  **500 ekstra for hole in one**. Laveste slagtall per løype ligger i
  `minigolf`-delen av `mem.json`.
- **Plukk opp kan slås av**: som standard slutter en bane etter åtte slag og
  teller minimum. Vil du heller spille til ballen går ned, sett *Plukk opp* til
  AV i oppsettet (eller trykk **P**).
- **Auto-sikte kan slås av**: som standard snur køllen seg selv mot hullet før
  hvert slag. Vil du heller sikte selv på hvert hull, sett *Auto-sikte* til AV
  i oppsettet (eller trykk **Z**) - da blir den sist valgte retningen stående,
  og på tee-en til et nytt hull peker køllen nøytralt oppover.
- **F** nullstiller hullet du spiller: slag på 0, ballen på utslaget - samme
  hull, samme bane.
- **Videre i stedet for gjentakelse**: når runden er ferdig, tar knappen
  **Videre** deg til neste bane (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), så
  de samme ni hullene aldri gjentas; ved siden av: **Om igjen** (samme bane) og
  **Oppsett**. Taster: Enter = videre, R = om igjen, S = oppsett.
- **Replay av runden**: til slutt viser **P** (eller knappen **Replay**) hele
  runden slag for slag. Med **S** havner den i arkivet (sidefeltknappen
  **Replays**).
- **Bygg og del dine egne hull**: fanen **MAPS** på oppsettskjermen fører til
  din egen samling - **Ny** åpner hullredigereren. Hvert hull får et navn og en
  **id** (små bokstaver, ingen mellomrom); id-en er samtidig filnavnet som
  foreslås ved deling. Til de sju klassiske hindringene kommer **åtte nye**:
  rør (flytter ballen til den andre enden), is, klebefelt, booster, magnet,
  enveisport, dreieskive og hopprampe. Hullets størrelse kan stilles fritt
  (60x80 til 160x240), **12 maler** gir et utgangspunkt, og angre/gjør om samt
  **Test** er med. **Del** skriver nøyaktig ett hull som en
  `.lamapgzmap`-fil - via lagringsdialogen eller rett i Nedlastinger-mappen,
  med navnet ditt som skaper. **Importer** leser den inn igjen og bytter
  automatisk til `-2` hvis id-en er tatt. Navn, id og skaper går alltid gjennom
  et **ordfilter på tvers av alle 14 språk**. Et hull spilles alene via
  **Spill**, eller hele samlingen via det femte banevalget **Egne**.

**Pinball**
- **Tre bord**: *Classic* (tre bumpere, én målrekke), *Space* (fire bumpere i
  rombe, to rekker) og *Lama* (åpen bane, seks mål i en bue); 3 eller 5 kuler per
  spill, for to spillere vekselvis kule for kule.
- **Alt en flipper trenger**: utskytingsbane med ladestolpe (for svakt? kula
  triller tilbake, og du får prøve igjen), to flippere, slingshots, målrekker,
  fire **L-A-M-A**-baner, fanger med kulelås, **multiball med jackpot**, seks
  sekunders **kuleredning**, dytt og **TILT**.
- **Multiplikator opptil x5** via ryddede rekker og fullførte baner; bumpere 100,
  slingshots 50, mål 250 - under multiball betaler bumperne 2 500 i jackpot.
- Flipperne styres med de tildelte venstre/høyre-tastene (samt venstre/høyre
  [Shift]) eller musa. Rekorden per bord ligger i `pinball`-delen av `mem.json`.

**Bowling**
- **Ti frames etter de offisielle reglene**, inkludert strikes, spares og
  bonuskastene i tiende frame (maksimum: 300). **Scorekortet** under overskriften
  viser hver frame med X, / og løpende sum.
- **Kast i fire trinn**: posisjon, vinkel, skru og kraft. Hver glider svinger av
  seg selv og låses med handlingstasten - eller stilles for hånd med
  venstre/høyre, som stopper svingingen.
- **Ekte kjeglefysikk**: ti kjegler som sirkler med masse som velter hverandre;
  en strike kommer av fysikk og ikke flaks. Banen er oljet fremst, så **hooken**
  griper først i siste tredjedel.
- Baneview i perspektiv med renner, siktepiler og kjeglefelt; tre
  vanskelighetsgrader (*Lett/Normal/Pro*) endrer gliderfart og spredning.
  Rekorden per grad ligger i `bowling`-delen av `mem.json`.
- **Replay av partiet**: til slutt viser **P** alle kastene på nytt, og **S**
  lagrer dem i arkivet (knappen **Replays**).

**Crossy Road**
- **Endeløse hopp** over gress (trær og steiner sperrer veien), veier med biler og
  lastebiler, elver med tømmerstokker og vannliljer og **togspor** der et tog
  suser forbi etter varsellys og bjelle – lenger fremme venter hele stasjoner med
  opptil 5 spor. Ruten bygges rad for rad, har alltid en farbar vei, og både tempo
  og trafikk øker.
- **Isometrisk voxel-stil**: figurer, kjøretøy og trær av skyggelagte klosser
  (forhåndsrendret for hver rutestørrelse), mykt følgende kamera, squash &
  stretch ved hopp, vannsprut, flatklemming-animasjon, fjær og glitrende mynter;
  fra rad 50 **dag/natt-skifte** med frontlys.
- **Ørnen**: kameraet kryper fremover – nøler du for lenge eller går mer enn tre
  rader tilbake, tar ørnen deg (en rød kant varsler først). Å drive ut av bildet
  på en tømmerstokk er også slutt.
- **Mynter og figurer**: innsamlede mynter (kjempemynt = 5) lagres og kjøper nye
  figurer i fanen **Figurer**: frosk, gris, pingvin, katt, rev, lama, robot,
  spøkelse og enhjørning (25 til 250 mynter); kyllingen er med fra start.
- **Moduser**: *Uendelig* (poeng = lengste rad, teller for rekorden) og *Dagens
  rute* (lik for alle i dag, også i nettleseren, med egen dagsrekord). Styring:
  piler/WASD, mellomrom/Enter/klikk = hopp fram; i oppsettet **H** = skygger,
  **N** = dag/natt. Mynter, figurer og dagsrekord ligger i seksjonen `crossy` i
  `mem.json`.

**Geometry Dash**
- **Rytme-plattformspill**: figuren suser av seg selv mot høyre – du bestemmer
  bare når det skal hoppes eller flys. **Fem former** – kube, skip, ball, UFO og
  bølge – pluss form-, tyngdekraft- og fartsportaler (0,5x til 3x),
  gule/rosa/blå **pads og kuler**, halvblokker, pigger, groper og fargetriggere.
- **8 innebygde baner** fra *Lett* til *Demon* («Lama Inferno») med **3 hemmelige
  mynter** hver. Hver bane kan bevislig klares: da den ble bygget, løste en
  løsningsalgoritme den med spillets ekte kode – med alle myntene og til og med
  forskjøvet med 1/240 sekund.
- **Presis fysikk**: fastkommaberegning med fast 240 Hz-steg; hvert trykk virker
  nøyaktig i steget der det skjedde – likt ved enhver bildefrekvens og
  bit-identisk i nettleseren.
- **Treningsmodus** (**P**) med automatiske og egne sjekkpunkter (**Z** setter,
  **X** sletter), forsøksteller, fremdriftslinje, eksplosjoner og øyeblikkelig
  omstart (**R**). Hver bane har sitt **eget lydspor** – bakgrunn, underlag og
  kuler pulserer i takt (musikken slås av med **M**).
- **Stjerner og mynter**: klarer du en bane i normal modus, får du stjernene, og
  hver mynt er verdt en stjerne til; rekorden er det **samlede antallet stjerner**
  (høyst 65). Beste resultater per bane, mynter, forsøk og hopp ligger i
  seksjonen `geodash` i `mem.json`.
- **Baneeditor** i fanen **BANER**: lerret med rutenett, palett med 6 grupper
  (blokker, farer, pads og kuler, portaler, fart, ekstra), rotering, angre/gjør
  om, oversiktsstripe, **test fra start eller herfra** og baneinnstillinger
  (startfart, startform, musikkstil, BPM, farger). Haken **«verifisert»** kommer
  først når du selv har klart banen din. **Del** skriver en `.lamapgzlevel`-fil,
  **Import** leser den inn igjen; banene lagres i `ugc.json` ved siden av dine
  egne minigolfbaner.

**Battleship**
- **Sjøslag på 10x10** med hangarskip (5 ruter), slagskip (4), krysser (3),
  ubåt (3) og destroyer (2) – den som først senker hele fiendens flåte, vinner.
- **Plasser flåten** med dra og slipp fra dokken: **R** eller høyreklikk roterer,
  forhåndsvisningen lyser grønt eller rødt, **X** plasserer alt tilfeldig, **C**
  tømmer brettet; den siste oppstillingen din foreslås igjen.
- **Regler i oppsettet** (lagres): *skip kan ligge inntil hverandre*, *salve* (like
  mange skudd per tur som du har skip flytende) og *skyt igjen etter et treff*.
- **KI med 3 nivåer**: Lett skyter tilfeldig, Middels følger opp treff
  systematisk, Vanskelig beregner et **sannsynlighetskart** med
  sjakkbrettparitet (i snitt ca. 70 / 60 / 45 skudd for en hel flåte). Eller **2
  spillere** ved samme datamaskin – en **overleveringsskjerm** skjuler begge
  flåtene før hver tur.
- **Grafikk**: radarsveip, animerte bølger, granater i bue, vannsprut,
  eksplosjoner med røyk og brennende ruter, en «SENKET!»-avsløring og en
  rundeoppsummering med skudd, treff og treffprosent. Rekorden teller **seirene
  dine mot KI-en** i én økt.

**Casino**
- **Rulett** (europeisk, 37 felt): alle klassiske innsatser ved klikk på et tall,
  en kant eller et hjørne – **plein** (35:1), cheval, transversale, carré, sixain,
  kolonne, dusin, rød/svart, partall/oddetall og manque/passe. Sjetongverdier
  1/5/25/100/500, høyreklikk fjerner sjetonger; **Snurr**, **Gjenta** (**R**),
  **Doble** (**D**) og **Tøm**. Kula spinner i spiral ned i feltet som er trukket
  på forhånd, og øverst står de siste 12 tallene.
- **Lama-automat**: 5 hjul x 3 rader, **10 gevinstlinjer**, **lama = wild**,
  **gullmynter = scatter** med 10 gratisspinn og doble gevinster, innsats per
  linje 1/2/5/10, **auto-spinn** (10/25), **turbo** og gevinsttabell.
  **Tilbakebetalingsprosenten er 96,1 %** – beregnet nøyaktig fra hjulstrimlene.
- **Lama-banken**: Casino, Blackjack og Poker deler én konto med
  **lama-sjetonger** (start 1000, seksjonen `casino` i `mem.json`); gamle
  sjetongsaldoer overføres automatisk. Innsatser trekkes med en gang, hvert spill
  fører sin egen balanse for rekorden, og går du blakk, får du et **banklån** opp
  til 1000.
- Konfetti, myntregn, big/mega/jackpot-bannere og gevinstlinje-animasjoner;
  prestasjonene **Blinkskudd** (vunnet plein i rulett) og **Lama-jackpot** (5
  lamaer på én linje).

Rekordene lagres i `highscores`-delen av `mem.json` (ved siden av koden) – sammen
med språket (delen `mem`).

### Grensesnittet

Hele grensesnittet er tegnet fra bunnen av (rent Tkinter + Pygame, ingen ekstra
pakker) og stylet som en moderne spill-launcher:

- **Spilliste i sidefeltet**: hver rad har sitt eget **minipiktogram** i spillets
  aksentfarge, viser gjeldende **rekord (★)** og reagerer med mykt animerte
  hover-effekter. Spillet som kjører, forblir uthevet; i små vinduer **ruller**
  listen med musehjulet.
- **Statuskort** nederst til venstre med en **status-LED** (grå = meny, grønn =
  kjører, gull = pause, rød = game over) og en **sanntids FPS-visning**.
- **Startskjerm** med nordlys, et parallakse-stjernefelt med stjerneskudd, en
  svevende logo med gnister i bane, et **klikkbart spillrutenett** rett under
  logoen (alle spill med hover-effekt i sin aksentfarge) og en **rekord-rulletekst**.
- **Effekter overalt**: myke skjermoverganger, gnister når du bekrefter et
  menyvalg, **konfettiregn ved ny rekord** og en ekte **uskarphet** bak
  pause-overlegget.
- Hvert spills **forspillskjerm** vises i spillets aksentfarge og viser forrige
  rekord som en chip. Med mange moduser og lav oppløsning blir den **kompakt**:
  Innstillinger, Wiki og Tilbake samles på én rad, og skriften tilpasser seg –
  ingenting havner lenger utenfor bildet.
- **Enhetlig utseende i spillet**: alle 46 spillene deler menyens temapalett og
  skrift – HUD-er, oppsettskjermer og overlegg følger designet valgt i
  alternativene (v4.1 / v4 / Classic), mens hver spillflate beholder sine
  identitetsfarger. Hvert spill håndterer nå oppløsningsendringer midt i spillet
  på en ren måte, og navnene i menyen er språkavhengige (f.eks. «Schach» → «Sjakk»).
- **Innebygd wiki** («LamaWiki»): detaljert hjelp for hvert spill (styring,
  moduser, poeng, tips) pluss generelle sider – med et **søkefelt**, kategorier,
  rullbare artikler og tastekapsler, på alle 14 språk. Nåbar via sidefelt-knappen
  **«Wiki / Hjelp»** og fra hvert spills forspillskjerm (åpner spillets side
  direkte).
- **Prestasjoner & statistikk**: **107 prestasjoner** i tre kategorier (23 mål
  på tvers av samlingen, 37 poengmilepæler og 47 spesielle øyeblikk som sjakkmatt
  mot KI-en, 4096-brikken, en T-Spin Double, 25 løste sjakkoppgaver, en Killer
  Sudoku eller lama-jackpoten; i 2048 og sjakk teller ikke partier med angre
  eller hint) med **gyllent varsel og fanfare**
  ved opplåsing - selv midt i spillet; gamle rekorder godskrives automatisk.
  I tillegg en **statistikk**-fane: total spilletid, partier, seire, rekorder,
  favorittspill og en tabell per spill sortert etter spilletid. Nås via
  knappen **«Prestasjoner & statistikk»** i sidefeltet.
- **Replays**: minigolf og bowling tar opp hver runde. Til slutt viser **P**
  opptaket, og **S** legger det i arkivet - tilgjengelig via sidefeltknappen
  **Replays** (en fane per spill, pause, hopp mellom sekvenser, tempo 0,5x til
  4x). Kan slås av ved første oppstart og i innstillingene.

### Betjening

- Velg et spill via knappen i menyen til venstre. Deretter vises en
  **forspillskjerm**: velg **Énspiller** eller **Flerspiller**, gå til
  **alternativene** eller tilbake. Piler/mus for å velge, Enter for å starte.
- **ESC** = pause / fortsett (i menyer: tilbake).
- **F11** (eller knappen «Fullskjerm på/av») = veksle fullskjerm. Pygame-visningen
  forblir innebygd og skaleres opp med bevart sideforhold (svarte kanter når
  sideforholdet avviker). Vinduet kan endres fritt i størrelse.
- **«Tilbake til menyen»** avslutter spillet og lagrer rekorden – det samme gjør
  et bytte til et annet spill via sidefeltet.
- **Faste ekstrataster**: i tillegg til de fem handlingene som kan bindes, har
  noen spill egne taster (f.eks. reserve **C** og roter mot venstre **Z** i
  Tetris, angre **U** i 2048, sjakk og sudoku). De virker bare hvis tasten ikke
  er bundet til en handling i alternativene, og står i oppsettshintet og i
  wikien. Holdte taster registreres riktig og slippes ved pause eller Alt-Tab –
  ingenting «henger» lenger.
- **«Avslutt»** lukker Pygame og Tkinter på en ren måte.

### Alternativer, styring og lyd

Alternativskjermen åpnes via knappen **«Alternativer / Styring»** (til venstre)
eller fra forspillskjermen. Den er organisert i **tre faner** (**Generelt /
Styring / Utseende**; bytt ved å klikke eller med Tab-tasten):

- **Generelt**: **lyd** på/av, **volum** og **haptikk** (gamepad-vibrasjon, virker
  bare med en tilkoblet kontroller) pluss **auto-oppløsning**, **oppløsning**,
  **FPS** og **språk** – hver byttes med Venstre/Høyre.
- **Styring**: **forhåndsinnstillinger** (*WASD + piler*, *WASD + IJKL*,
  *piler + WASD*) og **omdefinere hver enkelt tast** for spiller 1 og spiller 2:
  velg en rad, trykk Enter, trykk ønsket tast (Esc avbryter).
- **Utseende**: velg **UI-design** – **UI v4.2** (standard: Midnight Glass – en dyp
  midnattsgradient med langsomt drivende myke lys i indigo, turkis og magenta, fint
  filmkorn, spredte stjerner og paneler som matt glass med en lys kant),
  **UI v4.1** (som UI v4 men livligere – diskré stjerner pluss Saturn og et svart
  hull i bakgrunnen på startskjermen), **UI v4.1.1** (som v4.1, men et flislagt
  **sikksakkmønster** i svart og antrasitt i stedet for stjernehimmelen),
  **UI v4.1.2** (samme mønster i palettens blåtoner – aksentblå som dominerende
  farge, en mørkere blå som bunn), **UI v4.1.3** (samme mønster i indigoen fra UI
  v4 på svart), **UI v4.1.4** (i grafittonen fra UI v4 på svart), **UI v4** (et
  helt rolig, flatt grafittutseende med en enkelt indigo-aksent), **UI v3** (det
  forrige klassiske grensesnittet med stjernefelt, nordlys og glødeeffekter),
  **UI v2** (den aller første UI-fornyelsen: marineblå gradient, stjernehimmel og
  glødende knapper, helt uten animasjoner) eller **UI v1** (utseendet før
  UI-fornyelsen: ensfarget mørk bakgrunn, flate knapper, ingen effekter). Alle
  kortene viser en liten forhåndsvisning; valget slår inn umiddelbart på hele
  grensesnittet (spillområdet **og** sidefeltet) og lagres.

Innstillingene lagres permanent i `settings.json`. I **énspiller** styrer begge
tilordningene samme figur (standard: WASD *og* piler), i **flerspiller** én hver.
Alle spill har **lydeffekter** (prosedyregenerert, ingen ekstra filer nødvendig)
som kan dempes globalt.

### Prosjektstruktur

```
install-python.bat  Windows-oppsett: Python 3.13 + .venv + pygame
start.bat            Startskript (Windows)
start.sh             Startskript (Linux / macOS / Git Bash)
pyinstall.bat        EXE-bygg (Windows): pakker alt i builds\PyGameZ.exe
main.py              Tkinter-grensesnitt, Pygame-innbygging, sentral spilløkke
game_base.py         Spill-baseklasse (update/draw/handle_event) + InputEvent + hjelpere
settings.py          Laster/lagrer innstillinger (lyd/haptikk/tastebindinger/spillalternativer med kontrollregler) (JSON)
audio.py             Prosedyregenererte lydeffekter, musikksløyfer + gamepad-rumble
menu.py              Språk-, forspill- (modus) og alternativskjerm (lyd/styring)
highscore.py         Laste/lagre rekorder (del i mem.json)
store.py             Sentral lagringsfil mem.json (seksjoner: mem, highscores, stats, achievements + spillfremgang), atomisk med .bak-kopi
stats.py             Spillerstatistikk (partier, spilletid, seire, rekorder) per spill
achievements.py      Prestasjoner: definisjoner, opplåsing, varsel (toast)
progress.py          Skjerm for prestasjoner & statistikk (to faner, rullbar)
replay.py            Opptak og arkiv over replays (replay.json)
replayview.py        Replay-skjerm: arkivliste og avspilling
ugc.py               Eget innhold (minigolfbaner, Geometry Dash-nivåer): lagring, kontroll, eksport/import (ugc.json)
swear.py             Ordfilter for navn og id-er (lang/swear/*.yml, alle 14 språk)
filepick.py          Fildialoger ("Eksporter som ...", "Importer")
prestige.py          Prestige-system for Snake
competitive.py       Finjustering av Snakes Competitive-modus (nivåer, enarmet banditt, gambleepler)
ngb.py               Visuell tilpasning («mods»): hodefarge + koordinatrutenett + meny (mem-ngb.json)
lamabank.py          Lama-banken: felles sjetongkonto for Blackjack, Poker og Casino (seksjonen casino i mem.json)
seedrand.py          Tilfeldighetsgenerator med bit-identiske tall i Python og nettleseren (dagens moduser, nye oppgaver)
i18n.py              Oversettelsesmotor (laster lang/*.json, t("nøkkel"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Språkstrenger (én plassholdernøkkel per tekst)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Ordfilterlister per språk (regex, .yml)
lamawiki/
  lamawiki.py          Innebygd wiki (søk, kategorier, artikkelgjengiver)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-innhold (én side per spill + generelle sider)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Bygger ordlistene til Wordle på nytt (ordbøker + frekvenslister)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 bokstaver), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 bokstaver), 14 språk
devtools/            Utviklerverktøy (pakkes ikke inn i .exe-filen)
  merge_staging.py           Legger oversettelser og wikisider fra devtools/staging/ inn i alle 14 språkfilene
  build_chess_puzzles.py     Bygger de 200 sjakkoppgavene fra Lichess' oppgavedatabase (CC0)
  build_sudoku_killer.py     Genererer de 400 entydig løsbare Killer Sudokuene
  build_crossyroad_models.py Skriver voxel-modellene til Crossy Road for nettversjonen
  build_geodash_levels.py    Bygger de 8 Geometry Dash-banene og beviser med løsningsalgoritmen at hver kan klares, mynter inkludert
  build_geodash_solver.py    Løsningsalgoritme med spillets ekte stegkode (løsninger i geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Nivådata: snake-comp.json, chess-puzzles.json (+ kilde-README), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Samlet revisjon (input, seedrand Python = JS, lagring, språkfiler, forspillskjermer) + alle audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless-revisjoner per spill
  newgames_audit.py  blockjump_audit.py
```

Det valgte språket lagres i `mem.json` (i `mem`-delen, ved siden av
`highscores`-delen i samme fil) og lastes automatisk ved neste oppstart.

**Kilder og lisenser:** de 200 sjakkoppgavene kommer fra
[Lichess' oppgavedatabase](https://database.lichess.org/#puzzles) (lisens
**CC0 1.0**, fritt tilgjengelig – takk til lichess.org!); detaljer står i
`games/levels/chess-puzzles.README.md`. Kildene til Wordle-ordlistene står i
`woordlistz/README.md`.

### Plattformmerknader

Visningen kjører **off-screen**: pygame bruker dummy-videodriveren
(`SDL_VIDEODRIVER=dummy`), så den gjengir til en surface, og hvert bilde tegnes
som et bilde inn i en Tkinter-widget. Det finnes **ikke noe nativt SDL-vindu** som
kan kjempe med Tkinter om størrelse/posisjon. Resultatet er at vinduet oppfører
seg likt og stabilt overalt:

- **Windows**: prosessen gjøres i tillegg DPI-bevisst slik at visningen holder seg
  skarp på skalerte skjermer (125/150/200 %) og ikke «rister».
- **Linux/X11 og Wayland**: fungerer uten spesialtilfeller (ingen `SDL_WINDOWID`).
- **macOS**: fungerer også (tidligere ble det innebygde vinduet ikke vist her i det
  hele tatt).

---

### Installasjonsveiledning

Krav: **Python 3.9+** (anbefalt 3.12 eller 3.13) og **pygame ≥ 2.6**.

#### Windows (anbefalt: automatisk)

1. Åpne prosjektmappen og dobbeltklikk **`install-python.bat`**. Skriptet
   - sjekker om **Python 3.13** finnes, og installerer det ellers via
     **winget** (`winget install Python.Python.3.13`),
   - oppretter det virtuelle miljøet **`.venv`**,
   - installerer **pygame** fra `requirements.txt`.
2. Start deretter samlingen med **`start.bat`** (dobbeltklikk).

> Merk: hvis skriptet melder «ennå ikke tilgjengelig i dette vinduet», ble Python
> nettopp installert – åpne bare **et nytt terminalvindu** og kjør
> `install-python.bat` på nytt. Hvis **winget** ikke er tilgjengelig, installer
> Python 3.13 manuelt fra <https://www.python.org/downloads/> og huk av for
> **«Add python.exe to PATH»**.

#### Windows / Linux / macOS (manuell)

```bash
# 1. Sjekk Python (3.9+)
python --version

# 2. Opprett og aktiver et virtuelt miljø
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Installer avhengigheter
pip install -r requirements.txt
#   eller:  pip install "pygame>=2.6" (eller pygame-ce)
#                                   pip install pygame-ce
# 4. Start
python main.py
```

#### Linux / macOS med start.sh

```bash
# Sett opp Python + venv som over (steg 2 og 3), deretter:
chmod +x start.sh      # én gang, hvis ikke allerede kjørbar
./start.sh
```

På Linux installerer du Python ved behov via pakkebehandleren, f.eks.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); på macOS
f.eks. `brew install python`.

#### Bruke en annen Python-versjon

`install-python.bat` setter opp Python 3.13 som standard. Hvis du foretrekker 3.12
(eller en annen versjon), endre linjen `set "PYVER=3.13"` i filen til ønsket
versjon og winget-ID-en tilsvarende (`Python.Python.3.12`).

#### Bygge en frittstående EXE (Windows)

```bat
pyinstall.bat         :: bygger builds\PyGameZ.exe (alt i én fil)
```

`pyinstall.bat` bruker `.venv` (og oppretter den ved behov), installerer
**PyInstaller** automatisk og pakker hele spillet – Python, pygame, alle spill,
språk, wiki og logoer – i **én enkelt `PyGameZ.exe`** i mappen **`builds\`**.
Filen kjører på hvilken som helst Windows-PC uten Python installert og kan
kopieres fritt. Innstillinger og rekorder (`settings.json`, `mem.json`,
`mem-ngb.json`) opprettes ved siden av .exe-en mens du spiller.

#### Feilsøking

- **`pygame` ikke funnet** → er venv aktivert? Gjenta steg 3
  (`pip install -r requirements.txt`).
- **`python` gjenkjennes ikke (Windows)** → Python ble installert uten «Add to
  PATH»; installer på nytt og huk av boksen, eller bruk `py` i stedet for `python`.
- **Ingen lyd** → sjekk «Lyd» i alternativene; haptikk fungerer bare med en kontroller.
- **Vindu/innbygging på Linux** → se *Plattformmerknader* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ til toppen / back to top</a></b></div>

---

<a name="-svenska"></a>

## 🇸🇪 Svenska

En spelsamling för skrivbordet i Python: **Tkinter** står för fönstret och menyn,
**Pygame** bäddas in som spelyta inuti Tkinter-fönstret. Fyrtiosex spel med
gemensamma inställningar, fritt ombindbara kontroller, topplistor, procedurella
ljudeffekter och, för vissa titlar, ett flerspelarläge. Gränssnittet är
**flerspråkigt** – **14 språk** (tyska / engelska / franska / spanska /
portugisiska / polska / turkiska / danska / norska / svenska / finska / tjeckiska /
slovenska / kroatiska); språket väljs på en **välkomstskärm** vid första starten,
där du även kan ställa in **upplösning** och **ljud** (avstängt som standard);
förutom de tre huvudspråken gömmer sig alla övriga (spanska, portugisiska och de
nio ytterligare, svenska bland dem) bakom knappen **”Mer”**. Allt kan ändras
senare i inställningarna.

### Snabbstart

#### Windows

```bat
install-python.bat    :: engångskörning: ställ in Python 3.13 + .venv + pygame
start.bat             :: starta spelsamlingen
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # startar med .venv, annars systemets python3
```

`start.bat` / `start.sh` använder automatiskt den virtuella miljön `.venv` om den
finns, annars systemets Python. En utförlig steg-för-steg-guide finns längst ner
under **[Installationsguide](#installationsguide)**.

### Spelen

| Spel         | Lägen           | Kort beskrivning |
|--------------|-----------------|------------------|
| **Snake**    | 1 / 2 spelare   | Deluxe-Snake med 2D- och 3D-vy, boost, 6 spellägen (inkl. Competitive), gyllene äpplen och prestige |
| **Pong**     | 1 / 2 spelare   | Klassikern mot AI eller spelare 2, växlingsbart rörelseläge |
| **Air Hockey** | 1 / 2 spelare | 2D-fysik med impulsöverföring, musstyrning, AI och power-ups |
| **Tic-Tac-Toe** | 1 / 2 spelare | m,n,k-spel på 3x3 till 9x9, tre AI-nivåer **eller** lokalt X mot O |
| **Breakout** | 1 spelare       | Brick-breaker med olika stentyper, power-ups, combos och många banor |
| **Tetris**   | 1 / 2 spelare   | Moderna Guideline-regler (SRS, reserv, förhandsvisning av 5 bitar, T-Spins): Maraton, Sprint 40, Ultra 2:00, Versus mot AI (3 nivåer) eller två spelare med skräprader |
| **Invaders** | 1 spelare       | Space Invaders: rensa vågorna, skydda dina liv |
| **Asteroids** | 1 / 2 spelare  | Tröghetsfysik, vågor, UFO:n, power-ups, hyperrymd - solo eller co-op-duell |
| **Pac-Man**  | 1 spelare       | Trogen klon: 4 spök-AI:n, kraftpiller, tunnel, frukt, banor |
| **Flappy Bird** | 1 spelare    | Gravitationsflygning genom rör, mynt, sköld, dag/natt, medaljer |
| **Doodle Jump** | 1 spelare    | Automatiskt hopp uppåt, plattformstyper, fjädrar, propeller, monster |
| **2048**     | 1 spelare       | Sifferpussel där man skjuter brickor, från 3x3 till 8x8: Klassiskt, Tidsattack och Oändligt, ångra, mjuka animationer, sparade partier |
| **Minesweeper** | 1 spelare    | Klassikern med säkert första klick, chording, smiley och bästa tider |
| **Sudoku**      | 1 spelare    | 4 varianter (Klassisk, X-sudoku, Killer, Mini 6x6) med 400 banor var, dagens sudoku, upp till 3 stjärnor per bana, 4 hjälplägen, ångra, sparat spel |
| **Frogger**     | 1 spelare    | Väg + flod + 5 vikar, bonusfluga, krokodiler, tidsgräns, 3 svårighetsgrader |
| **Memory**      | 1 / 2 spelare | Hitta par på 4x4 upp till 8x6, vändningsanimation, solopoäng eller duell |
| **Patiens**     | 1 spelare    | 5 varianter (Klondike, Spider, FreeCell, Pyramid, TriPeaks) med dra och släpp och ångra |
| **Aim Trainer** | 1 spelare    | Avslappnat 3D-prickskytte: musen styr kameran, 4 lägen (precision/reflex/rörliga/chill), 3 teman inkl. ett svart hål |
| **Fyra i rad** | 1 / 2 spelare | Klassikern med fallanimation: 3 AI-nivåer (minimax) eller en lokal duell |
| **Tankduell**    | 1 / 2 spelare | 2D-arenaduell med studsande skott, power-ups, 4 arenor, AI med 3 nivåer |
| **Blackjack**    | 1 spelare    | Casino-blackjack med sko på 4 lekar, dubbla/dela och blackjack 3:2; spelas med lamamarker från den gemensamma Lamabanken |
| **Tunnel Racer** | 1 spelare    | 3D-neonrörsflygning: oändligt läge + 30 banor, styrning med tangenter eller mus, motion blur |
| **3D-labyrint**  | 1 spelare    | Förstapersons-raycaster (Wolfenstein-stil) med 50 seed-genererade banor, orbs, minikarta - eller en 2D-vy uppifrån |
| **Reversi**      | 1 / 2 spelare | Othello på 8x8: fånga och vänd brickor, 3 AI-nivåer (minimax) eller en lokal duell |
| **Yatzy**        | 1 / 2 spelare | Tärningsklassiker med 13 kategorier, övre bonus och yatzy; jakt på topplistan eller hotseat för 2 |
| **Wordle**       | 1 spelare    | Gissa ord med 4 till 7 bokstäver: Oändlig, Dagens ord, Dordle och Quordle, svårt läge, färgblind-palett, statistik med stapeldiagram, dela resultatet, riktiga ordlistor på 14 språk |
| **T-Rex Runner** | 1 spelare    | Oändlig ökenlöpning: variabelt hopp, ducka, kaktusar & pterodaktyler, dag/natt-cykel, stigande tempo, 3 svårighetsgrader |
| **Dam**          | 1 / 2 spelare | 3 regeluppsättningar (tysk 8×8, internationell 10×10, checkers), slagtvång & flygande dam, 3 AI-nivåer (minimax) eller en lokal duell |
| **Poker**        | 1 spelare    | 3 valbara varianter: Texas Hold'em mot AI, 5 Card Draw och Video Poker; satsningsrundor, blinds, lamamarker från den gemensamma Lamabanken |
| **Schack**       | 1 / 2 spelare | Fullständiga regler, Chess960 och schackklocka, 6 AI-nivåer, 200 problem från Lichess-databasen, ångra/tips, draglista, PGN-export eller lokal duell |
| **Kvarn**        | 1 / 2 spelare | Placera-/flytta-/flyga-faser, kvarnar & slag, valfri flygregel, 3 AI-nivåer eller en lokal duell |
| **Simon**        | 1 / 2 spelare | Senso-minnesspel: lägena Klassiskt/Speed/Reverse/Mixat + Duell, ljud av/på/mixat, 4/6/9 knappar, bästa per läge |
| **Biljard**      | 1 / 2 spelare | 8-ball, 9-ball & övning i 2D, fast 3D-vy eller fritt roterbar 3D-kamera; mjuk fysik, sikthjälp, 3 AI-nivåer |
| **Skjutpussel**  | 1 spelare    | 15-pusslet i 3x3/4x4/5x5: skjut de numrerade brickorna in i luckan, styrning med klick eller pilar, poäng från drag & tid |
| **Mastermind**   | 1 spelare    | Knäck den hemliga färgkoden (3 lägen: 4×6, klassiskt, 5×8), svarta/vita ledtrådsstift, oändlig streak som topplista |
| **Bubble Shooter** | 1 spelare  | Puzzle Bobble-klon: skjut matchande färger i grupper om tre, väggstudsar, fallande kluster, 3 svårighetsgrader |
| **Hänga gubbe**  | 1 spelare    | Gissa ordet innan galgen är färdig; skärmtangentbord, ordlistor per språk, 3 längdlägen, oändlig streak |
| **Block Jump**   | 1 spelare    | 3D-plattformsspel i Minecraft-stil: texturerad voxelvärld med Steve-figur, stegar, staket & slime-block, första-/tredjepersonskamera, motion blur, seed-genererade parkourbanor |
| **Tower Defense** | 1 spelare    | Slå tillbaka oändliga vågor på 4 kartor: upp till 11 torntyper med uppgraderingar, försäljning & A/B-specialisering, bossar, 3 lägen, aktiva förmågor |
| **Minigolf**    | 1 / 2 spelare | 360 banor på 40 slingor (18 handbyggda, 342 genererade): sand, ramper, vatten, kuddar, väderkvarnar & vandrande block; scorekort med par och hole-in-one-bonus; **egen hålredigerare** med 15 objekttyper, 12 mallar och delning som `.lamapgzmap` |
| **Pinball**     | 1 / 2 spelare | Flipperspel med 3 bord: bumpers, slingshots, mål, L-A-M-A-banor, multiboll med jackpot, bollräddning, knuff & tilt |
| **Bowling**     | 1 / 2 spelare | 10 frames med officiell strike/spare-räkning, äkta kägelfysik, skruv och bana i perspektiv, 3 svårighetsgrader |
| **Crossy Road** | 1 spelare     | Oändliga hopp över ängar, vägar, floder och järnvägsspår i isometrisk voxelstil: dag/natt, örn, 10 figurer att köpa, dagens bana |
| **Geometry Dash** | 1 spelare   | Rytmplattformspel med kub, skepp, boll, UFO och våg: 8 banor från Lätt till Demon med 3 hemliga mynt var, träningsläge, soundtrack per bana; **baneditor** med delning som `.lamapgzlevel` |
| **Battleship**  | 1 / 2 spelare | Sjöslag på 10x10: flottan placeras med dra och släpp, 3 regelreglage (beröring, salva, skjut igen), AI med 3 nivåer eller lokal duell med överlämningsskärm |
| **Casino**      | 1 spelare     | Europeisk roulette med alla klassiska insatser och Lamaautomat (5 hjul, 10 vinstlinjer, wild, gratissnurr); ett gemensamt konto med lamamarker för Blackjack och Poker |

**Flerspelarläge (2 spelare lokalt)** finns för **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (co-op-duell)**,
**Memory (duell)**, **Fyra i rad**, **Tankduell**, **Reversi**, **Yatzy**,
**Dam**, **Schack**, **Kvarn**, **Simon (duell)**, **Biljard**, **Minigolf**,
**Pinball**, **Bowling** och **Battleship** (med en överlämningsskärm som döljer
flottorna) - totalt 20 spel. Läget väljs direkt på förspelsskärmen
(*Enspelarläge / Flerspelarläge*); Tetris har dessutom **Versus mot AI**.
Webbversionen är endast för en spelare.

#### Funktionsdetaljer per spel

**Snake**
- **NYTT - 3D-vy** (tangent **V** i setup eller klick på *Vy*): brädet renderas
  som en 3D-scen i realtid - en **förföljarkamera** svävar bakom ormen och
  styrningen sker **relativt vyn** (vänster/höger = svänga, två snabba tryck =
  U-sväng). Med avståndsdimma, stjärnhimmel, schackrutigt golv, kantväggar,
  roterande matkristaller, 3D-partiklar och kameraskak vid krock; efter game over
  kretsar kameran långsamt kring ormen. Boost vidgar synfältet. Tillgängligt i 3D:
  *Klassiskt* och *Hinder* (väggarna är alltid solida där, 3D finns bara i
  enspelarläge). Vyn sparas i `settings.json`.
- **NYTT - 3D-kameraalternativ** (i 3D-setupen, klicka på raden *3D-kamera /
  smooth shake* eller tangent **K**): en särskild meny med **smooth shake**
  (mjukare kamera, betydligt mindre skakighet vid rörelse/svängning), justerbart
  **synfält (FOV)** och **kamerahöjd**, plus en växel för **skak vid svängning**
  (skärmskak vid vänster-/högersvängar på/av). Allt sparas i `settings.json`.
- **Boost**: **håll** boost-tangenten = turbo (dubbel hastighet), förbrukar
  uthållighet (stapel); när den är tom stängs boosten av och laddas upp igen.
  Standard S1 = Mellanslag/vänster Shift, S2 = Enter/höger Shift.
- **6 spellägen** (valbara i setup): *Klassiskt*, *Speed Rush* (blir snabbare för
  varje äpple), *Hinder* (dödliga block), *Portaler* (teleportörpar), *Time Attack*
  (60 sekunder, så många äpplen som möjligt) och *Competitive* (se nedan).
- **NYTT - Competitive** (enspelarläge): oändligt läge med en **nivåstege** - du
  börjar med exakt **ett** äpple och kan inte få fler i början; ju fler äpplen du
  samlar totalt, desto högre **nivå**, vilket hela tiden lägger till ännu ett
  samtidigt äpple på fältet och höjer poängmultiplikatorn.
  **Blå äpplen** öppnar en **enarmad bandit**: din längd är insatsen, hjulresultatet
  multiplicerar eller krymper den och får kortvarigt **extra äpplen** att dyka upp
  (jackpott vid tre lika symboler). **Lila äpplen** (spel) sätter en del av din
  **storlek** på spel och multiplicerar den delen slumpmässigt, resten förblir säker
  (ny storlek = storlek·(1-p) + storlek·p·faktor): **normal** satsar fasta 50 % med
  **x0.5 .. x1.5**, **HARDCORE** är mer riskabelt med en **75-90 %**-insats och
  **x0.25 .. x2.25**. Din **storlek** visas som ett **decimaltal uppe till vänster**
  och förs vidare exakt, så att vidare satsningar bygger på den. Det finns
  **15 nivåer** (multiplikator upp till x16, upp till 16 äpplen samtidigt); nivåerna
  ligger i `games/levels/snake-comp.json` och kan utökas där utan att röra koden,
  resten av finjusteringen ligger i `competitive.py`.
- **NYTT - HARDCORE** (växel i Competitive-setupen, tangent **H**): varje **boost
  äter av ormens längd**; en rödglödande **HARDCORE-text** markerar läget. Endast i
  Competitive; längden faller aldrig under minimum. Sparas i `settings.json`.
- **Gyllene äpplen** (tillfälliga) ger massor av poäng och fyller genast på boosten.
- Valfritt: **väggar man passerar genom** (wrap-around), bonusäpplen, **prestige**
  (enspelarläge, tangent **P**).
- **NYTT - Personalisera** (penselknapp längst upp till höger i setup, eller tangent
  **C**): en rent visuell meny (”mods” som *aldrig* ändrar spelet) med två flikar:
  - **Huvud**: ormens **huvudfärg** - 4 blå-turkosa förval (från mer blått till mer
    turkost), rött, orange och en **egen färg** via RGB-reglage.
  - **Rutnät (vägvisare)**: lägger ett **koordinatrutnät** över fältet - **radnummer**
    (vid vänster och höger kant) och **kolumnbokstäver** (upptill/nedtill). Så på stora
    bräden ser du direkt att t.ex. äpplet vid *8a* ligger på samma rad *8* som din egen
    position *8z*. Färgsekvensen (5 förval + två egna färger A/B) bestämmer färgtemat.
  - **Banner**: slå **på/av** multiplikatorbannern (t.ex. från det lila äpplet) och
    justera dess **storlek** (mindre/större) och **opacitet** (mer genomskinlig) - med
    förhandsvisning i realtid.
  Allt lagras i `mem-ngb.json`; all visuell personalisering går via modulen `ngb.py`.
- Utseende: rundad orm med ögon (huvudet är turkost som standard), boost-sken, partiklar.

**Pong**
- Enspelarläge mot AI, flerspelarläge = spelare 2 till höger. Först till 5 poäng.
- **Rörelseläge som kan växlas per kontrolluppsättning**: *Kontinuerligt* (tryck en
  gång -> fortsätter röra sig, standard) eller *Håll* (rör sig bara medan tangenten
  hålls nere). Växla: **X** = kontrolluppsättning 1, **N** = kontrolluppsättning 2
  (sparas i `settings.json`).
- Bollfysik med acceleration och vinkel beroende på träffpunkten.

**Air Hockey**
- **Äkta 2D-fysik**: runda klubbor och puck med impulsöverföring - pucken tar upp
  klubbans hastighet vid träff; väggar med studs, lätt isfriktion, mål som öppningar
  i sidoväggarna.
- **Musstyrning** i enspelarläge: klubban följer musen (valfri tangent växlar tillbaka
  till tangentbord). Tangentbord: riktningstangenter i 8 riktningar, flerspelarläge =
  S1 vänster (WASD), S2 höger (IJKL).
- **AI med tre nivåer** (Lätt/Medel/Svår): försvarar sitt eget mål, anfaller i sin egen
  halva och rundar pucken för att undvika självmål.
- **Power-ups** (kan stängas av): *XL* (större klubba), *MÅL* (motståndarens mål krymper),
  *>>* (snabbare klubba) - de tillhör den spelare som senast rörde pucken.
- Setup: svårighetsgrad, **mål för vinst** (3/5/7/10), power-ups på/av (sparas i
  `settings.json`). Efter varje mål gör den som släppte in avspark.
- Utseende: ljusspår efter pucken, partiklar, pulserande målgap, effektmarkeringar.

**Tic-Tac-Toe**
- Setup: svårighetsgrad (Lätt/Medel/Svår) och brädstorlek 3x3..9x9; vinstlängd
  K = 3 (3x3), 4 (4x4), annars 5.
- **1 spelare** mot AI (Svår på 3x3 är oslagbar) **eller 2 spelare** lokalt
  (X mot O, turas om med klick). Vid game over: Enter/klick = ny omgång,
  **S** = inställningar.

**Breakout**
- Stentyper: Normal, **Stål** (oförstörbar), **Bomb** (exploderar), **Guld**
  (extrapoäng).
- Power-ups: laser, eldklot, klibbig, sköld, mynt med flera; **combomultiplikator**.
- Effekter: partiklar, bollspår, skärmskak, poäng-popups, många banmönster.
- Setup: **1/2/3** = svårighetsgrad, **Vänster/Höger** = bollfärg, **Upp/Ner** =
  startbana, **M** = uppställning. Spel: mus/pilar, **Mellanslag** skjuter iväg bollen
  (avfyrar lasern), **P/Esc** = paus.

**Tetris**
- **Moderna Guideline-regler**: 10x20-plan, bitar ur en **7-påse**, **SRS-rotationssystemet** med äkta
  wall kicks (även för I-biten), rotation åt båda hållen, **reserv** (en gång per bit),
  **förhandsvisning av 5 bitar**, skuggbit och **lock delay** (0,5 s, högst 15 nollställningar).
- **Tre lägen** på förspelsskärmen: *Solo*, *Mot AI* och *2 spelare*. Solo erbjuder i inställningarna
  **Maraton** (startnivå 1-15, räknas till rekordet), **Sprint 40 rader** (bästa tid) och **Ultra 2
  minuter** (bästa poäng); rekorden sparas i sektionen `tetris` i `mem.json`.
- **Guideline-poäng**: singel till Tetris, **T-Spins** (hela och mini), **Back-to-Back** (x1,5),
  **combos** och **Perfect Clear** - med texter på skärmen, rensningsanimation, hard drop-spår,
  partiklar och nivåhöjningseffekt.
- **Versus med skräprader**: rensade rader skickar skräp till motståndaren (Tetris = 4, T-Spin Double
  = 4 …), inkommande skräp varslas i en varningsstapel och **kvittas** mot dina egna attacker; båda
  planen får samma bitordning. **AI:n** finns i 3 nivåer och tempot ökar var 40:e sekund.
- **Styrning**: Vänster/Höger med egen **DAS/ARR** (ställs in i inställningarna), Upp = rotera åt
  höger, Ner = soft drop, Åtgärd = hard drop; **C**/Shift = reserv, **Z**/**Y** = rotera åt vänster,
  **X** = rotera åt höger. Två spelare: spelare 1 lägger i reserv med **Q** och roterar åt vänster med
  **E**, spelare 2 med **höger Shift** / **höger Ctrl**. Efter spelet: **R** = igen, **S** =
  inställningar.

**Invaders** – två lägen (valbara på förspelsskärmen):
- **Klassiskt**: det klassiska alien-blocket; sedan valbart i setup-skärmen:
  **Rörelse** (bara vänster/höger *eller* fritt med WASD) och **Sikte** (alltid uppåt
  *eller* mot **musen** – då skjuter du dit muspekaren är). Förstörda aliens släpper
  ibland power-ups.
- **Arena (fri)**: fri rörelse i alla riktningar, fiender strömmar in från alla kanter;
  man siktar i rörelseriktningen, byt vapen med **1–4**.
Gemensamt: nivåsystem med en **boss** var fjärde bana, fyra vapen (blaster,
spridningsskott, snabbeld, laser), power-ups (extraliv, sköld, vapenuppgradering),
explosionseffekter, topplista.

**Asteroids**
- **Tröghetsfysik**: upp = framdrivning i blickriktningen, vänster/höger = rotera,
  skeppet fortsätter driva (lätt dämpning); allt wrappar runt skärmkanterna. Klassiskt
  **vektorutseende** med drivlåga och stjärnhimmel; varje sten har sin egen slumpmässiga
  polygonform.
- Stenar splittras i två mindre (3 storlekar, **20/50/100 poäng**), **vågor** med växande
  antal och en banner-annonsering.
- **UFO** (kan stängas av): korsar skärmen med jämna mellanrum och siktar på skeppen
  (siktfel beror på svårighetsgrad) - 200 poäng för att skjuta ner det.
- **Power-ups** (kan stängas av), släpps av förstörda stenar: **S**köld (6 s osårbar),
  **T** = trippelskott, **R** = snabbeld.
- **Hyperrymd** (ner-tangent): nödhopp till en slumpmässig position med 4 s återladdning -
  och 12 % risk att sprängas vid ankomst.
- 3 liv, säker återuppståndelse med osårbarhetsblink, **extraliv var 5000:e poäng**;
  explosionspartiklar och kameraskak.
- **Co-op-duell** (flerspelarläge): båda skeppen flyger samtidigt med separata liv och
  poäng - den med flest poäng vinner.
- Setup: svårighetsgrad, UFO:n på/av, power-ups på/av (sparas i `settings.json`).

**Pac-Man**
- **Klassisk 28x31-labyrint** i neonlook med prickar, 4 kraftpiller, tunnelvarp på
  sidorna och ett spökhus i mitten.
- **Fyra spöken med de ursprungliga beteendena** (mål-ruta-AI): *Blinky* jagar direkt,
  *Pinky* lägger sig i bakhåll (4 rutor framför), *Inky* använder en vektor genom Blinky,
  *Clyde* drar sig undan när han är nära.
- **Scatter/chase-faser** växlar (spökena vänder vid varje byte); ett **kraftpiller** gör
  spökena blå och ätbara (kedja 200/400/800/1600), sedan återvänder ögonen till huset.
- Spökhus med **stegvis frisläppning**, **frukt**bonusar (per bana), **3 liv**, **extraliv
  vid 10 000**, nivåsystem (blir snabbare), dödsanimation, READY/GAME OVER-skärmar.
- Setup: **svårighetsgrad** (Normal/Svår/Extrem) – spökfart & frightened-tid.
- Styrning: **pilar eller WASD**.  Enter = ny, S = setup.

**Flappy Bird**
- **Gravitationsfysik**: Mellanslag / Upp / W / **musklick** får fågeln att flaxa;
  den lutar beroende på stig-/fallhastighet.
- Oändliga **rörpar** med en lucka (+1 per rör); **mynt** (bonus) och en **sköld**-power-up
  (överlever en träff) dyker upp i luckorna.
- **Dag/natt-teman** växlar med poängen; drivande moln (parallax), rullande mark.
- Svårighetsgrad (Lätt/Normal/Svår): luckstorlek, tempo, rörens avstånd – luckan smalnar
  av något när poängen stiger.
- **Medaljer** (brons/silver/guld/platina) vid game over, kraschanimation med kameraskak,
  topplista.

**Doodle Jump**
- Doodlern **hoppar automatiskt** vid landning; du styr bara vänster/höger (med tröghet),
  kanterna wrappar runt, och kameran rullar uppåt medan du klättrar.
- **Plattformstyper**: grön (normal), blå (rörlig), brun (går sönder), vit (försvinner).
  **Fjädrar** ger ett superhopp, **propellerhatten** bär dig uppåt en kort stund (och gör
  dig oövervinnerlig).
- **Monster**: beröring är dödlig – men du kan **skjuta** dem med Upp / Mellanslag
  (bonuspoäng).
- Poäng = uppnådd höjd; svårigheten stiger med höjden. Topplista.
- Styrning: vänster/höger = flytta, Upp / Mellanslag = skjuta.

**2048**
- **Egen inställningsskärm** med brädstorlekar från **3x3 till 8x8** och tre lägen: *Klassiskt* (mål
  2048, sedan ”Spela vidare?”), *Tidsattack* (3 minuter, klockan startar vid första draget) och
  *Oändligt*.
- **Mjuka animationer**: brickor glider, slås ihop med ett ”pop” och växer fram; poäng-popups, gnistor
  från 128, en tryckvåg från 2048 och nya färger ända upp till 131072. Inmatningar under en animation
  sparas och körs direkt efteråt.
- **Ångra** (av / 3 per parti / obegränsat, tangent **U** eller Backspace) - den som använder det
  spelar utan rekord och utan brick-prestationer.
- **Spara och fortsätt**: det pågående partiet sparas automatiskt per storlek och läge; bästa poäng och
  största bricka per storlek/läge finns i sektionen `g2048` i `mem.json`.
- Styrning: pilar/WASD eller **svep** med mus/pekplatta, **R**/**N** = nytt parti, **Tab** =
  inställningar. Rekordet räknas bara i **4x4 Klassiskt** utan ångra.

**Minesweeper**
- Tre nivåer: **Nybörjare** (9x9, 10 minor), **Mellan** (16x16, 40), **Expert**
  (30x16, 99) - **bästa tiden per nivå** sparas och visas i setupen.
- **Första klicket är alltid säkert** (minorna placeras ut efteråt, 3x3-området runt
  klicket förblir fritt).
- **Vänsterklick** = avslöja, **högerklick** = flagga (valfritt med frågetecken-cykel),
  **F** = flagga under muspekaren, **R** = nytt spel.
- **Chording**: att klicka på en uppfylld siffra avslöjar de återstående grannarna.
- Klassiskt HUD: minräknare, **klickbar smiley** (förvånad/solglasögon/död), timer;
  felaktiga flaggor stryks över i slutet, konfetti vid vinst.
- Poäng = nivåns grundvärde minus sekunder.

**Sudoku**
- **4 varianter** med **400 banor** var (4 svårighetsgrader x 100): *Klassisk* (de kända
  seed-banorna - lösta banor förblir avbockade), *X-sudoku* (båda diagonalerna innehåller varje siffra
  exakt en gång), *Killer* (streckade burar med summa; 400 banor genererade i förväg) och *Mini 6x6*.
  Alla pussel har en **unik lösning** - bana 12 i ”Svår” är samma pussel på varje dator.
- **Dagens sudoku**: ett pussel om dagen för alla, likadant på datorn och i webbläsaren;
  svårighetsgraden beror på veckodagen (från måndag Lätt till lördag Expert), och den som löser varje
  dag bygger en svit.
- **Upp till 3 stjärnor per bana** (löst · utan fel och tips · dessutom under måltiden) och **bästa
  tid** i banvalet; **påbörjade pussel** sparas automatiskt och fortsätter nästa gång.
- **4 spellägen** (väljs innan start) med en poängmultiplikator: **Klassiskt** (x2,0 - inga
  hjälpmedel), **Anteckningar** (x1,5 - + blyertsanteckningar och automatiska kandidater), **Komfort**
  (x1,0 - + fel i rött, konflikter och felaktiga bursummor markerade, korrekta inmatningar låses fast),
  **Assistent** (x0,7 - + tips-tangent, max. 3). Med **3-felsgränsen** aktiverad (setup-alternativ)
  avslutar det tredje felet spelet.
- Styrning: pilar/WASD = ruta, **1-9** = siffra (även numeriskt tangentbord), **0/Delete/högerklick** =
  radera, **U**/**Z** = ångra, **Y** = gör om, **N** = anteckningar, **C** = fyll i kandidater
  automatiskt, **H** = tips, **M** = färgmarkör, **R** = starta om banan, **Q** = banval. Inmatning
  ”siffra först” (setup, **I**) och en **räknare för kvarvarande siffror** under varje siffra; fullt
  spelbart med musen. När spelet är slut visar **A** hela **lösningen**.
- Poäng = (bas för variant och svårighetsgrad - tid - fel - tips) x lägets multiplikator; alla
  varianter och dagens sudoku räknas till rekordet.

**Frogger**
- 5 trafikfiler (bilar/lastbilar) och 5 flodbanor (stockar, sköldpaddor som **dyker** på
  högre banor); 5 hemvikar upptill - fyll alla = nästa bana, allt går snabbare.
- Extra: **bonusfluga** (+200) i tomma vikar, **krokodiler** besätter vikar på högre banor,
  **tidsgränsstapel** per groda, extraliv vid 10 000.
- 3 svårighetsgrader (tempo, trafiktäthet, tid); poäng per ny rad, vik = 50 + tidsbonus,
  klar bana = +1000.

**Memory**
- Brädstorlekar **4x4, 6x6, 8x6**; motiven är form-färg-kombinationer helt ritade med
  primitiver; **vändningsanimation**, felaktiga par vänds tillbaka automatiskt.
- **Solo**: bas - 15 per drag - 2 per sekund (min. 100). **Duell** (lokal): turas om, ett
  par ger ett drag till, flest par vinner.

**Patiens**
- **5 varianter** på förspelsskärmen: Klondike (dra 1/3 som alternativ), Spider (1/2/4
  färger), FreeCell (supermove-gräns), Pyramid (13-par, 2 omgivningar) och TriPeaks
  (±1-kedja med combomultiplikator).
- **Dra och släpp** eller klick-klick, **högerklick** = till grundhögen, **U** = obegränsad
  ångra, **R** = ny giv, Mellanslag = talong.
- Korten renderas utan bildfiler (`games/cards.py`); alla varianter delar en topplista med
  variantspecifika formler.

**Aim Trainer**
- **Äkta mjukvaru-3D** (som Snakes 3D-läge): fast hårkors i skärmens mitt, **direkt
  1:1-mussikte som i en shooter** (pekarfångst: markören fångas inuti fönstret, Esc släpper
  den; justerbar känslighet, obegränsad yaw, pitch ±60°). Vänsterklick skjuter exakt genom
  mitten, med mynningsflamma, spårljus och träffpartiklar.
- **4 lägen**: precision (60 s, 3 klot, precisionsbonus), reflex (30 enskilda mål, statistik
  för reaktionstid), rörliga mål (banor + combomultiplikator upp till x4) och chill
  (oändligt, utan straff, **E** avslutar sessionen).
- **3 teman** (i setupen, sparas): **rymd** med en stjärnsfär, ett **svart hål med en lysande
  ring** och en planet (standard), en neonarena med golvrutnät och synthwave-sol, och en
  inomhusskyttebana.
- Känsligheten kan ändras mitt i spelet med **+/-**; dessutom en **justerbar motion blur**
  (0-80 %) för extra chill-look - båda sparas.

**Fyra i rad**
- 7x6-bräde med **fallanimation**, förhandsvisning vid hovring och en pulserande vinstlinje;
  mus, piltangenter eller direktval **1-7**.
- **3 AI-nivåer** (minimax med alfa-beta-sökning): Lätt missar hot med flit, Medel blockerar
  tillförlitligt, Svår planerar djupt framåt - eller **2 spelare** lokalt på samma enhet.
- Startspelaren växlar varje omgång; topplistan räknar dina **vinster mot AI:n** under en
  session.

**Tankduell**
- 2D-arenaduell: **skotten studsar en gång mot väggarna** (rikoschett) - träffa runt hörn
  (eller dig själv!). Först till 5 omgångar med nedräkning.
- **4 arenor** (Öppen, Kors, Pelare, Labyrint) eller slumpmässig rotation; **power-ups**:
  snabbeld, sköld, trippelskott.
- **AI med 3 nivåer** - den svåra siktar med försprång och studsar skotten mot väggarna med
  flit - eller **2 spelare** på ett tangentbord (S1 WASD+Mellanslag, S2 pilar+Enter).

**Blackjack**
- Äkta casinoregler: **sko med 4 lekar**, dealern stannar på 17, **blackjack betalar 3:2**,
  dealern tjuvkikar vid ess/10; **dubbla** och **en delning** (delade ess får ett kort var).
- **Lamamarker**: Blackjack spelar med kontot i **Lamabanken**, som delas med Poker och Casino (start
  1000, sparas permanent i `mem.json`). Insatsen dras så fort korten ges; under 10 marker tar Enter ett
  **banklån** som fyller på kontot till 1000.
- **Rekord** = högsta nivån på din egen **Blackjack-balans** (1000 plus allt som vunnits och
  förlorats i Blackjack) - vinster i roulette, på automaten eller i poker räknas inte här, och inte
  heller lån.
- Spelas via markerknappar och tangenter (**H**it / **S**tanna / **D**ubbla / dela **X**, **1-4** =
  insats, Backspace = nollställ insatsen, Enter = ge) med kortanimationer; dealerns dolda kort vänds
  nu på riktigt när det visas.

**Tunnel Racer**
- **3D-neonrörsflygning** (mjukvarurenderare som i Aim Trainer): balkar, block och
  **ringportar att träda igenom**, mynt på ideallinjen.
- **Två lägen**: oändligt (farten stiger till ett tak, topplista) och **30 seed-genererade
  banor** med mållinje, tidsbonus och avbockad progression.
- **Tangentstyrning** (standard) eller **direkt musstyrning** (pekarfångst, tangent **C**);
  dessutom justerbar **motion blur** (tangent **B**, 0-80 %) - allt sparas.

**3D-labyrint**
- **Förstapersons-raycaster i Wolfenstein-stil** (DDA, avståndsdimma, sprites) med mouselook
  + WASD, en **minikarta** (tangent **M**) och en grön pulserande utgång - eller en klassisk
  **2D-vy uppifrån** (tangent **V** i setupen).
- **50 seed-genererade banor** som hela tiden växer; utgången ligger alltid vid den punkt som
  är längst från starten, **orbs** längs vägen ger bonuspoäng.
- Poäng: 500 per bana + 100 per orb + tidsbonus; lösta banor bockas av och sessionens summa
  blir topplistan.

**Reversi**
- **Othello på 8x8**: lägg brickor som fångar motståndarens rader och vänd allt som stängs
  in; ogiltiga drag blockeras och en tur utan giltigt drag **passas automatiskt**.
- **Enspelarläge mot AI:n** (3 nivåer: negamax med alfa-beta, positionsviktning + rörlighet)
  **eller en lokal duell**, Svart mot Vitt.
- Giltiga rutor markeras; spela med **musen** eller markeringsramen (pilar + Mellanslag/Enter).
  Varje vinst mot AI:n ger en poäng till topplistan.

**Yatzy**
- **Tärningsklassiker**: 5 tärningar, upp till 3 kast per tur, **håll** tärningar individuellt,
  boka sedan en av de **13 kategorierna** (med en förhandsvisning av möjliga poäng).
- Komplett poängprotokoll: övre sektion med **63-poängsbonus (+35)**, tretal/fyrtal, kåk,
  liten/stor stege, **yatzy (50)** och Chans.
- **Enspelarläge som jakt på högsta totalen**, eller **hotseat för 2 spelare** med två protokoll
  sida vid sida; spela med musen eller tangenter (Mellanslag, 1-5, pilar, Enter).

**Wordle**
- Gissa det dolda ordet; färgåterkoppling (grön/gul/grå) med korrekt **räkning av dubbla
  bokstäver** och ett skärmtangentbord som färgas (QWERTZ för tyska, tjeckiska, slovenska och
  kroatiska, AZERTY för franska, annars QWERTY).
- **Fyra lägen**: *Oändlig* (ord efter ord med 6 försök vardera; varje löst ord ger poäng, det
  första olösta avslutar spelet), *Dagens ord* (ett ord om dagen per språk och längd - samma på
  datorn och i webbläsaren - med nedräkning och svit; ett påbörjat dagens ord sparas), *Dordle*
  (2 ord samtidigt på 7 försök) och *Quordle* (4 ord på 9 försök, tangenterna visar färgerna
  från alla brickor).
- **Inställningar** före varje parti: **ordlängd 4 till 7**, **svårt läge** (hittade ledtrådar
  måste användas vidare) och **färgblind-palett** (orange/blå); bredvid visas statistiken.
- **Riktiga ordlistor på alla 14 språk** (mappen `woordlistz/`, endast A-Z), med egna listor för
  varje längd: bara för 5 bokstäver nästan **34 000 lösningar** och över **213 000 tillåtna
  gissningar**, runt 134 000 lösningar i alla fyra längderna tillsammans. Lösningarna är vanliga
  ord utan namn, engelska rester och stötande ord; varje gissning kontrolleras mot listan -
  annat avvisas och raden skakar till.
- **Statistik** per språk, längd och läge: spelade partier, vinstprocent, aktuell och bästa svit
  och **fördelningen av försök som stapeldiagram** (sektionen `wordle` i `mem.json`). **Dela**
  (**C**) kopierar ett emoji-rutnät utan att avslöja ordet.
- Rekordet räknar bara *Oändlig* med 5 bokstäver; de andra längderna har egna bästa resultat.
  Prestationerna **Synsk** (högst 2 försök), **Ordvana** (7 dagens ord i rad) och **Fyrdubbelt
  geni** (Quordle löst).

**Poker**
- **3 varianter** på förspelsskärmen: **Texas Hold'em** mot 1-3 AI-motståndare med dealerknapp, blinds
  och fyra satsningsrundor, **5 Card Draw** (heads-up mot AI:n, ett kortbyte) och **Video Poker**
  (*Jacks or Better*, solo mot vinsttabellen).
- Handlingar via knappar eller tangenter: **F** = fold, **C** = check/call, **R** = raise, **A** =
  all-in; behåll/byt kort med klick eller **1-5**, **Enter** drar eller ger nästa hand.
- **Lamamarker** från den gemensamma **Lamabanken**: i början av en hand ligger ditt konto på bordet
  som stack, och det som går till potten dras direkt - lämnar du bordet mitt i en hand förlorar du
  bara din andel av potten. Pank (under big blind på 20, under 10 i Video Poker) = banklån upp till
  1000.
- **Rekord** = högsta nivån på din **Poker-balans** (1000 plus alla vinster och förluster i poker);
  prestationen **Chipleader** räknar också bara pokerbalansen.

**Schack**
- **Fullständigt schack**: alla pjäsdrag inklusive **rockad**, **en passant** och
  **bondeförvandling** (välj pjäs); **schack, schackmatt och patt** samt remi genom
  **50-dragsregeln**, **trefaldig ställningsupprepning**, **otillräckligt material** eller
  överenskommelse.
- **Tre lägen**: *parti* mot AI:n, *2 spelare* vid samma dator (brädet kan vändas efter varje drag)
  och **problem**.
- **Starkare AI utan hack** i 6 nivåer från *Nybörjare* till *Mästare*: iterativ fördjupning,
  transpositionstabell, lugnsökning, öppningsbok och en värdering med rörlighet, bondestruktur och
  kungssäkerhet. AI:n räknar i små portioner per bildruta - spelet hackar aldrig.
- **Inställningar**: färgval, **schackklocka** (ingen, 1+0, 3+2, 5+0, 10+5) och **Chess960** (alla 960
  utgångsställningar, numret står ovanför draglistan).
- **Sidopanel** med klockor, slagna pjäser, materialbalans och en scrollbar **draglista (SAN)**; dra och
  släpp, glidande pjäser, koordinater. Tangenter: **U** = ångra, **H** = tipspil, **O** = erbjud remi,
  **X** = ge upp, **F** = vänd brädet, efter partiet **P** = **PGN-export**.
- **Problem**: 200 problem i 5 steg (matt i 1/2/3, taktik I/II) från **Lichess fria problemdatabas
  (CC0)**, kontrollerade med spelets egen motor; i mattproblem räknas varje drag som sätter matt.
  Framstegen sparas i sektionen `chess` i `mem.json`.
- Ångra och tips gör ett parti ”assisterat”: rekordet räknar bara vinster utan hjälp mot AI:n (per
  session).

**Kvarn**
- **Kvarn** med alla tre faser: **utplacering** (9 pjäser var), **förflyttning** längs linjerna
  och **flygning** när man är nere på 3 pjäser (kan stängas av).
- En fullbordad **kvarn** tar bort en av motståndarens pjäser (helst en utanför en kvarn); du
  förlorar när du reducerats till under 3 pjäser eller inte kan dra.
- **3 AI-nivåer** (minimax med alfa-beta, fasanpassad utvärdering) eller en **lokal duell**; med
  dragtips, kvarnmarkering och en pjäsräknare.

**Simon**
- **Senso-minnesspel**: den upplysta sekvensen växer varje omgång och måste upprepas exakt.
- **Lägen**: *Klassiskt*, *Speed* (blir snabbare), *Reverse* (baklänges), *Mixat* (läget roterar
  varje omgång) och en **Duell** för två spelare (turas om att lägga till & upprepa).
- **Ljud** *av / på / mixat* (mixat tränar ditt visuella OCH auditiva minne), **4/6/9 knappar**
  som svårighetsgrad; **bästa poängen per läge** sparas. Spela med musen eller siffertangenterna
  1-9.

**Biljard**
- **8-ball**, **9-ball** och ett regelfritt **övningsläge**, mot AI:n (med sikthjälp) eller
  **två spelare lokalt**.
- **Tre fritt valbara vyer**: klassisk **2D uppifrån**, ett fast **3D-vinkelperspektiv** med
  skuggade bollar och en **fritt roterbar 3D-kamera** (höger musknapp). All rörelse är
  tidsstegsbaserad och **mjukt dämpad** (friktion, delsteg för att undvika genomträngning).
- **Stöt**: håll vänster musknapp för att ladda kraft, släpp för att stöta; en siktlinje och en
  kraftmätare hjälper till. **Boll i hand** efter en foul. Vyn (V) sparas i `settings.json`;
  vunna frames räknas till topplistan.

**Skjutpussel**
- 15-pusslet i tre storlekar: **3×3** (lätt), **4×4** (klassiskt) och **5×5** (svårt); skjut de
  numrerade brickorna in i den lediga luckan.
- Alltid lösbart (blandat med många slumpmässiga drag). Styr genom att **klicka** på en bricka i
  luckans rad/kolumn (hela linjen glider) eller med **piltangenterna**.
- Poäng = ett grundvärde per storlek minus drag och tid; när du löst det börjar ett nytt bräde
  direkt.

**Mastermind**
- Knäck den dolda **färgkoden**; efter varje gissning får du **svarta** stift (rätt färg +
  position) och **vita** stift (rätt färg, fel plats).
- **3 lägen**: Lätt (4 stift / 6 färger / 12 rader), Klassiskt (4/6/10) och Svår (5 stift / 8
  färger); dubbla färger tillåtna.
- Spela via färgpaletten (klick eller tangenter **1–8**), OK/Enter kontrollerar raden. **Oändlig
  streak** som i Wordle: varje knäckt kod ger poäng.

**Bubble Shooter**
- **Puzzle Bobble** på ett bikakerutnät: sikta med musen, skjut bubblor uppåt, **tre eller fler
  av en färg** spränger gruppen.
- Bubblor som tappar sin länk till taket **faller** (bonus); skott **studsar mot väggarna**, med
  förhandsvisning av nästa bubbla.
- **3 lägen** (4/5/6 färger, vissa med nedåtgående rader); game over vid den röda linjen.

**Hänga gubbe**
- Gissa ordet **bokstav för bokstav**; varje fel ritar en del av gubben, förlorat efter **6 fel**.
- **Ordlistor per språk** (endast A–Z), **3 längdlägen** (kort / mixat / långt); skriv eller klicka
  på skärmtangentbordet.
- **Oändlig streak**: varje gissat ord ger poäng (fler kvarvarande liv + längre ord = mer).

**Block Jump**
- **3D-plattformsspel i Minecraft-stil** (mjukvaru-3D som Snakes 3D-läge): hoppa över en svävande
  **voxelvärld** av block till det lysande målet.
- **Minecraft-skin**: alla block har äkta **pixeltexturer** (gräs, jord, sten,
  plankor, diamant, slime, trä); detaljnivån följer avståndet
  (**T** = hög/låg/av).
- Dessutom: **Steve**-figur med gånganimation (tredjepersonskamera), **hand** i
  förstaperson, **beacon-stråle** vid målet, roterande **guldtackor** som mynt,
  fyrkantig **sol**, **pixelmoln** och en HUD med **hjärtan**.
- Blocktyper: fasta block (gräs/jord/sten/trä), **stegar** (klättra), **staket** (hoppa över),
  **fjäderblock** (katapult) och **mynt**.
- Kameran är **förstaperson som i Minecraft som standard**, **V** växlar till en förföljarkamera;
  **mussikte** med pekarfångst, justerbar **motion blur** (**B**) och känslighet (**+/-**).
- **Seed-genererade parkourbanor** blir svårare; mål = poäng + tidsbonus, mynt +50, ett fall kostar
  ett liv (börja med 3). Styrning: WASD/pilar, **Mellanslag** för att hoppa.

**Tower Defense**
- **Oändligt vågförsvar** på **4 kartor** (Äng, Ravin, Korsning, Gatlopp), var
  och en med sin egen stig; låsta kartor låses upp med din bästa våg, en **boss**
  anländer var **8:e våg**.
- **3 lägen**: Klassisk (7 torn, huvudläget), Kompakt (4 torn, 2 nivåer) och
  Maximal (**11 torn**, **A/B-specialisering** på högsta nivån, specialfiender,
  aktiva förmågor **Meteor/Frostnova/Guldrush**).
- **11 torntyper** från pil till laser och guldbank, vart och ett med upp till
  **3 uppgraderingsnivåer**, försäljning återbetalar 70%; fiender med pansar,
  regenerering, delning, kamouflage, helandeaura och luftrutt.
- **Ekonomi**: guld per nedskjutning, vågbonus + 5% ränta; poäng per
  nedskjutning och våg. **F** = 2x tempo, **G** = räckvidder, högerklick
  avbryter.

**Minigolf**
- **360 banor på 40 slingor**: *Classic* och *Pro* med 9 handbyggda banor var,
  **Touren** med 38 slingor à 9 genererade banor (342 totalt) och stigande
  svårighet, därtill *Random* ur alltihop. Slinga 7, bana 3 ser likadan ut
  överallt - inget behöver sparas.
- **Underlag och hinder**: sand bromsar, ramper accelererar, vatten kostar ett
  straffslag, gummikuddar ger fart tillbaka, och väderkvarnar och vandrande block
  kräver timing. Fysiken går i delsteg med friktion precis som i biljard -
  ingenting hackar och ingenting glider genom sargen.
- **Styrning**: musen siktar, vänster musknapp nedtryckt laddar kraften och ett
  släpp slår (pilar + blanksteg funkar också). **R** avbryter ett laddat slag
  utan att slå. **G** växlar siktlinjen, **Z** auto-siktet, **P** Plocka upp.
- **Kraftlås (håll höger musknapp)**: fryser laddmätaren exakt där den står -
  gyllene, med procenten, ett hänglås och en pulserande ring runt bollen. Så
  väntar du på luckan i kvarnen med slaget färdigladdat. Släpper du laddas det
  vidare; en låst kraft överlever till och med slaget, och nästa vänsterklick
  slår med exakt det värdet.
- **Scorekort** till höger med par och slag per bana; till två spelar båda samma
  bana i tur och ordning. Poäng: 600 per bana, ±300 per slag under/över par,
  **500 extra för hole in one**. Lägsta slagantal per slinga ligger i avsnittet
  `minigolf` i `mem.json`.
- **Plocka upp går att stänga av**: som standard slutar en bana efter åtta slag
  och räknas som lägst. Vill du hellre spela tills bollen går i, sätt *Plocka
  upp* på AV i inställningarna (eller tryck **P**).
- **Auto-sikte går att stänga av**: som standard vrider sig klubban mot hålet
  före varje slag. Vill du hellre sikta själv på varje hål, sätt *Auto-sikte*
  på AV i inställningarna (eller tryck **Z**) - då står den senast valda
  riktningen kvar, och på ett nytt håls tee pekar klubban neutralt uppåt.
- **F** återställer det pågående hålet: slag till 0, bollen på utslaget - samma
  hål, samma bana.
- **Vidare i stället för upprepning**: när ronden är slut leder knappen
  **Vidare** till nästa bana (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), så att
  samma nio hål aldrig upprepas; bredvid: **Igen** (samma bana) och
  **Inställningar**. Tangenter: Enter = vidare, R = igen, S = inställningar.
- **Replay av rundan**: i slutet visar **P** (eller knappen **Replay**) hela
  rundan slag för slag. Med **S** hamnar den i arkivet (sidofältsknappen
  **Replays**).
- **Bygg och dela egna hål**: fliken **MAPS** på förberedelseskärmen leder
  till din egen samling - **Ny** öppnar hålredigeraren. Varje hål får ett namn
  och ett **id** (små bokstäver, inga mellanslag); id:t är samtidigt det
  filnamn som föreslås vid delning. Till de sju klassiska hindren kommer
  **åtta nya**: rör (flyttar bollen till andra änden), is, klibbfält, booster,
  magnet, enkelriktad grind, vridskiva och hopramp. Hålets storlek ställs in
  fritt (60x80 till 160x240), **12 mallar** ger en utgångspunkt, och
  ångra/gör om samt **Test** ingår. **Dela** skriver exakt ett hål som en
  `.lamapgzmap`-fil - via spara-dialogen eller rakt in i mappen Hämtade filer,
  med ditt namn som skapare. **Importera** läser in den igen och byter
  automatiskt till `-2` om id:t är taget. Namn, id och skapare går alltid
  genom ett **ordfilter över alla 14 språk**. Ett hål spelas ensamt via
  **Spela**, eller hela samlingen via det femte banvalet **Egna**.

**Pinball**
- **Tre bord**: *Classic* (tre bumpers, en målrad), *Space* (fyra bumpers i romb,
  två rader) och *Lama* (öppet fält, sex mål i en båge); 3 eller 5 bollar per
  spel, till två växelvis boll för boll.
- **Allt ett flipperspel behöver**: utskjutningsbana med laddstapel (för svagt?
  bollen rullar tillbaka och du får försöka igen), två flippers, slingshots,
  målrader, fyra **L-A-M-A**-banor, fångare med bollås, **multiboll med jackpot**,
  sex sekunders **bollräddning**, knuff och **TILT**.
- **Multiplikator upp till x5** via rensade rader och kompletta banor; bumpers
  100, slingshots 50, mål 250 - under multiboll betalar bumpers 2 500 i jackpot.
- Flipprarna körs med de tilldelade vänster/höger-tangenterna (samt
  vänster/höger [Shift]) eller musen. Rekordet per bord ligger i avsnittet
  `pinball` i `mem.json`.

**Bowling**
- **Tio frames enligt de officiella reglerna**, inklusive strikes, spares och
  tionde framens bonuskast (maximum: 300). **Scorekortet** under rubriken visar
  varje frame med X, / och löpande summa.
- **Kast i fyra steg**: position, vinkel, skruv och kraft. Varje reglage svänger
  av sig självt och låses med åtgärdstangenten - eller ställs för hand med
  vänster/höger, vilket stoppar svängningen.
- **Äkta kägelfysik**: tio käglor som cirklar med massa som välter varandra; en
  strike kommer av fysik och inte tur. Banan är oljad framtill, så **skruven**
  griper först i sista tredjedelen.
- Banvy i perspektiv med rännor, siktpilar och kägelfält; tre svårighetsgrader
  (*Lätt/Normal/Pro*) ändrar reglagens tempo och spridningen. Rekordet per grad
  ligger i avsnittet `bowling` i `mem.json`.
- **Replay av partiet**: i slutet visar **P** alla kast igen och **S** sparar dem
  i arkivet (knappen **Replays**).

**Crossy Road**
- **Oändliga hopp** över ängar (träd och stenar blockerar vägen), vägar med bilar och lastbilar, floder
  med stockar och näckrosor och **järnvägsspår** där ett tåg dundrar förbi efter varningsljus och
  klocka - längre fram väntar hela stationer med upp till 5 spår. Banan byggs rad för rad, har alltid
  en farbar väg, och både tempo och trafik ökar.
- **Isometrisk voxelstil**: figurer, fordon och träd av skuggade block (förrenderade för varje
  rutstorlek), mjukt följande kamera, squash & stretch vid hopp, vattenplask, tillplattningsanimation,
  fjädrar och glittrande mynt; från rad 50 **dag/natt-växling** med strålkastare.
- **Örnen**: kameran kryper framåt - den som dröjer för länge eller går mer än tre rader tillbaka tas
  av örnen (en röd kant varnar först). Att driva ut ur bilden på en stock är också slut.
- **Mynt och figurer**: insamlade mynt (jättemynt = 5) sparas och köper nya figurer i fliken
  **Figurer**: groda, gris, pingvin, katt, räv, lama, robot, spöke och enhörning (25 till 250 mynt);
  kycklingen finns med från start.
- **Lägen**: *Oändlig* (poäng = längsta raden, räknas till rekordet) och *Dagens bana* (samma för
  alla i dag, även i webbläsaren, med eget dagsrekord). Styrning: pilar/WASD, mellanslag/Enter/klick =
  hoppa framåt; i inställningarna **H** = skuggor, **N** = dag/natt. Mynt, figurer och dagsrekord
  sparas i sektionen `crossy` i `mem.json`.

**Geometry Dash**
- **Rytmplattformspel**: figuren rusar av sig själv åt höger - du bestämmer bara när den
  ska hoppa eller flyga. **Fem former** - kub, skepp, boll, UFO och våg - plus form-,
  gravitations- och fartportaler (0,5x till 3x), gula/rosa/blå **pads och klot**,
  halvblock, spikar, gropar och färgtriggers.
- **8 inbyggda banor** från *Lätt* till *Demon* (”Lama Inferno”) med **3 hemliga mynt**
  var. Varje bana går bevisligen att klara: när den byggdes löste en lösningsalgoritm den
  med spelets riktiga kod - med alla mynt och till och med förskjuten med 1/240 sekund.
- **Exakt fysik**: fixpunktsberäkning med fast 240 Hz-steg; varje tryck verkar exakt i det
  steg där det skedde - likadant vid varje bildfrekvens och bitidentiskt i webbläsaren.
- **Träningsläge** (**P**) med automatiska och egna kontrollpunkter (**Z** sätter, **X**
  tar bort), försöksräknare, förloppsindikator, explosioner och omedelbar omstart (**R**).
  Varje bana har sitt **eget soundtrack** - bakgrund, mark och klot pulserar i takt
  (musiken stängs av med **M**).
- **Stjärnor och mynt**: klarar du en bana i normalt läge får du dess stjärnor, och varje
  mynt är värt en stjärna till; rekordet är det **totala antalet stjärnor** (högst 65).
  Bästa resultat per bana, mynt, försök och hopp sparas i sektionen `geodash` i
  `mem.json`.
- **Baneditor** i fliken **BANOR**: rutnätsduk, palett med 6 grupper (block, faror, pads och
  klot, portaler, fart, extra), rotering, ångra/gör om, översiktsremsa, **test från start
  eller härifrån** och baninställningar (startfart, startform, musikstil, BPM, färger).
  Bocken **”verifierad”** kommer först när du själv har klarat din bana. **Dela** skriver
  en `.lamapgzlevel`-fil och **Importera** läser in den igen; banorna sparas i `ugc.json`
  bredvid dina egna minigolfbanor.

**Battleship**
- **Sjöslag på 10x10** med hangarfartyg (5 rutor), slagskepp (4), kryssare (3), ubåt (3) och
  jagare (2) - den som först sänker hela fiendens flotta vinner.
- **Placera flottan** med dra och släpp från dockan: **R** eller högerklick roterar, förhandsvisningen
  lyser grönt eller rött, **X** placerar allt slumpmässigt, **C** tömmer brädet; din senaste
  uppställning föreslås igen.
- **Regler i inställningarna** (sparas): *skepp får nudda varandra*, *salva* (lika många skott per tur
  som du har skepp flytande) och *skjut igen efter en träff*.
- **AI med 3 nivåer**: Lätt skjuter slumpmässigt, Medel följer upp träffar systematiskt, Svår beräknar
  en **sannolikhetskarta** med schackbrädesparitet (i snitt cirka 70 / 60 / 45 skott för en hel
  flotta). Eller **2 spelare** vid samma dator - en **överlämningsskärm** döljer båda flottorna före
  varje tur.
- **Grafik**: radarsvep, animerade vågor, granater i båge, plask, explosioner med rök och brinnande
  rutor, en ”SÄNKT!”-avslöjning och en rundsammanfattning med skott, träffar och träffsäkerhet.
  Rekordet räknar dina **vinster mot AI:n** under en session.

**Casino**
- **Roulette** (europeisk, 37 fack): alla klassiska insatser genom klick på ett nummer, en kant eller
  ett hörn - **plein** (35:1), cheval, transversale, carré, sixain, kolumn, dussin, rött/svart,
  jämnt/udda och manque/passe. Markvärden 1/5/25/100/500, högerklick tar bort marker; **Snurra**,
  **Upprepa** (**R**), **Dubbla** (**D**) och **Rensa**. Kulan snurrar i spiral ner i det fack som
  dragits i förväg, och överst visas de senaste 12 numren.
- **Lamaautomat**: 5 hjul x 3 rader, **10 vinstlinjer**, **lama = wild**, **guldmynt = scatter** med
  10 gratissnurr och dubbla vinster, insats per linje 1/2/5/10, **autosnurr** (10/25), **turbo** och
  vinsttabell. **Återbetalningsgraden är 96,1 %** - exakt beräknad från hjulremsorna.
- **Lamabanken**: Casino, Blackjack och Poker delar ett konto med **lamamarker** (start 1000,
  sektionen `casino` i `mem.json`); gamla marksaldon förs över automatiskt. Insatser dras direkt, varje
  spel för sin egen balans till rekordet, och går du pank får du ett **banklån** upp till 1000.
- Konfetti, myntregn, big/mega/jackpot-banderoller och vinstlinjeanimationer; prestationerna
  **Fullträff** (vunnen plein i roulette) och **Lamajackpot** (5 lamor på en linje).

Topplistorna sparas i avsnittet `highscores` i `mem.json` (bredvid koden) – tillsammans med
språket (avsnittet `mem`).

### Gränssnittet

Hela gränssnittet är ritat från grunden (ren Tkinter + Pygame, inga extra paket) och stylat som en
modern spel-launcher:

- **Spellista i sidofältet**: varje rad har sitt eget **minipiktogram** i spelets accentfärg, visar
  aktuell **topplista (★)** och reagerar med mjukt animerade hover-effekter. Det pågående spelet
  förblir färgmarkerat; i små fönster **scrollar** listan med mushjulet.
- **Statuskort** längst ner till vänster med en **status-LED** (grå = meny, grön = igång,
  guld = pausat, röd = game over) och en **live-FPS-avläsning**.
- **Viloskärm** med auroraljus, ett parallaxstjärnfält med stjärnfall, en svävande logotyp med
  kretsande gnistor, ett **klickbart spelrutnät** direkt under logotypen (alla spel med en
  hover-effekt i sin accentfärg) och en **topplisteticker**.
- **Effekter överallt**: mjuka skärmövergångar, gnistor när man bekräftar ett menyval, **konfettiregn
  vid nytt rekord** och en riktig **oskärpa** bakom pausöverlägget.
- Varje spels **förspelsskärm** visas i det spelets accentfärg och visar det tidigare rekordet som ett
  chip. Med många lägen och låg upplösning blir den **kompakt**: Inställningar, Wiki och Tillbaka hamnar
  på en rad och typsnittet anpassas - inget hamnar längre utanför bilden.
- **Enhetlig look i spelet**: alla 46 spel delar menyns temapalett och typsnitt - HUD:ar, setup-skärmar
  och överlägg följer den design som valts i alternativen (v4.1 / v4 / Classic), medan varje spelplan
  behåller sina identitetsfärger. Varje spel hanterar nu upplösningsbyten mitt i spelet snyggt, och
  menynamnen är språkanpassade (t.ex. ”Schach” → ”Schack”).
- **Inbyggt wiki** (”LamaWiki”): detaljerad hjälp för varje spel (styrning, lägen, poäng, tips) plus
  allmänna sidor - med en **sökruta**, kategorier, scrollbara artiklar och tangentchips, på alla 14
  språk. Nås via sidofältsknappen **”Wiki / Hjälp”** och från varje spels förspelsskärm (öppnar det
  spelets sida direkt).
- **Prestationer & statistik**: **107 prestationer** i tre kategorier (23 mål
  för hela samlingen, 37 poängmilstolpar och 47 speciella ögonblick som
  schackmatt mot AI:n, 4096-brickan, en T-Spin Double, 25 lösta schackproblem,
  ett Killer Sudoku eller lama-jackpotten; i 2048 och schack räknas inte partier
  med ångra eller tips) med **gyllene avisering och fanfar**
  vid upplåsning - även mitt i spelet; gamla rekord tillgodoräknas
  automatiskt. Dessutom en **statistik**-flik: total speltid, partier, segrar,
  rekord, favoritspel och en tabell per spel sorterad efter speltid. Nås via
  knappen **”Prestationer & statistik”** i sidofältet.
- **Replays**: minigolf och bowling spelar in varje runda. I slutet visar **P**
  repriser och **S** lägger dem i arkivet - nås via sidofältsknappen **Replays**
  (en flik per spel, paus, hopp mellan sekvenser, tempo 0,5x till 4x). Kan
  stängas av vid första starten och i inställningarna.

### Användning

- Välj ett spel via knappen i menyn till vänster. Sedan visas en **förspelsskärm**: välj
  **Enspelarläge** eller **Flerspelarläge**, gå till **alternativen** eller tillbaka. Pilar/mus för
  att välja, Enter startar.
- **ESC** = paus / återuppta (i menyer: tillbaka).
- **F11** (eller knappen ”Helskärm på/av”) = växla helskärm. Pygame-displayen förblir inbäddad och
  skalas upp med bibehållet bildförhållande (svarta kanter när bildförhållandet skiljer sig). Fönstret
  kan storleksändras fritt.
- **”Tillbaka till menyn”** avslutar spelet och sparar topplistan - det gör även ett byte till ett
  annat spel via sidofältet.
- **Fasta extratangenter**: utöver de fem handlingar som kan bindas har vissa spel egna tangenter
  (t.ex. reserv **C** och rotera åt vänster **Z** i Tetris, ångra **U** i 2048, schack och sudoku).
  De fungerar bara om tangenten inte är bunden till en handling i alternativen och står i
  inställningstipset och i wikin. Nedhållna tangenter känns igen korrekt och släpps vid paus eller
  Alt-Tab - inget ”hänger sig” längre.
- **”Avsluta”** stänger Pygame och Tkinter på ett rent sätt.

### Alternativ, styrning och ljud

Alternativskärmen öppnas via knappen **”Alternativ / Styrning”** (till vänster) eller från
förspelsskärmen. Den är organiserad i **tre flikar** (**Allmänt / Styrning / Utseende**; växla genom
att klicka eller med Tab-tangenten):

- **Allmänt**: **ljud** på/av, **volym** och **haptik** (gamepad-vibration, fungerar bara med en
  ansluten handkontroll) plus **autoupplösning**, **upplösning**, **FPS** och **språk** – var och en
  växlas med Vänster/Höger.
- **Styrning**: **förval** (*WASD + Pilar*, *WASD + IJKL*, *Pilar + WASD*) och **bind om varenda
  tangent** för spelare 1 och spelare 2: välj en rad, tryck Enter, tryck önskad tangent (Esc avbryter).
- **Utseende**: välj **UI-design** – **UI v4.2** (standard: Midnight Glass – en djup midnattsgradient
  med långsamt drivande mjuka ljus i indigo, turkos och magenta, fint filmkorn, glesa stjärnor och
  paneler som matt glas med en ljus kant), **UI v4.1** (som UI v4 men livligare – diskreta stjärnor
  plus Saturnus och ett svart hål i startskärmens bakgrund), **UI v4.1.1** (som v4.1, men ett kaklat
  **sicksackmönster** i svart och antracit i stället för stjärnhimlen), **UI v4.1.2** (samma mönster i
  palettens blå toner – accentblått som dominerande färg, ett mörkare blått som botten), **UI v4.1.3**
  (samma mönster i indigon från UI v4 på svart), **UI v4.1.4** (i grafittonen från UI v4 på svart),
  **UI v4** (en helt lugn, platt grafit-look med en enda indigo-accent), **UI v3** (det tidigare
  klassiska gränssnittet med stjärnhimmel, auroraljus och glödeffekter), **UI v2** (den allra första
  UI-omarbetningen: marinblå gradient, stjärnhimmel och glödande knappar, helt utan animationer) eller
  **UI v1** (utseendet före UI-omarbetningen: enfärgad mörk bakgrund, platta knappar, inga effekter).
  Alla kort visar en liten förhandsvisning; valet tillämpas omedelbart på hela gränssnittet (spelytan
  **och** sidofältet) och sparas.

Inställningarna sparas permanent i `settings.json`. I **enspelarläge** styr båda bindningarna samma
figur (standard: WASD *och* pilar), i **flerspelarläge** en var. Alla spel har **ljudeffekter**
(procedurellt genererade, inga extra filer behövs) som kan tystas globalt.

### Projektstruktur

```
install-python.bat  Windows-installation: Python 3.13 + .venv + pygame
start.bat            Startskript (Windows)
start.sh             Startskript (Linux / macOS / Git Bash)
pyinstall.bat        EXE-bygge (Windows): packar allt i builds\PyGameZ.exe
main.py              Tkinter-gränssnitt, Pygame-inbäddning, central spelloop
game_base.py         Spelbasklass (update/draw/handle_event) + InputEvent + hjälpfunktioner
settings.py          Läser/sparar inställningar (ljud/haptik/tangentbindningar/spelalternativ med kontrollregler) (JSON)
audio.py             Procedurella ljudeffekter, musikslingor + handkontrollsvibration
menu.py              Skärmar för språk, förspel (läge) och alternativ (ljud/styrning)
highscore.py         Ladda/spara topplistor (avsnitt i mem.json)
store.py             Central sparfil mem.json (sektioner: mem, highscores, stats, achievements + spelframsteg), atomär med .bak-kopia
stats.py             Spelarstatistik (partier, speltid, segrar, rekord) per spel
achievements.py      Prestationer: definitioner, upplåsning, avisering (toast)
progress.py          Skärm för prestationer & statistik (två flikar, rullbar)
replay.py            Inspelning och arkiv för replays (replay.json)
replayview.py        Replay-skärm: arkivlista och uppspelning
ugc.py               Eget innehåll (minigolfbanor, Geometry Dash-nivåer): lagring, kontroll, export/import (ugc.json)
swear.py             Ordfilter för namn och id (lang/swear/*.yml, alla 14 språk)
filepick.py          Fildialoger ("Exportera som ...", "Importera")
prestige.py          Prestige-system för Snake
competitive.py       Finjustering av Snakes Competitive-läge (nivåer, enarmad bandit, speläpplen)
ngb.py               Visuell personalisering ("mods"): huvudfärg + koordinatrutnät + meny (mem-ngb.json)
lamabank.py          Lamabanken: gemensamt markkonto för Blackjack, Poker och Casino (sektionen casino i mem.json)
seedrand.py          Slumpgenerator med bitidentiska tal i Python och webbläsaren (dagens lägen, nya pussel)
i18n.py              Översättningsmotor (laddar lang/*.json, t("nyckel"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Språksträngar (en platshållarnyckel per text)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Ordfilterlistor per språk (regex, .yml)
lamawiki/
  lamawiki.py          Inbyggt wiki (sökning, kategorier, artikelrenderare)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-innehåll (en sida per spel + allmänna sidor)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Bygger om Wordles ordlistor (ordböcker + frekvenslistor)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 bokstäver), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 bokstäver), 14 språk
devtools/            Utvecklarverktyg (packas inte in i .exe-filen)
  merge_staging.py           För in översättningar och wikisidor från devtools/staging/ i alla 14 språkfiler
  build_chess_puzzles.py     Bygger de 200 schackproblemen från Lichess problemdatabas (CC0)
  build_sudoku_killer.py     Genererar de 400 entydigt lösbara Killer Sudokun
  build_crossyroad_models.py Skriver Crossy Roads voxelmodeller för webbversionen
  build_geodash_levels.py    Bygger de 8 Geometry Dash-banorna och bevisar med lösningsalgoritmen att varje bana går att klara, mynt inräknade
  build_geodash_solver.py    Lösningsalgoritm med spelets riktiga stegkod (lösningar i geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Nivådata: snake-comp.json, chess-puzzles.json (+ käll-README), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Övergripande granskning (inmatning, seedrand Python = JS, sparande, språkfiler, förspelsskärmar) + alla audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless-granskningar per spel
  newgames_audit.py  blockjump_audit.py
```

Det valda språket sparas i `mem.json` (i avsnittet `mem`, bredvid avsnittet `highscores` i samma fil)
och laddas automatiskt vid nästa start.

**Källor och licenser:** de 200 schackproblemen kommer från
[Lichess problemdatabas](https://database.lichess.org/#puzzles) (licens **CC0 1.0**,
public domain - tack, lichess.org!); detaljer finns i `games/levels/chess-puzzles.README.md`.
Källorna till Wordles ordlistor anges i `woordlistz/README.md`.

### Plattformsanteckningar

Displayen körs **off-screen**: pygame använder dummy-videodrivrutinen (`SDL_VIDEODRIVER=dummy`),
renderar alltså till en surface, och varje bildruta ritas som en bild i en Tkinter-widget. Det finns
**inget nativt SDL-fönster** som skulle kunna slåss med Tkinter om storlek/position. Därför beter sig
fönstret likadant och stabilt överallt:

- **Windows**: processen görs dessutom DPI-medveten så att bilden förblir skarp på skalade skärmar
  (125/150/200 %) och inte ”skakar”.
- **Linux/X11 & Wayland**: fungerar utan specialfall (inget `SDL_WINDOWID`).
- **macOS**: fungerar också (tidigare visades det inbäddade fönstret inte alls här).

---

### Installationsguide

Krav: **Python 3.9+** (rekommenderas 3.12 eller 3.13) och **pygame ≥ 2.6**.

#### Windows (rekommenderas: automatiskt)

1. Öppna projektmappen och dubbelklicka på **`install-python.bat`**. Skriptet
   - kontrollerar om **Python 3.13** finns, och installerar det annars via
     **winget** (`winget install Python.Python.3.13`),
   - skapar den virtuella miljön **`.venv`**,
   - installerar **pygame** från `requirements.txt`.
2. Starta sedan samlingen med **`start.bat`** (dubbelklick).

> Obs: om skriptet rapporterar ”inte tillgängligt i det här fönstret ännu”, installerades Python precis
> – öppna helt enkelt **ett nytt terminalfönster** och kör `install-python.bat` igen. Om **winget** inte
> finns, installera Python 3.13 manuellt från <https://www.python.org/downloads/> och kryssa i
> **”Add python.exe to PATH”**.

#### Windows / Linux / macOS (manuellt)

```bash
# 1. Kontrollera Python (3.9+)
python --version

# 2. Skapa och aktivera en virtuell miljö
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Installera beroenden
pip install -r requirements.txt
#   eller:  pip install "pygame>=2.6" (eller pygame-ce)
#                                   pip install pygame-ce
# 4. Starta
python main.py
```

#### Linux / macOS med start.sh

```bash
# Konfigurera Python + venv som ovan (steg 2 och 3), sedan:
chmod +x start.sh      # en gång, om den inte redan är körbar
./start.sh
```

På Linux installerar du vid behov Python via pakethanteraren, t.ex.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); på macOS
t.ex. `brew install python`.

#### Använda en annan Python-version

`install-python.bat` installerar Python 3.13 som standard. Om du föredrar 3.12 (eller
en annan version), ändra raden `set "PYVER=3.13"` i filen till önskad version och
winget-ID:t därefter (`Python.Python.3.12`).

#### Bygga en fristående EXE (Windows)

```bat
pyinstall.bat         :: bygger builds\PyGameZ.exe (allt i en fil)
```

`pyinstall.bat` använder `.venv` (och skapar den vid behov), installerar automatiskt
**PyInstaller** och packar hela spelet - Python, pygame, alla spel, språk, wiki och
logotyper - i **en enda `PyGameZ.exe`** i mappen **`builds\`**. Filen körs på vilken
Windows-dator som helst utan installerad Python och kan kopieras fritt. Inställningar
och topplistor (`settings.json`, `mem.json`, `mem-ngb.json`) skapas bredvid .exe-filen
medan du spelar.

#### Felsökning

- **`pygame` hittas inte** → är venv aktiverad? Upprepa steg 3
  (`pip install -r requirements.txt`).
- **`python` känns inte igen (Windows)** → Python installerades utan ”Add to PATH”;
  installera om och kryssa i rutan, eller använd `py` istället för `python`.
- **Inget ljud** → kontrollera ”Ljud” i alternativen; haptik fungerar bara med en handkontroll.
- **Fönster/inbäddning på Linux** → se *Plattformsanteckningar* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ till toppen / back to top</a></b></div>

---

<a name="-suomi"></a>

## 🇫🇮 Suomi

Työpöytäpelikokoelma Pythonilla: **Tkinter** tarjoaa ikkunan ja valikon,
**Pygame** upotetaan pelinäytöksi Tkinter-ikkunan sisään. Neljäkymmentäkuusi
peliä, joilla on yhteiset asetukset, vapaasti uudelleenmääriteltävät ohjaimet,
ennätykset, proseduraaliset äänitehosteet ja osassa peleistä moninpeli.
Käyttöliittymä on **monikielinen** – **14 kieltä** (saksa / englanti / ranska /
espanja / portugali / puola / turkki / tanska / norja / ruotsi / suomi / tšekki /
sloveeni / kroatia); kieli valitaan ensimmäisellä käynnistyksellä
**tervetulonäytöllä**, jolla voi samalla asettaa **resoluution** ja **äänen**
(oletuksena pois päältä); kolmea pääkieltä lukuun ottamatta kaikki muut (espanja,
portugali ja yhdeksän lisäkieltä, suomi mukaan lukien) piiloutuvat
**"Lisää kieliä"** -painikkeen taakse. Kaiken voi muuttaa myöhemmin asetuksista.

### Pika-aloitus

#### Windows

```bat
install-python.bat    :: kerran: asenna Python 3.13 + .venv + pygame
start.bat             :: käynnistä pelikokoelma
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # käynnistyy .venv:llä, muuten järjestelmän python3:lla
```

`start.bat` / `start.sh` käyttävät automaattisesti virtuaaliympäristöä `.venv`,
jos se on olemassa, muuten järjestelmän Pythonia. Yksityiskohtainen vaiheittainen
ohje löytyy aivan alhaalta kohdasta **[Asennusopas](#asennusopas)**.

### Pelit

| Peli         | Tilat           | Lyhyt kuvaus |
|--------------|-----------------|--------------|
| **Snake**    | 1 / 2 pelaajaa  | Deluxe-Snake 2D- ja 3D-näkymällä, boostilla, 6 pelitilaa (ml. Competitive), kultaomenat ja prestige |
| **Pong**     | 1 / 2 pelaajaa  | Klassikko tekoälyä tai pelaajaa 2 vastaan, vaihdettava liikkumistila |
| **Air Hockey** | 1 / 2 pelaajaa | 2D-fysiikka liikemäärän siirrolla, hiiriohjaus, tekoäly ja power-upit |
| **Tic-Tac-Toe** | 1 / 2 pelaajaa | m,n,k-peli koossa 3x3–9x9, kolme tekoälyn tasoa **tai** paikallinen X vastaan O |
| **Breakout** | 1 pelaaja       | Tiilenmurskain tiililajeilla, power-upeilla, comboilla ja monilla kentillä |
| **Tetris**   | 1 / 2 pelaajaa  | Modernit Guideline-säännöt (SRS, varasto, 5 palan esikatselu, T-Spinit): Maraton, Sprintti 40, Ultra 2:00, Versus tekoälyä vastaan (3 tasoa) tai kaksinpeli roskariveillä |
| **Invaders** | 1 pelaaja       | Space Invaders: tyhjennä aallot, suojele elämiäsi |
| **Asteroids** | 1 / 2 pelaajaa | Hitausfysiikka, aallot, UFOt, power-upit, hyperavaruus – yksin tai yhteistyökaksintaisteluna |
| **Pac-Man**  | 1 pelaaja       | Uskollinen klooni: 4 haamun tekoälyä, voimapillerit, tunneli, hedelmät, kentät |
| **Flappy Bird** | 1 pelaaja    | Painovoimalento putkien läpi, kolikot, kilpi, päivä/yö, mitalit |
| **Doodle Jump** | 1 pelaaja    | Automaattinen ylöspäin hyppy, tasotyypit, jouset, potkuri, hirviöt |
| **2048**     | 1 pelaaja       | Numeroiden liu'utuspulma 3x3:sta 8x8:aan: Klassinen, Aikahaaste ja Loputon, kumoaminen, sulavat animaatiot, tallennetut pelit |
| **Minesweeper** | 1 pelaaja    | Klassikko turvallisella ensiklikkauksella, chording, hymiö ja parhaat ajat |
| **Sudoku**      | 1 pelaaja    | 4 muunnelmaa (Klassinen, X-sudoku, Killer, Mini 6x6), kussakin 400 kenttää, päivän sudoku, enintään 3 tähteä kentältä, 4 avustustilaa, kumoaminen, tallennus |
| **Frogger**     | 1 pelaaja    | Tie + joki + 5 poukamaa, bonuskärpänen, krokotiilit, aikaraja, 3 vaikeustasoa |
| **Memory**      | 1 / 2 pelaajaa | Etsi parit koossa 4x4–8x6, kääntöanimaatio, yksinpisteytys tai kaksintaistelu |
| **Pasianssi**   | 1 pelaaja    | 5 muunnelmaa (Klondike, Spider, FreeCell, Pyramidi, TriPeaks) vedä ja pudota -toiminnolla ja kumoamisella |
| **Aim Trainer** | 1 pelaaja    | Rento 3D-maaliammunta: hiiri ohjaa kameraa, 4 tilaa (tarkkuus/refleksi/liikkuvat/chill), 3 teemaa ml. musta aukko |
| **Neljän suora** | 1 / 2 pelaajaa | Klassikko putoamisanimaatiolla: 3 tekoälyn tasoa (minimax) tai paikallinen kaksintaistelu |
| **Tankkikaksintaistelu** | 1 / 2 pelaajaa | 2D-areenakaksintaistelu kimpoavilla laukauksilla, power-upit, 4 areenaa, tekoäly 3 tasolla |
| **Blackjack**    | 1 pelaaja    | Kasino-blackjack 4 pakan kengällä, tuplaus/jako ja 3:2-blackjack; pelataan yhteisen Laamapankin laamamerkeillä |
| **Tunnel Racer** | 1 pelaaja    | 3D-neonputkilento: loputon tila + 30 kenttää, näppäin- tai hiiriohjaus, motion blur |
| **3D-labyrintti** | 1 pelaaja   | Ensimmäisen persoonan raycaster (Wolfenstein-tyyli) 50 siemenpohjaisella kentällä, orbit, minikartta – tai 2D-yläkuva |
| **Reversi**      | 1 / 2 pelaajaa | Othello 8x8:ssa: saarrata ja käännä kiekot, 3 tekoälyn tasoa (minimax) tai paikallinen kaksintaistelu |
| **Yatzy**        | 1 / 2 pelaajaa | Noppaklassikko 13 kategorialla, yläbonus ja Yatzy; ennätysjahti tai 2 pelaajan hotseat |
| **Wordle**       | 1 pelaaja    | Arvaa 4–7-kirjaimisia sanoja: Loputon, Päivän sana, Dordle ja Quordle, vaikea tila, värisokeiden paletti, tilastot pylväsdiagrammilla, tuloksen jakaminen, aidot sanalistat 14 kielellä |
| **T-Rex Runner** | 1 pelaaja    | Loputon aavikkojuoksu: vaihteleva hyppy, kyykistys, kaktukset ja pterodaktyylit, päivä/yö-sykli, kasvava vauhti, 3 vaikeustasoa |
| **Tammi**        | 1 / 2 pelaajaa | 3 sääntökokoelmaa (saksalainen 8×8, kansainvälinen 10×10, checkers), pakkolyönti ja lentävä kuningatar, 3 tekoälyn tasoa (minimax) tai paikallinen kaksintaistelu |
| **Pokeri**       | 1 pelaaja    | 3 valittavaa muunnelmaa: Texas Hold'em tekoälyä vastaan, 5 Card Draw ja Video Poker; panostuskierrokset, blindit, yhteisen Laamapankin laamamerkit |
| **Shakki**       | 1 / 2 pelaajaa | Täydet säännöt, Chess960 ja shakkikello, 6 tekoälyn tasoa, 200 tehtävää Lichessin tietokannasta, kumoa/vihje, siirtolista, PGN-vienti tai paikallinen kaksintaistelu |
| **Mylly**        | 1 / 2 pelaajaa | Asettelu-/siirto-/lentovaiheet, myllyt ja lyönnit, valinnainen lentosääntö, 3 tekoälyn tasoa tai paikallinen kaksintaistelu |
| **Simon**        | 1 / 2 pelaajaa | Senso-muistipeli: Klassinen/Speed/Reverse/Mixed-tilat + kaksintaistelu, ääni pois/päällä/sekoitettu, 4/6/9 painiketta, paras per tila |
| **Biljardi**     | 1 / 2 pelaajaa | 8-ball, 9-ball ja harjoittelu 2D:ssä, kiinteä 3D-näkymä tai vapaasti kiertävä 3D-kamera; pehmeä fysiikka, tähtäysapu, 3 tekoälyn tasoa |
| **Liukupalapeli** | 1 pelaaja   | 15-pulma koossa 3x3/4x4/5x5: liu'uta numeroidut palat aukkoon, klikkaus- tai nuoliohjaus, pisteet siirroista ja ajasta |
| **Mastermind**    | 1 pelaaja   | Murra salainen värikoodi (3 tilaa: 4×6, klassinen, 5×8), mustat/valkoiset palautetapit, loputon putki -ennätys |
| **Bubble Shooter** | 1 pelaaja  | Puzzle Bobble -klooni: ammu samat värit kolmen ryhmiksi, seinäkimmokkeet, putoavat rykelmät, 3 vaikeustasoa |
| **Hirsipuu**       | 1 pelaaja  | Arvaa sana ennen kuin hirsipuu valmistuu; ruutunäppäimistö, kielikohtaiset sanalistat, 3 pituustilaa, loputon putki |
| **Block Jump**  | 1 pelaaja       | 3D-Minecraft-tyylinen tasoloikka: teksturoitu voxel-maailma, Steve-hahmo, tikkaat, aidat ja limapalikat, ensimmäisen/kolmannen persoonan kamera, motion blur, siemenpohjaiset parkour-kentät |
| **Tower Defense** | 1 pelaaja      | Torju loputtomia aaltoja 4 kartalla: jopa 11 tornityyppiä parannuksineen, myynteineen ja A/B-erikoistumisineen, pomoja, 3 tilaa, aktiivisia kykyjä |
| **Minigolf**    | 1 / 2 pelaajaa | 360 rataa 40 kierroksella (18 käsin rakennettua, 342 luotua): hiekkaa, ramppeja, vettä, puskureita, tuulimyllyjä ja vaeltavia lohkoja; tuloskortti parilla ja hole-in-one-bonuksella; **oma reikäeditori**, 15 objektityyppiä, 12 pohjaa ja jako `.lamapgzmap`-tiedostona |
| **Pinball**     | 1 / 2 pelaajaa | Flipperiautomaatti kolmella pöydällä: puskurit, slingshotit, maalit, L-A-M-A-kaistat, multiball ja jackpot, pallonpelastus, töytäisy ja tilt |
| **Bowling**     | 1 / 2 pelaajaa | 10 framea virallisella strike/spare-laskennalla, aito keilafysiikka, hook-kierre ja perspektiivinen rata, 3 vaikeustasoa |
| **Crossy Road** | 1 pelaaja     | Loputonta hyppimistä niittyjen, teiden, jokien ja rautateiden yli isometrisessä vokselityylissä: päivä/yö, kotka, 10 ostettavaa hahmoa, päivän reitti |
| **Geometry Dash** | 1 pelaaja   | Rytmitasohyppely kuutiolla, aluksella, pallolla, UFOlla ja aallolla: 8 kenttää Helposta Demoniin, kussakin 3 salaista kolikkoa, harjoitustila, oma soundtrack; **kenttäeditori** ja jakaminen `.lamapgzlevel`-tiedostona |
| **Battleship**  | 1 / 2 pelaajaa | Meritaistelu 10x10: laivaston sijoitus vetämällä, 3 sääntökytkintä (kosketus, salvo, uusi laukaus), tekoäly 3 tasolla tai paikallinen kaksintaistelu luovutusnäytöllä |
| **Casino**      | 1 pelaaja     | Eurooppalainen ruletti kaikilla klassisilla panoksilla ja Laama-automaatti (5 rullaa, 10 voittolinjaa, jokeri, ilmaiskierrokset); yksi laamamerkkitili Blackjackin ja Pokerin kanssa |

**Moninpeli (2 pelaajaa paikallisesti)** on saatavilla peleihin **Snake**,
**Pong**, **Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids
(yhteistyökaksintaistelu)**, **Memory (kaksintaistelu)**, **Neljän suora**,
**Tankkikaksintaistelu**, **Reversi**, **Yatzy**, **Tammi**, **Shakki**,
**Mylly**, **Simon (kaksintaistelu)**, **Biljardi**, **Minigolf**, **Pinball**,
**Bowling** ja **Battleship** (laivastot piilottavalla luovutusnäytöllä) –
yhteensä 20 peliä. Tila valitaan suoraan pelin esinäytöllä (*Yksinpeli /
Moninpeli*); Tetriksessä on lisäksi **Versus tekoälyä vastaan**. Selainversio on
pelkkää yksinpeliä.

#### Pelikohtaiset ominaisuudet

**Snake**
- **UUSI - 3D-näkymä** (näppäin **V** asetuksissa tai klikkaus kohtaan *Näkymä*):
  kenttä renderöidään reaaliaikaisena 3D-näkymänä – **seurantakamera** leijuu
  madon takana ja ohjaus tapahtuu **suhteessa katsesuuntaan** (vasen/oikea =
  käänny, kaksi nopeaa painallusta = U-käännös). Mukana etäisyyssumu, tähtitaivas,
  ruudukkolattia, reunamuurit, pyörivät ruokakiteet, 3D-partikkelit ja
  kameratärähdys törmäyksessä; game overin jälkeen kamera kiertää matoa hitaasti.
  Boostaus laajentaa näkökenttää. 3D:ssä käytettävissä: *Klassinen* ja *Esteet*
  (muurit ovat siellä aina kiinteät, 3D on vain yksinpelissä). Näkymä muistetaan
  tiedostossa `settings.json`.
- **UUSI - 3D-kamera-asetukset** (3D-asetuksissa klikkaa riviä *3D-kamera /
  smooth shake* tai näppäin **K**): oma valikko, jossa **smooth shake**
  (pehmeämpi kamera, huomattavasti vähemmän nykimistä liikkuessa/kääntyessä),
  säädettävä **näkökenttä (FOV)** ja **kameran korkeus** sekä **tärähdys
  käännöksessä** -kytkin (näyttötärähdys vasen/oikea-käännöksissä päälle/pois).
  Kaikki muistetaan tiedostossa `settings.json`.
- **Boost**: **pidä** boost-näppäintä pohjassa = turbo (kaksinkertainen vauhti),
  kuluttaa kestävyyttä (palkki); tyhjentyessä boost sammuu ja latautuu uudelleen.
  Oletus P1 = välilyönti/vasen Shift, P2 = Enter/oikea Shift.
- **6 pelitilaa** (valittavissa asetuksissa): *Klassinen*, *Speed Rush* (nopeutuu
  jokaisen omenan myötä), *Esteet* (tappavat lohkot), *Portaalit*
  (teleportteriparit), *Aika-ajo* (60 sekuntia, mahdollisimman monta omenaa) ja
  *Competitive* (katso alla).
- **UUSI - Competitive** (yksinpeli): loputon tila **tasonnousulla** – aloitat
  tasan **yhdellä** omenalla etkä voi aluksi saada useampaa; mitä enemmän omenoita
  keräät kaikkiaan, sitä korkeampi on **tasosi**, joka lisää kentälle jatkuvasti
  yhden samanaikaisen omenan lisää ja nostaa pistekerrointa. **Siniset omenat**
  avaavat **kolikkopelin**: panoksena on pituutesi, rullien tulos moninkertaistaa
  tai kutistaa sen ja saa hetkeksi ilmestymään **ylimääräisiä omenoita** (jättipotti
  kolmella samalla symbolilla). **Violetit omenat** (uhkapeli) panostavat osan
  **koostasi** ja moninkertaistavat sen osan sattumanvaraisesti, loppu pysyy
  turvassa (uusi koko = koko·(1-p) + koko·p·kerroin): **normaali** panostaa
  kiinteät 50 % kertoimella **x0.5 .. x1.5**, **HARDCORE** on riskialttiimpi
  **75–90 %** panoksella ja kertoimella **x0.25 .. x2.25**. **Kokosi** näkyy
  **desimaalilukuna vasemmassa yläkulmassa** ja siirtyy tarkasti eteenpäin, joten
  seuraavat vedot rakentuvat sen päälle. Tasoja on **15** (kerroin jopa x16,
  enintään 16 omenaa kerralla); tasot sijaitsevat tiedostossa
  `games/levels/snake-comp.json` ja niitä voi laajentaa siellä koodiin koskematta,
  muu hienosäätö on tiedostossa `competitive.py`.
- **UUSI - HARDCORE** (kytkin Competitive-asetuksissa, näppäin **H**): jokainen
  **boost syö madon pituutta**; punaisena hehkuva **HARDCORE-teksti** merkitsee
  tilan. Vain Competitive-tilassa; pituus ei koskaan laske alle vähimmäismäärän.
  Muistetaan tiedostossa `settings.json`.
- **Kultaomenat** (väliaikaiset) antavat paljon pisteitä ja täyttävät boostin heti.
- Valinnaiset **läpäistävät seinät**, bonusomenat, **prestige** (yksinpeli, näppäin **P**).
- **UUSI - Mukauttaminen** (sivellinpainike aivan asetusnäkymän oikeassa
  yläkulmassa tai näppäin **C**): pelkästään visuaalinen valikko ("modit", jotka
  *eivät koskaan* muuta peliä) kahdella välilehdellä:
  - **Pää**: madon **pään väri** – 4 sinivihertävää esiasetusta (enemmän sinisestä
    enemmän turkoosiin), punainen, oranssi ja **oma väri** RGB-liukusäätimillä.
  - **Ruudukko (suunnistus)**: lisää kentän päälle **koordinaattiruudukon** –
    **rivinumerot** (vasemmassa ja oikeassa reunassa) ja **sarakekirjaimet**
    (ylhäällä/alhaalla). Näin suurilla kentillä näet heti, että esimerkiksi omena
    kohdassa *8a* on samalla rivillä *8* kuin oma sijaintisi *8z*. Värisarja
    (5 esiasetusta + kaksi omaa väriä A/B) määrää väriteeman.
  - **Banneri**: kytke kerroinbanneri (esim. violetista omenasta) **päälle/pois**
    ja säädä sen **kokoa** (pienempi/suurempi) ja **läpinäkyvyyttä** (läpinäkyvämpi)
    – reaaliaikaisella esikatselulla.
  Kaikki tallentuu tiedostoon `mem-ngb.json`; koko visuaalinen mukauttaminen
  kulkee moduulin `ngb.py` kautta.
- Ulkoasu: pyöristetty mato silmineen (pää oletuksena turkoosi), boost-hehku, partikkelit.

**Pong**
- Yksinpeli tekoälyä vastaan, moninpeli = pelaaja 2 oikealla. Ensimmäisenä 5 pisteeseen.
- **Liikkumistila vaihdettavissa ohjainsarjoittain**: *Jatkuva* (paina kerran ->
  jatkaa liikkumista, oletus) tai *Pito* (liikkuu vain painettaessa).
  Vaihto: **X** = ohjainsarja 1, **N** = ohjainsarja 2 (muistetaan tiedostossa `settings.json`).
- Pallon fysiikka kiihdytyksellä ja osumakohdan mukaisella kulmalla.

**Air Hockey**
- **Aitoa 2D-fysiikkaa**: pyöreät mailat ja kiekko liikemäärän siirrolla – kiekko
  ottaa mailan nopeuden osumassa; laidat kimmokertoimineen, kevyt jääkitka, maalit
  sivuseinien aukkoina.
- **Hiiriohjaus** yksinpelissä: maila seuraa hiirtä (mikä tahansa näppäin palauttaa
  näppäimistöohjaukseen). Näppäimistö: suuntanäppäimet 8 suuntaan, moninpeli = P1
  vasemmalla (WASD), P2 oikealla (IJKL).
- **Tekoäly kolmella tasolla** (Helppo/Keskitaso/Vaikea): puolustaa omaa maaliaan,
  hyökkää omalla puoliskollaan ja kiertää kiekon välttääkseen omat maalit.
- **Power-upit** (voi kytkeä pois): *XL* (isompi maila), *TOR* (vastustajan maali
  kutistuu), *>>* (nopeampi maila) – ne kuuluvat pelaajalle, joka kosketti kiekkoa
  viimeksi.
- Asetukset: vaikeustaso, **maalit voittoon** (3/5/7/10), power-upit päälle/pois
  (tallennetaan tiedostoon `settings.json`). Jokaisen maalin jälkeen aloittaa
  maalin päästänyt pelaaja.
- Ulkoasu: kiekon valojälki, partikkelit, sykkivät maaliaukot, tehostemerkit.

**Tic-Tac-Toe**
- Asetukset: vaikeustaso (Helppo/Keskitaso/Vaikea) ja laudan koko 3x3..9x9;
  voittopituus K = 3 (3x3), 4 (4x4), muuten 5.
- **1 pelaaja** tekoälyä vastaan (Vaikea 3x3:ssa on voittamaton) **tai 2 pelaajaa**
  paikallisesti (X vastaan O, vuorotellen klikkaamalla). Pelin päättyessä:
  Enter/klikkaus = uusi kierros, **S** = asetukset.

**Breakout**
- Tiililajit: Normaali, **Teräs** (tuhoutumaton), **Pommi** (räjähtää), **Kulta**
  (lisäpisteet).
- Power-upit: laser, tulipallo, tarttuva, kilpi, kolikko ja muita; **combo-kerroin**.
- Tehosteet: partikkelit, pallojäljet, näyttötärähdys, pistenostot, monia kenttäkuvioita.
- Asetukset: **1/2/3** = vaikeustaso, **Vasen/Oikea** = pallon väri, **Ylös/Alas** =
  aloituskenttä, **M** = rakenne. Peli: hiiri/nuolet, **Välilyönti** laukaisee pallon
  (ampuu laserin), **P/Esc** = tauko.

**Tetris**
- **Modernit Guideline-säännöt**: 10x20-kenttä, palat **7 palan pussista**,
  **SRS-kiertojärjestelmä** aidoilla wall kickeillä (myös I-palalle), kierto
  kumpaankin suuntaan, **varasto** (kerran palaa kohden), **5 palan esikatselu**,
  haamupala ja **lock delay** (0,5 s, enintään 15 nollausta).
- **Kolme tilaa** esinäytöllä: *Yksinpeli*, *Tekoälyä vastaan* ja *2 pelaajaa*.
  Yksinpeli tarjoaa asetuksissa **Maratonin** (aloitustaso 1–15, lasketaan
  ennätykseen), **Sprintin 40 riviä** (paras aika) ja **Ultran 2 minuuttia** (paras
  tulos); parhaat tulokset tallentuvat `mem.json`-tiedoston osioon `tetris`.
- **Guideline-pisteytys**: singlestä Tetrikseen, **T-Spinit** (täydet ja mini),
  **Back-to-Back** (x1,5), **combot** ja **Perfect Clear** – ruututeksteillä,
  rivinpoistoanimaatiolla, kovan pudotuksen jäljellä, partikkeleilla ja
  tasonnousutehosteella.
- **Versus roskariveillä**: poistetut rivit lähettävät roskaa vastustajalle
  (Tetris = 4, T-Spin Double = 4 …), saapuva roska näkyy varoituspalkissa ja
  **kuittautuu** omilla hyökkäyksilläsi; molemmat kentät saavat saman
  palajärjestyksen. **Tekoälyssä** on 3 tasoa, ja vauhti kiihtyy 40 sekunnin
  välein.
- **Ohjaus**: Vasen/Oikea omalla **DAS/ARR**-säädöllä (asetuksissa), Ylös = kierto
  oikealle, Alas = pehmeä pudotus, Toiminto = kova pudotus; **C**/Shift = varasto,
  **Z**/**Y** = kierto vasemmalle, **X** = kierto oikealle. Kaksinpelissä pelaaja 1
  varastoi **Q**:lla ja kiertää vasemmalle **E**:llä, pelaaja 2 **oikealla
  Shiftillä** / **oikealla Ctrl:llä**. Pelin jälkeen: **R** = uudestaan, **S** =
  asetukset.

**Invaders** – kaksi tilaa (valittavissa esinäytöllä):
- **Klassinen**: perinteinen alien-lohko; sen jälkeen asetusnäytöllä valittavissa:
  **Liikkuminen** (vain vasen/oikea *tai* vapaa WASD:lla) ja **Tähtäys** (aina
  ylöspäin *tai* kohti **hiirtä** – tällöin ammut sinne, missä kursori on). Tuhotut
  alienit pudottavat toisinaan power-upeja.
- **Areena (vapaa)**: vapaa liikkuminen kaikkiin suuntiin, viholliset virtaavat
  sisään joka reunasta; tähtäät liikkumissuuntaan, vaihda ase näppäimillä **1–4**.
Yhteistä: tasojärjestelmä, jossa **pomo** joka 4. kentässä, neljä asetta (blaster,
hajalaukaus, pikatuli, laser), power-upit (lisäelämä, kilpi, aseen päivitys),
räjähdystehosteet, ennätys.

**Asteroids**
- **Hitausfysiikka**: ylös = työntö katsesuuntaan, vasen/oikea = pyöri, alus jatkaa
  ajelehtimista (kevyt vaimennus); kaikki kiertää näytön reunojen yli. Klassinen
  **vektoriulkoasu** työntöliekillä ja tähtitaivaalla; jokaisella lohkareella on
  oma satunnainen monikulmiomuoto.
- Lohkareet hajoavat kahdeksi pienemmäksi (3 kokoa, **20/50/100 pistettä**),
  **aallot** kasvavilla määrillä ja banneri-ilmoituksella.
- **UFO** (voi kytkeä pois): ylittää näytön säännöllisesti ja tähtää aluksiin
  (tähtäysvirhe riippuu vaikeustasosta) – 200 pistettä alasampumisesta.
- **Power-upit** (voi kytkeä pois), pudonneet tuhotuista lohkareista: **S**uojakilpi
  (6 s haavoittumaton), **T** = kolmoislaukaus, **R** = pikatuli.
- **Hyperavaruus** (alas-näppäin): hätäsiirto satunnaiseen paikkaan 4 s jäähtymisajalla
  – ja 12 % riski räjähtää perillä.
- 3 elämää, turvallinen uudelleensyntyminen haavoittumattomuusvilkkumisella,
  **lisäelämä joka 5000 pisteestä**; räjähdyspartikkelit ja kameratärähdys.
- **Yhteistyökaksintaistelu** (moninpeli): molemmat alukset lentävät yhtä aikaa
  erillisillä elämillä ja pisteillä – enemmän pisteitä saanut voittaa.
- Asetukset: vaikeustaso, UFOt päälle/pois, power-upit päälle/pois (tiedostossa `settings.json`).

**Pac-Man**
- **Klassinen 28x31-labyrintti** neonilmeessä pillereineen, 4 voimapilleriä,
  sivutunnelien läpikäynnit ja haamutalo keskellä.
- **Neljä haamua alkuperäisillä käytöksillä** (kohderuutu-tekoäly): *Blinky* jahtaa
  suoraan, *Pinky* väijyy (4 ruutua edellä), *Inky* käyttää Blinkyn kautta kulkevaa
  vektoria, *Clyde* väistää lähietäisyydeltä.
- **Scatter/chase-vaiheet** vuorottelevat (haamut kääntyvät jokaisessa vaihdossa);
  **voimapilleri** muuttaa haamut sinisiksi ja syötäviksi (ketju 200/400/800/1600),
  sitten silmät palaavat taloon.
- Haamutalo **porrastetulla vapautuksella**, **hedelmä**bonukset (per kenttä),
  **3 elämää**, **lisäelämä 10 000 pisteessä**, tasojärjestelmä (nopeutuu),
  kuolinanimaatio, READY/GAME OVER -näytöt.
- Asetukset: **vaikeustaso** (Normaali/Vaikea/Äärimmäinen) – haamujen nopeus ja pelotusaika.
- Ohjaus: **nuolet tai WASD**.  Enter = uusi, S = asetukset.

**Flappy Bird**
- **Painovoimafysiikka**: Välilyönti / Ylös / W / **hiiren klikkaus** saa linnun
  räpyttämään; se kallistuu nousu-/laskuvauhdin mukaan.
- Loputtomat **putkiparit** aukolla (+1 per putki); **kolikot** (bonus) ja
  **kilpi**-power-up (selviää yhdestä osumasta) ilmestyvät aukkoihin.
- **Päivä/yö-teemat** vaihtuvat pistemäärän mukaan; ajelehtivat pilvet (parallaksi),
  vierivä maa.
- Vaikeustaso (Helppo/Normaali/Vaikea): aukon koko, vauhti, putkiväli – aukko kapenee
  hieman pistemäärän noustessa.
- **Mitalit** (pronssi/hopea/kulta/platina) game overissa, törmäysanimaatio
  kameratärähdyksellä, ennätys.

**Doodle Jump**
- Doodler **hyppää automaattisesti** laskeutuessaan; ohjaat vain vasen/oikea
  (hitaudella), reunat kiertävät ympäri, ja kamera vierii ylös noustessasi.
- **Tasotyypit**: vihreä (normaali), sininen (liikkuva), ruskea (hajoaa), valkoinen
  (katoaa). **Jouset** antavat superhypyn, **potkurihattu** kantaa hetkeksi ylös
  (ja tekee haavoittumattomaksi).
- **Hirviöt**: kosketus on tappava – mutta voit **ampua** ne näppäimillä Ylös /
  Välilyönti (bonuspisteet).
- Pisteet = saavutettu korkeus; vaikeus kasvaa korkeuden myötä. Ennätys.
- Ohjaus: vasen/oikea = liiku, Ylös / Välilyönti = ammu.

**2048**
- **Oma asetusnäyttö**: laudan koko **3x3:sta 8x8:aan** ja kolme tilaa:
  *Klassinen* (tavoite 2048, sen jälkeen "Jatketaanko?"), *Aikahaaste* (3
  minuuttia, kello käynnistyy ensimmäisestä siirrosta) ja *Loputon*.
- **Sulavat animaatiot**: laatat liukuvat, yhdistyvät "pop"-efektillä ja kasvavat
  esiin; pistepopupit, kipinät 128:sta alkaen, paineaalto 2048:sta alkaen ja uudet
  värit aina 131072:een asti. Animaation aikana annetut syötteet puskuroidaan.
- **Kumoaminen** (pois / 3 per peli / rajaton, näppäin **U** tai Backspace) – sen
  käyttäjä pelaa ilman ennätystä ja ilman laattasaavutuksia.
- **Tallenna ja jatka**: käynnissä oleva peli tallentuu automaattisesti koon ja
  tilan mukaan; parhaat tulokset ja suurin laatta koon/tilan mukaan löytyvät
  `mem.json`-tiedoston osiosta `g2048`.
- Ohjaus: nuolet/WASD tai **pyyhkäisy** hiirellä/kosketuslevyllä, **R**/**N** = uusi
  peli, **Tab** = asetukset. Ennätys lasketaan vain **4x4 Klassisessa** ilman
  kumoamista.

**Minesweeper**
- Kolme tasoa: **Aloittelija** (9x9, 10 miinaa), **Edistynyt** (16x16, 40),
  **Ekspertti** (30x16, 99) – **paras aika per taso** tallennetaan ja näytetään
  asetuksissa.
- **Ensimmäinen klikkaus on aina turvallinen** (miinat sijoitetaan vasta sen
  jälkeen, klikkauksen ympärillä oleva 3x3-alue pysyy vapaana).
- **Vasen klikkaus** = paljasta, **oikea klikkaus** = lippu (valinnaisesti
  kysymysmerkkikierrolla), **F** = lippu kursorin alle, **R** = uusi peli.
- **Chording**: valmiin numeron klikkaaminen paljastaa loput naapurit.
- Klassinen HUD: miinalaskuri, **klikattava hymiö** (hämmästynyt/aurinkolasit/kuollut),
  ajastin; väärät liput yliviivataan lopussa, konfetti voitossa.
- Pisteet = tason perusarvo miinus sekunnit.

**Sudoku**
- **4 muunnelmaa**, kussakin **400 kenttää** (4 vaikeustasoa x 100): *Klassinen*
  (tutut siemenpohjaiset kentät – ratkaistut pysyvät merkittyinä), *X-sudoku*
  (molemmat lävistäjät sisältävät jokaisen numeron tasan kerran), *Killer*
  (katkoviivahäkit summineen; 400 valmiiksi luotua kenttää) ja *Mini 6x6*.
  Jokainen pulma on **yksiselitteisesti ratkeava** – "Vaikean" kenttä 12 on sama
  pulma jokaisella tietokoneella.
- **Päivän sudoku**: yksi pulma päivässä kaikille, sama tietokoneella ja
  selaimessa; vaikeus riippuu viikonpäivästä (maanantain Helposta lauantain
  Ekspertiin), ja päivittäin ratkaisemalla kerryttää putken.
- **Enintään 3 tähteä kentältä** (ratkaistu · ilman virheitä ja vihjeitä ·
  lisäksi alle tavoiteajan) ja **paras aika** kenttävalinnassa; **kesken jääneet
  pulmat** tallentuvat automaattisesti ja jatkuvat seuraavalla kerralla.
- **4 pelitilaa** (valitaan ennen aloitusta) pistekertoimella: **Klassinen**
  (x2,0 - ei apuja), **Muistiinpanot** (x1,5 - + lyijykynämuistiinpanot ja
  automaattiset ehdokkaat), **Mukavuus** (x1,0 - + väärät numerot punaisella,
  ristiriidat ja väärät häkkisummat merkitään, oikeat syötteet lukittuvat),
  **Avustus** (x0,7 - + vihjenäppäin, enint. 3). Kun **3 virheen raja** on
  käytössä (asetusvaihtoehto), kolmas virhe päättää pelin.
- Ohjaus: nuolet/WASD = ruutu, **1-9** = numero (myös numeronäppäimistö),
  **0/Delete/oikea klikkaus** = pyyhi, **U**/**Z** = kumoa, **Y** = tee uudelleen,
  **N** = muistiinpanot, **C** = täytä ehdokkaat automaattisesti, **H** = vihje,
  **M** = värimerkki, **R** = aloita kenttä alusta, **Q** = kenttävalinta. Syöttö
  "numero ensin" (asetukset, **I**) ja **jäljellä olevien numeroiden laskuri**
  jokaisen numeron alla; täysin pelattavissa hiirellä. Pelin päätyttyä **A**
  näyttää koko **ratkaisun**.
- Pisteet = (muunnelman ja vaikeustason perusarvo - aika - virheet - vihjeet) x
  tilan kerroin; kaikki muunnelmat ja päivän sudoku lasketaan ennätykseen.

**Frogger**
- 5 liikennekaistaa (autot/kuorma-autot) ja 5 jokikaistaa (tukit, kilpikonnat jotka
  **sukeltavat** korkeammilla tasoilla); ylhäällä 5 kotipoukamaa – kaikkien
  täyttäminen = seuraava taso, kaikki nopeutuu.
- Lisät: **bonuskärpänen** (+200) tyhjissä poukamissa, **krokotiilit** valtaavat
  poukamia korkeammilla tasoilla, **aikarajapalkki** per sammakko, lisäelämä 10 000
  pisteessä.
- 3 vaikeustasoa (vauhti, liikennetiheys, aika); pisteet per uusi rivi, poukama =
  50 + aikabonus, kenttä valmis = +1000.

**Memory**
- Laudan koot **4x4, 6x6, 8x6**; kuviot ovat muoto-väri-yhdistelmiä, jotka on
  piirretty kokonaan primitiiveillä; **kääntöanimaatio**, epäsopivat parit kääntyvät
  takaisin automaattisesti.
- **Yksin**: perusarvo - 15 per siirto - 2 per sekunti (väh. 100). **Kaksintaistelu**
  (paikallinen): vuorotellen, osuma antaa uuden vuoron, eniten pareja voittaa.

**Pasianssi**
- **5 muunnelmaa** esinäytöllä: Klondike (nosto 1/3 valintana), Spider (1/2/4 maata),
  FreeCell (supersiirtoraja), Pyramidi (13-parit, 2 uudelleenjakoa) ja TriPeaks
  (±1-ketju combo-kertoimella).
- **Vedä ja pudota** tai klikkaus-klikkaus, **oikea klikkaus** = perustukseen,
  **U** = rajaton kumoaminen, **R** = uusi jako, Välilyönti = pakka.
- Kortit renderöidään ilman kuvatiedostoja (`games/cards.py`); kaikki muunnelmat
  jakavat yhden ennätyslistan muunnelmakohtaisilla kaavoilla.

**Aim Trainer**
- **Aitoa ohjelmistopohjaista 3D:tä** (kuten Snaken 3D-tila): kiinteä tähtäin näytön
  keskellä, **suora 1:1-hiiriohjaus kuten räiskintäpelissä** (osoittimen kaappaus:
  kursori pysyy ikkunan sisällä, Esc vapauttaa sen; säädettävä herkkyys, rajaton
  yaw, pitch ±60°). Vasen klikkaus ampuu tarkalleen keskeltä, suuliekillä,
  luotijäljellä ja osumapartikkeleilla.
- **4 tilaa**: tarkkuus (60 s, 3 palloa, tarkkuusbonus), refleksi (30 yksittäistä
  maalia, reaktioaikatilastot), liikkuvat maalit (radat + combo-kerroin jopa x4) ja
  chill (loputon, ei rangaistusta, **E** päättää istunnon).
- **3 teemaa** (asetuksissa, tallennetaan): **avaruus** tähtipallolla, **musta aukko
  hehkuvalla renkaalla** ja planeetalla (oletus), neonareena lattiaruudukolla ja
  synthwave-auringolla, sekä sisätiloissa oleva ampumarata.
- Herkkyyttä voi muuttaa kesken pelin näppäimillä **+/-**; lisäksi **säädettävä
  motion blur** (0-80 %) ylimääräiseen chill-ilmeeseen – molemmat tallennetaan.

**Neljän suora**
- 7x6-lauta **kiekon putoamisanimaatiolla**, kohdistuksen esikatselulla ja sykkivällä
  voittolinjalla; hiiri, nuolinäppäimet tai suora valinta **1-7**.
- **3 tekoälyn tasoa** (minimax alfa-beta-haulla): Helppo ohittaa tarkoituksella
  uhkia, Keskitaso torjuu luotettavasti, Vaikea suunnittelee syvälle – tai
  **2 pelaajaa** paikallisesti samalla laitteella.
- Aloittava pelaaja vaihtuu joka kierroksella; ennätys laskee **voitot tekoälyä
  vastaan** yhden istunnon aikana.

**Tankkikaksintaistelu**
- 2D-areenakaksintaistelu: **laukaukset kimpoavat seinistä kerran** (kimmoke) – osu
  nurkan takaa (tai itseesi!). Ensimmäisenä 5 kierrosta lähtölaskennalla.
- **4 areenaa** (Avoin, Risti, Pylväät, Labyrintti) tai satunnaiskierto;
  **power-upit**: pikatuli, kilpi, kolmoislaukaus.
- **Tekoäly 3 tasolla** – vaikea ennakoi laukauksiaan ja kimmottaa ne tahallaan
  seinistä – tai **2 pelaajaa** yhdellä näppäimistöllä (P1 WASD+Välilyönti,
  P2 nuolet+Enter).

**Blackjack**
- Aidot kasinosäännöt: **4 pakan kenkä**, jakaja jää 17:ään, **blackjack maksaa
  3:2**, jakajan kurkistus ässällä/10:llä; **tuplaus** ja **yksi jako** (jaetut ässät
  saavat yhden kortin kumpikin).
- **Laamamerkit**: Blackjack pelaa **Laamapankin** tilillä, joka on yhteinen
  Pokerin ja Casinon kanssa (alku 1000, tallentuu pysyvästi `mem.json`-tiedostoon).
  Panos veloitetaan heti jaettaessa; alle 10 merkin kohdalla Enter ottaa
  **pankkilainan**, joka täyttää tilin taas 1000:een.
- **Ennätys** = oman **Blackjack-saldosi** huippu (1000 plus kaikki Blackjackissa
  voitettu ja hävitty) – voitot ruletissa, automaatissa tai pokerissa eivät
  lasketa tähän, eivätkä myöskään lainat.
- Pelataan pelimerkkipainikkeilla ja näppäimillä (**H**it/**S**tand/**D**ouble/jako
  **X**, **1-4** = panos, Backspace = tyhjennä panos, Enter = jaa)
  korttianimaatioilla; jakajan piilokortti kääntyy nyt oikeasti paljastettaessa.

**Tunnel Racer**
- **3D-neonputkilento** (ohjelmistorenderöijä kuten Aim Trainerissa): palkit, lohkot
  ja **läpi pujoteltavat rengasportit**, kolikot ihannelinjalla.
- **Kaksi tilaa**: loputon (vauhti nousee kattoon asti, ennätys) ja **30
  siemenpohjaista kenttää** maaliviivalla, aikabonuksella ja merkityllä
  edistymisellä.
- **Näppäinohjaus** (oletus) tai **suora hiiriohjaus** (osoittimen kaappaus, näppäin
  **C**); lisäksi säädettävä **motion blur** (näppäin **B**, 0-80 %) – kaikki
  tallennetaan.

**3D-labyrintti**
- **Ensimmäisen persoonan raycaster Wolfenstein-tyyliin** (DDA, etäisyyssumu,
  spritet) mouselookilla + WASD, **minikartta** (näppäin **M**) ja vihreänä sykkivä
  uloskäynti – tai klassinen **2D-yläkuva** (näppäin **V** asetuksissa).
- **50 siemenpohjaista kenttää**, jotka kasvavat jatkuvasti; uloskäynti sijaitsee
  aina kauimpana lähdöstä, matkan varrella olevat **orbit** antavat bonuspisteitä.
- Pisteytys: 500 per kenttä + 100 per orb + aikabonus; ratkaistut kentät merkitään
  ja istunnon summa muodostaa ennätyksen.

**Reversi**
- **Othello 8x8:ssa**: aseta kiekkoja, jotka saartavat vastustajan rivit ja käännä
  kaikki saarretut; laittomat siirrot on estetty ja vuoro ilman laillista siirtoa
  **ohitetaan automaattisesti**.
- **Yksinpeli tekoälyä vastaan** (3 tasoa: negamax alfa-betalla, sijaintipainotus +
  liikkuvuus) **tai paikallinen kaksintaistelu**, Musta vastaan Valkoinen.
- Lailliset ruudut korostetaan; pelaa **hiirellä** tai valintakehyksellä (nuolet +
  Välilyönti/Enter). Jokainen voitto tekoälyä vastaan lasketaan yhtenä pisteenä
  ennätykseen.

**Yatzy**
- **Noppaklassikko**: 5 noppaa, enintään 3 heittoa per vuoro, **pidä** nopat
  yksitellen, sitten kirjaa yksi **13 kategoriasta** (mahdollisten pisteiden
  reaaliaikaisella esikatselulla).
- Täysi pistelappu: yläosa **63 pisteen bonuksella (+35)**, kolmoset/neloset,
  täyskäsi, pieni/suuri suora, **Yatzy (50)** ja Sattuma.
- **Yksinpeli ennätysjahtina** korkeimpaan loppusummaan, tai **2 pelaajan hotseat**
  kahdella pistelapulla vierekkäin; pelaa hiirellä tai näppäimillä (Välilyönti, 1-5,
  nuolet, Enter).

**Wordle**
- Arvaa piilotettu sana; värillinen palaute (vihreä/keltainen/harmaa) oikealla
  **toistuvien kirjainten laskennalla** ja värittyvällä ruutunäppäimistöllä
  (QWERTZ saksalle, tšekille, sloveenille ja kroatialle, AZERTY ranskalle, muuten
  QWERTY).
- **Neljä pelitilaa**: *Loputon* (sana toisensa perään, kullekin 6 arvausta;
  jokainen ratkaistu sana antaa pisteitä ja ensimmäinen ratkaisematon päättää
  pelin), *Päivän sana* (yksi sana päivässä kielen ja pituuden mukaan – sama
  tietokoneella ja selaimessa – lähtölaskennalla ja putkella; aloitettu päivän
  sana tallentuu), *Dordle* (2 sanaa yhtä aikaa 7 arvauksella) ja *Quordle* (4
  sanaa 9 arvauksella, näppäimet näyttävät kaikkien ruudukoiden värit).
- **Asetukset** ennen jokaista peliä: **sanan pituus 4–7**, **vaikea tila**
  (löydetyt vihjeet on käytettävä uudelleen) ja **värisokeiden paletti**
  (oranssi/sininen); vieressä näkyvät tilastot.
- **Aidot sanalistat kaikilla 14 kielellä** (kansio `woordlistz/`, vain A-Z),
  jokaiselle pituudelle omat: pelkästään 5 kirjaimen sanoja lähes **34 000
  ratkaisua** ja yli **213 000 sallittua arvausta**, kaikkien neljän pituuden
  yhteensä noin 134 000 ratkaisua. Ratkaisut ovat yleisiä sanoja ilman nimiä,
  englanninkielisiä jäänteitä ja loukkaavia sanoja; jokainen arvaus tarkistetaan
  listasta – muut hylätään ja rivi tärähtää hetken.
- **Tilastot** kielen, pituuden ja pelitilan mukaan: pelit, voittoprosentti,
  nykyinen ja paras putki sekä **arvausten jakauma pylväsdiagrammina**
  (`mem.json`-tiedoston osio `wordle`). **Jaa** (**C**) kopioi emojiruudukon
  leikepöydälle paljastamatta sanaa.
- Ennätykseen lasketaan vain *Loputon* 5 kirjaimella; muilla pituuksilla on omat
  parhaat tuloksensa. Saavutukset **Selvännäkijä** (enintään 2 arvausta),
  **Sanatapa** (7 päivän sanaa peräkkäin) ja **Nelinkertainen nero** (Quordle
  ratkaistu).

**Pokeri**
- **3 muunnelmaa** esinäytöllä: **Texas Hold'em** 1–3 tekoälyvastustajaa vastaan
  jakajanapilla, blindeillä ja neljällä panostuskierroksella, **5 Card Draw**
  (kaksin tekoälyä vastaan, yksi korttien vaihto) ja **Video Poker** (*Jacks or
  Better*, yksin voittotaulukkoa vastaan).
- Toiminnot painikkeilla tai näppäimillä: **F** = luovuta, **C** = sökö/maksa,
  **R** = korota, **A** = all-in; korttien pito/vaihto klikkaamalla tai **1-5**,
  **Enter** vaihtaa kortit tai jakaa seuraavan käden.
- Yhteisen **Laamapankin** **laamamerkit**: käden alussa tilisi on pöydällä
  pinona, ja pottiin menevä summa veloitetaan heti – pöydästä poistuminen kesken
  käden maksaa vain osuutesi potista. Rahat loppu (alle 20:n ison blindin, Video
  Pokerissa alle 10) = pankkilaina 1000:een.
- **Ennätys** = **Pokeri-saldosi** huippu (1000 plus kaikki pokerin voitot ja
  tappiot); myös **Merkkijohtaja**-saavutus laskee vain pokerisaldon.

**Shakki**
- **Täysi shakki**: kaikki nappuloiden siirrot mukaan lukien **linnoitus**,
  **ohestalyönti** ja **sotilaan korotus** (valitse nappula); **shakki, matti ja
  patti** sekä tasapelit **50 siirron säännöllä**, **kolminkertaisella toistolla**,
  **riittämättömällä materiaalilla** tai sopimuksella.
- **Kolme tilaa**: *peli* tekoälyä vastaan, *2 pelaajaa* samalla koneella (lauta
  voi kääntyä jokaisen siirron jälkeen) ja **tehtävät**.
- **Vahvempi, nykimätön tekoäly** 6 tasolla *Aloittelijasta* *Mestariin*:
  iteratiivinen syventäminen, transpositiotaulu, lepohaku, avauskirja ja arvio,
  joka huomioi liikkuvuuden, sotilasrakenteen ja kuninkaan turvallisuuden.
  Tekoäly laskee pienissä paloissa kuvaa kohden – peli ei koskaan nyi.
- **Asetukset**: värin valinta, **shakkikello** (ei, 1+0, 3+2, 5+0, 10+5) ja
  **Chess960** (kaikki 960 alkuasemaa, numero näkyy siirtolistan yläpuolella).
- **Sivupaneeli** kelloilla, lyödyillä nappuloilla, materiaalitasapainolla ja
  vieritettävällä **siirtolistalla (SAN)**; vedä ja pudota, liukuvat nappulat,
  koordinaatit. Näppäimet: **U** = kumoa, **H** = vihjenuoli, **O** = tarjoa
  tasapeliä, **X** = luovuta, **F** = käännä lauta, pelin jälkeen **P** =
  **PGN-vienti**.
- **Tehtävät**: 200 tehtävää 5 vaiheessa (matti 1/2/3 siirrolla, taktiikka I/II)
  **Lichessin vapaasta tehtävätietokannasta (CC0)**, tarkistettu pelin omalla
  moottorilla; mattitehtävissä kelpaa jokainen mattiin johtava siirto. Edistyminen
  tallentuu `mem.json`-tiedoston osioon `chess`.
- Kumoaminen ja vihje tekevät pelistä "avustetun": ennätykseen lasketaan vain
  voitot tekoälyä vastaan ilman apua (istuntoa kohden).

**Mylly**
- **Mylly** kaikilla kolmella vaiheella: **asettelu** (9 nappulaa kummallakin),
  **siirto** viivoja pitkin ja **lentäminen**, kun jäljellä on enää 3 nappulaa (voi
  kytkeä pois).
- Valmis **mylly** poistaa vastustajan nappulan (mieluiten myllyn ulkopuolisen);
  häviät, kun nappuloita on alle 3 tai et voi siirtää.
- **3 tekoälyn tasoa** (minimax alfa-betalla, vaihetietoinen arviointi) tai
  **paikallinen kaksintaistelu**; siirtovihjeillä, myllyjen korostuksella ja
  nappulalaskurilla.

**Simon**
- **Senso-muistipeli**: valaistu sarja kasvaa joka kierroksella ja se on toistettava
  tarkalleen.
- **Tilat**: *Klassinen*, *Speed* (nopeutuu), *Reverse* (takaperin), *Mixed* (tila
  vaihtuu joka kierroksella) ja kahden pelaajan **kaksintaistelu** (vuorotellen lisää
  ja toista).
- **Ääni** *pois / päällä / sekoitettu* (sekoitettu harjoittaa sekä visuaalista ETTÄ
  kuulomuistia), **4/6/9 painiketta** vaikeutena; **paras tulos per tila**
  tallennetaan. Pelaa hiirellä tai numeronäppäimillä 1-9.

**Biljardi**
- **8-ball**, **9-ball** ja säännötön **harjoittelu**tila, tekoälyä vastaan
  (tähtäysavulla) tai **kaksi pelaajaa paikallisesti**.
- **Kolme vapaasti valittavaa näkymää**: klassinen **2D-yläkuva**, kiinteä
  **3D-vinoperspektiivi** varjostetuilla palloilla ja **vapaasti kiertävä 3D-kamera**
  (hiiren oikea painike). Kaikki liike on aika-askelpohjaista ja **pehmeästi
  vaimennettua** (kitka, osa-askeleet läpitunneloitumisen estämiseksi).
- **Lyönti**: pidä hiiren vasenta painiketta pohjassa ladataksesi voimaa, vapauta
  lyödäksesi; tähtäysviiva ja voimamittari auttavat. **Pallo kädessä** virheen
  jälkeen. Näkymä (V) muistetaan tiedostossa `settings.json`; voitetut erät lasketaan
  ennätykseen.

**Liukupalapeli**
- 15-pulma kolmessa koossa: **3×3** (helppo), **4×4** (klassinen) ja **5×5**
  (vaikea); liu'uta numeroidut palat vapaaseen aukkoon.
- Aina ratkaistavissa (sekoitettu monilla satunnaisilla siirroilla). Ohjaa
  **klikkaamalla** palaa aukon rivillä/sarakkeessa (koko linja liukuu) tai
  **nuolinäppäimillä**.
- Pisteet = kokokohtainen perusarvo miinus siirrot ja aika; ratkaisu aloittaa heti
  uuden laudan.

**Mastermind**
- Murra piilotettu **värikoodi**; jokaisen arvauksen jälkeen saat **mustia** tappeja
  (oikea väri + sijainti) ja **valkoisia** tappeja (oikea väri, väärä paikka).
- **3 tilaa**: Helppo (4 tappia / 6 väriä / 12 riviä), Klassinen (4/6/10) ja Vaikea
  (5 tappia / 8 väriä); toistuvat värit sallittu.
- Pelaa väripaletilla (klikkaus tai näppäimet **1–8**), OK/Enter tarkistaa rivin.
  **Loputon putki** kuten Wordlessa: jokainen murrettu koodi antaa pisteitä.

**Bubble Shooter**
- **Puzzle Bobble** hunajakennoruudukossa: tähtää hiirellä, ammu kuplia ylös, **kolme
  tai useampi samaa väriä** puhkaisee ryhmän.
- Kuplat, jotka menettävät yhteytensä kattoon, **putoavat** (bonus); laukaukset
  **kimpoavat seinistä**, seuraavan kuplan esikatselulla.
- **3 tilaa** (4/5/6 väriä, osassa laskeutuvat rivit); game over punaisella viivalla.

**Hirsipuu**
- Arvaa sana **kirjain kerrallaan**; jokainen virhe piirtää osan hirsipuusta, häviät
  **6 virheen** jälkeen.
- **Kielikohtaiset sanalistat** (vain A–Z), **3 pituustilaa** (lyhyt / sekoitettu /
  pitkä); kirjoita tai klikkaa ruutunäppäimistöä.
- **Loputon putki**: jokainen arvattu sana antaa pisteitä (enemmän elämiä jäljellä +
  pidempi sana = enemmän).

**Block Jump**
- **3D-Minecraft-tyylinen tasoloikka** (ohjelmistopohjainen 3D kuten Snaken 3D-tila):
  loiki kelluvan **voxel-maailman** lohkojen yli hehkuvaan maaliin.
- **Minecraft-ulkoasu**: kaikilla palikoilla on aidot **pikselitekstuurit**
  (ruoho, multa, kivi, lankut, timantti, lima, puu); yksityiskohtien taso
  seuraa etäisyyttä (**T** = korkea/matala/pois).
- Lisäksi: kävelyanimoitu **Steve**-hahmo (kolmannen persoonan kamera), **käsi**
  ensimmäisessä persoonassa, **majakkasäde** maalissa, pyörivät **kultaharkot**
  kolikkoina, neliömäinen **aurinko**, **pikselipilvet** ja **sydämillä**
  varustettu HUD.
- Lohkotyypit: kiinteät lohkot (ruoho/multa/kivi/puu), **tikkaat** (kiipeä), **aidat**
  (hyppää yli), **jousiblokit** (sinkoa) ja **kolikot**.
- Kamera **oletuksena ensimmäisessä persoonassa kuten Minecraftissa**, **V** vaihtaa
  seurantakameraan; **hiiriohjaus** osoittimen kaappauksella, säädettävä **motion
  blur** (**B**) ja herkkyys (**+/-**).
- **Siemenpohjaiset parkour-kentät** vaikeutuvat; maali = pisteet + aikabonus, kolikot
  +50, putoaminen maksaa elämän (aloitat 3:lla). Ohjaus: WASD/nuolet, **Välilyönti**
  hyppää.

**Tower Defense**
- **Loputon aaltopuolustus** **4 kartalla** (Niitty, Kanjoni, Risteys,
  Kujanjuoksu), jokaisella oma polkunsa; lukitut kartat avautuvat parhaalla
  aallollasi, **pomo** saapuu joka **8. aallolla**.
- **3 tilaa**: Klassinen (7 tornia, päätila), Kompakti (4 tornia, 2 tasoa) ja
  Maksimi (**11 tornia**, **A/B-erikoistuminen** korkeimmalla tasolla,
  erikoisviholliset, aktiiviset kyvyt **Meteori/Pakkasnova/Kultaryntäys**).
- **11 tornityyppiä** nuolesta laseriin ja kultapankkiin, kussakin jopa
  **3 parannustasoa**, myynti palauttaa 70%; vihollisilla panssaria,
  uusiutumista, jakautumista, naamiointia, parannusaura ja ilmareitti.
- **Talous**: kultaa jokaisesta tuhotusta, aaltobonus + 5% korkoa; pisteitä
  tuhotuista ja aalloista. **F** = 2x nopeus, **G** = kantamat, oikea painike
  peruu.

**Minigolf**
- **360 rataa 40 kierroksella**: *Classic* ja *Pro*, joissa on yhdeksän käsin
  rakennettua rataa kumpikin, **Tour** 38 kierroksella à yhdeksän luotua rataa
  (yhteensä 342) ja nousevalla vaikeudella, lisäksi *Random* koko joukosta.
  Kierros 7, rata 3 näyttää kaikkialla samalta - mitään ei tarvitse tallentaa.
- **Alustat ja esteet**: hiekka jarruttaa, rampit kiihdyttävät, vesi maksaa
  rangaistuslyönnin, kumipuskurit antavat vauhtia takaisin, ja tuulimyllyt ja
  vaeltavat lohkot vaativat ajoitusta. Fysiikka lasketaan osa-askelin kitkan
  kanssa kuten biljardissa - mikään ei nyki eikä läpäise laitaa.
- **Ohjaus**: hiiri tähtää, vasen painike pohjassa lataa voiman ja irrotus lyö
  (nuolet + välilyönti käyvät myös). **R** peruu ladatun lyönnin lyömättä.
  **G** vaihtaa tähtäysviivan, **Z** autotähtäyksen, **P** pallon noston.
- **Voimalukko (pidä oikeaa painiketta pohjassa)**: jäädyttää latauspalkin
  täsmälleen siihen, missä se on - kullanvärisenä, prosentteineen, lukkokuvake
  ja sykkivä rengas pallon ympärillä. Näin odotat myllyn aukkoa lyönti valmiina.
  Kun päästät irti, lataus jatkuu; lukittu voima säilyy myös lyönnin yli, ja
  seuraava vasen napsautus lyö juuri sillä arvolla.
- **Tuloskortti** oikealla parin ja ratakohtaisten lyöntien kanssa; kaksinpelissä
  molemmat pelaavat saman radan vuorotellen. Pisteet: 600 radalta, ±300 jokaisesta
  parin alittavasta/ylittävästä lyönnistä, **500 lisää hole in onesta**. Pienin
  lyöntimäärä kierrosta kohti on tiedoston `mem.json` osiossa `minigolf`.
- **Noston voi kytkeä pois**: oletuksena rata päättyy kahdeksan lyönnin jälkeen
  ja lasketaan pienimmällä arvolla. Jos haluat pelata upotukseen asti, aseta
  *Pallon nosto* asetusruudussa POIS (tai paina **P**).
- **Autotähtäyksen voi kytkeä pois**: oletuksena maila kääntyy ennen jokaista
  lyöntiä itsestään reikää kohti. Jos haluat tähdätä joka reiällä itse, aseta
  *Autotähtäys* asetusruudussa POIS (tai paina **Z**) - silloin viimeksi
  valittu suunta jää voimaan, ja uuden reiän avauspaikalla maila osoittaa
  neutraalisti ylös.
- **F** nollaa käynnissä olevan reiän: lyönnit nollaan, pallo avauspaikalle - sama
  reikä, sama rata.
- **Jatka toiston sijaan**: kierroksen lopussa **Seuraava**-painike vie
  seuraavalle radalle (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), joten sama
  yhdeksän reiän sarja ei toistu; vieressä **Uudelleen** (sama rata) ja
  **Asetukset**. Näppäimet: Enter = jatka, R = uudelleen, S = asetukset.
- **Kierroksen uusinta**: lopuksi **P** (tai painike **Uusinta**) näyttää koko
  kierroksen lyönti lyönniltä. **S** vie sen arkistoon (sivupalkin painike
  **Uusinnat**).
- **Rakenna ja jaa omia reikiä**: valmistelunäytön **MAPS**-välilehti vie
  omaan kokoelmaasi - **Uusi** avaa reikäeditorin. Jokainen reikä saa nimen ja
  **id**:n (pienet kirjaimet, ei välilyöntejä); id on samalla jaettaessa
  ehdotettu tiedostonimi. Seitsemän klassisen esteen rinnalle tulee **kahdeksan
  uutta**: putki (siirtää pallon toiseen päähän), jää, tahmea alue, kiihdytin,
  magneetti, yksisuuntainen portti, pyörölevy ja hyppyri. Reiän koko on
  vapaasti säädettävissä (60x80 - 160x240), **12 pohjaa** antaa lähtökohdan, ja
  kumoa/tee uudelleen sekä **Testi** kuuluvat mukaan. **Jaa** kirjoittaa
  täsmälleen yhden reiän `.lamapgzmap`-tiedostoksi - tallennusikkunan kautta
  tai suoraan Lataukset-kansioon, nimesi tekijänä. **Tuo** lukee sen takaisin
  ja siirtyy automaattisesti `-2`:een, jos id on varattu. Nimi, id ja tekijä
  kulkevat aina **kaikki 14 kieltä kattavan sanasuodattimen** läpi. Yksi reikä
  pelataan **Pelaa**-napista, koko kokoelma viidennellä ratavalinnalla
  **Omat**.

**Pinball**
- **Kolme pöytää**: *Classic* (kolme puskuria, yksi maalirivi), *Space* (neljä
  puskuria vinoneliössä, kaksi riviä) ja *Lama* (avoin kenttä, kuusi maalia
  kaaressa); 3 tai 5 palloa per peli, kaksinpelissä vuorotellen pallo kerrallaan.
- **Kaikki mitä flipperi tarvitsee**: laukaisukaista latauspalkilla (liian heikko?
  pallo palaa ja saat yrittää uudelleen), kaksi flipperiä, slingshotit, maalirivit,
  neljä **L-A-M-A**-kaistaa, lukko pallolle, **multiball ja jackpot**, kuuden
  sekunnin **pallonpelastus**, töytäisy ja **TILT**.
- **Kerroin aina x5 asti** kaadetuista riveistä ja täysistä kaistoista; puskurit
  100, slingshotit 50, maalit 250 - multiballissa puskurit maksavat 2 500 pisteen
  jackpotin.
- Flipperit toimivat määritetyillä vasen/oikea-näppäimillä (myös vasen/oikea
  [Shift]) tai hiirellä. Pöytäkohtainen ennätys on tiedoston `mem.json` osiossa
  `pinball`.

**Bowling**
- **Kymmenen framea virallisilla säännöillä**, mukaan lukien striket, sparet ja
  kymmenennen framen bonusheitot (enintään: 300). **Tuloskortti** otsikon alla
  näyttää jokaisen framen merkeillä X, / ja juoksevan summan.
- **Heitto neljässä vaiheessa**: sijainti, kulma, kierre ja voima. Jokainen säädin
  heiluu itsestään ja lukitaan toimintonäppäimellä - tai asetetaan käsin
  vasen/oikea-näppäimillä, jolloin heilunta pysähtyy.
- **Aito keilafysiikka**: kymmenen keilaa ympyröinä, joilla on massa ja jotka
  kaatavat toisiaan; strike syntyy fysiikasta eikä onnesta. Rata on edestä öljytty,
  joten **hook** puree vasta viimeisellä kolmanneksella.
- Perspektiivinen ratanäkymä kouruineen, tähtäysnuolineen ja keila-alueineen;
  kolme vaikeustasoa (*Helppo/Normaali/Pro*) muuttavat säätimien vauhtia ja
  hajontaa. Tasokohtainen ennätys on tiedoston `mem.json` osiossa `bowling`.
- **Pelin uusinta**: lopuksi **P** näyttää kaikki heitot uudelleen ja **S**
  tallentaa ne arkistoon (painike **Uusinnat**).

**Crossy Road**
- **Loputonta hyppimistä** niittyjen (puut ja kivet tukkivat tien), autojen ja
  rekkojen täyttämien teiden, tukkien ja lumpeiden täplittämien jokien ja
  **rautateiden** yli, joilla juna syöksyy ohi varoitusvalon ja kellon jälkeen –
  myöhemmin odottavat kokonaiset asemat jopa 5 raiteella. Reitti syntyy rivi
  kerrallaan, siinä on aina kuljettava polku, ja vauhti ja liikenne kasvavat.
- **Isometrinen vokselityyli**: hahmot, ajoneuvot ja puut varjostetuista
  kuutioista (esirenderöity jokaiselle ruutukoolle), pehmeästi seuraava kamera,
  squash & stretch hypätessä, vesiroiskeet, litistymisanimaatio, höyhenet ja
  kimaltelevat kolikot; rivistä 50 alkaen **päivän ja yön vaihtelu** ajovaloineen.
- **Kotka**: kamera etenee hiljalleen – liian kauan vitkutteleva tai yli kolme
  riviä peruuttava joutuu kotkan nappaamaksi (punainen reuna varoittaa ensin).
  Tukin mukana kuvan ulkopuolelle ajautuminen päättää myös pelin.
- **Kolikot ja hahmot**: kerätyt kolikot (jättikolikko = 5) tallentuvat, ja niillä
  ostetaan uusia hahmoja **Hahmot**-välilehdeltä: sammakko, possu, pingviini,
  kissa, kettu, laama, robotti, aave ja yksisarvinen (25–250 kolikkoa); kana on
  mukana alusta asti.
- **Tilat**: *Loputon* (pisteet = pisin rivi, lasketaan ennätykseen) ja *Päivän
  reitti* (tänään sama kaikille, myös selaimessa, omalla päivän ennätyksellä).
  Ohjaus: nuolet/WASD, välilyönti/Enter/klikkaus = hyppy eteenpäin; asetuksissa
  **H** = varjot, **N** = päivä/yö. Kolikot, hahmot ja päivän ennätys tallentuvat
  `mem.json`-tiedoston osioon `crossy`.

**Geometry Dash**
- **Rytmitasohyppely**: hahmo kiitää itsestään oikealle – sinä päätät vain, milloin
  hypätään tai lennetään. **Viisi muotoa** – kuutio, alus, pallo, UFO ja aalto –
  sekä muoto-, painovoima- ja nopeusportaalit (0,5x–3x), keltaiset/pinkit/siniset
  **ponnahtimet ja pallot**, puolikaspalikat, piikit, kuopat ja värilaukaisimet.
- **8 valmista kenttää** *Helposta* *Demoniin* ("Lama Inferno"), kussakin **3
  salaista kolikkoa**. Jokainen kenttä on todistetusti läpäistävissä:
  rakennusvaiheessa ratkaisija läpäisi sen pelin oikealla koodilla – kaikkine
  kolikkoineen ja jopa 1/240 sekuntia siirrettynä.
- **Tarkka fysiikka**: kiintolukulaskenta kiinteällä 240 Hz:n askeleella; jokainen
  painallus vaikuttaa juuri siinä askeleessa, jossa se tapahtui – sama millä
  tahansa kuvataajuudella ja bitilleen sama selaimessa.
- **Harjoitustila** (**P**) automaattisilla ja omilla tarkistuspisteillä (**Z**
  asettaa, **X** poistaa), yrityslaskuri, edistymispalkki, räjähdykset ja
  välitön uusintayritys (**R**). Jokaisella kentällä on **oma soundtrack** – tausta,
  maa ja pallot sykkivät tahdissa (musiikin saa pois **M**:llä).
- **Tähdet ja kolikot**: kun läpäiset kentän normaalitilassa, saat sen tähdet, ja
  jokainen kolikko on yhden lisätähden arvoinen; ennätys on **tähtien
  kokonaismäärä** (enintään 65). Kenttäkohtaiset parhaat, kolikot, yritykset ja
  hypyt tallentuvat `mem.json`-tiedoston osioon `geodash`.
- **Kenttäeditori** **KENTÄT**-välilehdellä: ruudukkopohja, 6 ryhmän paletti
  (palikat, vaarat, ponnahtimet ja pallot, portaalit, nopeus, lisät), kääntö,
  kumoa/tee uudelleen, yleiskuvapalkki, **testaus alusta tai tästä kohdasta** ja
  kentän asetukset (aloitusnopeus, aloitusmuoto, musiikkityyli, BPM, värit).
  **"Varmistettu"**-merkki tulee vasta, kun olet itse läpäissyt kenttäsi.
  **Jaa** kirjoittaa `.lamapgzlevel`-tiedoston ja **Tuo** lukee sen takaisin;
  kentät tallentuvat `ugc.json`-tiedostoon omien minigolfratojen viereen.

**Battleship**
- **Meritaistelu 10x10-ruudukolla**: lentotukialus (5 ruutua), taistelulaiva (4),
  risteilijä (3), sukellusvene (3) ja hävittäjä (2) – voittaja on se, joka upottaa
  ensin koko vihollislaivaston.
- **Laivaston sijoitus** vetämällä telakasta: **R** tai oikea klikkaus kääntää,
  esikatselu hehkuu vihreänä tai punaisena, **X** sijoittaa kaiken satunnaisesti,
  **C** tyhjentää laudan; viimeisin sijoittelusi ehdotetaan uudelleen.
- **Säännöt asetuksissa** (tallentuvat): *laivat saavat koskettaa toisiaan*,
  *salvo* (vuorossa yhtä monta laukausta kuin omia laivoja on pinnalla) ja *osuman
  jälkeen ammu uudelleen*.
- **Tekoäly 3 tasolla**: Helppo ampuu satunnaisesti, Keskitaso jahtaa osumia
  järjestelmällisesti, Vaikea laskee **todennäköisyyskartan** shakkilautapariteetilla
  (keskimäärin noin 70 / 60 / 45 laukausta koko laivastoon). Tai **2 pelaajaa**
  samalla koneella – **luovutusnäyttö** piilottaa molemmat laivastot ennen
  jokaista vuoroa.
- **Grafiikka**: tutkapyyhkäisy, animoidut aallot, kaarevat ammukset, roiskeet,
  räjähdykset savuineen ja palavat ruudut, "UPPOSI!"-paljastus sekä kierroksen
  yhteenveto laukauksista, osumista ja osumatarkkuudesta. Ennätys laskee
  **voittosi tekoälyä vastaan** yhden istunnon aikana.

**Casino**
- **Ruletti** (eurooppalainen, 37 lokeroa): kaikki klassiset panokset
  klikkaamalla numeroa, reunaa tai kulmaa – **plein** (35:1), cheval,
  transversale, carré, sixain, sarake, tusina, punainen/musta, parillinen/pariton ja
  manque/passe. Merkkiarvot 1/5/25/100/500, oikea klikkaus poistaa merkkejä;
  **Pyöritä**, **Toista** (**R**), **Tuplaa** (**D**) ja **Tyhjennä**. Kuula kiertää
  spiraalina ennalta arvottuun lokeroon, ja ylhäällä näkyvät 12 viimeisintä numeroa.
- **Laama-automaatti**: 5 rullaa x 3 riviä, **10 voittolinjaa**, **laama = jokeri**,
  **kultakolikot = scatter**, joista 10 ilmaiskierrosta tuplavoitoin, panos per
  linja 1/2/5/10, **automaattipyöritys** (10/25), **turbo** ja voittotaulukko.
  **Palautusprosentti on 96,1 %** – laskettu tarkasti rullanauhoista.
- **Laamapankki**: Casino, Blackjack ja Poker jakavat yhden **laamamerkkitilin**
  (alku 1000, `mem.json`-tiedoston osio `casino`); vanhat merkkisaldot siirtyvät
  automaattisesti. Panokset veloitetaan heti, jokainen peli pitää ennätystään
  varten omaa saldoaan, ja rahojen loppuessa saat **pankkilainan** 1000:een.
- Konfetti, kolikkosade, big/mega/jackpot-bannerit ja voittolinja-animaatiot;
  saavutukset **Napakymppi** (voitettu plein ruletissa) ja **Laamajackpot** (5 laamaa
  samalla linjalla).

Ennätykset tallennetaan tiedoston `mem.json` osioon `highscores` (koodin vieressä)
– yhdessä kielen kanssa (osio `mem`).

### Käyttöliittymä

Koko käyttöliittymä on piirretty alusta asti (pelkkä Tkinter + Pygame, ei
ylimääräisiä paketteja) ja viimeistelty modernin pelilauncherin tyyliin:

- **Sivupalkin peliluettelo**: jokaisella rivillä on oma **minipiktogrammi** pelin
  korostusvärissä, näyttää nykyisen **ennätyksen (★)** ja reagoi pehmeästi animoiduilla
  hover-tehosteilla. Käynnissä oleva peli pysyy värillä korostettuna; pienissä
  ikkunoissa luettelo **vierii** hiiren rullalla.
- **Tilakortti** vasemmassa alakulmassa **tila-LEDillä** (harmaa = valikko, vihreä =
  käynnissä, kulta = tauko, punainen = game over) ja **reaaliaikaisella FPS-lukemalla**.
- **Aloitusnäyttö** revontulivaloilla, parallaksitähtikentällä tähdenlentoineen,
  kelluvalla logolla kiertävine kipinöineen, **klikattavalla peliruudukolla** heti
  logon alla (kaikki pelit hover-tehosteella korostusvärissään) ja **ennätysten
  kulkupalkilla**.
- **Tehosteita kaikkialla**: pehmeät näyttösiirtymät, kipinöitä valikon vahvistuksessa,
  **konfettisade uudesta ennätyksestä** ja aito **sumennus** taukopeiton takana.
- Jokaisen pelin **esinäyttö** ilmestyy kyseisen pelin korostusvärissä ja näyttää
  edellisen ennätyksen chippinä. Kun tiloja on paljon ja resoluutio pieni, se
  muuttuu **tiiviiksi**: Asetukset, Wiki ja Takaisin siirtyvät samalle riville ja
  fontti mukautuu – mikään ei enää valu kuvan ulkopuolelle.
- **Yhtenäinen pelin ilme**: kaikki 46 peliä jakavat valikon teemapaletin ja fontin –
  HUDit, asetusnäytöt ja peittokerrokset noudattavat asetuksissa valittua ulkoasua
  (v4.1 / v4 / Klassinen), kun taas jokainen pelikenttä säilyttää identiteettivärinsä.
  Jokainen peli käsittelee nyt siististi resoluution vaihdon kesken pelin, ja valikon
  nimet mukautuvat kieleen (esim. "Schach" → "Shakki").
- **Pelin sisäinen wiki** ("LamaWiki"): yksityiskohtainen ohje jokaiselle pelille
  (ohjaus, tilat, pisteytys, vinkit) sekä yleiset sivut – **hakukentällä**,
  kategorioilla, vieritettävillä artikkeleilla ja näppäinsiruilla, kaikilla 14
  kielellä. Saavutettavissa sivupalkin **"Wiki / Ohje"** -painikkeella ja jokaisen
  pelin esinäytöstä (avaa suoraan kyseisen pelin sivun).
- **Saavutukset ja tilastot**: **107 saavutusta** kolmessa kategoriassa (23 koko
  kokoelman tavoitetta, 37 pistevirstanpylvästä ja 47 erityistä hetkeä kuten
  shakkimatti tekoälylle, 4096-laatta, T-Spin Double, 25 ratkaistua
  shakkitehtävää, Killer Sudoku tai laamajättipotti; 2048:ssa ja shakissa
  kumoamista tai vihjeitä käyttäneitä pelejä ei lasketa); avautuessa **kultainen
  ilmoitus ja fanfaari** - myös kesken pelin; vanhat ennätykset hyvitetään
  automaattisesti. Lisäksi **tilastot**-välilehti: kokonaispeliaika,
  pelikerrat, voitot, ennätykset, lempipeli ja peliajan mukaan järjestetty
  pelitaulukko. Avautuu sivupalkin painikkeesta **"Saavutukset ja tilastot"**.
- **Uusinnat**: minigolf ja keilailu tallentavat jokaisen kierroksen. Lopuksi
  **P** näyttää uusinnan ja **S** vie sen arkistoon, joka avautuu sivupalkin
  painikkeesta **Uusinnat** (välilehti per peli, tauko, hypyt jaksojen välillä,
  nopeus 0,5x - 4x). Voidaan kytkeä pois ensikäynnistyksessä ja asetuksissa.

### Käyttö

- Valitse peli vasemmalla olevan valikon painikkeella. Sen jälkeen ilmestyy
  **esinäyttö**: valitse **Yksinpeli** tai **Moninpeli**, siirry **asetuksiin** tai
  takaisin. Nuolet/hiiri valitsemiseen, Enter aloittaa.
- **ESC** = tauko / jatka (valikoissa: takaisin).
- **F11** (tai painike "Koko näyttö päälle/pois") = koko näyttö päälle/pois.
  Pygame-näyttö pysyy upotettuna ja skaalataan kuvasuhde säilyttäen (mustat palkit,
  jos kuvasuhde poikkeaa). Ikkunan kokoa voi muuttaa vapaasti.
- **"Takaisin valikkoon"** päättää pelin ja tallentaa ennätyksen – samoin
  siirtyminen toiseen peliin sivupalkista.
- **Kiinteät lisänäppäimet**: viiden määritettävän toiminnon lisäksi joissakin
  peleissä on omia näppäimiä (esim. varasto **C** ja kierto vasemmalle **Z**
  Tetriksessä, kumoa **U** 2048:ssa, shakissa ja Sudokussa). Ne toimivat vain,
  jos näppäintä ei ole asetuksissa määritetty millekään toiminnolle, ja ne
  kerrotaan asetusvihjeessä ja wikissä. Pohjassa pidetyt näppäimet tunnistetaan
  oikein ja vapautetaan tauolla tai Alt-Tabilla – mikään ei enää "jumitu".
- **"Lopeta"** sulkee Pygamen ja Tkinterin siististi.

### Asetukset, ohjaus ja ääni

Asetusnäyttö avautuu **"Asetukset / Ohjaus"** -painikkeella (vasemmalla) tai
esinäytöstä. Se on jaettu **kolmeen välilehteen** (**Yleiset / Ohjaus / Ulkoasu**;
vaihda klikkaamalla tai Tab-näppäimellä):

- **Yleiset**: **ääni** päälle/pois, **äänenvoimakkuus** ja **haptiikka** (peliohjaimen
  värinä, toimii vain kytketyllä ohjaimella) sekä **automaattinen resoluutio**,
  **resoluutio**, **FPS** ja **kieli** – kukin vaihdetaan Vasen/Oikea-näppäimillä.
- **Ohjaus**: **esiasetukset** (*WASD + nuolet*, *WASD + IJKL*, *Nuolet + WASD*) ja
  **jokaisen yksittäisen näppäimen uudelleenmääritys** pelaajalle 1 ja pelaajalle 2:
  valitse rivi, paina Enter, paina haluttua näppäintä (Esc peruuttaa).
- **Ulkoasu**: valitse **käyttöliittymän ulkoasu** – **UI v4.2** (oletus: Midnight
  Glass – syvä keskiyön liukuväri, jossa ajelehtii hitaasti pehmeitä valoja indigona,
  turkoosina ja magentana, lisäksi hieno filmirae, harvat tähdet ja paneelit kuin
  himmeää lasia valoreunalla), **UI v4.1** (kuten UI v4, mutta eloisampi – hillityt
  tähdet sekä Saturnus ja musta aukko aloitusnäytön taustalla), **UI v4.1.1** (kuten
  v4.1, mutta tähtitaivaan tilalla laatoitettu **siksak-kuvio** mustana ja
  antrasiittina), **UI v4.1.2** (sama kuvio paletin sinisillä – korostussininen
  hallitsevana värinä ja tummempi sininen pohjana), **UI v4.1.3** (sama kuvio UI v4:n
  indigolla mustalla pohjalla), **UI v4.1.4** (UI v4:n grafiittisävyllä mustalla
  pohjalla), **UI v4** (täysin rauhallinen, litteä grafiitti-ilme yhdellä
  indigokorostuksella), **UI v3** (aiempi klassinen käyttöliittymä tähtitaivaalla,
  revontulivaloilla ja hehkutehosteilla), **UI v2** (ensimmäinen käyttöliittymän
  uudistus: laivastonsininen liukuväri, tähtitaivas ja hehkuvat painikkeet, täysin
  ilman animaatioita) tai **UI v1** (ulkoasu ennen käyttöliittymän uudistusta:
  yksivärinen tumma tausta, litteät painikkeet, ei tehosteita). Kaikki kortit
  näyttävät pienen esikatselun; valinta vaikuttaa heti koko käyttöliittymään (pelialue
  **ja** sivupalkki) ja tallennetaan.

Asetukset tallennetaan pysyvästi tiedostoon `settings.json`. **Yksinpelissä** molemmat
määritykset ohjaavat samaa hahmoa (oletus: WASD *ja* nuolet), **moninpelissä** yksi
kummallekin. Kaikissa peleissä on **äänitehosteet** (proseduraalisesti tuotettuja, ei
ylimääräisiä tiedostoja) jotka voi mykistää globaalisti.

### Projektin rakenne

```
install-python.bat  Windows-asennus: Python 3.13 + .venv + pygame
start.bat            Käynnistysskripti (Windows)
start.sh             Käynnistysskripti (Linux / macOS / Git Bash)
pyinstall.bat        EXE-käännös (Windows): pakkaa kaiken tiedostoon builds\PyGameZ.exe
main.py              Tkinter-käyttöliittymä, Pygame-upotus, keskeinen pelisilmukka
game_base.py         Pelin perusluokka (update/draw/handle_event) + InputEvent + apurit
settings.py          Asetusten lataus/tallennus (ääni/haptiikka/näppäimet/peliasetukset tarkistussäännöillä) (JSON)
audio.py             Proseduraaliset äänitehosteet, musiikkisilmukat + peliohjaimen tärinä
menu.py              Kieli-, esinäyttö- (tila) ja asetusnäytöt (ääni/ohjaus)
highscore.py         Ennätysten lataus/tallennus (osio tiedostossa mem.json)
store.py             Keskitetty tallennustiedosto mem.json (osiot: mem, highscores, stats, achievements + pelien edistyminen), atominen .bak-varmuuskopiolla
stats.py             Pelaajatilastot (pelikerrat, peliaika, voitot, ennätykset) peliä kohden
achievements.py      Saavutukset: määritykset, avaaminen, ilmoitus (toast)
progress.py          Saavutukset ja tilastot -näyttö (kaksi välilehteä, vieritettävä)
replay.py            Uusintojen tallennus ja arkisto (replay.json)
replayview.py        Uusintanäyttö: arkistolista ja toisto
ugc.py               Oma sisältö (minigolfradat, Geometry Dash -tasot): tallennus, tarkistus, vienti/tuonti (ugc.json)
swear.py             Sanasuodatin nimille ja id:ille (lang/swear/*.yml, 14 kieltä)
filepick.py          Tiedostoikkunat ("Vie nimellä ...", "Tuo")
prestige.py          Snaken prestige-järjestelmä
competitive.py       Snaken Competitive-tilan hienosäätö (tasot, kolikkopeli, veto-omenat)
ngb.py               Visuaalinen mukauttaminen ("modit"): pään väri + koordinaattiruudukko + valikko (mem-ngb.json)
lamabank.py          Laamapankki: Blackjackin, Pokerin ja Casinon yhteinen merkkitili (osio casino mem.json-tiedostossa)
seedrand.py          Satunnaislukugeneraattori, joka tuottaa bitilleen samat luvut Pythonissa ja selaimessa (päivän tilat, uudet pulmat)
i18n.py              Käännösmoottori (lataa lang/*.json, t("avain"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Kielitekstit (yksi paikkamerkkiavain per teksti)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Sanasuodatinlistat kielittäin (regex, .yml)
lamawiki/
  lamawiki.py          Pelin sisäinen wiki (haku, kategoriat, artikkelirenderöijä)
  de.json  en.json  fr.json  es.json  pt.json   Wiki-sisältö (yksi sivu per peli + yleiset sivut)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Rakentaa Wordlen sanalistat uudelleen (sanakirjat + taajuuslistat)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 kirjainta), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 kirjainta), 14 kieltä
devtools/            Kehitystyökalut (eivät tule .exe-tiedostoon)
  merge_staging.py           Vie devtools/staging/-kansion käännökset ja wikisivut kaikkiin 14 kielitiedostoon
  build_chess_puzzles.py     Rakentaa 200 shakkitehtävää Lichessin tehtävätietokannasta (CC0)
  build_sudoku_killer.py     Luo 400 yksiselitteisesti ratkeavaa Killer Sudokua
  build_crossyroad_models.py Kirjoittaa Crossy Roadin vokselimallit selainversiota varten
  build_geodash_levels.py    Rakentaa 8 Geometry Dash -kenttää ja todistaa ratkaisijalla, että jokainen on läpäistävissä kolikoineen
  build_geodash_solver.py    Ratkaisija pelin oikealla askelkoodilla (ratkaisut tiedostossa geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Tasodata: snake-comp.json, chess-puzzles.json (+ lähde-README), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Kokonaistarkastus (syöte, seedrand Python = JS, tallennus, kielitiedostot, esinäytöt) + kaikki audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless-tarkastukset peleittäin
  newgames_audit.py  blockjump_audit.py
```

Valittu kieli tallennetaan tiedostoon `mem.json` (osioon `mem`, saman tiedoston osion
`highscores` viereen) ja ladataan automaattisesti seuraavalla käynnistyksellä.

**Lähteet ja lisenssit:** 200 shakkitehtävää on peräisin
[Lichessin tehtävätietokannasta](https://database.lichess.org/#puzzles) (lisenssi
**CC0 1.0**, vapaa käyttö – kiitos, lichess.org!); tarkemmat tiedot ovat tiedostossa
`games/levels/chess-puzzles.README.md`. Wordlen sanalistojen lähteet kerrotaan
tiedostossa `woordlistz/README.md`.

### Alustahuomiot

Näyttö toimii **ruudun ulkopuolella**: pygame käyttää dummy-videoajuria
(`SDL_VIDEODRIVER=dummy`), joten se renderöi pinnalle, ja jokainen ruutu piirretään
kuvana Tkinter-widgetiin. **Natiivia SDL-ikkunaa ei ole**, joka voisi kilpailla
Tkinterin kanssa koosta/sijainnista. Tämän ansiosta ikkuna käyttäytyy kaikkialla
samalla tavalla ja vakaasti:

- **Windows**: prosessi tehdään lisäksi DPI-tietoiseksi, jotta näyttö pysyy terävänä
  skaalatuilla näytöillä (125/150/200 %) eikä "tärise".
- **Linux/X11 ja Wayland**: toimii ilman erikoistapauksia (ei `SDL_WINDOWID`).
- **macOS**: toimii myös (aiemmin upotettu ikkuna ei näkynyt täällä lainkaan).

---

### Asennusopas

Vaatimus: **Python 3.9+** (suositeltu 3.12 tai 3.13) ja **pygame ≥ 2.6**.

#### Windows (suositeltu: automaattinen)

1. Avaa projektikansio ja kaksoisnapsauta **`install-python.bat`**. Skripti
   - tarkistaa, onko **Python 3.13** asennettu, ja jos ei, asentaa sen
     **wingetillä** (`winget install Python.Python.3.13`),
   - luo virtuaaliympäristön **`.venv`**,
   - asentaa **pygamen** tiedostosta `requirements.txt`.
2. Käynnistä sitten kokoelma **`start.bat`**-tiedostolla (kaksoisnapsautus).

> Huom: jos skripti ilmoittaa "ei vielä käytettävissä tässä ikkunassa", Python
> asennettiin juuri – avaa vain **uusi pääte/ikkuna** ja suorita `install-python.bat`
> uudelleen. Jos **wingetiä** ei ole, asenna Python 3.13 manuaalisesti osoitteesta
> <https://www.python.org/downloads/> ja rastita **"Add python.exe to PATH"**.

#### Windows / Linux / macOS (manuaalinen)

```bash
# 1. Tarkista Python (3.9+)
python --version

# 2. Luo ja aktivoi virtuaaliympäristö
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Asenna riippuvuudet
pip install -r requirements.txt
#   tai:  pip install "pygame>=2.6" (tai pygame-ce)
#                                    pip install pygame-ce
# 4. Käynnistä
python main.py
```

#### Linux / macOS ja start.sh

```bash
# Asenna Python + venv kuten yllä (vaiheet 2 ja 3), sitten:
chmod +x start.sh      # kerran, jos ei vielä suoritettava
./start.sh
```

Linuxissa asenna Python tarvittaessa pakettienhallinnalla, esim.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); macOS:ssä esim.
`brew install python`.

#### Toisen Python-version käyttö

`install-python.bat` asentaa oletuksena Python 3.13:n. Jos suosit 3.12:ta (tai muuta
versiota), muuta tiedostossa rivi `set "PYVER=3.13"` haluttuun versioon ja
winget-tunnus vastaavasti (`Python.Python.3.12`).

#### Itsenäisen EXE:n rakentaminen (Windows)

```bat
pyinstall.bat         :: rakentaa builds\PyGameZ.exe (kaikki yhdessä tiedostossa)
```

`pyinstall.bat` käyttää `.venv`-ympäristöä (ja luo sen tarvittaessa), asentaa
automaattisesti **PyInstallerin** ja pakkaa koko pelin – Pythonin, pygamen, kaikki
pelit, kielet, wikin ja logot – **yhdeksi `PyGameZ.exe`-tiedostoksi** kansioon
**`builds\`**. Tiedosto toimii millä tahansa Windows-koneella ilman asennettua
Pythonia ja sen voi kopioida vapaasti. Asetukset ja ennätykset (`settings.json`,
`mem.json`, `mem-ngb.json`) luodaan .exe:n viereen pelatessa.

#### Vianetsintä

- **`pygame` ei löydy** → onko venv aktivoitu? Toista vaihe 3
  (`pip install -r requirements.txt`).
- **`python` ei tunnistu (Windows)** → Python asennettiin ilman "Add to PATH"; asenna
  uudelleen ruutu rastitettuna, tai käytä `py` komennon `python` sijaan.
- **Ei ääntä** → tarkista "Ääni" asetuksista; haptiikka toimii vain ohjaimella.
- **Ikkuna/upotus Linuxissa** → katso *Alustahuomiot* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ takaisin ylös / back to top</a></b></div>

---

<a name="-cestina"></a>

## 🇨🇿 Čeština

Sbírka desktopových her v Pythonu: **Tkinter** tvoří okno a menu, **Pygame** je
vložen jako herní obrazovka do okna Tkinteru. Čtyřicet šest her se sdílenými
nastaveními, volně přemapovatelným ovládáním, nejlepšími skóre, procedurálními
zvukovými efekty a u některých titulů i režimem pro více hráčů. Rozhraní je
**vícejazyčné** – **14 jazyků** (němčina / angličtina / francouzština /
španělština / portugalština / polština / turečtina / dánština / norština /
švédština / finština / čeština / slovinština / chorvatština); jazyk se volí na
**uvítací obrazovce** při prvním spuštění, kde lze zároveň nastavit
**rozlišení** a **zvuk** (ve výchozím stavu vypnutý); kromě tří hlavních jazyků
se všechny ostatní (španělština, portugalština a devět dalších, mezi nimi i
čeština) skrývají za tlačítkem **„Více"**. Vše lze později kdykoli změnit v
nastavení.

### Rychlý start

#### Windows

```bat
install-python.bat    :: jednorázově: nastaví Python 3.13 + .venv + pygame
start.bat             :: spustí sbírku her
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # spustí s .venv, jinak systémový python3
```

`start.bat` / `start.sh` automaticky použijí virtuální prostředí `.venv`, pokud
existuje, jinak systémový Python. Podrobný návod krok za krokem je úplně dole v
části **[Průvodce instalací](#průvodce-instalací)**.

### Hry

| Hra          | Režimy          | Stručný popis |
|--------------|-----------------|---------------|
| **Snake**    | 1 / 2 hráči     | Luxusní Snake s 2D a 3D pohledem, turbem, 6 herními režimy (vč. Kompetitivního), zlatými jablky a prestiží |
| **Pong**     | 1 / 2 hráči     | Klasika proti AI nebo hráči 2, přepínatelný režim pohybu |
| **Air Hockey** | 1 / 2 hráči   | 2D fyzika s přenosem impulzu, ovládání myší, AI a power-upy |
| **Tic-Tac-Toe** | 1 / 2 hráči  | Hra m,n,k na 3x3 až 9x9, tři úrovně AI **nebo** lokálně X proti O |
| **Breakout** | 1 hráč          | Rozbíječka cihel s druhy cihel, power-upy, comby a mnoha úrovněmi |
| **Tetris**   | 1 / 2 hráči     | Moderní pravidla Guideline (SRS, odložení kostky, náhled 5 kostek, T-Spiny): Maraton, Sprint 40, Ultra 2:00, Versus proti AI (3 úrovně) nebo ve dvou s odpadními řadami |
| **Invaders** | 1 hráč          | Space Invaders: vyčisti vlny, chraň své životy |
| **Asteroids** | 1 / 2 hráči    | Fyzika setrvačnosti, vlny, UFO, power-upy, hyperprostor - sólo nebo kooperativní duel |
| **Pac-Man**  | 1 hráč          | Věrný klon: 4 AI duchů, mocenské pilulky, tunel, ovoce, úrovně |
| **Flappy Bird** | 1 hráč       | Gravitační let mezi trubkami, mince, štít, den/noc, medaile |
| **Doodle Jump** | 1 hráč       | Automatický skok vzhůru, typy plošin, pružiny, vrtule, příšery |
| **2048**     | 1 hráč          | Hlavolam s posouváním čísel od 3x3 do 8x8: Klasický, Na čas a Nekonečný, vracení tahů, plynulé animace, ukládání partií |
| **Minesweeper** | 1 hráč       | Klasika s bezpečným prvním klikem, chordingem, smajlíkem a nejlepšími časy |
| **Sudoku**      | 1 hráč       | 4 varianty (Klasické, X-sudoku, Killer, Mini 6x6), každá se 400 úrovněmi, Sudoku dne, až 3 hvězdy za úroveň, 4 režimy pomoci, vracení tahů, uložená hra |
| **Frogger**     | 1 hráč       | Silnice + řeka + 5 zátok, bonusová moucha, krokodýli, časový limit, 3 obtížnosti |
| **Memory**      | 1 / 2 hráči  | Hledej dvojice na 4x4 až 8x6, animace otočení, sólo hodnocení nebo duel |
| **Solitér**     | 1 hráč       | 5 variant (Klondike, Spider, FreeCell, Pyramida, TriPeaks) s táhni a pusť a krokem zpět |
| **Aim Trainer** | 1 hráč       | Pohodová 3D střelba na terče: myš řídí kameru, 4 režimy (přesnost/reflex/pohyblivé/chill), 3 témata vč. černé díry |
| **Čtyři v řadě** | 1 / 2 hráči | Klasika s animací padání disku: 3 úrovně AI (minimax) nebo lokální duel |
| **Tankový duel** | 1 / 2 hráči | 2D duel v aréně s odrazovými střelami, power-upy, 4 arény, AI se 3 úrovněmi |
| **Blackjack**    | 1 hráč      | Kasino blackjack se shoe ze 4 balíčků, zdvojení/rozdělení a blackjackem 3:2; hraje se s lama žetony společné Lama banky |
| **Tunnel Racer** | 1 hráč      | 3D let neonovým tunelem: nekonečný režim + 30 úrovní, ovládání klávesami nebo myší, motion blur |
| **3D bludiště**  | 1 hráč      | Raycaster z první osoby (styl Wolfenstein) s 50 úrovněmi ze semínka, orby, minimapa - nebo 2D pohled shora |
| **Reversi**      | 1 / 2 hráči | Othello na 8x8: sevři a otoč kameny, 3 úrovně AI (minimax) nebo lokální duel |
| **Kniffel (Yahtzee)** | 1 / 2 hráči | Kostková klasika se 13 kategoriemi, horním bonusem a Yahtzee; hon za rekordem nebo hotseat pro 2 |
| **Wordle**       | 1 hráč      | Hádání slov o 4 až 7 písmenech: Nekonečno, Slovo dne, Dordle a Quordle, těžký režim, paleta pro barvoslepé, statistiky s grafem, sdílení výsledku, skutečné seznamy slov ve 14 jazycích |
| **T-Rex Runner** | 1 hráč      | Nekonečný běh pouští: variabilní skok, krčení, kaktusy a pterodaktylové, cyklus den/noc, rostoucí rychlost, 3 obtížnosti |
| **Dáma**         | 1 / 2 hráči | 3 sady pravidel (německá 8×8, mezinárodní 10×10, checkers), povinné braní a létající dámy, 3 úrovně AI (minimax) nebo lokální duel |
| **Poker**        | 1 hráč      | 3 volitelné varianty: Texas Hold'em proti AI, 5 Card Draw a Video Poker; sázková kola, blindy, lama žetony společné Lama banky |
| **Šachy**        | 1 / 2 hráči | Kompletní pravidla, Chess960 a šachové hodiny, 6 úrovní AI, 200 úloh z databáze Lichess, vrácení tahu/nápověda, seznam tahů, export PGN nebo lokální duel |
| **Mlýn**         | 1 / 2 hráči | Fáze pokládání/posouvání/létání, mlýny a braní, volitelné pravidlo létání, 3 úrovně AI nebo lokální duel |
| **Simon**        | 1 / 2 hráči | Pamětová hra Senso: režimy Klasický/Speed/Reverse/Smíšený + Duel, zvuk vyp/zap/smíšený, 4/6/9 polí, nejlepší skóre pro každý režim |
| **Kulečník**     | 1 / 2 hráči | 8-ball, 9-ball a trénink ve 2D, pevný 3D pohled nebo volně otočná 3D kamera; plynulá fyzika, asistence míření, 3 úrovně AI |
| **Posuvné puzzle** | 1 hráč    | Patnáctka v 3x3/4x4/5x5: posouvej číslované dlaždice do mezery, ovládání klikem nebo šipkami, body podle tahů a času |
| **Mastermind**     | 1 hráč    | Rozlušti tajný barevný kód (3 režimy: 4×6, klasický, 5×8), černé/bílé kolíky zpětné vazby, nejlepší skóre s nekonečnou sérií |
| **Bubble Shooter** | 1 hráč    | Klon Puzzle Bobble: střílej stejné barvy do trojic, odrazy od stěn, padající shluky, 3 obtížnosti |
| **Hangman**        | 1 hráč    | Uhodni slovo, než se dokreslí šibenice; klávesnice na obrazovce, seznamy slov podle jazyka, 3 délkové režimy, nekonečná série |
| **Block Jump**   | 1 hráč      | 3D plošinovka ve stylu Minecraftu: texturovaný voxelový svět se Stevem, žebříky, ploty a slizovými bloky, kamera z první/třetí osoby, motion blur, parkourové úrovně ze semínka |
| **Tower Defense** | 1 hráč      | Odrážej nekonečné vlny na 4 mapách: až 11 typů věží s vylepšeními, prodejem a specializací A/B, bossové, 3 režimy, aktivní schopnosti |
| **Minigolf**    | 1 / 2 hráči | 360 drah na 40 hřištích (18 ručních, 342 vytvořených): písek, rampy, voda, odrazníky, mlýny a bloudící bloky; karta s parem a bonusem za hole in one; **vlastní editor drah** s 15 typy objektů, 12 šablonami a sdílením jako `.lamapgzmap` |
| **Pinball**     | 1 / 2 hráči | Pinballový automat se 3 stoly: bumpery, slingshoty, terče, dráhy L-A-M-A, multiball s jackpotem, záchrana koule, šťouch a tilt |
| **Bowling**     | 1 / 2 hráči | 10 framů s oficiálním počítáním strike/spare, skutečnou fyzikou kuželek, hookem a dráhou v perspektivě, 3 obtížnosti |
| **Crossy Road** | 1 hráč        | Nekonečné skákání přes louky, silnice, řeky a koleje v izometrickém voxelovém stylu: den/noc, orel, 10 postav ke koupi, denní trasa |
| **Geometry Dash** | 1 hráč      | Rytmická plošinovka s kostkou, lodí, míčem, UFO a vlnou: 8 levelů od Lehkého po Démona, v každém 3 tajné mince, tréninkový režim, soundtrack pro každý level; **editor levelů** se sdílením jako `.lamapgzlevel` |
| **Battleship**  | 1 / 2 hráči   | Námořní bitva 10x10: rozmístění flotily přetažením, 3 přepínače pravidel (dotyk, salva, střelba znovu), AI se 3 úrovněmi nebo lokální duel s předávací obrazovkou |
| **Casino**      | 1 hráč        | Evropská ruleta se všemi klasickými sázkami a Lama automat (5 válců, 10 výherních linií, divoký symbol, volné otočky); jeden účet lama žetonů s Blackjackem a Pokerem |

**Více hráčů (2 hráči lokálně)** je k dispozici pro **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (kooperativní
duel)**, **Memory (duel)**, **Čtyři v řadě**, **Tankový duel**, **Reversi**,
**Kniffel**, **Dáma**, **Šachy**, **Mlýn**, **Simon (duel)**, **Kulečník**,
**Minigolf**, **Pinball**, **Bowling** a **Battleship** (s předávací obrazovkou,
která skryje flotily) - celkem 20 her. Režim se volí přímo na přípravné
obrazovce (*Jeden hráč / Více hráčů*); Tetris navíc nabízí **Versus proti AI**.
Webová verze je jen pro jednoho hráče.

#### Podrobnosti k jednotlivým hrám

**Snake**
- **NOVÉ - 3D pohled** (klávesa **V** v nastavení nebo klik na *Pohled*): hrací
  plocha se vykresluje jako 3D scéna v reálném čase - **sledovací kamera** se
  vznáší za hadem a řídí se **vzhledem ke směru pohledu** (vlevo/vpravo =
  zatočit, dvě rychlá stisknutí = otočka o 180°). S mlhou do dálky, hvězdnou
  oblohou, šachovnicovou podlahou, mantinely, rotujícími krystaly potravy, 3D
  částicemi a otřesem kamery při nárazu; po konci hry kamera pomalu obíhá hada.
  Turbo rozšiřuje zorné pole. V 3D dostupné: *Klasický* a *Překážky* (zdi jsou
  tam vždy pevné, 3D je jen pro jednoho hráče). Pohled se ukládá do
  `settings.json`.
- **NOVÉ - Možnosti 3D kamery** (v 3D nastavení klikni na řádek *3D kamera /
  Smooth-Shake* nebo klávesa **K**): samostatná nabídka se **Smooth-Shake**
  (jemnější kamera, výrazně méně cukání při pohybu/otáčení), nastavitelným
  **zorným polem (FOV)** a **výškou kamery** a přepínačem **otřes při zatáčení**
  (otřes obrazu při zatáčení vlevo/vpravo zap/vyp). Vše se ukládá do
  `settings.json`.
- **Turbo**: **podrž** klávesu turba = dvojnásobná rychlost, spotřebovává výdrž
  (ukazatel); po vyčerpání se turbo vypne a znovu se nabíjí. Výchozí H1 =
  Mezerník/levý Shift, H2 = Enter/pravý Shift.
- **6 herních režimů** (na výběr v nastavení): *Klasický*, *Speed-Rush* (s
  každým jablkem rychlejší), *Překážky* (smrtící bloky), *Portály* (dvojice
  teleportů), *Časovka* (60 sekund, co nejvíce jablek) a *Kompetitivní* (viz
  níže).
- **NOVÉ - Kompetitivní** (jeden hráč): nekonečný režim s **postupem v úrovních**
  - začínáš s přesně **jedním** jablkem a víc jich zpočátku mít nemůžeš; čím víc
  jablek celkem posbíráš, tím vyšší je tvá **úroveň**, která postupně přidává na
  plochu další současné jablko a zvyšuje násobitel skóre. **Modrá jablka**
  otevírají **automat**: sázkou je tvá délka, výsledek válců ji znásobí nebo
  zmenší a na chvíli nechá objevit **jablka navíc** (jackpot při třech stejných
  symbolech). **Fialová jablka** (sázka) dají v sázku část tvé **velikosti** a
  tuto část náhodně znásobí, zbytek zůstává v bezpečí (nová velikost =
  velikost·(1-p) + velikost·p·faktor): **normálně** pevných 50 % s
  **x0.5 .. x1.5**, v **HARDCORE** riskantněji se sázkou **75-90 %** a
  **x0.25 .. x2.25**. **Velikost** se zobrazuje jako **desetinné číslo vlevo
  nahoře** a přenáší se přesně, takže na ní další sázky staví. Je **15 úrovní**
  (násobitel až x16, až 16 jablek najednou); úrovně žijí v
  `games/levels/snake-comp.json` a lze je tam rozšířit bez zásahu do kódu,
  zbytek doladění je v `competitive.py`.
- **NOVÉ - HARDCORE** (přepínač v nastavení Kompetitivního režimu, klávesa **H**):
  každé **turbo ukusuje z délky** tvého hada; režim značí červeně zářící **nápis
  HARDCORE**. Jen v Kompetitivním režimu; délka nikdy neklesne pod minimum.
  Ukládá se do `settings.json`.
- **Zlatá jablka** (dočasná) dávají spoustu bodů a okamžitě doplní turbo.
- Volitelně **průchozí zdi**, bonusová jablka, **prestiž** (jeden hráč, klávesa **P**).
- **NOVÉ - Přizpůsobení** (tlačítko štětce úplně vpravo nahoře v nastavení, nebo
  klávesa **C**): čistě vizuální nabídka („mody", které *nikdy* nemění hru) se
  dvěma záložkami:
  - **Hlava**: **barva hlavy** hada - 4 modrotyrkysové předlohy (od více modré po
    více tyrkysovou), červená, oranžová a **vlastní barva** přes posuvníky RGB.
  - **Mřížka (rozcestník)**: překryje pole **souřadnicovou mřížkou** - **čísla
    řádků** (u levého a pravého okraje) a **písmena sloupců** (nahoře/dole). Na
    velkých polích tak hned vidíš, že např. jablko na *8a* leží ve stejném řádku
    *8* jako tvá vlastní pozice *8z*. Pořadí barev (5 předloh + dvě vlastní barvy
    A/B) určuje barevné téma.
  - **Banner**: **zapnout/vypnout** banner násobitele (např. z fialového jablka)
    a nastavit jeho **velikost** (menší/větší) a **průhlednost** (průhlednější) -
    s živým náhledem.
  Vše se ukládá do `mem-ngb.json`; veškeré vizuální přizpůsobení běží přes modul
  `ngb.py`.
- Vzhled: zaoblený had s očima (hlava je ve výchozím stavu tyrkysová), záře
  turba, částice.

**Pong**
- Jeden hráč proti AI, více hráčů = hráč 2 vpravo. Do 5 bodů.
- **Režim pohybu přepínatelný pro každou sadu ovládání**: *Trvalý* (stiskni
  jednou -> pohybuje se dál, výchozí) nebo *Držení* (pohybuje se jen po dobu
  držení). Přepínání: **X** = ovládání 1, **N** = ovládání 2 (ukládá se do
  `settings.json`).
- Fyzika míčku se zrychlením a úhlem podle místa dopadu.

**Air Hockey**
- **Skutečná 2D fyzika**: kulaté pálky a puk s přenosem impulzu - puk při zásahu
  přebírá rychlost pálky; mantinely s odrazivostí, mírné ledové tření, branky
  jako otvory v bočních stěnách.
- **Ovládání myší** pro jednoho hráče: pálka sleduje myš (jakákoli klávesa
  přepne zpět na klávesnici). Klávesnice: směrové klávesy v 8 směrech, více
  hráčů = H1 vlevo (WASD), H2 vpravo (IJKL).
- **AI se třemi úrovněmi** (Lehká/Střední/Těžká): brání vlastní branku, útočí na
  své polovině a objíždí puk, aby se vyhnula vlastním gólům.
- **Power-upy** (lze vypnout): *XL* (větší pálka), *TOR* (soupeřova branka se
  zmenší), *>>* (rychlejší pálka) - patří hráči, který se puku dotkl naposledy.
- Nastavení: obtížnost, **góly do vítězství** (3/5/7/10), power-upy zap/vyp
  (ukládá se do `settings.json`). Po každém gólu rozehrává ten, kdo ho dostal.
- Vzhled: světelná stopa puku, částice, pulzující ústí branek, ukazatele efektů.

**Tic-Tac-Toe**
- Nastavení: obtížnost (Lehká/Střední/Těžká) a velikost pole 3x3..9x9; vítězná
  délka K = 3 (3x3), 4 (4x4), jinak 5.
- **1 hráč** proti AI (Těžká na 3x3 je neporazitelná) **nebo 2 hráči** lokálně
  (X proti O, střídavě klikáním). Po konci hry: Enter/klik = nové kolo,
  **S** = nastavení.

**Breakout**
- Druhy cihel: Normální, **Ocel** (nezničitelná), **Bomba** (vybuchne), **Zlato**
  (body navíc).
- Power-upy: laser, ohnivá koule, lepkavá, štít, mince a další; **násobitel
  comba**.
- Efekty: částice, stopy míčku, otřes obrazu, vyskakující body, mnoho vzorů
  úrovní.
- Nastavení: **1/2/3** = obtížnost, **Vlevo/Vpravo** = barva míčku,
  **Nahoru/Dolů** = počáteční úroveň, **M** = uspořádání. Hra: myš/šipky,
  **Mezerník** vypustí míček (vystřelí laser), **P/Esc** = pauza.

**Tetris**
- **Moderní pravidla Guideline**: pole 10x20, kostky ze **sáčku po 7**, **systém
  otáčení SRS** se skutečnými wall kicky (i pro kostku I), otáčení oběma směry,
  **odložení kostky** (jednou za kostku), **náhled 5 kostek**, stín kostky a
  **lock delay** (0,5 s, nejvýše 15 resetů).
- **Tři režimy** na přípravné obrazovce: *Sólo*, *Proti AI* a *2 hráči*. Sólo
  nabízí v nastavení **Maraton** (počáteční úroveň 1–15, počítá se do rekordu),
  **Sprint 40 řad** (nejlepší čas) a **Ultra 2 minuty** (nejlepší skóre); nejlepší
  výsledky se ukládají do sekce `tetris` v `mem.json`.
- **Bodování podle Guideline**: od Single po Tetris, **T-Spiny** (plné i mini),
  **Back-to-Back** (x1,5), **komba** a **Perfect Clear** - s nápisy na obrazovce,
  animací mazání řad, stopou hard dropu, částicemi a efektem postupu na další
  úroveň.
- **Versus s odpadními řadami**: smazané řady posílají soupeři odpad (Tetris = 4,
  T-Spin Double = 4 …), příchozí odpad ohlásí varovný pruh a vlastní útoky ho
  **započítají**; obě pole dostávají stejné pořadí kostek. **AI** má 3 úrovně a
  tempo se zvyšuje každých 40 sekund.
- **Ovládání**: Vlevo/Vpravo s vlastním **DAS/ARR** (nastavitelným v nastavení),
  Nahoru = otočit doprava, Dolů = soft drop, Akce = hard drop; **C**/Shift =
  odložit, **Z**/**Y** = otočit doleva, **X** = otočit doprava. Ve dvou hráč 1
  odkládá klávesou **Q** a otáčí doleva **E**, hráč 2 **pravým Shiftem** /
  **pravým Ctrl**. Po hře: **R** = znovu, **S** = nastavení.

**Invaders** – dva režimy (na výběr na přípravné obrazovce):
- **Klasický**: klasický blok mimozemšťanů; poté v nastavení na výběr: **pohyb**
  (jen vlevo/vpravo *nebo* volně s WASD) a **míření** (vždy nahoru *nebo* na
  **myš** – pak střílíš tam, kde je kurzor). Zničení mimozemšťané někdy upustí
  power-upy.
- **Aréna (volná)**: volný pohyb všemi směry, nepřátelé se valí ze všech okrajů;
  míříš ve směru pohybu, zbraň přepínáš pomocí **1–4**.
Společné: systém úrovní s **bossem** v každé 4. úrovni, čtyři zbraně (blaster,
rozptylová střela, rychlopalba, laser), power-upy (život navíc, štít, vylepšení
zbraně), efekty explozí, nejlepší skóre.

**Asteroids**
- **Fyzika setrvačnosti**: Nahoru = tah ve směru pohledu, Vlevo/Vpravo = otáčení,
  loď dál plachtí (mírné tlumení); vše přechází přes okraje obrazovky. Klasický
  **vektorový vzhled** s plamenem trysky a hvězdnou oblohou; každý balvan má
  vlastní náhodný mnohoúhelníkový tvar.
- Balvany se tříští na dva menší (3 velikosti, **20/50/100 bodů**), **vlny** s
  rostoucím počtem a oznámením v banneru.
- **UFO** (lze vypnout): pravidelně přelétá obrazovku a míří na lodě (chyba
  míření podle obtížnosti) - 200 bodů za sestřelení.
- **Power-upy** (lze vypnout), padají ze zničených balvanů: **Š**tít (6 s
  nezranitelnost), **T** = trojitá střela, **R** = rychlopalba.
- **Hyperprostor** (klávesa Dolů): nouzový skok na náhodnou pozici se 4s
  prodlevou - a 12% rizikem, že se přitom roztříštíš.
- 3 životy, bezpečné oživení s blikáním nezranitelnosti, **život navíc za
  každých 5000 bodů**; částice explozí a otřes kamery.
- **Kooperativní duel** (více hráčů): obě lodě létají zároveň s oddělenými
  životy a body - vyhrává, kdo má víc bodů.
- Nastavení: obtížnost, UFO zap/vyp, power-upy zap/vyp (v `settings.json`).

**Pac-Man**
- **Klasické bludiště 28x31** v neonovém stylu s kuličkami, 4 mocenskými
  pilulkami, bočními tunely a domečkem duchů uprostřed.
- **Čtyři duchové s původním chováním** (AI cílového políčka): *Blinky* honí
  přímo, *Pinky* číhá v záloze (4 políčka napřed), *Inky* využívá vektor přes
  Blinkyho, *Clyde* se zblízka stahuje.
- **Fáze scatter/chase** se střídají (duchové se při každé změně otočí);
  **mocenská pilulka** promění duchy na modré a jedlé (řetěz 200/400/800/1600),
  poté se jejich oči vracejí do domečku.
- Domeček duchů s **postupným vypouštěním**, **ovocnými** bonusy (podle úrovně),
  **3 životy**, **život navíc při 10 000**, systém úrovní (zrychluje se),
  animace smrti, obrazovky READY/GAME OVER.
- Nastavení: **obtížnost** (Normální/Těžká/Extrémní) – rychlost duchů a doba
  vystrašení.
- Ovládání: **šipky nebo WASD**.  Enter = nová hra, S = nastavení.

**Flappy Bird**
- **Gravitační fyzika**: Mezerník / Nahoru / W / **klik myší** rozmává ptákovi
  křídly; naklání se podle rychlosti stoupání/klesání.
- Nekonečné **dvojice trubek** s mezerou (+1 za trubku); v mezerách se objevují
  **mince** (bonus) a power-up **štít** (přežije jednu kolizi).
- **Denní/noční témata** se mění se skóre; plující mraky (parallax), rolující
  země.
- Obtížnost (Lehká/Normální/Těžká): velikost mezery, rychlost, rozestup trubek –
  mezera se s rostoucím skóre mírně zužuje.
- **Medaile** (bronzová/stříbrná/zlatá/platinová) po konci hry, animace nárazu s
  otřesem kamery, nejlepší skóre.

**Doodle Jump**
- Doodler **skáče automaticky** při dopadu; ovládáš jen vlevo/vpravo (se
  setrvačností), okraje se obtáčejí (**wrap-around**), kamera stoupá spolu s
  tebou.
- **Typy plošin**: zelená (normální), modrá (pohyblivá), hnědá (rozbije se),
  bílá (zmizí). **Pružiny** dají superskok, **vrtulová čepice** tě na chvíli
  automaticky vynese vzhůru (a udělá nezranitelným).
- **Příšery**: dotek je smrtelný – ale můžeš je **sestřelit** klávesou Nahoru /
  Mezerník (body navíc).
- Body = dosažená výška; obtížnost roste s výškou. Nejlepší skóre.
- Ovládání: vlevo/vpravo = pohyb, Nahoru / Mezerník = střelba.

**2048**
- **Vlastní obrazovka nastavení** s deskami od **3x3 do 8x8** a třemi režimy:
  *Klasický* (cíl 2048, pak „Hrát dál?"), *Na čas* (3 minuty, hodiny se spustí
  prvním tahem) a *Nekonečný*.
- **Plynulé animace**: dlaždice kloužou, spojují se s efektem „pop" a vyrůstají;
  vyskakující body, jiskry od 128, tlaková vlna od 2048 a nové barvy až do
  131072. Vstupy během animace se uloží do fronty.
- **Vracení tahů** (vypnuto / 3 za hru / neomezeně, klávesa **U** nebo
  Backspace) - kdo ho použije, hraje bez rekordu a bez úspěchů za dlaždice.
- **Uložení a pokračování**: rozehraná partie se automaticky ukládá pro každou
  velikost a režim; nejlepší skóre a největší dlaždice podle velikosti/režimu jsou
  v sekci `g2048` souboru `mem.json`.
- Ovládání: šipky/WASD nebo **tažení** myší/touchpadem, **R**/**N** = nová hra,
  **Tab** = nastavení. Rekord se počítá jen v **4x4 Klasickém** bez vracení tahů.

**Minesweeper**
- Tři úrovně: **Začátečník** (9x9, 10 min), **Pokročilý** (16x16, 40), **Expert**
  (30x16, 99) - **nejlepší čas pro každou úroveň** se ukládá a zobrazuje v
  nastavení.
- **První klik je vždy bezpečný** (miny se rozmístí až poté, oblast 3x3 kolem
  kliknutí zůstává volná).
- **Levý klik** = odkrýt, **pravý klik** = vlajka (volitelně s cyklem otazníku),
  **F** = vlajka pod kurzorem, **R** = nová hra.
- **Chording**: klik na splněné číslo odkryje zbývající sousedy.
- Klasický HUD: počítadlo min, **klikatelný smajlík** (překvapený/sluneční
  brýle/mrtvý), časovač; špatné vlajky se na konci přeškrtnou, konfety při
  vítězství.
- Body = základní hodnota úrovně minus sekundy.

**Sudoku**
- **4 varianty**, každá se **400 úrovněmi** (4 obtížnosti x 100): *Klasické*
  (známé úrovně ze semínka - vyřešené zůstávají zaškrtnuté), *X-sudoku* (obě
  úhlopříčky obsahují každou číslici právě jednou), *Killer* (čárkované klece se
  součtem; 400 předem vygenerovaných úrovní) a *Mini 6x6*. Každý hlavolam má
  **jediné řešení** - úroveň 12 v „Těžké" je na každém PC stejný hlavolam.
- **Sudoku dne**: jeden hlavolam denně pro všechny, stejný na PC i v prohlížeči;
  obtížnost závisí na dni v týdnu (od pondělní Lehké po sobotní Expert) a
  každodenním řešením budeš budovat sérii.
- **Až 3 hvězdy za úroveň** (vyřešeno · bez chyb a nápověd · navíc pod cílovým
  časem) a **nejlepší čas** v přehledu úrovní; **rozehrané hlavolamy** se
  automaticky ukládají a příště pokračují.
- **4 herní režimy** (volba před startem) s násobitelem skóre: **Klasický**
  (x2,0 - bez pomůcek), **Poznámky** (x1,5 - + tužkové poznámky a automatičtí
  kandidáti), **Komfort** (x1,0 - + špatné číslice červeně, vyznačené konflikty a
  chybné součty klecí, správné zápisy se uzamknou), **Asistent** (x0,7 - +
  nápověda, max. 3). Se zapnutým **limitem 3 chyb** (volba v nastavení) třetí
  chyba ukončí partii.
- Ovládání: šipky/WASD = buňka, **1-9** = číslice (i numerická klávesnice),
  **0/Delete/pravý klik** = mazat, **U**/**Z** = zpět, **Y** = znovu, **N** =
  poznámky, **C** = automaticky doplnit kandidáty, **H** = nápověda, **M** =
  barevná značka, **R** = restart úrovně, **Q** = výběr úrovní. Zadávání
  „nejdřív číslice" (nastavení, **I**) a **počítadlo zbývajících číslic** pod
  každou číslicí; plně hratelné myší. Po skončení hry **A** ukáže celé
  **řešení**.
- Body = (základ varianty a obtížnosti - čas - chyby - nápovědy) x násobitel
  režimu; do rekordu se počítají všechny varianty i Sudoku dne.

**Frogger**
- 5 dopravních pruhů (auta/kamiony) a 5 říčních pruhů (klády, želvy, které se ve
  vyšších úrovních **potápějí**); nahoře 5 cílových zátok - zaplnit všechny =
  další úroveň, vše se zrychlí.
- Extra: **bonusová moucha** (+200) v prázdných zátokách, **krokodýli** obsazují
  zátoky ve vyšších úrovních, **ukazatel časového limitu** pro každou žábu,
  život navíc při 10 000.
- 3 obtížnosti (rychlost, hustota provozu, čas); body za každou novou řadu,
  zátoka = 50 + časový bonus, dokončení úrovně = +1000.

**Memory**
- Velikosti pole **4x4, 6x6, 8x6**; motivy z kombinací tvaru a barvy, kreslené
  výhradně primitivy; **animace otočení**, neshodné dvojice se automaticky otočí
  zpět.
- **Sólo**: základ - 15 za tah - 2 za sekundu (min. 100). **Duel** (lokální):
  střídavě, nalezená dvojice = hraješ znovu, vyhrává, kdo má nejvíc dvojic.

**Solitér**
- **5 variant** na přípravné obrazovce: Klondike (dobírání 1/3 jako volba),
  Spider (1/2/4 barvy), FreeCell (limit supertahů), Pyramida (dvojice na 13, 2
  rozdání) a TriPeaks (řetěz ±1 s násobitelem comba).
- **Táhni a pusť** nebo klik-klik, **pravý klik** = na základ, **U** = neomezené
  zpět, **R** = nové rozdání, Mezerník = balíček.
- Karty se vykreslují bez obrázkových souborů (`games/cards.py`); všechny
  varianty sdílejí jeden seznam nejlepších skóre se vzorci specifickými pro
  variantu.

**Aim Trainer**
- **Skutečné softwarové 3D** (jako 3D režim Snaku): pevný zaměřovač uprostřed
  obrazovky, **přímé ovládání myší 1:1 jako ve střílečce** (zachycení kurzoru:
  kurzor je uvězněn v okně, Esc jej uvolní; nastavitelná citlivost, neomezený
  yaw, pitch ±60°). Levý klik střílí přesně středem, se šlehnutím z hlavně,
  dráhou střely a částicemi zásahu.
- **4 režimy**: Přesnost (60 s, 3 koule, bonus za přesnost), Reflex (30
  jednotlivých terčů, statistika reakčního času), Pohyblivé terče (dráhy +
  násobitel comba až x4) a Chill (nekonečně, bez postihu, **E** ukončí).
- **3 témata** (v nastavení, uloží se): **Vesmír** s hvězdnou koulí, **černou
  dírou se zářivým prstencem** a planetou (výchozí), neonová aréna s mřížkou na
  podlaze a synthwave sluncem a vnitřní střelnice.
- Citlivost lze měnit i za hry pomocí **+/-**; k tomu **nastavitelný motion
  blur** (0-80 %) pro extra chill vzhled - obojí se ukládá.

**Čtyři v řadě**
- Pole 7x6 s **animací padání disku**, náhledem při najetí a pulzující vítěznou
  linií; myš, šipky nebo přímá volba **1-7**.
- **3 úrovně AI** (minimax s prořezáváním alfa-beta): Lehká záměrně přehlíží
  hrozby, Střední spolehlivě blokuje, Těžká plánuje hluboko dopředu - nebo
  **2 hráči** lokálně na stejném zařízení.
- Začínající hráč se každé kolo střídá; nejlepší skóre počítá **výhry proti AI**
  v rámci jedné relace.

**Tankový duel**
- 2D duel v aréně: **střely se jednou odrazí od stěn** (ricochet) - trefíš za
  rohem (nebo sám sebe!). Do 5 kol s odpočtem.
- **4 arény** (Otevřená, Kříž, Sloupy, Bludiště) nebo náhodné střídání;
  **power-upy**: rychlopalba, štít, trojitá střela.
- **AI se 3 úrovněmi** - těžká míří s předstihem a záměrně střílí přes mantinel -
  nebo **2 hráči** na jedné klávesnici (H1 WASD+Mezerník, H2 šipky+Enter).

**Blackjack**
- Skutečná kasino pravidla: **shoe se 4 balíčky**, krupiér stojí na 17,
  **blackjack platí 3:2**, nahlédnutí krupiéra při esu/desítce; **zdvojení** a
  **jedno rozdělení** (rozdělená esa dostanou po jedné kartě).
- **Lama žetony**: Blackjack hraje s účtem **Lama banky**, který sdílí s Pokerem
  a Casinem (start 1000, trvale uložený v `mem.json`). Sázka se strhne hned při
  rozdání; pod 10 žetonů si Enter vezme **bankovní úvěr**, který účet doplní na
  1000.
- **Rekord** = nejvyšší stav tvé **bilance v Blackjacku** (1000 plus vše, co se v
  Blackjacku vyhrálo a prohrálo) - výhry v ruletě, na automatu nebo v pokeru se
  sem nepočítají, úvěry také ne.
- Ovládání pomocí tlačítek žetonů a kláves (**H**it/**S**tand/**D**ouble/rozdělit
  **X**, **1-4** = sázka, Backspace = zrušit sázku, Enter = rozdat) s animacemi
  karet; skrytá karta krupiéra se při odkrytí teď opravdu otočí.

**Tunnel Racer**
- **3D let neonovým tunelem** (softwarový renderer jako u Aim Traineru): tyče,
  bloky a **prstencové brány k proplétání**, mince na ideální stopě.
- **Dva režimy**: Nekonečný (rychlost stoupá až ke stropu, nejlepší skóre) a
  **30 úrovní ze semínka** s cílem, časovým bonusem a zaškrtnutým postupem.
- **Ovládání klávesami** (výchozí) nebo **přímé ovládání myší** (zachycení
  kurzoru, klávesa **C**); k tomu nastavitelný **motion blur** (klávesa **B**,
  0-80 %) - vše se ukládá.

**3D bludiště**
- **Raycaster z pohledu první osoby ve stylu Wolfensteina** (DDA, mlha do dálky,
  sprity) s mouselookem + WASD, **minimapou** (klávesa **M**) a zeleně
  pulzujícím východem - nebo klasický **2D pohled shora** (klávesa **V** v
  nastavení).
- **50 úrovní ze semínka**, které stále rostou; východ leží vždy v
  nejvzdálenějším bodě od startu, **orby** cestou dávají bonusové body.
- Body: 500 za úroveň + 100 za orb + časový bonus; vyřešené úrovně se
  zaškrtávají a součet relace se stává nejlepším skóre.

**Reversi**
- **Othello na 8x8**: pokládej kameny, které sevřou soupeřovy řady, a otoč vše
  uzavřené; neplatné tahy jsou zablokované a tah bez možného pohybu se
  **automaticky přeskočí**.
- **Jeden hráč proti AI** (3 úrovně: negamax s alfa-beta, poziční vážení +
  mobilita) **nebo lokální duel**, Černý proti Bílému.
- Platná pole jsou zvýrazněná; hraj **myší** nebo výběrovým rámečkem (šipky +
  Mezerník/Enter). Každá výhra proti AI počítá jeden bod do nejlepšího skóre.

**Kniffel (Yahtzee)**
- **Kostková klasika**: 5 kostek, až 3 hody za tah, kostky lze jednotlivě
  **podržet**; poté zapiš jednu ze **13 kategorií** (s živým náhledem možných
  bodů).
- Kompletní zápisník: horní část s **bonusem za 63 (+35)**, trojice/čtveřice,
  full house, malá/velká postupka, **Yahtzee (50)** a Šance.
- **Jeden hráč jako hon za nejlepším skóre** o nejvyšší součet, nebo **hotseat
  pro 2 hráče** se dvěma zápisníky vedle sebe; hraj myší nebo klávesami
  (Mezerník, 1-5, šipky, Enter).

**Wordle**
- Uhodni skryté slovo; barevná zpětná vazba (zelená/žlutá/šedá) se správným
  **počítáním zdvojených písmen** a klávesnicí na obrazovce, která se obarvuje
  (QWERTZ pro němčinu, češtinu, slovinštinu a chorvatštinu, AZERTY pro
  francouzštinu, jinak QWERTY).
- **Čtyři režimy**: *Nekonečno* (slovo za slovem, vždy 6 pokusů; každé vyřešené
  slovo dává body, první nevyřešené hru ukončí), *Slovo dne* (jedno slovo denně
  pro každý jazyk a délku - stejné na PC i v prohlížeči - s odpočtem a sérií;
  rozehrané slovo dne se uloží), *Dordle* (2 slova najednou na 7 pokusů) a
  *Quordle* (4 slova na 9 pokusů, klávesy ukazují barvy všech mřížek).
- **Nastavení** před každou hrou: **délka slova 4 až 7**, **těžký režim**
  (nalezené nápovědy musíš dál používat) a **paleta pro barvoslepé**
  (oranžová/modrá); vedle je vidět statistika.
- **Skutečné seznamy slov ve všech 14 jazycích** (složka `woordlistz/`, jen A-Z),
  pro každou délku vlastní: jen u 5 písmen téměř **34 000 řešení** a přes
  **213 000 povolených slov**, napříč všemi čtyřmi délkami zhruba 134 000 řešení.
  Řešení jsou běžná slova bez jmen, anglických zbytků a urážlivých výrazů; každý
  pokus se ověřuje proti seznamu - ostatní se odmítne a řádek se krátce zatřese.
- **Statistika** podle jazyka, délky a režimu: hry, podíl výher, aktuální a
  nejlepší série a **rozložení pokusů jako sloupcový graf** (sekce `wordle` v
  `mem.json`). **Sdílet** (**C**) zkopíruje mřížku emoji, aniž by prozradila
  řešení.
- Do rekordu se počítá jen *Nekonečno* s 5 písmeny; ostatní délky mají vlastní
  nejlepší výsledky. Úspěchy **Jasnovidec** (nejvýš 2 pokusy), **Slovní zvyk**
  (7 slov dne za sebou) a **Čtyřnásobný génius** (vyřešené Quordle).

**Poker**
- **3 varianty** na přípravné obrazovce: **Texas Hold'em** proti 1–3 soupeřům s
  AI s tlačítkem dealera, blindy a čtyřmi sázkovými koly, **5 Card Draw** (heads-up
  proti AI, jedna výměna karet) a **Video Poker** (*Jacks or Better*, sólo proti
  výplatní tabulce).
- Akce tlačítky nebo klávesami: **F** = fold, **C** = check/call, **R** = raise,
  **A** = all-in; držení/výměna karet kliknutím nebo **1-5**, **Enter** dobírá
  nebo rozdá další ruku.
- **Lama žetony** společné **Lama banky**: na začátku ruky leží tvůj účet na stole
  jako stack a co jde do banku, strhne se hned - odchod od stolu uprostřed ruky tě
  stojí jen tvůj podíl v banku. Bez peněz (méně než big blind 20, ve Video Pokeru
  méně než 10) = bankovní úvěr do 1000.
- **Rekord** = nejvyšší stav **bilance v Pokeru** (1000 plus všechny výhry a
  prohry v pokeru); úspěch **Chipleader** počítá také jen tuto bilanci.

**Šachy**
- **Kompletní šachy**: všechny tahy figur včetně **rošády**, **braní mimochodem**
  a **proměny pěšce** (figuru lze zvolit); **šach, mat a pat** plus remízy podle
  **pravidla 50 tahů**, **trojího opakování pozice**, **nedostatku materiálu** nebo
  dohodou.
- **Tři režimy**: *partie* proti AI, *2 hráči* u jednoho počítače (deska se může
  po každém tahu otočit) a **úlohy**.
- **Silnější AI bez zasekávání** v 6 úrovních od *Začátečníka* po *Mistra*:
  iterativní prohlubování, transpoziční tabulka, klidové prohledávání, kniha
  zahájení a hodnocení s mobilitou, pěšcovou strukturou a bezpečností krále. AI
  počítá v malých dávkách na snímek - hra se nikdy nezasekne.
- **Nastavení**: volba barvy, **šachové hodiny** (bez, 1+0, 3+2, 5+0, 10+5) a
  **Chess960** (všech 960 výchozích postavení, číslo je nad seznamem tahů).
- **Boční panel** s hodinami, sebranými figurami, materiálovou bilancí a
  posuvným **seznamem tahů (SAN)**; táhni a pusť, klouzající figury, souřadnice.
  Klávesy: **U** = vrátit tah, **H** = šipka nápovědy, **O** = nabídnout remízu,
  **X** = vzdát se, **F** = otočit desku, po partii **P** = **export PGN**.
- **Úlohy**: 200 úloh v 5 stupních (mat 1/2/3 tahem, taktika I/II) z **volné
  databáze úloh Lichess (CC0)**, ověřených vlastním enginem; u matových úloh platí
  každý tah, který dá mat. Postup je v sekci `chess` souboru `mem.json`.
- Vrácení tahu a nápověda dělají z partie „asistovanou": do rekordu se počítají
  jen výhry proti AI bez pomoci (za relaci).

**Mlýn**
- **Mlýn** se všemi třemi fázemi: **pokládání** (po 9 kamenech), **posouvání** po
  liniích a **létání** při pouhých 3 kamenech (lze vypnout).
- Uzavřený **mlýn** odebere soupeři kámen (přednostně mimo mlýn); prohráváš,
  když klesneš pod 3 kameny nebo nemůžeš táhnout.
- **3 úrovně AI** (minimax s alfa-beta, hodnocení podle fáze) nebo **lokální
  duel**; s nápovědami tahů, zvýrazněním mlýnů a počítadlem kamenů.

**Simon**
- **Pamětová hra Senso**: rozsvícená sekvence se každé kolo prodlouží a musí být
  přesně zopakována.
- **Režimy**: *Klasický*, *Speed* (zrychluje se), *Reverse* (pozpátku),
  *Smíšený* (režim se každé kolo mění) a **Duel** pro dva (střídavě přidávat a
  opakovat).
- **Zvuk** *vypnutý / zapnutý / smíšený* (smíšený trénuje zrakovou I sluchovou
  paměť), **4/6/9 polí** jako obtížnost; **nejlepší skóre pro každý režim** se
  ukládá. Hraj myší nebo číselnými klávesami 1-9.

**Kulečník**
- **8-ball**, **9-ball** a bezpravidlový **tréninkový** režim, proti AI (s
  asistencí míření) nebo **dva hráči lokálně**.
- **Tři volně volitelné pohledy**: klasický **2D pohled shora**, pevná **3D
  šikmá perspektiva** se stínovanými koulemi a **volně otočná 3D kamera** (pravé
  tlačítko myši). Veškerý pohyb je založen na časových krocích a **plynule
  tlumen** (tření, dílčí kroky proti protunelování).
- **Strkání**: podrž levé tlačítko myši pro nabití síly, uvolněním udeříš;
  pomáhá zaměřovací linie a ukazatel síly. Po faulu **koule v ruce**. Pohled (V)
  se ukládá do `settings.json`; vyhrané framy se počítají do nejlepšího skóre.

**Posuvné puzzle**
- Patnáctka ve třech velikostech: **3×3** (lehká), **4×4** (klasická) a **5×5**
  (těžká); posouvej číslované dlaždice do volné mezery.
- Vždy řešitelné (zamícháno mnoha náhodnými tahy). Ovládání **klikem** na
  dlaždici v řádku/sloupci mezery (posune se celá linie) nebo **šipkami**.
- Body = základní hodnota podle velikosti minus tahy a čas; po vyřešení hned
  pokračuje nové pole.

**Mastermind**
- Rozlušti skrytý **barevný kód**; po každém tipu dostaneš **černé** kolíky
  (správná barva + pozice) a **bílé** kolíky (správná barva, špatné místo).
- **3 režimy**: Lehký (4 kolíky / 6 barev / 12 řad), Klasický (4/6/10) a Těžký
  (5 kolíků / 8 barev); barvy se mohou opakovat.
- Ovládání přes barevnou paletu (klik nebo klávesy **1–8**), OK/Enter vyhodnotí
  řadu. **Nekonečná série** jako u Wordle: každý rozluštěný kód dává body.

**Bubble Shooter**
- **Puzzle Bobble** na plástvové mřížce: miř myší, střílej bubliny vzhůru, **tři
  a více stejné barvy** skupinu odpálí.
- Bubliny, které ztratí spojení se stropem, **spadnou** (bonus); střely se
  **odrážejí od stěn**, s náhledem další bubliny.
- **3 režimy** (4/5/6 barev, některé s postupujícími řadami); konec hry u
  červené linie.

**Hangman**
- Uhodni slovo **písmeno po písmenu**; každá chyba dokreslí část oběšence, po
  **6 chybách** prohráváš.
- **Seznamy slov podle jazyka** (jen A–Z), **3 délkové režimy** (krátká /
  smíšené / dlouhá); piš nebo klikej na klávesnici na obrazovce.
- **Nekonečná série**: každé uhodnuté slovo dává body (více zbývajících životů +
  delší slovo = víc).

**Block Jump**
- **3D plošinovka ve stylu Minecraftu** (softwarové 3D jako 3D režim Snaku):
  skákej přes plovoucí **voxelový svět** z bloků až k zářícímu cíli.
- **Minecraftí vzhled**: všechny bloky mají skutečné **pixelové textury**
  (tráva, hlína, kámen, prkna, diamant, sliz, dřevo); úroveň detailů se řídí
  vzdáleností (**T** = vysoké/nízké/vypnuté).
- K tomu: postava **Steva** s animací chůze (kamera z třetí osoby), **ruka**
  v první osobě, **paprsek majáku** u cíle, rotující **zlaté ingoty** místo
  mincí, čtvercové **slunce**, **pixelové mraky** a HUD se **srdíčky**.
- Typy bloků: pevné bloky (tráva/hlína/kámen/dřevo), **žebříky** (šplhání),
  **ploty** (přeskočení), **pružinové bloky** (vymrštění) a **mince**.
- Kamera **ve výchozím stavu z první osoby jako v Minecraftu**, **V** přepíná na
  sledovací kameru; **rozhlížení myší** se zachycením kurzoru, nastavitelný
  **motion blur** (**B**) a citlivost (**+/-**).
- **Parkourové úrovně ze semínka** se ztěžují; cíl = body + časový bonus, mince
  +50, pád stojí život (start se 3). Ovládání: WASD/šipky, **Mezerník** skok.

**Tower Defense**
- **Nekonečná obrana proti vlnám** na **4 mapách** (Louka, Kaňon, Křižovatka,
  Ulička), každá s vlastní cestou; zamčené mapy odemyká tvá nejlepší vlna,
  každou **8. vlnu** dorazí **boss**.
- **3 režimy**: Klasický (7 věží, hlavní režim), Kompaktní (4 věže, 2 úrovně) a
  Maximální (**11 věží**, **specializace A/B** na nejvyšší úrovni, speciální
  nepřátelé, aktivní schopnosti **Meteor/Mrazivá nova/Zlatá horečka**).
- **11 typů věží** od šípů po laser a zlatou banku, každá s až **3 úrovněmi
  vylepšení**, prodej vrací 70%; nepřátelé s pancířem, regenerací, dělením,
  maskováním, léčivou aurou a vzdušnou trasou.
- **Ekonomika**: zlato za sestřel, bonus za vlnu + 5% úrok; body za sestřely a
  vlny. **F** = 2x rychlost, **G** = dosahy, pravé tlačítko ruší.

**Minigolf**
- **360 drah na 40 hřištích**: *Classic* a *Pro* s devíti ručně postavenými
  drahami každé, **Tour** s 38 hřišti po devíti vytvořených drahách (celkem 342)
  a stoupající obtížností, k tomu *Random* ze všeho dohromady. Hřiště 7, dráha 3
  vypadá všude stejně - nic se kvůli tomu neukládá.
- **Povrchy a překážky**: písek brzdí, rampy zrychlují, voda stojí trestný úder,
  gumové odrazníky vracejí rychlost a mlýny s bloudícími bloky vyžadují
  načasování. Fyzika běží v dílčích krocích s třením jako u kulečníku - nic
  neskáče a nic neprojde mantinelem.
- **Ovládání**: myš míří, držené levé tlačítko nabíjí sílu a puštění odpálí
  (fungují i šipky + mezerník). **R** zruší nabitý úder bez odpalu. **G**
  přepíná zaměřovací čáru, **Z** automíření, **P** zvednutí.
- **Zámek síly (držené pravé tlačítko)**: zmrazí ukazatel nabíjení přesně tam,
  kde je - zlatý, s procenty, visacím zámkem a pulzujícím kroužkem kolem míčku.
  Tak počkáš s nabitým úderem na mezeru ve mlýně. Puštění nabíjí dál; uzamčená
  síla přežije i úder a další kliknutí levým tlačítkem odpálí přesně touto
  hodnotou.
- **Karta skóre** vpravo s parem a údery na dráhu; ve dvou hraje každý stejnou
  dráhu po sobě. Body: 600 za dráhu, ±300 za úder pod/nad par, **500 navíc za
  hole in one**. Nejnižší počet úderů na hřiště je v sekci `minigolf` souboru
  `mem.json`.
- **Zvednutí lze vypnout**: ve výchozím nastavení dráha končí po osmi úderech a
  počítá se za minimum. Kdo chce hrát až do zahrání, přepne *Zvednutí* v
  nastavení na VYP (nebo stiskne **P**).
- **Automíření lze vypnout**: ve výchozím nastavení se hůl před každým úderem
  sama natočí k jamce. Kdo si chce zamířit každou dráhu sám, přepne
  *Automíření* v nastavení na VYP (nebo stiskne **Z**) - pak zůstane naposledy
  zvolený směr a na odpališti nové dráhy míří hůl neutrálně nahoru.
- **F** vynuluje rozehranou dráhu: rány na 0, míček na odpaliště - stejná dráha,
  stejné hřiště.
- **Dále místo opakování**: na konci kola vede tlačítko **Dále** na další hřiště
  (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), takže se stejná devítka drah
  neopakuje; vedle **Znovu** (stejné hřiště) a **Nastavení**. Klávesy: Enter =
  dále, R = znovu, S = nastavení.
- **Záznam kola**: na konci **P** (nebo tlačítko **Záznam**) přehraje celé kolo
  úder po úderu. Klávesa **S** ho uloží do archivu (tlačítko **Záznamy** v
  postranním panelu).
- **Stavba a sdílení vlastních drah**: záložka **MAPS** na přípravné obrazovce
  vede k tvé sbírce - **Nová** otevře editor drah. Každá dráha dostane název a
  **id** (malá písmena, bez mezer); id je zároveň navržený název souboru při
  sdílení. K sedmi klasickým překážkám přibývá **osm nových**: roura (přenese
  míček na druhý konec), led, lepivé pole, urychlovač, magnet, jednosměrná
  brána, otočný talíř a skokanský můstek. Velikost dráhy lze volně nastavit
  (60x80 až 160x240), **12 šablon** dá výchozí bod a zpět/znovu i **Test**
  patří k výbavě. **Sdílet** zapíše přesně jednu dráhu jako soubor
  `.lamapgzmap` - přes okno pro uložení nebo rovnou do složky Stažené, s tvým
  jménem jako autora. **Importovat** ji načte zpět a při obsazeném id
  automaticky přejde na `-2`. Název, id i autor procházejí vždy **filtrem slov
  přes všech 14 jazyků**. Dráha se hraje samostatně přes **Hrát**, nebo celá
  sbírka přes pátou volbu hřiště **Vlastní**.

**Pinball**
- **Tři stoly**: *Classic* (tři bumpery, jedna řada terčů), *Space* (čtyři
  bumpery v kosočtverci, dvě řady) a *Lama* (otevřené pole, šest terčů v oblouku);
  3 nebo 5 koulí na partii, ve dvou se hráči střídají po koulích.
- **Vše, co flipper potřebuje**: odpalovací dráha s ukazatelem síly (příliš
  slabě? koule se vrátí a smíš znovu), dvě pálky, slingshoty, řady terčů, čtyři
  dráhy **L-A-M-A**, past se zámkem koule, **multiball s jackpotem**, šest sekund
  **záchrany koule**, šťouch a **TILT**.
- **Násobitel až x5** za sražené řady a kompletní dráhy; bumpery 100, slingshoty
  50, terče 250 - během multiballu platí bumpery jackpot 2 500.
- Pálky jedou na přiřazených klávesách vlevo/vpravo (i levý/pravý [Shift]) nebo
  na myši. Rekord každého stolu je v sekci `pinball` souboru `mem.json`.

**Bowling**
- **Deset framů podle oficiálních pravidel** včetně striků, sparů a bonusových
  hodů v desátém framu (maximum: 300). **Karta skóre** pod záhlavím ukazuje každý
  frame se značkami X, / a průběžným součtem.
- **Hod ve čtyřech krocích**: pozice, úhel, rotace a síla. Každý posuvník se sám
  houpe a zafixuje se akční klávesou - nebo se nastaví ručně vlevo/vpravo, čímž
  se houpání zastaví.
- **Skutečná fyzika kuželek**: deset kuželek jako kruhy s hmotností, které se
  navzájem porážejí; strike vzniká z fyziky, ne ze štěstí. Dráha je vpředu
  naolejovaná, takže **hook** zabere až v poslední třetině.
- Pohled na dráhu v perspektivě se žlaby, šipkami a plochou kuželek; tři
  obtížnosti (*Snadná/Normální/Pro*) mění tempo posuvníků a rozptyl. Rekord pro
  každou obtížnost je v sekci `bowling` souboru `mem.json`.
- **Záznam partie**: na konci **P** přehraje všechny hody znovu a **S** je uloží
  do archivu (tlačítko **Záznamy**).

**Crossy Road**
- **Nekonečné skákání** přes louky (stromy a kameny blokují cestu), silnice s auty
  a náklaďáky, řeky s kmeny a lekníny a **koleje**, po kterých po výstražném
  světle a zvonku přiřítí vlak - dál čekají celá nádraží až s 5 kolejemi. Trasa
  vzniká řadu po řadě, vždy má průchozí cestu a tempo i provoz rostou.
- **Izometrický voxelový styl**: postavy, vozidla a stromy ze stínovaných kostek
  (předrenderované pro každou velikost pole), plynule sledující kamera, squash &
  stretch při skocích, šplouchání vody, animace rozmáčknutí, peří a třpytivé
  mince; od řady 50 **střídání dne a noci** se světlomety.
- **Orel**: kamera se pomalu posouvá vpřed - kdo příliš otálí nebo couvne o víc
  než tři řady, toho odnese orel (předem varuje červený okraj). Odplavání na kmeni
  mimo obraz také končí hru.
- **Mince a postavy**: sebrané mince (obří mince = 5) se ukládají a kupují nové
  postavy v záložce **Postavy**: žába, prase, tučňák, kočka, liška, lama, robot,
  duch a jednorožec (25 až 250 mincí); kuře je k dispozici od začátku.
- **Režimy**: *Nekonečno* (body = nejvzdálenější řada, počítá se do rekordu) a
  *Denní trasa* (dnes pro všechny stejná, i v prohlížeči, s vlastním denním
  rekordem). Ovládání: šipky/WASD, mezerník/Enter/klik = skok vpřed; v nastavení
  **H** = stíny, **N** = den/noc. Mince, postavy a denní rekord jsou v sekci
  `crossy` souboru `mem.json`.

**Geometry Dash**
- **Rytmická plošinovka**: postavička sama uhání doprava - ty rozhoduješ jen o
  tom, kdy skočit nebo letět. **Pět podob** - kostka, loď, míč, UFO a vlna - k
  tomu portály podoby, gravitace a rychlosti (0,5x až 3x), žluté/růžové/modré
  **odrazy a koule**, poloviční bloky, bodáky, jámy a barevné spouštěče.
- **8 vestavěných levelů** od *Lehkého* po *Démona* („Lama Inferno"), v každém **3
  tajné mince**. Každý level je prokazatelně zvládnutelný: při stavbě ho řešič
  dohrál se skutečným kódem hry - se všemi mincemi a dokonce i při posunu o
  1/240 sekundy.
- **Přesná fyzika**: výpočty s pevnou řádovou čárkou v pevném kroku 240 Hz;
  každý stisk působí přesně v kroku, ve kterém nastal - stejně při jakékoli
  snímkové frekvenci a bitově shodně v prohlížeči.
- **Tréninkový režim** (**P**) s automatickými i vlastními checkpointy (**Z**
  postaví, **X** smaže), počítadlem pokusů, ukazatelem postupu, explozemi a
  okamžitým restartem (**R**). Každý level má **vlastní soundtrack** - pozadí,
  země i koule pulzují do rytmu (hudbu vypneš klávesou **M**).
- **Hvězdy a mince**: kdo level dokončí v normálním režimu, získá jeho hvězdy, a
  každá mince má cenu další hvězdy; rekordem je **celkový počet hvězd** (nejvýš
  65). Nejlepší výsledky levelů, mince, pokusy a skoky jsou v sekci `geodash`
  souboru `mem.json`.
- **Editor levelů** na záložce **LEVELY**: plátno s mřížkou, paleta se 6 skupinami
  (bloky, nebezpečí, odrazy a koule, portály, rychlost, doplňky), otáčení,
  zpět/znovu, přehledový pruh, **test od startu nebo odsud** a nastavení levelu
  (počáteční rychlost a podoba, hudební styl, BPM, barvy). Značka **„ověřeno"**
  přibude, až když svůj level sám dohraješ. **Sdílet** zapíše soubor
  `.lamapgzlevel`, **Import** ho zase načte; levely se ukládají do `ugc.json`
  vedle vlastních minigolfových jamek.

**Battleship**
- **Námořní bitva 10x10** s letadlovou lodí (5 polí), bitevní lodí (4), křižníkem
  (3), ponorkou (3) a torpédoborcem (2) - vyhrává ten, kdo první potopí celou
  nepřátelskou flotilu.
- **Rozmístění flotily** přetažením z doku: **R** nebo pravý klik otáčí, náhled
  svítí zeleně nebo červeně, **X** rozmístí vše náhodně, **C** vyprázdní desku;
  poslední rozestavení se nabídne znovu.
- **Pravidla v nastavení** (ukládají se): *lodě se smějí dotýkat*, *salva* (tolik
  výstřelů za tah, kolik vlastních lodí je na hladině) a *po zásahu střílíš znovu*.
- **AI se 3 úrovněmi**: Lehká střílí náhodně, Střední systematicky dorazí
  zásahy, Těžká počítá **mapu pravděpodobnosti** s paritou šachovnice (v průměru
  asi 70 / 60 / 45 výstřelů na celou flotilu). Nebo **2 hráči** u jednoho počítače -
  **předávací obrazovka** před každým tahem skryje obě flotily.
- **Grafika**: radarové skenování, animované vlny, granáty v oblouku, šplouchnutí,
  exploze s kouřem a hořící pole, odhalení „POTOPENO!" a souhrn kola se
  střelami, zásahy a přesností. Rekord počítá tvé **výhry proti AI** během jedné
  relace.

**Casino**
- **Ruleta** (evropská, 37 polí): všechny klasické sázky kliknutím na číslo,
  hranu nebo roh - **plein** (35:1), cheval, transversale, carré, sixain, sloupec,
  tucet, červená/černá, sudá/lichá a manque/passe. Žetony 1/5/25/100/500, pravý
  klik žetony odebere; **Roztočit**, **Opakovat** (**R**), **Zdvojit** (**D**) a
  **Smazat**. Kulička se ve spirále skutálí do předem vylosovaného pole a nahoře je
  vidět posledních 12 čísel.
- **Lama automat**: 5 válců x 3 řady, **10 výherních linií**, **lama = divoký
  symbol**, **zlaté mince = scatter** s 10 volnými otočkami a dvojnásobnými
  výhrami, sázka na linii 1/2/5/10, **automatické otáčení** (10/25), **turbo** a
  výherní tabulka. **Výplatní poměr je 96,1 %** - vypočtený přesně z pásů válců.
- **Lama banka**: Casino, Blackjack a Poker sdílejí jeden účet **lama žetonů**
  (start 1000, sekce `casino` v `mem.json`); staré stavy žetonů se převezmou
  automaticky. Sázky se strhnou hned, každá hra si pro rekord vede vlastní bilanci
  a při bankrotu dostaneš **bankovní úvěr** do 1000.
- Konfety, déšť mincí, bannery big/mega/jackpot a animace výherních linií;
  úspěchy **Terno** (výherní plein v ruletě) a **Lama jackpot** (5 lam na jedné
  linii).

Nejlepší skóre se ukládají do sekce `highscores` souboru `mem.json` (vedle
kódu) – spolu s jazykem (sekce `mem`).

### Rozhraní

Celé rozhraní je nakreslené od základu (čistý Tkinter + Pygame, žádné balíčky
navíc) a vyladěné do stylu moderního herního spouštěče:

- **Postranní panel se seznamem her**: každý řádek má vlastní **mini-piktogram**
  v akcentové barvě hry, ukazuje aktuální **nejlepší skóre (★)** a reaguje jemně
  animovanými efekty při najetí. Běžící hra zůstává barevně označená; v malých
  oknech se seznam **posouvá** kolečkem myši.
- **Stavová karta** vlevo dole se **stavovou LED** (šedá = menu, zelená = běží,
  zlatá = pauza, červená = game over) a **živým ukazatelem FPS**.
- **Úvodní obrazovka** s polárními světly, parallaxovým hvězdným polem včetně
  padajících hvězd, plovoucím logem s obíhajícími jiskrami, **klikatelnou
  mřížkou her** hned pod logem (všechny hry s efektem při najetí v jejich
  akcentové barvě) a **běžícím pásem nejlepších skóre**.
- **Efekty všude**: jemné přechody obrazovek, jiskry při potvrzení v menu,
  **déšť konfet při novém rekordu** a skutečné **rozostření** za překrytím
  pauzy.
- **Přípravná obrazovka** každé hry se zobrazuje v její akcentové barvě a ukazuje
  dosavadní rekord jako čip. Při mnoha režimech a malém rozlišení se stane
  **kompaktní**: Možnosti, Wiki a Zpět se přesunou do jedné řady a písmo se
  přizpůsobí - nic už nepřetéká z obrazu.
- **Jednotný vzhled ve hře**: všech 46 her sdílí paletu témat a písmo menu -
  HUDy, obrazovky nastavení a překrytí následují design zvolený v nastavení
  (v4.1 / v4 / Klasický), zatímco každé hrací pole si ponechává své identitní
  barvy. Každá hra nyní čistě zvládne změnu rozlišení uprostřed partie a názvy
  her v menu se přizpůsobují jazyku (např. „Schach" → „Šachy").
- **Vestavěná wiki** („LamaWiki"): podrobná nápověda ke každé hře (ovládání,
  režimy, body, tipy) plus obecné stránky - s **vyhledávacím polem**,
  kategoriemi, rolovatelnými články a čipy kláves, ve všech 14 jazycích.
  Dostupná přes tlačítko **„Wiki / Nápověda"** v postranním panelu a z přípravné
  obrazovky každé hry (otevře přímo její stránku).
- **Úspěchy a statistiky**: **107 úspěchů** ve třech kategoriích (23 cílů
  napříč sbírkou, 37 bodových milníků a 47 výjimečných okamžiků jako šachmat AI,
  dlaždice 4096, T-Spin Double, 25 vyřešených šachových úloh, Killer Sudoku nebo
  lamí jackpot; ve 2048 a v šachu se partie s vrácením tahu či nápovědou
  nepočítají) se **zlatým oznámením a fanfárou** při odemčení - i
  uprostřed hry; staré rekordy se započítají automaticky. K tomu záložka
  **statistik**: celkový herní čas, partie, výhry, rekordy, oblíbená hra a
  tabulka her seřazená podle času. Dostupné tlačítkem **„Úspěchy a
  statistiky"** v postranním panelu.
- **Záznamy**: minigolf a bowling nahrávají každé kolo. Na konci **P** ukáže
  záznam a **S** ho uloží do archivu - dostupného tlačítkem **Záznamy** v
  postranním panelu (karta na hru, pauza, skoky mezi sekvencemi, rychlost 0,5x
  až 4x). Lze vypnout při prvním spuštění i v možnostech.

### Ovládání

- Hru vyber tlačítkem v menu vlevo. Poté se objeví **přípravná obrazovka**: zvol
  **Jeden hráč** nebo **Více hráčů**, přejdi do **nastavení** nebo zpět.
  Šipky/myš pro výběr, Enter spustí.
- **ESC** = pauza / pokračovat (v menu: zpět).
- **F11** (nebo tlačítko „Celá obrazovka zap/vyp") = přepnutí celé obrazovky.
  Zobrazení Pygame zůstává vložené a zvětšuje se se zachováním poměru stran
  (černé pruhy při odlišném poměru). Okno lze libovolně měnit.
- **„Zpět do menu"** ukončí hru a uloží nejlepší skóre - stejně jako přepnutí do
  jiné hry přes postranní panel.
- **Pevné přídavné klávesy**: kromě pěti přiřaditelných akcí mají některé hry
  vlastní klávesy (např. odložení **C** a otočení doleva **Z** v Tetrisu,
  vrácení tahu **U** ve 2048, šachu a Sudoku). Fungují jen tehdy, když klávesa
  není v možnostech přiřazena žádné akci, a najdeš je v nápovědě nastavení i ve
  wiki. Držené klávesy se správně rozpoznají a při pauze nebo Alt-Tab se uvolní -
  nic se už „nezasekne".
- **„Ukončit"** čistě zavře Pygame a Tkinter.

### Nastavení, ovládání a zvuk

Obrazovka nastavení se otevírá tlačítkem **„Nastavení / Ovládání"** (vlevo) nebo
z přípravné obrazovky. Je rozdělena do **tří záložek** (**Obecné / Ovládání /
Vzhled**; přepínání klikem nebo klávesou Tab):

- **Obecné**: **zvuk** zap/vyp, **hlasitost** a **haptika** (vibrace gamepadu,
  funguje jen s připojeným ovladačem) a dále **automatické rozlišení**,
  **rozlišení**, **FPS** a **jazyk** – vše přepínatelné pomocí Vlevo/Vpravo.
- **Ovládání**: **předlohy** (*WASD + šipky*, *WASD + IJKL*, *šipky + WASD*) a
  **libovolné přemapování každé klávesy** pro hráče 1 a hráče 2: zvol řádek,
  stiskni Enter, stiskni požadovanou klávesu (Esc zruší).
- **Vzhled**: zvol **design rozhraní** – **UI v4.2** (výchozí: Midnight Glass
  – hluboký půlnoční přechod s pomalu plujícími měkkými světly v indigové,
  tyrkysové a purpurové, jemné filmové zrno, řídké hvězdy a panely jako matné
  sklo se světelným okrajem), **UI v4.1** (jako UI v4, ale živější – jemné
  hvězdy plus Saturn a černá díra v pozadí úvodní obrazovky), **UI v4.1.1**
  (jako v4.1, ale místo hvězdné oblohy dlaždicový **cikcak vzor** v černé a
  antracitové), **UI v4.1.2** (stejný vzor v modrých odstínech palety –
  akcentní modrá jako dominantní barva, tmavší modrá jako podklad),
  **UI v4.1.3** (stejný vzor v indigu z UI v4 na černé), **UI v4.1.4** (v
  grafitovém tónu UI v4 na černé), **UI v4** (naprosto klidný, plochý
  grafitový vzhled s jediným indigovým akcentem), **UI v3** (dřívější klasické
  rozhraní s hvězdnou oblohou, polárními světly a efekty záře), **UI v2**
  (úplně první přepracování rozhraní: námořnicky modrý přechod, hvězdná obloha
  a zářící tlačítka, zcela bez animací) nebo **UI v1** (vzhled před
  přepracováním UI: jednobarevné tmavé pozadí, plochá tlačítka, žádné efekty).
  Všechny karty ukazují malý náhled; volba se okamžitě projeví na celém
  rozhraní (herní plocha **i** postranní panel) a uloží se.

Nastavení se trvale ukládá do `settings.json`. V režimu **jeden hráč** obě
přiřazení ovládají stejnou postavu (výchozí: WASD *a* šipky), ve **více hráčích**
každé jednu. Všechny hry mají **zvukové efekty** (generované procedurálně, bez
nutnosti dalších souborů), které lze globálně ztlumit.

### Struktura projektu

```
install-python.bat  Instalace pro Windows: Python 3.13 + .venv + pygame
start.bat            Spouštěcí skript (Windows)
start.sh             Spouštěcí skript (Linux / macOS / Git Bash)
pyinstall.bat        Sestavení EXE (Windows): zabalí vše do builds\PyGameZ.exe
main.py              Rozhraní Tkinter, vložení Pygame, centrální herní smyčka
game_base.py         Základní třída her (update/draw/handle_event) + InputEvent + pomocníci
settings.py          Načítání/ukládání nastavení (zvuk/haptika/klávesy/herní volby s kontrolními pravidly) (JSON)
audio.py             Procedurální zvukové efekty, hudební smyčky + vibrace gamepadu
menu.py              Obrazovky jazyka, přípravy (režim) a nastavení (zvuk/ovládání)
highscore.py         Načtení/uložení nejlepších skóre (sekce v mem.json)
store.py             Centrální soubor uložení mem.json (sekce: mem, highscores, stats, achievements + postup her), atomicky se zálohou .bak
stats.py             Statistiky hráče (partie, herní čas, výhry, rekordy) na hru
achievements.py      Úspěchy: definice, odemykání, oznámení (toast)
progress.py          Obrazovka úspěchů a statistik (dvě záložky, posuvná)
replay.py            Nahrávání a archiv záznamů (replay.json)
replayview.py        Obrazovka záznamů: seznam archivu a přehrávání
ugc.py               Vlastní obsah (minigolfové jamky, úrovně Geometry Dash): úložiště, kontrola, export/import (ugc.json)
swear.py             Filtr slov pro názvy a id (lang/swear/*.yml, všech 14 jazyků)
filepick.py          Dialogy souborů ("Exportovat jako ...", "Importovat")
prestige.py          Systém prestiže pro Snake
competitive.py       Parametry Kompetitivního režimu Snaku (úrovně, automat, sázková jablka)
ngb.py               Vizuální přizpůsobení („mody"): barva hlavy + souřadnicová mřížka + menu (mem-ngb.json)
lamabank.py          Lama banka: společný účet žetonů pro Blackjack, Poker a Casino (sekce casino v mem.json)
seedrand.py          Generátor náhodných čísel s bitově shodnými čísly v Pythonu i v prohlížeči (denní režimy, nové hlavolamy)
i18n.py              Překladový engine (načítá lang/*.json, t("klíč"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Jazykové řetězce (jeden zástupný klíč na text)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Seznamy filtru slov pro každý jazyk (regex, .yml)
lamawiki/
  lamawiki.py          Vestavěná wiki (vyhledávání, kategorie, renderer článků)
  de.json  en.json  fr.json  es.json  pt.json   Obsah wiki (jedna stránka na hru + obecné stránky)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Znovu sestaví seznamy slov pro Wordle (slovníky + frekvenční seznamy)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 písmen), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 písmen), 14 jazyků
devtools/            Vývojářské nástroje (nebalí se do .exe)
  merge_staging.py           Nahraje překlady a wiki stránky z devtools/staging/ do všech 14 jazykových souborů
  build_chess_puzzles.py     Sestaví 200 šachových úloh z databáze úloh Lichess (CC0)
  build_sudoku_killer.py     Vygeneruje 400 Killer Sudoku s jediným řešením
  build_crossyroad_models.py Zapíše voxelové modely Crossy Road pro webovou verzi
  build_geodash_levels.py    Sestaví 8 levelů Geometry Dash a řešičem dokáže, že každý je zvládnutelný i s mincemi
  build_geodash_solver.py    Řešič se skutečným kódem kroku (řešení v geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Data úrovní: snake-comp.json, chess-puzzles.json (+ README se zdroji), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Celkový audit (vstup, seedrand Python = JS, ukládání, jazykové soubory, přípravné obrazovky) + všechny audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless audity pro jednotlivé hry
  newgames_audit.py  blockjump_audit.py
```

Zvolený jazyk se ukládá do `mem.json` (v sekci `mem`, vedle sekce `highscores`
téhož souboru) a při dalším spuštění se automaticky načte.

**Zdroje a licence:** 200 šachových úloh pochází z
[databáze úloh Lichess](https://database.lichess.org/#puzzles) (licence
**CC0 1.0**, volné dílo - díky, lichess.org!); podrobnosti jsou v
`games/levels/chess-puzzles.README.md`. Zdroje seznamů slov pro Wordle uvádí
`woordlistz/README.md`.

### Poznámky k platformám

Zobrazení běží **mimo obrazovku** (off-screen): pygame používá dummy video
ovladač (`SDL_VIDEODRIVER=dummy`), takže vykresluje do surface a každý snímek se
kreslí jako obrázek do widgetu Tkinteru. **Neexistuje žádné nativní okno SDL**,
které by s Tkinterem bojovalo o velikost/pozici. Díky tomu se okno chová všude
stejně a stabilně:

- **Windows**: proces se navíc nastaví jako DPI-aware, aby zobrazení zůstalo
  ostré na škálovaných displejích (125/150/200 %) a „nekmitalo".
- **Linux/X11 a Wayland**: funguje bez zvláštních případů (bez `SDL_WINDOWID`).
- **macOS**: funguje také (dříve se zde vložené okno vůbec nezobrazovalo).

---

### Průvodce instalací

Požadavek: **Python 3.9+** (doporučeno 3.12 nebo 3.13) a **pygame ≥ 2.6**.

#### Windows (doporučeno: automaticky)

1. Otevři složku projektu a dvakrát klikni na **`install-python.bat`**.
   Skript
   - zkontroluje, zda je přítomen **Python 3.13**, a jinak jej nainstaluje přes
     **winget** (`winget install Python.Python.3.13`),
   - vytvoří virtuální prostředí **`.venv`**,
   - nainstaluje **pygame** z `requirements.txt`.
2. Poté spusť sbírku pomocí **`start.bat`** (dvojklik).

> Poznámka: pokud skript hlásí „v tomto okně ještě není k dispozici", Python byl
> právě nainstalován – jednoduše otevři **nové okno/terminál** a spusť
> `install-python.bat` znovu. Pokud **winget** není k dispozici, nainstaluj
> Python 3.13 ručně z <https://www.python.org/downloads/> a zaškrtni
> **„Add python.exe to PATH"**.

#### Windows / Linux / macOS (ručně)

```bash
# 1. Zkontroluj Python (3.9+)
python --version

# 2. Vytvoř a aktivuj virtuální prostředí
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Nainstaluj závislosti
pip install -r requirements.txt
#   nebo:  pip install "pygame>=2.6" (nebo pygame-ce)
#                                     pip install pygame-ce
# 4. Spusť
python main.py
```

#### Linux / macOS pomocí start.sh

```bash
# Nastav Python + venv jako výše (kroky 2 a 3), poté:
chmod +x start.sh      # jednorázově, pokud ještě není spustitelný
./start.sh
```

Na Linuxu v případě potřeby nainstaluj Python přes správce balíčků, např.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); na macOS
např. `brew install python`.

#### Použití jiné verze Pythonu

`install-python.bat` ve výchozím stavu nastaví Python 3.13. Kdo preferuje 3.12
(nebo jinou verzi), změní v souboru řádek `set "PYVER=3.13"` na požadovanou
verzi a odpovídajícím způsobem i winget ID (`Python.Python.3.12`).

#### Sestavení samostatného EXE (Windows)

```bat
pyinstall.bat         :: sestaví builds\PyGameZ.exe (vše v jednom souboru)
```

`pyinstall.bat` používá `.venv` (a v případě potřeby ji vytvoří), automaticky
doinstaluje **PyInstaller** a zabalí kompletní hru - Python, pygame, všechny
hry, jazyky, wiki a loga - do **jediného souboru `PyGameZ.exe`** ve složce
**`builds\`**. Soubor běží na jakémkoli PC s Windows bez nainstalovaného Pythonu
a lze jej volně kopírovat. Nastavení a nejlepší skóre (`settings.json`,
`mem.json`, `mem-ngb.json`) si .exe vytvoří vedle sebe během hraní.

#### Řešení problémů

- **`pygame` nenalezen** → je aktivované venv? Zopakuj krok 3
  (`pip install -r requirements.txt`).
- **`python` není rozpoznán (Windows)** → Python byl nainstalován bez „Add to
  PATH"; přeinstaluj a zaškrtni políčko, nebo použij `py` místo `python`.
- **Žádný zvuk** → zkontroluj „Zvuk" v nastavení; haptika funguje jen s
  ovladačem.
- **Okno/vložení na Linuxu** → viz *Poznámky k platformám* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ zpět nahoru / back to top</a></b></div>

---

<a name="-slovenscina"></a>

## 🇸🇮 Slovenščina

Zbirka namiznih iger v Pythonu: **Tkinter** poskrbi za okno in meni, **Pygame**
je kot prikaz igre vgrajen v okno Tkinter. Šestinštirideset iger s skupnimi
možnostmi, prosto nastavljivim krmiljenjem, rekordi, proceduralnimi zvočnimi
učinki in, pri nekaterih naslovih, večigralskim načinom. Vmesnik je
**večjezičen** – **14 jezikov** (nemščina / angleščina / francoščina /
španščina / portugalščina / poljščina / turščina / danščina / norveščina /
švedščina / finščina / češčina / slovenščina / hrvaščina); jezik se izbere na
**pozdravnem zaslonu** ob prvem zagonu, kjer je mogoče nastaviti tudi
**ločljivost** in **zvok** (privzeto izklopljen); poleg glavnih jezikov se vsi
dodatni jeziki (med njimi slovenščina) ob prvem zagonu skrivajo za gumbom
**»Več jezikov«**. Vse je mogoče pozneje kadar koli spremeniti v možnostih.

### Hitri začetek

#### Windows

```bat
install-python.bat    :: enkratno: nastavi Python 3.13 + .venv + pygame
start.bat             :: zaženi zbirko iger
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # zažene z .venv, sicer sistemski python3
```

`start.bat` / `start.sh` samodejno uporabita navidezno okolje `.venv`, če
obstaja, sicer sistemski Python. Na dnu dokumenta je podroben vodnik po korakih:
**[Navodila za namestitev](#navodila-za-namestitev)**.

### Igre

| Igra         | Načini          | Kratek opis |
|--------------|-----------------|-------------------|
| **Snake**    | 1 / 2 igralca   | Deluxe Snake z 2D- in 3D-pogledom, pospeškom, 6 igralnimi načini (vkl. Tekmovalni), zlatimi jabolki in prestižem |
| **Pong**     | 1 / 2 igralca   | Klasika proti UI ali igralcu 2, preklopljiv način gibanja |
| **Air Hockey** | 1 / 2 igralca | 2D-fizika s prenosom gibalne količine, upravljanje z miško, UI in power-ups |
| **Tic-Tac-Toe** | 1 / 2 igralca | Igra m,n,k na 3x3 do 9x9, tri jakosti UI **ali** lokalno X proti O |
| **Breakout** | 1 igralec       | Razbijanje opek z vrstami opek, power-ups, kombinacijami in številnimi ravnmi |
| **Tetris**   | 1 / 2 igralca   | Sodobna pravila Guideline (SRS, shramba, predogled 5 kock, T-Spini): Maraton, Šprint 40, Ultra 2:00, Versus proti UI (3 jakosti) ali v dvoje z vrsticami smeti |
| **Invaders** | 1 igralec       | Space Invaders: očisti vale, zaščiti svoja življenja |
| **Asteroids** | 1 / 2 igralca  | Fizika vztrajnosti, vali, NLP-ji, power-ups, hiperprostor - sam ali kooperativni dvoboj |
| **Pac-Man**  | 1 igralec       | Zvest klon: 4 UI-ji duhov, močne tablete, tunel, sadje, ravni |
| **Flappy Bird** | 1 igralec    | Gravitacijski let skozi cevi, kovanci, ščit, dan/noč, medalje |
| **Doodle Jump** | 1 igralec    | Samodejni skok navzgor, vrste ploščadi, vzmeti, propeler, pošasti |
| **2048**     | 1 igralec       | Sestavljanka z drsenjem števil od 3x3 do 8x8: Klasično, Na čas in Neskončno, razveljavitev, tekoče animacije, shranjene partije |
| **Minesweeper** | 1 igralec    | Klasika z varnim prvim klikom, chording, smeškom in najboljšimi časi |
| **Sudoku**      | 1 igralec    | 4 različice (Klasični, X-sudoku, Killer, Mini 6x6) s po 400 ravnmi, Sudoku dneva, do 3 zvezdice na raven, 4 načini pomoči, razveljavitev, shranjena igra |
| **Frogger**     | 1 igralec    | Cesta + reka + 5 zalivov, bonus muha, krokodili, časovna omejitev, 3 težavnosti |
| **Memory**      | 1 / 2 igralca | Poišči pare na 4x4 do 8x6, animacija obračanja, samostojno točkovanje ali dvoboj |
| **Pasjansa**    | 1 igralec    | 5 različic (Klondike, Spider, FreeCell, Piramida, TriPeaks) s povleci in spusti ter razveljavitvijo |
| **Aim Trainer** | 1 igralec    | Sproščeno 3D-streljanje v tarče: miška vodi kamero, 4 načini (natančnost/refleks/premikajoče/chill), 3 teme vkl. črno luknjo |
| **Štiri v vrsto** | 1 / 2 igralca | Klasika z animacijo padanja: 3 jakosti UI (minimax) ali lokalni dvoboj |
| **Tankovski dvoboj** | 1 / 2 igralca | 2D-dvoboj v areni z odbojnimi streli, power-ups, 4 arene, UI s 3 jakostmi |
| **Blackjack**    | 1 igralec    | Kazinojski blackjack s 4-paketnim čeveljcem, podvojitev/delitev in blackjack 3:2; igra se z lama žetoni skupne Lama banke |
| **Tunnel Racer** | 1 igralec    | 3D-let skozi neonsko cev: neskončni način + 30 ravni, upravljanje s tipkami ali miško, motion blur |
| **3D-labirint**  | 1 igralec    | Raycaster v prvi osebi (slog Wolfenstein) s 50 ravnmi s semenom, orbi, minimapo - ali 2D-pogled od zgoraj |
| **Reversi**      | 1 / 2 igralca | Othello na 8x8: ujemi in obrni ploščke, 3 jakosti UI (minimax) ali lokalni dvoboj |
| **Kniffel (Yahtzee)** | 1 / 2 igralca | Klasika s kockami s 13 kategorijami, zgornjim bonusom in Yahtzeejem; lov na rekord ali dvoigralski hotseat |
| **Wordle**       | 1 igralec    | Ugibanje besed s 4 do 7 črkami: Neskončno, Beseda dneva, Dordle in Quordle, težki način, paleta za barvno slepe, statistika z grafom, deljenje rezultata, pravi seznami besed v 14 jezikih |
| **T-Rex Runner** | 1 igralec    | Neskončni tek po puščavi: spremenljiv skok, priklek, kaktusi in pterodaktili, cikel dan/noč, naraščajoča hitrost, 3 težavnosti |
| **Dama**         | 1 / 2 igralca | 3 pravila (nemška 8×8, mednarodna 10×10, checkers), obvezno jemanje in leteča dama, 3 jakosti UI (minimax) ali lokalni dvoboj |
| **Poker**        | 1 igralec    | 3 izbirne različice: Texas Hold'em proti UI, 5 Card Draw in Video Poker; krogi stav, blindi, lama žetoni skupne Lama banke |
| **Šah**          | 1 / 2 igralca | Vsa pravila, Chess960 in šahovska ura, 6 jakosti UI, 200 ugank iz baze Lichess, razveljavitev/namig, seznam potez, izvoz PGN ali lokalni dvoboj |
| **Mlin**         | 1 / 2 igralca | Faze postavljanja/premikanja/letenja, mlini in jemanja, izbirno pravilo letenja, 3 jakosti UI ali lokalni dvoboj |
| **Simon**        | 1 / 2 igralca | Spominska igra Senso: načini Klasični/Speed/Reverse/Mešani + Dvoboj, zvok izkl./vkl./mešan, 4/6/9 polj, najboljši po načinu |
| **Biljard**      | 1 / 2 igralca | 8-ball, 9-ball in vadba v 2D, fiksni 3D-pogled ali prosto vrtljiva 3D-kamera; gladka fizika, pomoč pri merjenju, 3 jakosti UI |
| **Drsna sestavljanka** | 1 igralec | Igra 15 v 3x3/4x4/5x5: drsi oštevilčene ploščice v vrzel, upravljanje s klikom ali puščicami, točke po potezah in času |
| **Mastermind**     | 1 igralec  | Razbij skrivno barvno kodo (3 načini: 4×6, klasični, 5×8), črni/beli povratni zatiči, neskončni niz kot rekord |
| **Bubble Shooter** | 1 igralec  | Klon Puzzle Bobble: streljaj enake barve v skupine po tri, odboji od sten, padajoče gruče, 3 težavnosti |
| **Vislice**        | 1 igralec  | Ugani besedo, preden so vislice končane; zaslonska tipkovnica, seznami besed po jeziku, 3 dolžinski načini, neskončni niz |
| **Block Jump**  | 1 igralec       | 3D-ploščadnica v slogu Minecrafta: teksturiran voksel svet s Stevom, lestvami, ograjami in sluzastimi bloki, kamera v prvi/tretji osebi, motion blur, ravni parkourja s semenom |
| **Tower Defense** | 1 igralec      | Odbijaj neskončne valove na 4 zemljevidih: do 11 vrst stolpov z nadgradnjami, prodajo in specializacijo A/B, bossi, 3 načini, aktivne sposobnosti |
| **Minigolf**    | 1 / 2 igralca | 360 stez na 40 igriščih (18 ročnih, 342 ustvarjenih): pesek, klančine, voda, odbijači, mlini in tavajoči bloki; kartica s parom in bonusom za hole in one; **lasten urejevalnik stez** s 15 vrstami predmetov, 12 predlogami in deljenjem kot `.lamapgzmap` |
| **Pinball**     | 1 / 2 igralca | Pinball avtomat s 3 mizami: odbijači, slingshoti, tarče, steze L-A-M-A, multiball z jackpotom, rešitev krogle, sunek in tilt |
| **Bowling**     | 1 / 2 igralca | 10 framov z uradnim štetjem strike/spare, pravo fiziko kegljev, hookom in stezo v perspektivi, 3 težavnosti |
| **Crossy Road** | 1 igralec     | Neskončno skakanje čez travnike, ceste, reke in tire v izometričnem voksel slogu: dan/noč, orel, 10 likov za nakup, dnevna proga |
| **Geometry Dash** | 1 igralec   | Ritmična ploščadna igra s kocko, ladjo, žogo, NLP in valom: 8 stopenj od Lahke do Demona s po 3 skrivnimi kovanci, način vaje, glasba za vsako stopnjo; **urejevalnik stopenj** z deljenjem kot `.lamapgzlevel` |
| **Battleship**  | 1 / 2 igralca | Pomorska bitka 10x10: postavitev flote z vlečenjem, 3 stikala pravil (dotik, salva, ponovni strel), UI s 3 jakostmi ali lokalni dvoboj z zaslonom za predajo |
| **Casino**      | 1 igralec     | Evropska ruleta z vsemi klasičnimi stavami in Lama avtomat (5 kolutov, 10 dobitnih linij, divji simbol, brezplačni vrtljaji); en račun lama žetonov z Blackjackom in Pokrom |

**Večigralski način (2 igralca lokalno)** je na voljo za igre **Snake**,
**Pong**, **Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**,
**Asteroids (kooperativni dvoboj)**, **Memory (dvoboj)**, **Štiri v vrsto**,
**Tankovski dvoboj**, **Reversi**, **Kniffel**, **Dama**, **Šah**, **Mlin**,
**Simon (dvoboj)**, **Biljard**, **Minigolf**, **Pinball**, **Bowling** in
**Battleship** (z zaslonom za predajo, ki skrije flote) - skupaj 20 iger. Način
se izbere kar na predigralnem zaslonu (*En igralec / Več igralcev*); Tetris
ponuja še **Versus proti UI**. Spletna različica je samo za enega igralca.

#### Podrobnosti po igrah

**Snake**
- **NOVO - 3D-pogled** (tipka **V** v setupu ali klik na *Pogled*): plošča se
  izriše kot 3D-scena v realnem času - **kamera zasledovalka** lebdi za kačo,
  krmiljenje pa je **glede na pogled** (levo/desno = obrat, dva hitra pritiska =
  obrat nazaj). Z meglo razdalje, zvezdnim nebom, šahovskim tlom, robnimi
  zidovi, vrtečimi se kristali hrane, 3D-delci in tresenjem kamere ob trku; po
  game overju kamera počasi kroži okoli kače. Pospešek razširi vidno polje. Na
  voljo v 3D: *Klasični* in *Ovire* (zidovi so tam vedno trdni, 3D je samo za
  enega igralca). Pogled se shrani v `settings.json`.
- **NOVO - Možnosti 3D-kamere** (v 3D-setupu klikni vrstico *3D-kamera / smooth
  shake* ali tipko **K**): namenski meni s **smooth shake** (nežnejša kamera,
  veliko manj tresenja pri gibanju/obračanju), nastavljivim **vidnim poljem
  (FOV)** in **višino kamere** ter stikalom **tresenje pri obračanju** (tresenje
  zaslona ob obratih levo/desno vkl./izkl.). Vse shranjeno v `settings.json`.
- **Pospešek**: tipko za pospešek **drži** = turbo (dvojna hitrost), troši
  vzdržljivost (vrstica); ko je prazna, se pospešek izklopi in znova napolni.
  Privzeto I1 = preslednica/leva shift, I2 = enter/desna shift.
- **6 igralnih načinov** (izbirni v setupu): *Klasični*, *Speed Rush* (hitrejši
  z vsakim jabolkom), *Ovire* (smrtonosni bloki), *Portali* (pari teleporterjev),
  *Time Attack* (60 sekund, čim več jabolk) in *Tekmovalni* (glej spodaj).
- **NOVO - Tekmovalni** (en igralec): neskončni način z **vzpenjanjem po ravneh**
  - začneš z natanko **enim** jabolkom in sprva ne moreš dobiti več; več ko jih
  skupno zbereš, višja je tvoja **raven**, ki na polje sproti dodaja še eno
  hkratno jabolko in zvišuje množitelj točk. **Modra jabolka** odprejo **igralni
  avtomat**: vložek je tvoja dolžina, izid koluta jo pomnoži ali skrči in za
  kratek čas sproži pojav **dodatnih jabolk** (jackpot ob treh enakih simbolih).
  **Vijolična jabolka** (igranje na srečo) na kocko postavijo delež tvoje
  **velikosti** in ta del naključno pomnožijo, preostanek ostane varen (nova
  velikost = velikost·(1-p) + velikost·p·faktor): **normalno** stavi fiksnih
  50 % z **x0.5 .. x1.5**, **HARDCORE** je bolj tvegan z vložkom **75-90 %** in
  **x0.25 .. x2.25**. Tvoja **velikost** je prikazana kot **decimalka zgoraj
  levo** in se natančno prenaša naprej, tako da nadaljnje stave gradijo nanjo.
  Ravni je **15** (množitelj do x16, do 16 jabolk hkrati); ravni živijo v
  `games/levels/snake-comp.json` in jih je tam mogoče razširiti brez poseganja v
  kodo, preostala fina nastavitev pa je v `competitive.py`.
- **NOVO - HARDCORE** (stikalo v Tekmovalnem setupu, tipka **H**): vsak
  **pospešek požre dolžino** tvoje kače; rdeč žareč **napis HARDCORE** označuje
  način. Samo v Tekmovalnem; dolžina nikoli ne pade pod najmanjšo. Shranjeno v
  `settings.json`.
- **Zlata jabolka** (začasna) dajo veliko točk in v hipu napolnijo pospešek.
- Izbirno: **prehodni zidovi**, bonus jabolka, **prestiž** (en igralec, tipka **P**).
- **NOVO - Prilagodi** (gumb s čopičem povsem zgoraj desno v setupu ali tipka
  **C**): izključno vizualni meni (»modi«, ki *nikoli* ne spremenijo igranja) z
  dvema zavihkoma:
  - **Glava**: **barva glave** kače - 4 modro-turkizne prednastavitve (od bolj
    modre do bolj turkizne), rdeča, oranžna in **lastna barva** prek drsnikov RGB.
  - **Mreža (kažipot)**: čez polje položi **koordinatno mrežo** - **številke
    vrstic** (na levem in desnem robu) in **črke stolpcev** (zgoraj/spodaj). Tako
    na velikih ploščah takoj vidiš, da je npr. jabolko na *8a* v isti vrstici *8*
    kot tvoj položaj *8z*. Zaporedje barv (5 prednastavitev + dve lastni barvi
    A/B) določa barvno temo.
  - **Pasica**: pasico množitelja (npr. od vijoličnega jabolka) **vklopi/izklopi**
    ter nastavi njeno **velikost** (manjša/večja) in **prosojnost** (bolj
    prozorna) - s predogledom v živo.
  Vse je shranjeno v `mem-ngb.json`; vsa vizualna prilagoditev teče prek modula
  `ngb.py`.
- Videz: zaobljena kača z očmi (glava je privzeto turkizna), sij pospeška, delci.

**Pong**
- Za enega igralca proti UI, večigralsko = igralec 2 na desni. Do 5 točk.
- **Način gibanja preklopljiv za vsak nabor tipk**: *Zvezno* (pritisni enkrat ->
  se premika naprej, privzeto) ali *Držanje* (premika se le, dokler držiš).
  Preklop: **X** = nabor tipk 1, **N** = nabor tipk 2 (shranjeno v `settings.json`).
- Fizika žogice s pospeševanjem in kotom glede na točko dotika.

**Air Hockey**
- **Prava 2D-fizika**: okrogla loparja in plošček s prenosom gibalne količine -
  plošček ob udarcu prevzame hitrost loparja; stene z odbojnostjo, rahlo ledeno
  trenje, vrata kot odprtine v stranskih stenah.
- **Upravljanje z miško** za enega igralca: lopar sledi miški (katera koli tipka
  preklopi nazaj na tipkovnico). Tipkovnica: smerne tipke v 8 smereh,
  večigralsko = I1 levo (WASD), I2 desno (IJKL).
- **UI s tremi jakostmi** (Lahka/Srednja/Težka): brani svoja vrata, napada na
  svoji polovici in obkroži plošček, da prepreči avtogole.
- **Power-ups** (lahko izklopiš): *XL* (večji lopar), *TOR* (nasprotnikova vrata
  se skrčijo), *>>* (hitrejši lopar) - pripadajo igralcu, ki se je zadnji dotaknil
  ploščka.
- Setup: težavnost, **golov do zmage** (3/5/7/10), power-ups vkl./izkl. (shranjeno
  v `settings.json`). Po vsakem golu začne tisti, ki ga je prejel.
- Videz: svetleča sled ploščka, delci, utripajoča usta vrat, oznake učinkov.

**Tic-Tac-Toe**
- Setup: težavnost (Lahka/Srednja/Težka) in velikost plošče 3x3..9x9; zmagovalna
  dolžina K = 3 (3x3), 4 (4x4), sicer 5.
- **1 igralec** proti UI (Težka na 3x3 je nepremagljiva) **ali 2 igralca** lokalno
  (X proti O, izmenično s klikom). Ob koncu igre: enter/klik = nova runda,
  **S** = nastavitve.

**Breakout**
- Vrste opek: navadna, **jeklena** (neuničljiva), **bomba** (eksplodira), **zlata**
  (dodatne točke).
- Power-ups: laser, ognjena krogla, lepljivost, ščit, kovanec in drugo;
  **množitelj kombinacij**.
- Učinki: delci, sledi žogice, tresenje zaslona, pojavna okna s točkami, številni
  vzorci ravni.
- Setup: **1/2/3** = težavnost, **levo/desno** = barva žogice, **gor/dol** =
  začetna raven, **M** = razporeditev. Igra: miška/puščice, **preslednica** izstreli
  žogico (sproži laser), **P/Esc** = pavza.

**Tetris**
- **Sodobna pravila Guideline**: igrišče 10x20, kocke iz **vrečke po 7**, **sistem
  vrtenja SRS** s pravimi wall kicki (tudi za kocko I), vrtenje v obe smeri,
  **shramba** (enkrat na kocko), **predogled 5 kock**, senčna kocka in **lock
  delay** (0,5 s, največ 15 ponastavitev).
- **Trije načini** na predigralnem zaslonu: *Solo*, *Proti AI* in *2 igralca*.
  Solo v nastavitvah ponuja **Maraton** (začetna stopnja 1–15, šteje za rekord),
  **Šprint 40 vrstic** (najboljši čas) in **Ultra 2 minuti** (najboljši rezultat);
  najboljši rezultati se shranijo v razdelek `tetris` v `mem.json`.
- **Točkovanje Guideline**: od singla do Tetrisa, **T-Spini** (polni in mini),
  **Back-to-Back** (x1,5), **kombinacije** in **Perfect Clear** - z napisi na
  zaslonu, animacijo brisanja vrstic, sledjo trdega spusta, delci in učinkom
  napredovanja.
- **Versus z vrsticami smeti**: počiščene vrstice pošiljajo nasprotniku smeti
  (Tetris = 4, T-Spin Double = 4 …), prihajajoče smeti napove opozorilni trak,
  lastni napadi pa jih **odštejejo**; obe igrišči dobita enako zaporedje kock.
  **UI** ima 3 jakosti, hitrost pa se poveča vsakih 40 sekund.
- **Upravljanje**: levo/desno z lastnim **DAS/ARR** (nastavljivo v nastavitvah),
  gor = vrtenje desno, dol = mehki spust, akcija = trdi spust; **C**/Shift =
  shramba, **Z**/**Y** = vrtenje levo, **X** = vrtenje desno. V dvoje igralec 1
  shranjuje s **Q** in vrti levo z **E**, igralec 2 z **desnim Shiftom** /
  **desnim Ctrl**. Po igri: **R** = znova, **S** = nastavitve.

**Invaders** – dva načina (izbirna na predigralnem zaslonu):
- **Klasični**: klasičen blok vesoljcev; nato izbirno v setupu: **gibanje** (samo
  levo/desno *ali* prosto z WASD) in **merjenje** (vedno navzgor *ali* proti
  **miški** – takrat streljaš tja, kjer je kazalec). Uničeni vesoljci včasih
  spustijo power-ups.
- **Arena (prosto)**: prosto gibanje v vse smeri, sovražniki pritekajo z vseh
  robov; meriš v smeri gibanja, orožje menjaš s **1–4**.
Skupno: sistem ravni z **bossom** vsako 4. raven, štiri orožja (blaster,
razpršeni strel, hitri ogenj, laser), power-ups (dodatno življenje, ščit,
nadgradnja orožja), učinki eksplozij, rekord.

**Asteroids**
- **Fizika vztrajnosti**: gor = potisk v smeri pogleda, levo/desno = obrat, ladja
  drsi naprej (rahlo dušenje); vse se ovija okoli robov zaslona. Klasičen
  **vektorski videz** s plamenom potiska in zvezdnim nebom; vsak balvan ima svojo
  naključno poligonsko obliko.
- Balvani se razletijo na dva manjša (3 velikosti, **20/50/100 točk**), **vali**
  z naraščajočim številom in napovedjo v pasici.
- **NLP** (lahko izklopiš): redno prečka zaslon in meri na ladje (napaka merjenja
  glede na težavnost) - 200 točk za sestrel.
- **Power-ups** (lahko izklopiš), padejo iz uničenih balvanov: **S** = ščit (6 s
  neranljiv), **T** = trojni strel, **R** = hitri ogenj.
- **Hiperprostor** (tipka dol): zasilni skok na naključni položaj s 4 s premora -
  in 12 % tveganja, da se ob prihodu raztreščiš.
- 3 življenja, varno oživljanje z utripanjem neranljivosti, **dodatno življenje
  vsakih 5000 točk**; delci eksplozije in tresenje kamere.
- **Kooperativni dvoboj** (večigralsko): obe ladji letita hkrati z ločenimi
  življenji in točkami - zmaga tisti z več točkami.
- Setup: težavnost, NLP-ji vkl./izkl., power-ups vkl./izkl. (v `settings.json`).

**Pac-Man**
- **Klasičen labirint 28x31** v neonskem videzu s pikami, 4 močnimi tabletami,
  stranskimi tuneli (warp) in hišo duhov na sredini.
- **Štirje duhovi z izvirnimi vedenji** (UI ciljne ploščice): *Blinky* lovi
  naravnost, *Pinky* pripravi zasedo (4 ploščice naprej), *Inky* uporabi vektor
  skozi Blinkyja, *Clyde* se umakne, ko je blizu.
- **Faze scatter/chase** se izmenjujejo (duhovi ob vsaki menjavi obrnejo smer);
  **močna tableta** duhove obarva modro in jih naredi užitne (veriga
  200/400/800/1600), nato se oči vrnejo v hišo.
- Hiša duhov s **stopnjevanim izpuščanjem**, **sadje** kot bonus (na raven),
  **3 življenja**, **dodatno življenje pri 10.000**, sistem ravni (postaja
  hitrejši), animacija smrti, zaslona READY/GAME OVER.
- Setup: **težavnost** (Normalna/Težka/Ekstremna) – hitrost duhov in čas
  prestrašenosti.
- Upravljanje: **puščice ali WASD**.  Enter = novo, S = setup.

**Flappy Bird**
- **Fizika gravitacije**: preslednica / gor / W / **klik z miško** povzroči, da
  ptica zaplahuta; nagiba se glede na hitrost vzpona/padca.
- Neskončni **pari cevi** z režo (+1 na cev); **kovanci** (bonus) in power-up
  **ščit** (preživi en trk) se pojavijo v režah.
- **Temi dan/noč** se menjata s številom točk; plavajoči oblaki (paralaksa),
  drseča tla.
- Težavnost (Lahka/Normalna/Težka): velikost reže, hitrost, razmik cevi – reža se
  z naraščanjem točk nekoliko zoži.
- **Medalje** (bron/srebro/zlato/platina) ob game overju, animacija trka s
  tresenjem kamere, rekord.

**Doodle Jump**
- Doodler **samodejno skoči** ob pristanku; krmiliš le levo/desno (z vztrajnostjo),
  robovi se ovijajo, kamera se ob vzponu pomika navzgor.
- **Vrste ploščadi**: zelena (navadna), modra (premikajoča), rjava (se zlomi),
  bela (izgine). **Vzmeti** dajo super odskok, **propelerski klobuk** te za hip
  ponese navzgor (in te naredi neranljivega).
- **Pošasti**: dotik je smrtonosen – lahko pa jih **ustreliš** z gor / preslednico
  (dodatne točke).
- Točke = dosežena višina; težavnost narašča z višino. Rekord.
- Upravljanje: levo/desno = premik, gor / preslednica = strel.

**2048**
- **Lasten nastavitveni zaslon** s ploščami od **3x3 do 8x8** in tremi načini:
  *Klasično* (cilj 2048, nato »Igraš naprej?«), *Na čas* (3 minute, ura se zažene
  ob prvi potezi) in *Neskončno*.
- **Tekoče animacije**: ploščice drsijo, se združijo z učinkom »pop« in zrastejo v
  polje; pojavne točke, iskre od 128, udarni val od 2048 in nove barve vse do
  131072. Vnosi med animacijo se shranijo v vrsto.
- **Razveljavitev** (izklopljeno / 3 na igro / neomejeno, tipka **U** ali
  vračalka) - kdor jo uporabi, igra brez rekorda in brez dosežkov za ploščice.
- **Shrani in nadaljuj**: tekoča partija se samodejno shrani za vsako velikost in
  način; najboljši rezultati in največja ploščica po velikosti/načinu so v
  razdelku `g2048` v `mem.json`.
- Upravljanje: puščice/WASD ali **poteg** z miško/sledilno ploščico, **R**/**N** =
  nova igra, **Tab** = nastavitve. Rekord šteje le v **4x4 Klasično** brez
  razveljavitve.

**Minesweeper**
- Tri ravni: **Začetnik** (9x9, 10 min), **Napredni** (16x16, 40), **Strokovnjak**
  (30x16, 99) - **najboljši čas na raven** se shrani in prikaže v setupu.
- **Prvi klik je vedno varen** (mine se razporedijo šele po njem, območje 3x3
  okoli klika ostane prosto).
- **Levi klik** = razkrij, **desni klik** = zastavica (izbirno s ciklom vprašaja),
  **F** = zastavica pod kazalcem, **R** = nova igra.
- **Chording**: klik na izpolnjeno število razkrije preostale sosede.
- Klasičen HUD: števec min, **klikljiv smeško** (presenečen/sončna očala/mrtev),
  časovnik; napačne zastavice so na koncu prečrtane, konfeti ob zmagi.
- Točke = osnovna vrednost ravni minus sekunde.

**Sudoku**
- **4 različice** s po **400 ravnmi** (4 težavnosti x 100): *Klasični* (znane
  ravni s semenom - rešene ostanejo odkljukane), *X-sudoku* (obe diagonali
  vsebujeta vsako števko natanko enkrat), *Killer* (črtkane kletke z vsoto; 400
  vnaprej ustvarjenih ravni) in *Mini 6x6*. Vsaka uganka ima **enolično
  rešitev** - raven 12 v »Težka« je na vsakem računalniku ista uganka.
- **Sudoku dneva**: ena uganka na dan za vse, enaka na računalniku in v
  brskalniku; težavnost je odvisna od dneva v tednu (od ponedeljkove Lahke do
  sobotne Strokovnjak), z vsakodnevnim reševanjem pa gradiš niz.
- **Do 3 zvezdice na raven** (rešeno · brez napak in namigov · poleg tega pod
  ciljnim časom) in **najboljši čas** v izbiri ravni; **začete uganke** se
  samodejno shranijo in se naslednjič nadaljujejo.
- **4 igralni načini** (izbrani pred začetkom) z množiteljem točk: **Klasični**
  (x2.0 - brez pomoči), **Zapiski** (x1.5 - + zapiski s svinčnikom in samodejni
  kandidati), **Udobje** (x1.0 - + napačne števke rdeče, označeni konflikti in
  napačne vsote kletk, pravilni vnosi se zaklenejo), **Pomočnik** (x0.7 - + tipka
  za namig, največ 3). Z vklopljeno **omejitvijo 3 napak** (možnost v setupu)
  tretja napaka konča igro.
- Upravljanje: puščice/WASD = celica, **1-9** = števka (tudi numerična
  tipkovnica), **0/Delete/desni klik** = izbriši, **U**/**Z** = razveljavi, **Y** =
  uveljavi znova, **N** = zapiski, **C** = samodejno vpiši kandidate, **H** = namig,
  **M** = barvna oznaka, **R** = ponovi raven, **Q** = izbira ravni. Vnos »najprej
  števka« (setup, **I**) in **števec manjkajočih števk** pod vsako števko; povsem
  igralno z miško. Po koncu igre **A** razkrije celotno **rešitev**.
- Točke = (osnova različice in težavnosti - čas - napake - namigi) x množitelj
  načina; vse različice in Sudoku dneva štejejo za rekord.

**Frogger**
- 5 prometnih pasov (avtomobili/tovornjaki) in 5 rečnih pasov (debla, želve, ki se
  na višjih ravneh **potopijo**); zgoraj 5 domačih zalivov - napolni vse =
  naslednja raven, vse se pospeši.
- Dodatki: **bonus muha** (+200) v praznih zalivih, **krokodili** na višjih ravneh
  zasedejo zalive, **vrstica časovne omejitve** na vsako žabo, dodatno življenje
  pri 10.000.
- 3 težavnosti (hitrost, gostota prometa, čas); točke na vsako novo vrsto, zaliv =
  50 + časovni bonus, dokončana raven = +1000.

**Memory**
- Velikosti plošče **4x4, 6x6, 8x6**; motivi so kombinacije oblika-barva, v celoti
  narisani s primitivi; **animacija obračanja**, neujemajoči se pari se samodejno
  obrnejo nazaj.
- **Samostojno**: osnova - 15 na potezo - 2 na sekundo (najm. 100). **Dvoboj**
  (lokalno): izmenične poteze, ujemanje prinese še eno potezo, zmaga tisti z
  največ pari.

**Pasjansa**
- **5 različic** na predigralnem zaslonu: Klondike (možnost vlečenja 1/3), Spider
  (1/2/4 barve), FreeCell (omejitev supermove), Piramida (pari do 13, 2 ponovni
  delitvi) in TriPeaks (veriga ±1 z množiteljem kombinacij).
- **Povleci in spusti** ali klik-klik, **desni klik** = na temelj, **U** =
  neomejena razveljavitev, **R** = nova delitev, preslednica = kup.
- Karte se izrišejo brez slikovnih datotek (`games/cards.py`); vse različice si
  delijo en seznam rekordov z različici lastnimi formulami.

**Aim Trainer**
- **Pravi programski 3D** (kot 3D-način igre Snake): fiksni križec na sredini
  zaslona, **neposredno merjenje z miško 1:1 kot v strelski igri** (zajem kazalca:
  kazalec je ujet znotraj okna, Esc ga sprosti; nastavljiva občutljivost, neomejen
  yaw, pitch ±60°). Levi klik ustreli natanko skozi sredino, z bliskom cevi,
  sledilko in delci zadetka.
- **4 načini**: natančnost (60 s, 3 krogle, bonus za natančnost), refleks (30
  posamičnih tarč, statistika reakcijskega časa), premikajoče tarče (poti +
  množitelj kombinacij do x4) in chill (neskončno, brez kazni, **E** konča sejo).
- **3 teme** (v setupu, shranjene): **vesolje** z zvezdno kroglo, **črno luknjo s
  svetlečim obročem** in planetom (privzeto), neonska arena s talno mrežo in
  soncem synthwave ter notranje strelišče.
- Občutljivost lahko spreminjaš tudi med igro s **+/-**; poleg tega **nastavljiv
  motion blur** (0-80 %) za še bolj chill videz - oboje se shrani.

**Štiri v vrsto**
- Plošča 7x6 z **animacijo padanja**, predogledom ob prehodu miške in utripajočo
  zmagovalno črto; miška, puščice ali neposredna izbira **1-7**.
- **3 jakosti UI** (minimax z iskanjem alfa-beta): Lahka namenoma spregleda
  grožnje, Srednja zanesljivo blokira, Težka načrtuje globoko naprej - ali
  **2 igralca** lokalno na isti napravi.
- Začetni igralec se menja vsako rundo; rekord šteje tvoje **zmage proti UI** v
  eni seji.

**Tankovski dvoboj**
- 2D-dvoboj v areni: **streli se enkrat odbijejo od sten** (ricochet) - zadeneš
  okrog vogalov (ali samega sebe!). Na 5 dobljenih rund z odštevanjem.
- **4 arene** (Odprta, Križ, Stebri, Labirint) ali naključna rotacija;
  **power-ups**: hitri ogenj, ščit, trojni strel.
- **UI s 3 jakostmi** - težka strelja z upoštevanjem gibanja in namenoma odbija ob
  stene - ali **2 igralca** na eni tipkovnici (I1 WASD+preslednica, I2 puščice+enter).

**Blackjack**
- Prava kazinojska pravila: **čeveljec s 4 paketi**, delivec ostane pri 17,
  **blackjack plača 3:2**, delivčev peek ob asu/10; **podvojitev** in **ena
  delitev** (razdeljena asa dobita vsak po eno karto).
- **Lama žetoni**: Blackjack igra z računom **Lama banke**, ki si ga deli s
  Pokrom in Casinom (začetek 1000, trajno shranjen v `mem.json`). Stava se odšteje
  takoj ob deljenju; pod 10 žetoni enter vzame **bančni kredit**, ki račun spet
  napolni do 1000.
- **Rekord** = najvišje stanje tvoje **bilance v Blackjacku** (1000 plus vse, kar
  je bilo v Blackjacku dobljeno in izgubljeno) - dobitki na ruleti, avtomatu ali v
  pokru tu ne štejejo, krediti prav tako ne.
- Igra se prek gumbov za žetone in tipk (**H**it/**S**tand/**D**ouble/delitev
  **X**, **1-4** = stava, vračalka = počisti stavo, enter = razdeli) z animacijami
  kart; delivčeva zakrita karta se ob razkritju zdaj res obrne.

**Tunnel Racer**
- **3D-let skozi neonsko cev** (programski izrisovalnik kot pri Aim Trainerju):
  drogovi, bloki in **obročni prehodi za predevanje**, kovanci na idealni liniji.
- **Dva načina**: neskončni (hitrost narašča do stropa, rekord) in **30 ravni s
  semenom** s ciljno črto, časovnim bonusom in odkljukanim napredkom.
- **Upravljanje s tipkami** (privzeto) ali **neposredno upravljanje z miško**
  (zajem kazalca, tipka **C**); poleg tega nastavljiv **motion blur** (tipka **B**,
  0-80 %) - vse se shrani.

**3D-labirint**
- **Raycaster v prvi osebi v slogu Wolfenstein** (DDA, megla razdalje, spriti) z
  mouselookom + WASD, **minimapo** (tipka **M**) in zeleno utripajočim izhodom -
  ali klasičen **2D-pogled od zgoraj** (tipka **V** v setupu).
- **50 ravni s semenom**, ki nenehno rastejo; izhod je vedno na točki, najbolj
  oddaljeni od začetka, **orbi** ob poti dajejo bonus točke.
- Točkovanje: 500 na raven + 100 na orb + časovni bonus; rešene ravni so
  odkljukane, vsota seje pa postane rekord.

**Reversi**
- **Othello na 8x8**: postavljaj ploščke, ki ujamejo nasprotnikove vrste, in obrni
  vse ujeto; nedovoljene poteze so blokirane, poteza brez možne poteze pa se
  **samodejno preskoči**.
- **En igralec proti UI** (3 jakosti: negamax z alfa-beta, položajno uteževanje +
  mobilnost) **ali lokalni dvoboj**, črni proti belim.
- Dovoljena polja so poudarjena; igraj z **miško** ali izbirnim okvirjem (puščice +
  preslednica/enter). Vsaka zmaga proti UI šteje eno točko k rekordu.

**Kniffel (Yahtzee)**
- **Klasika s kockami**: 5 kock, do 3 meti na potezo, kocke **zadržiš** posamično,
  nato vpišeš eno od **13 kategorij** (s predogledom možnih točk v živo).
- Cel točkovni list: zgornji del z **bonusom 63 točk (+35)**, tris/poker, full
  house, mala/velika lestvica, **Yahtzee (50)** in Chance.
- **En igralec kot lov na najvišjo vsoto** ali **dvoigralski hotseat** z dvema
  listoma drug ob drugem; igraj z miško ali tipkami (preslednica, 1-5, puščice,
  enter).

**Wordle**
- Ugani skrito besedo; barvna povratna informacija (zeleno/rumeno/sivo) s
  **pravilnim štetjem podvojenih črk** in zaslonsko tipkovnico, ki se obarva
  (QWERTZ za nemščino, češčino, slovenščino in hrvaščino, AZERTY za francoščino,
  sicer QWERTY).
- **Štirje načini**: *Neskončno* (beseda za besedo, vsakič 6 poskusov; vsaka
  rešena beseda prinese točke, prva nerešena konča igro), *Beseda dneva* (ena
  beseda na dan za vsak jezik in dolžino - enaka na računalniku in v brskalniku -
  z odštevanjem in nizom; začeta beseda dneva se shrani), *Dordle* (2 besedi hkrati
  v 7 poskusih) in *Quordle* (4 besede v 9 poskusih, tipke kažejo barve vseh mrež).
- **Nastavitve** pred vsako igro: **dolžina besede 4 do 7**, **težki način**
  (najdene namige je treba uporabiti naprej) in **paleta za barvno slepe**
  (oranžna/modra); ob strani je statistika.
- **Pravi seznami besed v vseh 14 jezikih** (mapa `woordlistz/`, samo A-Z), za
  vsako dolžino posebej: samo pri 5 črkah skoraj **34.000 rešitev** in več kot
  **213.000 dovoljenih besed**, v vseh štirih dolžinah skupaj približno 134.000
  rešitev. Rešitve so pogoste besede brez imen, angleških ostankov in žaljivih
  besed; vsak poskus se preveri s seznamom - drugo je zavrnjeno in vrstica se na
  kratko strese.
- **Statistika** po jeziku, dolžini in načinu: igre, delež zmag, trenutni in
  najdaljši niz ter **razporeditev poskusov kot stolpčni graf** (razdelek `wordle`
  v `mem.json`). **Deli** (**C**) kopira mrežo emojijev, ne da bi izdala rešitev.
- Za rekord šteje le *Neskončno* s 5 črkami; druge dolžine imajo svoje najboljše
  rezultate. Dosežki **Jasnovidec** (največ 2 poskusa), **Besedna navada** (7
  besed dneva zapored) in **Štirikratni genij** (rešen Quordle).

**Poker**
- **3 različice** na predigralnem zaslonu: **Texas Hold'em** proti 1–3 nasprotnikom
  UI z gumbom delivca, blindi in štirimi krogi stav, **5 Card Draw** (ena na ena
  proti UI, ena menjava kart) in **Video Poker** (*Jacks or Better*, solo proti
  tabeli izplačil).
- Dejanja z gumbi ali tipkami: **F** = odstop (fold), **C** = check/call, **R** =
  dvig, **A** = all-in; zadrži/zamenjaj karte s klikom ali **1-5**, **enter**
  dobi karte ali razdeli naslednjo roko.
- **Lama žetoni** skupne **Lama banke**: na začetku roke je tvoj račun na mizi kot
  kup, kar gre v pot, pa se takoj odšteje - če mizo zapustiš sredi roke, izgubiš
  le svoj delež v potu. Brez denarja (manj kot veliki blind 20, v Video Pokru manj
  kot 10) = bančni kredit do 1000.
- **Rekord** = najvišje stanje **bilance v Pokru** (1000 plus vsi dobitki in
  izgube v pokru); dosežek **Chipleader** prav tako šteje le to bilanco.

**Šah**
- **Popolni šah**: vse poteze figur, vključno z **rošado**, **en passant** in
  **pretvorbo kmeta** (izbereš figuro); **šah, mat in pat** ter remi po **pravilu
  50 potez**, **trikratni ponovitvi položaja**, **nezadostnem materialu** ali
  dogovoru.
- **Trije načini**: *partija* proti UI, *2 igralca* za istim računalnikom (plošča
  se lahko po vsaki potezi obrne) in **uganke**.
- **Močnejša UI brez zatikanja** v 6 jakostih od *Začetnika* do *Mojstra*:
  iterativno poglabljanje, transpozicijska tabela, umirjeno iskanje, knjiga otvoritev
  in ocena z mobilnostjo, strukturo kmetov in varnostjo kralja. UI računa v majhnih
  obrokih na sličico - igra se nikoli ne zatika.
- **Nastavitve**: izbira barve, **šahovska ura** (brez, 1+0, 3+2, 5+0, 10+5) in
  **Chess960** (vseh 960 začetnih postavitev, številka je nad seznamom potez).
- **Stranska plošča** z urama, zajetimi figurami, materialno bilanco in drsnim
  **seznamom potez (SAN)**; povleci in spusti, drseče figure, koordinate. Tipke:
  **U** = razveljavi, **H** = puščica namiga, **O** = ponudi remi, **X** = predaja,
  **F** = obrni ploščo, po partiji **P** = **izvoz PGN**.
- **Uganke**: 200 ugank v 5 stopnjah (mat v 1/2/3, taktika I/II) iz **proste baze
  ugank Lichess (CC0)**, preverjenih z lastnim pogonom; pri matnih ugankah velja
  vsaka poteza, ki matira. Napredek je v razdelku `chess` v `mem.json`.
- Razveljavitev in namig naredita partijo »podprto«: rekord šteje le zmage proti
  UI brez pomoči (na sejo).

**Mlin**
- **Mlin** z vsemi tremi fazami: **postavljanje** (po 9 figur), **premikanje**
  vzdolž črt in **letenje**, ko ti ostanejo le 3 figure (lahko izklopiš).
- Sklenjen **mlin** odstrani nasprotnikovo figuro (po možnosti tako zunaj mlina);
  izgubiš, ko padeš pod 3 figure ali se ne moreš premakniti.
- **3 jakosti UI** (minimax z alfa-beta, oceni glede na fazo) ali **lokalni
  dvoboj**; z namigi potez, poudarjanjem mlinov in števcem figur.

**Simon**
- **Spominska igra Senso**: osvetljeno zaporedje raste vsako rundo in ga je treba
  natanko ponoviti.
- **Načini**: *Klasični*, *Speed* (postaja hitrejši), *Reverse* (nazaj), *Mešani*
  (način se menja vsako rundo) in dvoigralski **Dvoboj** (izmenično dodajanje in
  ponavljanje).
- **Zvok** *izkl. / vkl. / mešan* (mešani trenira tvoj vizualni IN slušni spomin),
  **4/6/9 polj** kot težavnost; **najboljši rezultat na način** se shrani. Igraj z
  miško ali številskimi tipkami 1-9.

**Biljard**
- **8-ball**, **9-ball** in način **vadbe** brez pravil, proti UI (s pomočjo pri
  merjenju) ali **dva igralca lokalno**.
- **Trije prosto izbirni pogledi**: klasičen **2D od zgoraj**, fiksna
  **3D-poševna perspektiva** z osenčenimi kroglami in **prosto vrtljiva
  3D-kamera** (desni gumb miške). Vse gibanje temelji na časovnih korakih in je
  **gladko dušeno** (trenje, podkoraki proti pretunelanju).
- **Udarec**: drži levi gumb miške za polnjenje moči, spusti za udarec; pomagata
  ciljna črta in merilnik moči. Po prekršku **žogica v roki**. Pogled (V) se shrani
  v `settings.json`; dobljene igre štejejo k rekordu.

**Drsna sestavljanka**
- Igra 15 v treh velikostih: **3×3** (lahko), **4×4** (klasično) in **5×5**
  (težko); drsi oštevilčene ploščice v prosto vrzel.
- Vedno rešljivo (premešano z mnogo naključnimi potezami). Upravljanje s **klikom**
  na ploščico v vrstici/stolpcu vrzeli (zdrsne cela linija) ali s **puščicami**.
- Točke = osnovna vrednost na velikost minus poteze in čas; po rešitvi se takoj
  začne nova plošča.

**Mastermind**
- Razbij skrito **barvno kodo**; po vsakem ugibanju dobiš **črne** zatiče (prava
  barva + položaj) in **bele** zatiče (prava barva, napačno mesto).
- **3 načini**: Lahki (4 zatiči / 6 barv / 12 vrstic), Klasični (4/6/10) in Težki
  (5 zatičev / 8 barv); podvojene barve so dovoljene.
- Igra prek barvne palete (klik ali tipke **1–8**), OK/enter oceni vrstico.
  **Neskončni niz** kot pri Wordlu: vsaka razbita koda prinese točke.

**Bubble Shooter**
- **Puzzle Bobble** na satasti mreži: meri z miško, streljaj mehurčke navzgor,
  **tri ali več enake barve** razpočijo skupino.
- Mehurčki, ki izgubijo povezavo s stropom, **padejo** (bonus); streli se
  **odbijajo od sten**, s predogledom naslednjega mehurčka.
- **3 načini** (4/5/6 barv, nekateri s spuščajočimi se vrstami); game over pri
  rdeči črti.

**Vislice**
- Ugani besedo **črko za črko**; vsaka napaka nariše del obešenca, izgubljeno po
  **6 napakah**.
- **Seznami besed po jeziku** (samo A–Z), **3 dolžinski načini** (kratke / mešane /
  dolge); tipkaj ali klikaj zaslonsko tipkovnico.
- **Neskončni niz**: vsaka uganjena beseda prinese točke (več preostalih življenj +
  daljša beseda = več).

**Block Jump**
- **3D-ploščadnica v slogu Minecrafta** (programski 3D kot 3D-način igre Snake):
  skači čez lebdeč **voksel svet** blokov do žarečega cilja.
- **Minecraftov videz**: vsi bloki imajo prave **pikselske teksture** (trava,
  zemlja, kamen, deske, diamant, sluz, les); raven podrobnosti sledi razdalji
  (**T** = visoko/nizko/izklopljeno).
- Poleg tega: lik **Steve** z animacijo hoje (kamera v tretji osebi), **roka**
  v prvi osebi, **žarek svetilnika** pri cilju, vrteče se **zlate palice**
  namesto kovancev, kvadratno **sonce**, **pikselni oblaki** in HUD s **srci**.
- Vrste blokov: trdni bloki (trava/zemlja/kamen/les), **lestve** (plezanje),
  **ograje** (preskoči), **vzmetni bloki** (izstrelijo) in **kovanci**.
- Kamera **privzeto v prvi osebi kot v Minecraftu**, **V** preklopi na kamero
  zasledovalko; **pogled z miško** z zajemom kazalca, nastavljiv **motion blur**
  (**B**) in občutljivost (**+/-**).
- **Ravni parkourja s semenom** postajajo težje; cilj = točke + časovni bonus,
  kovanci +50, padec stane življenje (začneš s 3). Upravljanje: WASD/puščice,
  **preslednica** za skok.

**Tower Defense**
- **Neskončna obramba pred valovi** na **4 zemljevidih** (Travnik, Kanjon,
  Križišče, Špalir), vsak s svojo potjo; zaklenjene zemljevide odklene tvoj
  najboljši val, vsak **8. val** prikoraka **boss**.
- **3 načini**: Klasični (7 stolpov, glavni način), Kompaktni (4 stolpi,
  2 stopnji) in Maksimalni (**11 stolpov**, **specializacija A/B** na najvišji
  stopnji, posebni sovražniki, aktivne sposobnosti **Meteor/Ledena nova/Zlata
  mrzlica**).
- **11 vrst stolpov** od puščic do laserja in zlate banke, vsak z do
  **3 stopnjami nadgradnje**, prodaja vrne 70%; sovražniki z oklepom,
  regeneracijo, delitvijo, prikrivanjem, zdravilno avro in zračno potjo.
- **Ekonomija**: zlato za sestrelitev, bonus za val + 5% obresti; točke za
  sestrelitve in valove. **F** = 2x hitrost, **G** = dosegi, desni klik
  prekliče.

**Minigolf**
- **360 stez na 40 igriščih**: *Classic* in *Pro* z devetimi ročno zgrajenimi
  stezami vsako, **Tour** z 38 igrišči po devet ustvarjenih stez (skupaj 342) in
  naraščajočo težavnostjo, poleg tega *Random* iz vsega skupaj. Igrišče 7,
  steza 3 je povsod enaka - shranjevati ni treba ničesar.
- **Podlage in ovire**: pesek zavira, klančine pospešujejo, voda stane kazenski
  udarec, gumijasti odbijači vračajo hitrost, mlini in tavajoči bloki pa zahtevajo
  občutek za čas. Fizika teče v podkorakih s trenjem kot pri biljardu - nič ne
  poskakuje in nič ne gre skozi bando.
- **Upravljanje**: miška cilja, držana leva tipka polni moč, spust pa udari
  (delujejo tudi puščice + preslednica). **R** prekliče napolnjen udarec brez
  udarca. **G** preklopi ciljno črto, **Z** samodejno merjenje, **P**
  pobiranje.
- **Zaklep moči (držana desna tipka)**: zamrzne polnilno črto točno tam, kjer
  je - zlata, z odstotki, ključavnico in utripajočim obročem okoli žogice. Tako
  z napolnjenim udarcem počakaš na vrzel v mlinu. Ko spustiš, se polni naprej;
  zaklenjena moč preživi tudi udarec, naslednji levi klik pa udari točno s to
  vrednostjo.
- **Kartica rezultatov** desno s parom in udarci na stezo; v dvoje vsak igra isto
  stezo po vrsti. Točke: 600 na stezo, ±300 za udarec pod/nad par, **500 dodatnih
  za hole in one**. Najnižje število udarcev na igrišče je v razdelku `minigolf`
  datoteke `mem.json`.
- **Pobiranje je mogoče izklopiti**: privzeto se steza konča po osmih udarcih in
  šteje najmanj. Kdor raje igra do zadetka, nastavi *Pobiranje* v nastavitvah na
  IZKLOP (ali pritisne **P**).
- **Samodejni cilj je mogoče izklopiti**: privzeto se palica pred vsakim
  udarcem sama obrne proti luknji. Kdor raje meri sam na vsaki stezi, nastavi
  *Samodejni cilj* v nastavitvah na IZKLOP (ali pritisne **Z**) - takrat ostane
  nazadnje izbrana smer, na začetku nove steze pa palica kaže nevtralno
  navzgor.
- **F** ponastavi stezo v igri: udarci na 0, žogica na začetek - ista steza, isto
  igrišče.
- **Naprej namesto ponavljanja**: ob koncu kroga gumb **Naprej** vodi na naslednje
  igrišče (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), tako da se ista deveterica
  stez ne ponavlja; poleg **Še enkrat** (isto igrišče) in **Nastavitve**. Tipke:
  Enter = naprej, R = še enkrat, S = nastavitve.
- **Posnetek kroga**: na koncu **P** (ali gumb **Posnetek**) predvaja cel krog
  udarec za udarcem. S tipko **S** gre v arhiv (gumb **Posnetki** v stranski
  vrstici).
- **Gradnja in deljenje lastnih stez**: zavihek **MAPS** na pripravljalnem
  zaslonu vodi do tvoje zbirke - **Nova** odpre urejevalnik stez. Vsaka steza
  dobi ime in **id** (male črke, brez presledkov); id je hkrati predlagano ime
  datoteke pri deljenju. K sedmim klasičnim oviram pride **osem novih**: cev
  (prestavi žogico na drugi konec), led, lepljivo polje, pospeševalnik,
  magnet, enosmerna vrata, vrtljiva plošča in skakalnica. Velikost steze je
  prosto nastavljiva (60x80 do 160x240), **12 predlog** da izhodišče, zraven
  sta razveljavi/uveljavi in **Test**. **Deli** zapiše natanko eno stezo kot
  datoteko `.lamapgzmap` - prek okna za shranjevanje ali naravnost v mapo
  Prenosi, s tvojim imenom kot ustvarjalca. **Uvozi** jo prebere nazaj in ob
  zasedenem id samodejno preide na `-2`. Ime, id in ustvarjalec gredo vedno
  skozi **besedni filter čez vseh 14 jezikov**. Stezo igraš posamično prek
  **Igraj**, celotno zbirko pa prek pete izbire igrišča **Lastne**.

**Pinball**
- **Tri mize**: *Classic* (trije odbijači, ena vrsta tarč), *Space* (štirje
  odbijači v rombu, dve vrsti) in *Lama* (odprto polje, šest tarč v loku); 3 ali 5
  krogel na igro, v dvoje izmenično krogla za kroglo.
- **Vse, kar flipper potrebuje**: izstrelitveni kanal z merilnikom moči (prešibko?
  krogla se vrne in smeš znova), dva loparja, slingshoti, vrste tarč, štiri steze
  **L-A-M-A**, past z zaklepom krogle, **multiball z jackpotom**, šest sekund
  **rešitve krogle**, sunek in **TILT**.
- **Množitelj do x5** prek podrtih vrst in dokončanih stez; odbijači 100,
  slingshoti 50, tarče 250 - med multiballom odbijači plačajo jackpot 2.500.
- Loparja delujeta na dodeljenih tipkah levo/desno (tudi levi/desni [Shift]) ali
  z miško. Rekord vsake mize je v razdelku `pinball` datoteke `mem.json`.

**Bowling**
- **Deset framov po uradnih pravilih**, vključno s striki, spari in bonus meti v
  desetem framu (največ: 300). **Kartica** pod glavo prikazuje vsak frame z X, /
  in tekočo vsoto.
- **Met v štirih korakih**: položaj, kot, vrtenje in moč. Vsak drsnik niha sam in
  se zaklene s tipko za dejanje - ali pa se nastavi ročno z levo/desno, kar
  nihanje ustavi.
- **Prava fizika kegljev**: deset kegljev kot krogi z maso, ki podirajo drug
  drugega; strike nastane iz fizike in ne iz sreče. Steza je spredaj naoljena,
  zato **hook** prime šele v zadnji tretjini.
- Pogled na stezo v perspektivi z žlebovi, puščicami in poljem kegljev; tri
  težavnosti (*Lahka/Normalna/Pro*) spremenijo hitrost drsnikov in raztros.
  Rekord za vsako težavnost je v razdelku `bowling` datoteke `mem.json`.
- **Posnetek partije**: na koncu **P** znova pokaže vse mete, **S** pa jih
  shrani v arhiv (gumb **Posnetki**).

**Crossy Road**
- **Neskončno skakanje** čez travnike (drevesa in kamni zapirajo pot), ceste z
  avtomobili in tovornjaki, reke z debli in lokvanji ter **tire**, po katerih po
  opozorilni luči in zvoncu pridrvi vlak - naprej čakajo cele postaje z do 5 tiri.
  Proga nastaja vrstico za vrstico, vedno ima prehodno pot, hitrost in promet pa
  naraščata.
- **Izometrični voksel slog**: liki, vozila in drevesa iz senčenih kock
  (vnaprej izrisani za vsako velikost polja), mehko sledeča kamera, squash &
  stretch pri skokih, pljuski vode, animacija sploščenja, perje in bleščeči
  kovanci; od vrstice 50 **menjava dneva in noči** z žarometi.
- **Orel**: kamera se počasi pomika naprej - kdor predolgo oklijeva ali se vrne za
  več kot tri vrstice, ga zgrabi orel (prej opozori rdeč rob). Če te deblo odnese
  čez rob slike, je igre prav tako konec.
- **Kovanci in liki**: zbrani kovanci (velikanski kovanec = 5) se shranijo in z
  njimi v zavihku **Liki** kupuješ nove like: žabo, prašiča, pingvina, mačko,
  lisico, lamo, robota, duha in samoroga (25 do 250 kovancev); piščanec je na voljo
  od začetka.
- **Načini**: *Neskončno* (točke = najdaljša vrstica, šteje za rekord) in *Dnevna
  proga* (danes enaka za vse, tudi v brskalniku, z lastnim dnevnim rekordom).
  Upravljanje: puščice/WASD, preslednica/enter/klik = skok naprej; v nastavitvah
  **H** = sence, **N** = dan/noč. Kovanci, liki in dnevni rekord so v razdelku
  `crossy` v `mem.json`.

**Geometry Dash**
- **Ritmična ploščadna igra**: lik sam drvi v desno - ti odločaš le, kdaj skočiti
  ali leteti. **Pet oblik** - kocka, ladja, žoga, NLP in val - poleg tega portali
  oblike, težnosti in hitrosti (0,5x do 3x), rumeni/roza/modri **odskoki in
  krogle**, polbloki, konice, jame in barvni sprožilci.
- **8 vgrajenih stopenj** od *Lahko* do *Demon* (»Lama Inferno«) s po **3
  skrivnimi kovanci**. Vsaka stopnja je dokazano rešljiva: ob gradnji jo je
  reševalec opravil s pravo kodo igre - z vsemi kovanci in celo z zamikom 1/240
  sekunde.
- **Natančna fizika**: računanje s stalno vejico v stalnem koraku 240 Hz; vsak
  pritisk učinkuje natanko v koraku, v katerem se je zgodil - enako pri vsaki
  hitrosti sličic in bitno enako v brskalniku.
- **Način vaje** (**P**) s samodejnimi in lastnimi kontrolnimi točkami (**Z**
  postavi, **X** izbriše), števcem poskusov, vrstico napredka, eksplozijami in
  takojšnjim ponovnim začetkom (**R**). Vsaka stopnja ima **svojo glasbo** -
  ozadje, tla in krogle utripajo v ritmu (glasbo izklopiš z **M**).
- **Zvezdice in kovanci**: kdor stopnjo opravi v običajnem načinu, dobi njene
  zvezdice, vsak kovanec pa je vreden še eno zvezdico; rekord je **skupno število
  zvezdic** (največ 65). Najboljši rezultati stopenj, kovanci, poskusi in skoki so
  v razdelku `geodash` v `mem.json`.
- **Urejevalnik stopenj** v zavihku **STOPNJE**: platno z mrežo, paleta s 6
  skupinami (bloki, nevarnosti, odskoki in krogle, portali, hitrost, dodatki),
  vrtenje, razveljavi/uveljavi, pregledni trak, **test od začetka ali od tu** in
  nastavitve stopnje (začetna hitrost in oblika, glasbeni slog, BPM, barve).
  Kljukica **»preverjeno«** se pojavi šele, ko svojo stopnjo sam opraviš.
  **Deli** zapiše datoteko `.lamapgzlevel`, **Uvozi** jo spet prebere; stopnje se
  shranijo v `ugc.json` poleg lastnih minigolf prog.

**Battleship**
- **Pomorska bitka 10x10** z letalonosilko (5 polj), bojno ladjo (4), križarko
  (3), podmornico (3) in rušilcem (2) - zmaga, kdor prvi potopi celotno sovražno
  floto.
- **Postavitev flote** z vlečenjem iz doka: **R** ali desni klik zavrti, predogled
  sveti zeleno ali rdeče, **X** vse postavi naključno, **C** izprazni ploščo;
  zadnja postavitev se znova predlaga.
- **Pravila v nastavitvah** (se shranijo): *ladje se lahko dotikajo*, *salva*
  (toliko strelov na potezo, kolikor lastnih ladij še plava) in *po zadetku
  streljaš še enkrat*.
- **UI s 3 jakostmi**: Lahko strelja naključno, Srednje sistematično dotolče
  zadetke, Težko izračuna **zemljevid verjetnosti** s parnostjo šahovnice (v
  povprečju približno 70 / 60 / 45 strelov za celo floto). Ali **2 igralca** za
  istim računalnikom - **zaslon za predajo** pred vsako potezo skrije obe floti.
- **Grafika**: radarski preplet, animirani valovi, granate v loku, pljuski,
  eksplozije z dimom in gorečimi polji, razkritje »POTOPLJENO!« ter povzetek
  kroga s streli, zadetki in natančnostjo. Rekord šteje tvoje **zmage proti UI** v
  eni seji.

**Casino**
- **Ruleta** (evropska, 37 polj): vse klasične stave s klikom na število, rob ali
  kot - **plein** (35:1), cheval, transversale, carré, sixain, stolpec, ducat,
  rdeče/črno, sodo/liho in manque/passe. Žetoni 1/5/25/100/500, desni klik žetone
  odstrani; **Zavrti**, **Ponovi** (**R**), **Podvoji** (**D**) in **Počisti**.
  Kroglica se v spirali zakotali v vnaprej izžrebano polje, zgoraj pa je vidnih
  zadnjih 12 števil.
- **Lama avtomat**: 5 kolutov x 3 vrstice, **10 dobitnih linij**, **lama = divji
  simbol**, **zlatniki = scatter** z 10 brezplačnimi vrtljaji in dvojnimi dobitki,
  stava na linijo 1/2/5/10, **samodejno vrtenje** (10/25), **turbo** in tabela
  dobitkov. **Stopnja vračila je 96,1 %** - natančno izračunana iz trakov kolutov.
- **Lama banka**: Casino, Blackjack in Poker si delijo en račun **lama žetonov**
  (začetek 1000, razdelek `casino` v `mem.json`); stara stanja žetonov se prenesejo
  samodejno. Stave se odštejejo takoj, vsaka igra vodi svojo bilanco za rekord, ob
  bankrotu pa dobiš **bančni kredit** do 1000.
- Konfeti, dež kovancev, pasice big/mega/jackpot in animacije dobitnih linij;
  dosežka **Polni zadetek** (dobljen plein na ruleti) in **Lama jackpot** (5 lam na
  eni liniji).

Rekordi se shranijo v razdelku `highscores` datoteke `mem.json` (poleg kode) –
skupaj z jezikom (razdelek `mem`).

### Vmesnik

Ves vmesnik je izrisan iz nič (čisti Tkinter + Pygame, brez dodatnih paketov) in
oblikovan kot sodoben zaganjalnik iger:

- **Stranska vrstica s seznamom iger**: vsaka vrstica ima svoj **mini piktogram**
  v poudarni barvi igre, prikazuje trenutni **rekord (★)** in se odziva z gladko
  animiranimi učinki ob prehodu miške. Tekoča igra ostane obarvana; pri majhnih
  oknih se seznam **pomika** s kolescem miške.
- **Kartica stanja** spodaj levo z **LED-diodo stanja** (siva = meni, zelena =
  teče, zlata = pavza, rdeča = game over) in **prikazom FPS v živo**.
- **Zaslon v mirovanju** z aurorami, paralaksnim zvezdnim poljem z utrinki,
  lebdečim logotipom z iskrami v orbiti, **klikljivo mrežo iger** tik pod
  logotipom (vse igre z učinkom ob prehodu miške v svoji poudarni barvi) in
  **drsečim trakom rekordov**.
- **Učinki povsod**: mehki prehodi med zasloni, iskre ob potrditvi vnosa v meniju,
  **dež konfetov ob novem rekordu** in pravo **zamegljenje** za prekrivnim slojem
  pavze.
- **Predigralni zaslon** vsake igre se pojavi v njeni poudarni barvi in prejšnji
  rekord prikaže kot čip. Pri veliko načinih in majhni ločljivosti postane
  **kompakten**: Možnosti, Wiki in Nazaj se postavijo v eno vrsto, pisava pa se
  prilagodi - nič več ne sega čez rob slike.
- **Poenoten videz v igri**: vseh 46 iger si deli barvno paleto in pisavo menija -
  HUD-i, nastavitveni zasloni in prekrivni sloji sledijo izbranemu dizajnu v
  možnostih (v4.1 / v4 / Klasični), medtem ko vsako igrišče obdrži svoje
  identitetne barve. Vsaka igra zdaj čisto obvlada spremembo ločljivosti med igro,
  imena v meniju pa so odvisna od jezika (npr. »Schach« → »Chess« / »Šah«).
- **Vgrajeni wiki** (»LamaWiki«): podrobna pomoč za vsako igro (upravljanje,
  načini, točkovanje, nasveti) ter splošne strani - z **iskalnim poljem**,
  kategorijami, drsečimi članki in čipi tipk, v vseh 14 jezikih. Dostopen prek
  gumba **»Wiki / Pomoč«** v stranski vrstici in z predigralnega zaslona vsake
  igre (odpre neposredno njeno stran).
- **Dosežki in statistika**: **107 dosežkov** v treh kategorijah (23 ciljev na
  ravni zbirke, 37 točkovnih mejnikov in 47 posebnih trenutkov, kot so šah-mat
  UI, ploščica 4096, T-Spin Double, 25 rešenih šahovskih ugank, Killer Sudoku ali
  lamji jackpot; v 2048 in šahu partije z razveljavitvijo ali namigom ne štejejo) z **zlatim obvestilom in fanfaro** ob
  odklepanju - tudi sredi igre; stari rekordi se upoštevajo samodejno. Poleg
  tega zavihek **statistike**: skupni čas igranja, partije, zmage, rekordi,
  najljubša igra in tabela iger, razvrščena po času. Na voljo prek gumba
  **»Dosežki in statistika«** v stranski vrstici.
- **Posnetki**: minigolf in bowling snemata vsak krog. Na koncu **P** pokaže
  ponovitev, **S** pa jo shrani v arhiv - dosegljiv z gumbom **Posnetki** v
  stranski vrstici (zavihek na igro, pavza, skoki med sekvencami, hitrost 0,5x
  do 4x). Izklopljivo ob prvem zagonu in v možnostih.

### Upravljanje

- Izberi igro z gumbom v meniju na levi. Nato se pojavi **predigralni zaslon**:
  izberi **En igralec** ali **Več igralcev**, pojdi na **možnosti** ali nazaj.
  Puščice/miška za izbiro, enter zažene.
- **ESC** = pavza / nadaljuj (v menijih: nazaj).
- **F11** (ali gumb »Celozaslonsko vkl./izkl.«) = preklop celozaslonskega načina.
  Prikaz Pygame ostane vgrajen in se poveča z ohranitvijo razmerja stranic (črni
  robovi, kadar se razmerje razlikuje). Okno je mogoče prosto spreminjati po
  velikosti.
- **»Nazaj v meni«** konča igro in shrani rekord - enako velja za preklop na
  drugo igro prek stranske vrstice.
- **Stalne dodatne tipke**: poleg petih dodeljivih dejanj imajo nekatere igre
  svoje tipke (npr. shramba **C** in vrtenje v levo **Z** v Tetrisu,
  razveljavitev **U** v 2048, šahu in Sudokuju). Delujejo le, če tipka v
  možnostih ni dodeljena nobenemu dejanju, navedene pa so v namigu nastavitev in
  v wikiju. Držane tipke se pravilno zaznajo in ob pavzi ali Alt-Tab sprostijo -
  nič se ne »zatakne« več.
- **»Izhod«** čisto zapre Pygame in Tkinter.

### Možnosti, upravljanje in zvok

Zaslon možnosti se odpre z gumbom **»Možnosti / Upravljanje«** (na levi) ali s
predigralnega zaslona. Organiziran je v **tri zavihke** (**Splošno / Upravljanje /
Videz**; preklopiš s klikom ali s tipko Tab):

- **Splošno**: **zvok** vkl./izkl., **glasnost** in **haptika** (vibracija
  krmilnika, učinkovita le s priključenim krmilnikom) ter **samodejna ločljivost**,
  **ločljivost**, **FPS** in **jezik** – vsakega preklopiš z levo/desno.
- **Upravljanje**: **prednastavitve** (*WASD + puščice*, *WASD + IJKL*,
  *puščice + WASD*) in **preslikava vsake posamezne tipke** za igralca 1 in
  igralca 2: izberi vrstico, pritisni enter, pritisni želeno tipko (Esc prekliče).
- **Videz**: izberi **dizajn vmesnika** – **UI v4.2** (privzeto: Midnight Glass –
  globok polnočni preliv s počasi plavajočimi mehkimi lučmi v indigo, turkizni in
  magenta barvi, fino filmsko zrno, redke zvezde in plošče kot motno steklo s
  svetlobnim robom), **UI v4.1** (kot UI v4, a bolj živahno – nevpadljive zvezde
  ter Saturn in črna luknja v ozadju začetnega zaslona), **UI v4.1.1** (kot v4.1,
  a namesto zvezdnega neba tlakovan **cikcak vzorec** v črni in antracitni),
  **UI v4.1.2** (isti vzorec v modrih odtenkih palete – poudarna modra kot
  prevladujoča barva, temnejša modra kot podlaga), **UI v4.1.3** (isti vzorec v
  indigo barvi UI v4 na črni), **UI v4.1.4** (v grafitnem tonu UI v4 na črni),
  **UI v4** (povsem umirjen, ploski grafitni videz z enim samim indigo poudarkom),
  **UI v3** (prejšnji klasični vmesnik z zvezdnim nebom, aurorami in učinki
  žarenja), **UI v2** (prva prenova vmesnika: mornarsko moder preliv, zvezdno nebo
  in žareči gumbi, povsem brez animacij) ali **UI v1** (videz pred prenovo
  vmesnika: enobarvno temno ozadje, ploski gumbi, brez učinkov). Vse kartice
  prikažejo majhen predogled; izbira takoj učinkuje na ves vmesnik (igralno
  območje **in** stransko vrstico) in se shrani.

Nastavitve se trajno shranijo v `settings.json`. V načinu za **enega igralca** obe
preslikavi upravljata isti lik (privzeto: WASD *in* puščice), v **večigralskem**
vsaka svojega. Vse igre imajo **zvočne učinke** (proceduralno ustvarjene, brez
dodatnih datotek), ki jih je mogoče globalno utišati.

### Zgradba projekta

```
install-python.bat  Namestitev za Windows: Python 3.13 + .venv + pygame
start.bat            Zagonska skripta (Windows)
start.sh             Zagonska skripta (Linux / macOS / Git Bash)
pyinstall.bat        Gradnja EXE (Windows): vse zapakira v builds\PyGameZ.exe
main.py              Vmesnik Tkinter, vgradnja Pygame, osrednja igralna zanka
game_base.py         Osnovni razred iger (update/draw/handle_event) + InputEvent + pomočniki
settings.py          Nalaganje/shranjevanje nastavitev (zvok/haptika/tipke/možnosti iger s pravili preverjanja) (JSON)
audio.py             Proceduralni zvočni učinki, glasbene zanke + vibriranje igralne ploščice
menu.py              Zasloni za jezik, predigro (način) in možnosti (zvok/upravljanje)
highscore.py         Nalaganje/shranjevanje rekordov (razdelek v mem.json)
store.py             Osrednja datoteka shranjevanja mem.json (razdelki: mem, highscores, stats, achievements + napredek iger), atomsko z varnostno kopijo .bak
stats.py             Statistika igralca (partije, čas igranja, zmage, rekordi) na igro
achievements.py      Dosežki: definicije, odklepanje, obvestilo (toast)
progress.py          Zaslon dosežkov in statistike (dva zavihka, drsni)
replay.py            Snemanje in arhiv posnetkov (replay.json)
replayview.py        Zaslon posnetkov: seznam arhiva in predvajanje
ugc.py               Lastna vsebina (minigolf proge, stopnje Geometry Dash): shranjevanje, preverjanje, izvoz/uvoz (ugc.json)
swear.py             Besedni filter za imena in id (lang/swear/*.yml, vseh 14 jezikov)
filepick.py          Pogovorna okna za datoteke ("Izvozi kot ...", "Uvozi")
prestige.py          Sistem prestiža za Snake
competitive.py       Nastavitve za Tekmovalni način igre Snake (ravni, igralni avtomat, jabolka za stave)
ngb.py               Vizualna prilagoditev (»modi«): barva glave + koordinatna mreža + meni (mem-ngb.json)
lamabank.py          Lama banka: skupni račun žetonov za Blackjack, Poker in Casino (razdelek casino v mem.json)
seedrand.py          Generator naključnih števil z bitno enakimi števili v Pythonu in brskalniku (dnevni načini, nove uganke)
i18n.py              Prevajalski pogon (naloži lang/*.json, t("ključ"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Jezikovni nizi (en ključ na besedilo)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Seznami besednega filtra po jezikih (regex, .yml)
lamawiki/
  lamawiki.py          Vgrajeni wiki (iskanje, kategorije, izrisovalnik člankov)
  de.json  en.json  fr.json  es.json  pt.json   Vsebina wikija (ena stran na igro + splošne strani)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Znova zgradi sezname besed za Wordle (slovarji + seznami pogostosti)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 črk), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 črk), 14 jezikov
devtools/            Razvijalska orodja (ne pakirajo se v .exe)
  merge_staging.py           Prenese prevode in wiki strani iz devtools/staging/ v vseh 14 jezikovnih datotek
  build_chess_puzzles.py     Zgradi 200 šahovskih ugank iz baze ugank Lichess (CC0)
  build_sudoku_killer.py     Ustvari 400 Killer Sudokujev z enolično rešitvijo
  build_crossyroad_models.py Zapiše voksel modele Crossy Road za spletno različico
  build_geodash_levels.py    Zgradi 8 stopenj Geometry Dash in z reševalcem dokaže, da je vsaka rešljiva skupaj s kovanci
  build_geodash_solver.py    Reševalec s pravo kodo koraka (rešitve v geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Podatki o ravneh: snake-comp.json, chess-puzzles.json (+ README z viri), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Celovita revizija (vnos, seedrand Python = JS, shranjevanje, jezikovne datoteke, predigralni zasloni) + vsi audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless revizije za posamezne igre
  newgames_audit.py  blockjump_audit.py
```

Izbrani jezik se shrani v `mem.json` (v razdelku `mem`, poleg razdelka
`highscores` v isti datoteki) in se ob naslednjem zagonu samodejno naloži.

**Viri in licence:** 200 šahovskih ugank izvira iz
[baze ugank Lichess](https://database.lichess.org/#puzzles) (licenca
**CC0 1.0**, javna domena - hvala, lichess.org!); podrobnosti so v
`games/levels/chess-puzzles.README.md`. Viri seznamov besed za Wordle so navedeni
v `woordlistz/README.md`.

### Opombe glede platform

Prikaz teče **zunaj zaslona** (off-screen): pygame uporablja navidezni video
gonilnik (`SDL_VIDEODRIVER=dummy`), torej izrisuje v površino, vsak okvir pa je
kot slika narisan v gradnik Tkinter. **Nativnega okna SDL ni**, ki bi se s Tkinter
borilo za velikost/položaj. Zato se okno povsod obnaša enako in stabilno:

- **Windows**: proces je dodatno označen kot DPI-aware, da prikaz na skaliranih
  zaslonih (125/150/200 %) ostane oster in se ne »trese«.
- **Linux/X11 in Wayland**: deluje brez posebnih primerov (brez `SDL_WINDOWID`).
- **macOS**: prav tako deluje (prej se vgrajeno okno tu sploh ni prikazalo).

---

### Navodila za namestitev

Zahteva: **Python 3.9+** (priporočeno 3.12 ali 3.13) in **pygame ≥ 2.6**.

#### Windows (priporočeno: samodejno)

1. Odpri mapo projekta in dvoklikni **`install-python.bat`**.
   Skripta
   - preveri, ali je **Python 3.13** prisoten, sicer pa ga namesti prek
     **winget** (`winget install Python.Python.3.13`),
   - ustvari navidezno okolje **`.venv`**,
   - namesti **pygame** iz `requirements.txt`.
2. Nato zaženi zbirko z **`start.bat`** (dvoklik).

> Opomba: če skripta javi »še ni na voljo v tem oknu«, je bil Python pravkar
> nameščen – preprosto odpri **novo terminalsko okno** in znova zaženi
> `install-python.bat`. Če **winget** ni na voljo, Python 3.13 ročno namesti z
> <https://www.python.org/downloads/> in obkljukaj **»Add python.exe to PATH«**.

#### Windows / Linux / macOS (ročno)

```bash
# 1. Preveri Python (3.9+)
python --version

# 2. Ustvari in aktiviraj navidezno okolje
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Namesti odvisnosti
pip install -r requirements.txt
#   ali:  pip install "pygame>=2.6" (ali pygame-ce)
#                                   pip install pygame-ce
# 4. Zaženi
python main.py
```

#### Linux / macOS s start.sh

```bash
# Nastavi Python + venv kot zgoraj (koraka 2 in 3), nato:
chmod +x start.sh      # enkrat, če še ni izvedljiva
./start.sh
```

Na Linuxu Python po potrebi namesti prek upravitelja paketov, npr.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); na macOS
npr. `brew install python`.

#### Uporaba druge različice Pythona

`install-python.bat` privzeto nastavi Python 3.13. Če imaš raje 3.12 (ali drugo
različico), v datoteki spremeni vrstico `set "PYVER=3.13"` na želeno različico in
temu ustrezno ID winget (`Python.Python.3.12`).

#### Izdelava samostojnega EXE (Windows)

```bat
pyinstall.bat         :: zgradi builds\PyGameZ.exe (vse v eni datoteki)
```

`pyinstall.bat` uporabi `.venv` (in ga po potrebi ustvari), samodejno namesti
**PyInstaller** in celotno igro - Python, pygame, vse igre, jezike, wiki in
logotipe - zapakira v **eno samo `PyGameZ.exe`** v mapi **`builds\`**. Datoteka
teče na katerem koli računalniku Windows brez nameščenega Pythona in jo je mogoče
prosto kopirati. Nastavitve in rekordi (`settings.json`, `mem.json`,
`mem-ngb.json`) se med igranjem ustvarijo poleg datoteke .exe.

#### Odpravljanje težav

- **`pygame` ni najden** → je venv aktiviran? Ponovi korak 3
  (`pip install -r requirements.txt`).
- **`python` ni prepoznan (Windows)** → Python je bil nameščen brez »Add to
  PATH«; ponovno namesti in obkljukaj polje ali uporabi `py` namesto `python`.
- **Ni zvoka** → v možnostih preveri »Zvok«; haptika deluje le s krmilnikom.
- **Okno/vgradnja v Linuxu** → glej *Opombe glede platform* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ nazaj na vrh / back to top</a></b></div>

---

<a name="-hrvatski"></a>

## 🇭🇷 Hrvatski

Zbirka desktop igara u Pythonu: **Tkinter** pruža prozor i izbornik, a **Pygame**
je ugrađen kao prikaz igre unutar Tkinter prozora. Četrdeset i šest igara sa
zajedničkim opcijama, slobodno preslagivim upravljanjem, rekordima,
proceduralnim zvučnim efektima i, kod nekih naslova, višeigračkim modom. Sučelje
je **višejezično** – **14 jezika** (njemački / engleski / francuski / španjolski /
portugalski / poljski / turski / danski / norveški / švedski / finski / češki /
slovenski / hrvatski); jezik se bira na **pozdravnom zaslonu** pri prvom
pokretanju, koji ujedno omogućuje postavljanje **razlučivosti** i **zvuka** (po
zadanome isključen); osim triju glavnih jezika, svi ostali (španjolski,
portugalski i devet dodatnih) skriveni su iza gumba **„Više"**. Sve se kasnije
može promijeniti u opcijama.

### Brzi početak

#### Windows

```bat
install-python.bat    :: jednokratno: postavi Python 3.13 + .venv + pygame
start.bat             :: pokreni zbirku igara
```

#### Linux / macOS / Git Bash

```bash
./start.sh            # pokreće s .venv, inače sustavski python3
```

`start.bat` / `start.sh` automatski koriste virtualno okruženje `.venv` ako
postoji, inače sustavski Python. Detaljni vodič korak po korak nalazi se na samom
dnu pod **[Vodič za instalaciju](#vodič-za-instalaciju)**.

### Igre

| Igra         | Modovi          | Kratki opis |
|--------------|-----------------|-------------|
| **Snake**    | 1 / 2 igrača    | Deluxe Snake s 2D i 3D prikazom, turbo, 6 modova igre (uklj. Competitive), zlatne jabuke i prestiž |
| **Pong**     | 1 / 2 igrača    | Klasik protiv AI-ja ili igrača 2, preklopivi način kretanja |
| **Air Hockey** | 1 / 2 igrača  | 2D fizika s prijenosom impulsa, upravljanje mišem, AI i power-ups |
| **Tic-Tac-Toe** | 1 / 2 igrača | m,n,k igra na 3x3 do 9x9, tri razine AI-ja **ili** lokalno X protiv O |
| **Breakout** | 1 igrač         | Razbijanje cigli s vrstama cigli, power-ups, kombinacijama i mnogo razina |
| **Tetris**   | 1 / 2 igrača    | Moderna Guideline pravila (SRS, spremište, pregled 5 kocki, T-Spinovi): Maraton, Sprint 40, Ultra 2:00, Versus protiv AI-ja (3 razine) ili u dvoje s linijama smeća |
| **Invaders** | 1 igrač         | Space Invaders: očistite valove, zaštitite živote |
| **Asteroids** | 1 / 2 igrača   | Fizika inercije, valovi, NLO-i, power-ups, hiperprostor - solo ili kooperativni dvoboj |
| **Pac-Man**  | 1 igrač         | Vjeran klon: 4 AI-ja duhova, super-pilule, tunel, voće, razine |
| **Flappy Bird** | 1 igrač      | Gravitacijski let kroz cijevi, novčići, štit, dan/noć, medalje |
| **Doodle Jump** | 1 igrač      | Automatski skok prema gore, vrste platformi, opruge, propeler, čudovišta |
| **2048**     | 1 igrač         | Slagalica s klizanjem brojeva od 3x3 do 8x8: Klasično, Utrka s vremenom i Beskonačno, poništavanje, tečne animacije, spremljene partije |
| **Minesweeper** | 1 igrač      | Klasik sa sigurnim prvim klikom, chordingom, smileyjem i najboljim vremenima |
| **Sudoku**      | 1 igrač      | 4 varijante (Klasični, X-sudoku, Killer, Mini 6x6) s po 400 razina, Sudoku dana, do 3 zvjezdice po razini, 4 moda pomoći, poništavanje, spremljena igra |
| **Frogger**     | 1 igrač      | Cesta + rijeka + 5 uvala, bonus muha, krokodili, vremensko ograničenje, 3 težine |
| **Memory**      | 1 / 2 igrača | Pronađite parove na 4x4 do 8x6, animacija okretanja, solo bodovanje ili dvoboj |
| **Pasijans**    | 1 igrač      | 5 varijanti (Klondike, Spider, FreeCell, Piramida, TriPeaks) s povuci-i-ispusti i poništavanjem |
| **Aim Trainer** | 1 igrač      | Opušteno 3D gađanje meta: miš upravlja kamerom, 4 moda (preciznost/refleks/pokretne/chill), 3 teme uklj. crnu rupu |
| **Četiri u nizu** | 1 / 2 igrača | Klasik s animacijom padanja žetona: 3 razine AI-ja (minimax) ili lokalni dvoboj |
| **Tenkovski dvoboj** | 1 / 2 igrača | 2D dvoboj u areni s rikošetnim hicima, power-ups, 4 arene, AI s 3 razine |
| **Blackjack**    | 1 igrač     | Kasino blackjack sa shoeom od 4 špila, udvostručavanje/dijeljenje i blackjack 3:2; igra se lama žetonima zajedničke Lama banke |
| **Tunnel Racer** | 1 igrač     | Let kroz 3D neonsku cijev: beskonačni mod + 30 razina, upravljanje tipkama ili mišem, motion blur |
| **3D labirint**  | 1 igrač     | Raycaster iz prvog lica (stil Wolfensteina) s 50 razina sa sjemenom, orbovi, minimapa - ili 2D pogled odozgo |
| **Reversi**      | 1 / 2 igrača | Othello na 8x8: zarobite i okrenite žetone, 3 razine AI-ja (minimax) ili lokalni dvoboj |
| **Yahtzee**      | 1 / 2 igrača | Klasik s kockicama s 13 kategorija, gornji bonus i Yahtzee; lov na rekord ili hotseat za 2 igrača |
| **Wordle**       | 1 igrač     | Pogađanje riječi od 4 do 7 slova: Beskonačno, Riječ dana, Dordle i Quordle, teški način, paleta za daltoniste, statistika s grafikonom, dijeljenje rezultata, pravi popisi riječi na 14 jezika |
| **T-Rex Runner** | 1 igrač     | Beskonačna pustinjska trka: promjenjiv skok, saginjanje, kaktusi i pterodaktili, izmjena dan/noć, rastuća brzina, 3 težine |
| **Dame**         | 1 / 2 igrača | 3 skupa pravila (njemačke 8×8, međunarodne 10×10, checkers), obavezno uzimanje i leteća dama, 3 razine AI-ja (minimax) ili lokalni dvoboj |
| **Poker**        | 1 igrač     | 3 varijante po izboru: Texas Hold'em protiv AI-ja, 5 Card Draw i Video Poker; runde klađenja, blindovi, lama žetoni zajedničke Lama banke |
| **Šah**          | 1 / 2 igrača | Potpuna pravila, Chess960 i šahovski sat, 6 razina AI-ja, 200 zadataka iz Lichess baze, poništavanje/savjet, popis poteza, izvoz PGN-a ili lokalni dvoboj |
| **Mlin**         | 1 / 2 igrača | Faze postavljanja/pomicanja/letenja, mlinovi i uzimanja, opcionalno pravilo letenja, 3 razine AI-ja ili lokalni dvoboj |
| **Simon**        | 1 / 2 igrača | Senso igra pamćenja: modovi Klasično/Speed/Reverse/Miješano + Dvoboj, zvuk isklj./uklj./miješan, 4/6/9 polja, najbolji po modu |
| **Biljar**       | 1 / 2 igrača | 8-ball, 9-ball i vježba u 2D-u, fiksni 3D prikaz ili slobodno rotirajuća 3D kamera; glatka fizika, pomoć pri ciljanju, 3 razine AI-ja |
| **Klizna slagalica** | 1 igrač | Igra 15 u 3x3/4x4/5x5: klizite numerirane pločice u prazninu, upravljanje klikom ili strelicama, bodovi prema potezima i vremenu |
| **Mastermind**     | 1 igrač   | Razbijte tajni kôd boja (3 moda: 4×6, klasično, 5×8), crni/bijeli čavlići povratne informacije, beskonačni niz kao rekord |
| **Bubble Shooter** | 1 igrač   | Klon Puzzle Bobblea: ispaljujte jednake boje u skupine od tri, odbijanje od zidova, padajuće skupine, 3 težine |
| **Vješala**        | 1 igrač   | Pogodite riječ prije nego što se dovrši vješalo; zaslonska tipkovnica, popisi riječi po jeziku, 3 moda duljine, beskonačni niz |
| **Block Jump**  | 1 igrač        | 3D platformer u stilu Minecrafta: teksturirani voxel svijet sa Steveom, ljestvama, ogradama i sluzavim blokovima, kamera iz prvog/trećeg lica, motion blur, parkour razine generirane sjemenom |
| **Tower Defense** | 1 igrač        | Odbijaj beskonačne valove na 4 karte: do 11 vrsta tornjeva s nadogradnjama, prodajom i specijalizacijom A/B, bossovi, 3 načina, aktivne sposobnosti |
| **Minigolf**    | 1 / 2 igrača | 360 staza na 40 terena (18 ručnih, 342 generirane): pijesak, rampe, voda, odbojnici, vjetrenjače i lutajući blokovi; kartica s parom i bonusom za hole in one; **vlastiti uređivač staza** s 15 vrsta objekata, 12 predložaka i dijeljenjem kao `.lamapgzmap` |
| **Pinball**     | 1 / 2 igrača | Pinball automat s 3 stola: odbojnici, slingshotovi, mete, staze L-A-M-A, multiball s jackpotom, spašavanje kugle, gurkanje i tilt |
| **Bowling**     | 1 / 2 igrača | 10 frameova sa službenim brojanjem strike/spare, pravom fizikom čunjeva, hookom i stazom u perspektivi, 3 težine |
| **Crossy Road** | 1 igrač       | Beskonačno skakanje preko livada, cesta, rijeka i tračnica u izometrijskom voxel stilu: dan/noć, orao, 10 likova za kupnju, dnevna staza |
| **Geometry Dash** | 1 igrač     | Ritmička platformska igra s kockom, brodom, loptom, NLO-om i valom: 8 razina od Lako do Demon s po 3 tajna novčića, način vježbe, glazba za svaku razinu; **uređivač razina** s dijeljenjem kao `.lamapgzlevel` |
| **Battleship**  | 1 / 2 igrača  | Pomorska bitka 10x10: raspoređivanje flote povlačenjem, 3 prekidača pravila (dodir, salva, ponovni hitac), AI s 3 razine ili lokalni dvoboj sa zaslonom za predaju |
| **Casino**      | 1 igrač       | Europski rulet sa svim klasičnim ulozima i Lama automat (5 valjaka, 10 dobitnih linija, džoker, besplatne vrtnje); jedan račun lama žetona s Blackjackom i Pokerom |

**Više igrača (2 igrača lokalno)** dostupno je za **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (kooperativni
dvoboj)**, **Memory (dvoboj)**, **Četiri u nizu**, **Tenkovski dvoboj**,
**Reversi**, **Yahtzee**, **Dame**, **Šah**, **Mlin**, **Simon (dvoboj)**,
**Biljar**, **Minigolf**, **Pinball**, **Bowling** i **Battleship** (sa zaslonom za
predaju koji skriva flote) - ukupno 20 igara. Mod se bira izravno na zaslonu
pripreme (*Jedan igrač / Više igrača*); Tetris uz to nudi **Versus protiv AI-ja**.
Web-inačica je samo za jednog igrača.

#### Detalji značajki po igri

**Snake**
- **NOVO - 3D prikaz** (tipka **V** u setupu ili klik na *Prikaz*): ploča se
  iscrtava kao 3D scena u stvarnom vremenu - **prateća kamera** lebdi iza zmije,
  a upravljanje je **relativno u odnosu na pogled** (lijevo/desno = skretanje,
  dva brza pritiska = okret za 180°). Uz maglu udaljenosti, zvjezdano nebo,
  šahovnicu na podu, rubne zidove, rotirajuće kristale hrane, 3D čestice i
  trešnju kamere pri sudaru; nakon game overa kamera polako kruži oko zmije.
  Turbo širi vidno polje. Dostupno u 3D-u: *Klasično* i *Prepreke* (ondje su
  zidovi uvijek čvrsti, 3D postoji samo za jednog igrača). Prikaz se pamti u
  `settings.json`.
- **NOVO - Opcije 3D kamere** (u 3D setupu kliknite redak *3D kamera / smooth
  shake*, ili tipka **K**): zaseban izbornik sa **smooth shakeom** (blaža
  kamera, znatno manje podrhtavanja pri kretanju/skretanju), podesivim **vidnim
  poljem (FOV)** i **visinom kamere**, uz prekidač **trešnja pri skretanju**
  (uključivanje/isključivanje trešnje ekrana pri skretanju lijevo/desno). Sve se
  pamti u `settings.json`.
- **Turbo**: **držite** tipku turba = turbo (dvostruka brzina), troši
  izdržljivost (traka); kad se isprazni, turbo se isključuje i ponovno se puni.
  Zadano I1 = razmaknica/lijevi Shift, I2 = Enter/desni Shift.
- **6 modova igre** (odabir u setupu): *Klasično*, *Speed Rush* (ubrzava sa
  svakom jabukom), *Prepreke* (smrtonosni blokovi), *Portali* (parovi
  teleportera), *Utrka s vremenom* (60 sekundi, što više jabuka) i *Competitive*
  (vidi dolje).
- **NOVO - Competitive** (jedan igrač): beskonačni mod s **usponom po razinama** -
  počinjete s točno **jednom** jabukom i isprva ih ne možete imati više; što više
  jabuka ukupno skupite, to je viša vaša **razina**, koja neprekidno dodaje još
  jednu istovremenu jabuku na polje i podiže množitelj bodova. **Plave jabuke**
  otvaraju **slot machine**: ulog je vaša duljina, rezultat na valjcima je množi
  ili smanjuje i nakratko izaziva pojavu **dodatnih jabuka** (jackpot uz tri
  jednaka simbola). **Ljubičaste jabuke** (kockanje) stavljaju dio vaše
  **veličine** na kocku i taj dio nasumično množe, dok ostatak ostaje siguran
  (nova veličina = veličina·(1-p) + veličina·p·faktor): **normalno** ulaže
  fiksnih 50 % uz **x0.5 .. x1.5**, a **HARDCORE** je rizičniji s ulogom od
  **75-90 %** i **x0.25 .. x2.25**. Vaša **veličina** prikazuje se kao
  **decimalni broj gore lijevo** i prenosi se točno, pa se sljedeće oklade
  nadovezuju na nju. Postoji **15 razina** (množitelj do x16, do 16 jabuka
  istovremeno); razine se nalaze u `games/levels/snake-comp.json` i ondje se mogu
  proširiti bez diranja koda, dok ostatak ugađanja živi u `competitive.py`.
- **NOVO - HARDCORE** (prekidač u Competitive setupu, tipka **H**): svaki
  **turbo jede duljinu** vaše zmije; crveni užareni **natpis HARDCORE** označava
  taj mod. Samo u Competitiveu; duljina nikad ne pada ispod minimuma. Pamti se u
  `settings.json`.
- **Zlatne jabuke** (privremene) daju puno bodova i istog trena napune turbo.
- Opcionalno **prolazni zidovi**, bonus jabuke, **prestiž** (jedan igrač, tipka **P**).
- **NOVO - Personalizacija** (gumb s kistom u samom gornjem desnom kutu setupa,
  ili tipka **C**): isključivo vizualni izbornik („modovi" koji *nikad* ne
  mijenjaju igru) s dvije kartice:
  - **Glava**: **boja glave** zmije - 4 plavo-tirkizne predloške (od više plave do
    više tirkizne), crvena, narančasta i **vlastita boja** preko RGB klizača.
  - **Mreža (putokaz)**: prekriva polje **koordinatnom mrežom** - **brojevi
    redaka** (na lijevom i desnom rubu) i **slova stupaca** (gore/dolje). Tako na
    velikim pločama odmah vidite da je npr. jabuka na *8a* u istom retku *8* kao i
    vaš položaj *8z*. Slijed boja (5 predložaka + dvije vlastite boje A/B)
    određuje temu boja.
  - **Baner**: uključite/isključite baner množitelja (npr. od ljubičaste jabuke)
    te podesite njegovu **veličinu** (manje/veće) i **prozirnost** (prozirnije) -
    uz pretpregled uživo.
  Sve se sprema u `mem-ngb.json`; sva vizualna personalizacija ide kroz modul
  `ngb.py`.
- Izgled: zaobljena zmija s očima (glava je po zadanome tirkizna), sjaj turba,
  čestice.

**Pong**
- Jedan igrač protiv AI-ja, više igrača = igrač 2 desno. Do 5 bodova.
- **Način kretanja koji se prebacuje po skupu tipki**: *Neprekidno* (pritisnete
  jednom -> nastavlja se kretati, zadano) ili *Držanje* (kreće se samo dok
  držite). Prebacivanje: **X** = skup tipki 1, **N** = skup tipki 2 (pamti se u
  `settings.json`).
- Fizika lopte s ubrzanjem i kutom ovisno o točki udara.

**Air Hockey**
- **Prava 2D fizika**: okrugli udarači i pak s prijenosom impulsa - pak preuzima
  brzinu udarača pri udaru; bande s restitucijom, blago trenje leda, golovi kao
  otvori u bočnim zidovima.
- **Upravljanje mišem** za jednog igrača: udarač prati miš (svaka tipka vraća na
  tipkovnicu). Tipkovnica: tipke smjera u 8 smjerova, više igrača = I1 lijevo
  (WASD), I2 desno (IJKL).
- **AI s tri razine** (Lako/Srednje/Teško): brani vlastiti gol, napada u vlastitoj
  polovici i zaobilazi pak kako bi izbjegao autogolove.
- **Power-ups** (mogu se isključiti): *XL* (veći udarač), *GOL* (protivnikov gol
  se smanjuje), *>>* (brži udarač) - pripadaju igraču koji je zadnji dodirnuo pak.
- Setup: težina, **golovi do pobjede** (3/5/7/10), power-ups uklj./isklj.
  (spremljeno u `settings.json`). Nakon svakog gola servira onaj tko ga je primio.
- Izgled: svjetlosni trag paka, čestice, pulsirajuća usta golova, oznake efekata.

**Tic-Tac-Toe**
- Setup: težina (Lako/Srednje/Teško) i veličina ploče 3x3..9x9; pobjednička
  duljina K = 3 (3x3), 4 (4x4), inače 5.
- **1 igrač** protiv AI-ja (Teško na 3x3 je nepobjedivo) **ili 2 igrača** lokalno
  (X protiv O, naizmjenično klikom). Na game overu: Enter/klik = nova runda,
  **S** = postavke.

**Breakout**
- Vrste cigli: Normalna, **Čelik** (neuništiva), **Bomba** (eksplodira), **Zlato**
  (dodatni bodovi).
- Power-ups: laser, vatrena kugla, ljepljiva, štit, novčić i drugo; **množitelj
  kombinacija**.
- Efekti: čestice, tragovi lopte, trešnja ekrana, skočni bodovi, mnogo uzoraka
  razina.
- Setup: **1/2/3** = težina, **Lijevo/Desno** = boja lopte, **Gore/Dolje** =
  početna razina, **M** = raspored. Igra: miš/strelice, **razmaknica** lansira
  loptu (ispaljuje laser), **P/Esc** = pauza.

**Tetris**
- **Moderna Guideline pravila**: polje 10x20, kocke iz **vrećice od 7**, **SRS
  sustav rotacije** sa stvarnim wall kickovima (i za kocku I), okretanje u oba
  smjera, **spremište** (jednom po kocki), **pregled 5 kocki**, sjena kocke i
  **lock delay** (0,5 s, najviše 15 resetiranja).
- **Tri načina** na zaslonu pripreme: *Solo*, *Protiv AI* i *2 igrača*. Solo u
  postavkama nudi **Maraton** (početna razina 1–15, broji se za rekord), **Sprint
  40 linija** (najbolje vrijeme) i **Ultra 2 minute** (najbolji rezultat); najbolji
  rezultati spremaju se u odjeljak `tetris` u `mem.json`.
- **Bodovanje po Guidelineu**: od singlea do Tetrisa, **T-Spinovi** (puni i mini),
  **Back-to-Back** (x1,5), **kombinacije** i **Perfect Clear** - s natpisima na
  zaslonu, animacijom brisanja linija, tragom hard dropa, česticama i efektom
  prelaska na višu razinu.
- **Versus s linijama smeća**: očišćene linije šalju smeće protivniku (Tetris = 4,
  T-Spin Double = 4 …), dolazno smeće najavljuje traka upozorenja, a vlastiti
  napadi ga **poništavaju**; oba polja dobivaju isti redoslijed kocki. **AI** ima
  3 razine, a tempo raste svakih 40 sekundi.
- **Upravljanje**: Lijevo/Desno s vlastitim **DAS/ARR** (podesivo u postavkama),
  Gore = okretanje udesno, Dolje = soft drop, Akcija = hard drop; **C**/Shift =
  spremište, **Z**/**Y** = okretanje ulijevo, **X** = okretanje udesno. U dvoje
  igrač 1 sprema tipkom **Q** i okreće ulijevo tipkom **E**, igrač 2 **desnim
  Shiftom** / **desnim Ctrlom**. Nakon igre: **R** = ponovno, **S** = postavke.

**Invaders** – dva moda (odabir na zaslonu pripreme):
- **Klasično**: klasični blok vanzemaljaca; zatim se u setup zaslonu bira:
  **Kretanje** (samo lijevo/desno *ili* slobodno s WASD) i **Ciljanje** (uvijek
  prema gore *ili* prema **mišu** – tada pucate ondje gdje je pokazivač). Uništeni
  vanzemaljci ponekad ispuštaju power-ups.
- **Arena (slobodno)**: slobodno kretanje u svim smjerovima, neprijatelji nadiru
  sa svih rubova; ciljate u smjeru kretanja, oružje mijenjate s **1–4**.
Zajedničko: sustav razina s **bossom** na svakoj 4. razini, četiri oružja
(blaster, rasprsni hitac, brza paljba, laser), power-ups (dodatni život, štit,
nadogradnja oružja), efekti eksplozije, rekord.

**Asteroids**
- **Fizika inercije**: gore = potisak u smjeru gledanja, lijevo/desno = rotacija,
  brod nastavlja plutati (blago prigušenje); sve prelazi preko rubova ekrana.
  Klasičan **vektorski izgled** s plamenom potiska i zvjezdanim nebom; svaka
  stijena ima vlastiti nasumični poligon.
- Stijene se raspadaju na dvije manje (3 veličine, **20/50/100 bodova**),
  **valovi** s rastućim brojem i najavom u baneru.
- **NLO** (može se isključiti): povremeno prelazi ekran i cilja na brodove
  (pogreška u ciljanju ovisi o težini) - 200 bodova za obaranje.
- **Power-ups** (mogu se isključiti), ispadaju iz uništenih stijena: **S** = štit
  (6 s neranjivosti), **T** = trostruki hitac, **R** = brza paljba.
- **Hiperprostor** (tipka dolje): hitni skok na nasumičan položaj uz 4 s hlađenja
  - i 12 % rizika da se pritom razbijete.
- 3 života, sigurno oživljavanje uz treptanje neranjivosti, **dodatni život
  svakih 5000 bodova**; čestice eksplozije i trešnja kamere.
- **Kooperativni dvoboj** (više igrača): oba broda lete istovremeno, s odvojenim
  životima i bodovima - pobjeđuje onaj s više bodova.
- Setup: težina, NLO-i uklj./isklj., power-ups uklj./isklj. (spremljeno u
  `settings.json`).

**Pac-Man**
- **Klasični labirint 28x31** u neonskom izgledu s kuglicama, 4 super-pilule,
  bočnim tunelima za teleport i kućicom duhova u sredini.
- **Četiri duha s izvornim ponašanjima** (AI ciljne pločice): *Blinky* juri
  izravno, *Pinky* postavlja zasjedu (4 pločice ispred), *Inky* koristi vektor
  kroz Blinkyja, *Clyde* se povlači kad je preblizu.
- **Faze scatter/chase** izmjenjuju se (duhovi se okreću pri svakoj promjeni);
  **super-pilula** čini duhove plavima i jestivima (lanac 200/400/800/1600),
  nakon čega im se oči vraćaju u kućicu.
- Kućica duhova s **postupnim izlaskom**, **voće** kao bonus (po razini),
  **3 života**, **dodatni život na 10.000**, sustav razina (ubrzava), animacija
  smrti, zasloni READY/GAME OVER.
- Setup: **težina** (Normalno/Teško/Ekstremno) – brzina duhova i trajanje straha.
- Upravljanje: **strelice ili WASD**.  Enter = novo, S = setup.

**Flappy Bird**
- **Fizika gravitacije**: razmaknica / Gore / W / **klik miša** tjera pticu da
  zamahne krilima; naginje se ovisno o brzini uspona/pada.
- Beskonačni **parovi cijevi** s prolazom (+1 po cijevi); **novčići** (bonus) i
  power-up **štit** (preživi jedan sudar) pojavljuju se u prolazima.
- **Dnevne/noćne teme** mijenjaju se s rezultatom; oblaci koji plutaju (parallax),
  pomično tlo.
- Težina (Lako/Normalno/Teško): veličina prolaza, brzina, razmak cijevi – prolaz
  se malo sužava kako rezultat raste.
- **Medalje** (bronca/srebro/zlato/platina) na game overu, animacija sudara uz
  trešnju kamere, rekord.

**Doodle Jump**
- Doodler **automatski skače** pri doskoku; upravljate samo lijevo/desno (uz
  inerciju), rubovi se spajaju, a kamera se pomiče prema gore kako se penjete.
- **Vrste platformi**: zelena (normalna), plava (pomična), smeđa (lomi se), bijela
  (nestaje). **Opruge** daju super odskok, a **šešir s propelerom** nakratko vas
  nosi prema gore (i čini vas neranjivim).
- **Čudovišta**: dodir je smrtonosan – ali ih možete **ustrijeliti** s Gore /
  razmaknica (dodatni bodovi).
- Rezultat = dosegnuta visina; težina raste s visinom. Rekord.
- Upravljanje: lijevo/desno = kretanje, Gore / razmaknica = pucanje.

**2048**
- **Vlastiti zaslon postavki** s pločama od **3x3 do 8x8** i tri načina:
  *Klasično* (cilj 2048, zatim „Nastaviti igru?"), *Utrka s vremenom* (3 minute,
  sat kreće s prvim potezom) i *Beskonačno*.
- **Tečne animacije**: pločice klize, spajaju se uz efekt „pop" i izrastaju;
  skočni bodovi, iskre od 128, udarni val od 2048 i nove boje sve do 131072. Unosi
  tijekom animacije spremaju se u red.
- **Poništavanje** (isključeno / 3 po igri / neograničeno, tipka **U** ili
  Backspace) - tko ga koristi, igra bez rekorda i bez postignuća za pločice.
- **Spremi i nastavi**: partija u tijeku automatski se sprema za svaku veličinu i
  način; najbolji rezultati i najveća pločica po veličini/načinu nalaze se u
  odjeljku `g2048` u `mem.json`.
- Upravljanje: strelice/WASD ili **povlačenje** mišem/touchpadom, **R**/**N** = nova
  igra, **Tab** = postavke. Rekord se računa samo u **4x4 Klasično** bez
  poništavanja.

**Minesweeper**
- Tri razine: **Početnik** (9x9, 10 mina), **Napredni** (16x16, 40), **Stručnjak**
  (30x16, 99) - **najbolje vrijeme po razini** sprema se i prikazuje u setupu.
- **Prvi klik je uvijek siguran** (mine se raspoređuju tek nakon toga, područje
  3x3 oko klika ostaje prazno).
- **Lijevi klik** = otkrivanje, **desni klik** = zastavica (opcionalno s ciklusom
  upitnika), **F** = zastavica pod pokazivačem, **R** = nova igra.
- **Chording**: klik na zadovoljen broj otkriva preostale susjede.
- Klasičan HUD: brojač mina, **smiley na klik** (iznenađen/sunčane naočale/mrtav),
  štoperica; pogrešne zastavice na kraju se precrtavaju, konfeti pri pobjedi.
- Bodovi = osnovna vrijednost razine minus sekunde.

**Sudoku**
- **4 varijante** s po **400 razina** (4 težine x 100): *Klasični* (poznate razine
  sa sjemenom - riješene ostaju označene), *X-sudoku* (obje dijagonale sadrže
  svaku znamenku točno jednom), *Killer* (isprekidani kavezi sa zbrojem; 400
  unaprijed generiranih razina) i *Mini 6x6*. Svaka zagonetka ima **jedinstveno
  rješenje** - razina 12 „Teško" ista je zagonetka na svakom računalu.
- **Sudoku dana**: jedna zagonetka dnevno za sve, ista na računalu i u
  pregledniku; težina ovisi o danu u tjednu (od ponedjeljka Lako do subote
  Stručnjak), a svakodnevnim rješavanjem gradi se niz.
- **Do 3 zvjezdice po razini** (riješeno · bez pogrešaka i savjeta · uz to ispod
  ciljanog vremena) i **najbolje vrijeme** u izboru razina; **započete zagonetke**
  automatski se spremaju i nastavljaju sljedeći put.
- **4 moda igre** (biraju se prije početka) s množiteljem bodova: **Klasično**
  (x2.0 - bez pomoći), **Bilješke** (x1.5 - + olovkom bilješke i automatski
  kandidati), **Udobnost** (x1.0 - + pogrešne znamenke crveno, označeni sukobi i
  pogrešni zbrojevi kaveza, točni unosi se zaključavaju), **Asistent** (x0.7 - +
  tipka za savjet, maks. 3). S uključenim **ograničenjem od 3 pogreške** (opcija u
  setupu) treća pogreška završava partiju.
- Upravljanje: strelice/WASD = ćelija, **1-9** = znamenka (i numerička
  tipkovnica), **0/Delete/desni klik** = brisanje, **U**/**Z** = poništi, **Y** =
  ponovi, **N** = bilješke, **C** = automatski upiši kandidate, **H** = savjet,
  **M** = oznaka bojom, **R** = ponovno pokreni razinu, **Q** = izbor razina. Unos
  „najprije znamenka" (setup, **I**) i **brojač preostalih znamenki** ispod svake
  znamenke; potpuno igrivo mišem. Nakon kraja partije **A** otkriva cjelovito
  **rješenje**.
- Bodovi = (osnova varijante i težine - vrijeme - pogreške - savjeti) x množitelj
  moda; sve varijante i Sudoku dana računaju se za rekord.

**Frogger**
- 5 prometnih traka (auti/kamioni) i 5 riječnih traka (trupci, kornjače koje
  **rone** na višim razinama); na vrhu 5 ciljnih uvala - napunite sve = sljedeća
  razina, sve ubrzava.
- Dodaci: **bonus muha** (+200) u praznim uvalama, **krokodili** zauzimaju uvale
  na višim razinama, **traka vremenskog ograničenja** po žabi, dodatni život na
  10.000.
- 3 težine (brzina, gustoća prometa, vrijeme); bodovi po novom retku, uvala = 50 +
  vremenski bonus, razina dovršena = +1000.

**Memory**
- Veličine ploče **4x4, 6x6, 8x6**; motivi su kombinacije oblika i boja, u
  potpunosti nacrtani primitivima; **animacija okretanja**, nesparene karte
  automatski se okreću natrag.
- **Solo**: osnova - 15 po potezu - 2 po sekundi (min. 100). **Dvoboj** (lokalno):
  naizmjenični potezi, pogodak = ponovni potez, pobjeđuje onaj s najviše parova.

**Pasijans**
- **5 varijanti** na zaslonu pripreme: Klondike (opcija vučenja 1/3), Spider
  (1/2/4 boje), FreeCell (ograničenje super-poteza), Piramida (parovi do 13,
  2 ponovna dijeljenja) i TriPeaks (lanac ±1 s množiteljem kombinacija).
- **Povuci i ispusti** ili klik-klik, **desni klik** = na temelj, **U** =
  neograničeno poništavanje, **R** = novo dijeljenje, razmaknica = špil.
- Karte se iscrtavaju bez slikovnih datoteka (`games/cards.py`); sve varijante
  dijele jednu ljestvicu rekorda s formulama specifičnima za varijantu.

**Aim Trainer**
- **Pravi softverski 3D** (kao Snakeov 3D mod): fiksni nišan u središtu ekrana,
  **izravno gledanje mišem 1:1 kao u pucačini** (hvatanje pokazivača: kursor je
  zarobljen unutar prozora, Esc ga otpušta; podesiva osjetljivost, neograničen
  yaw, pitch ±60°). Lijevi klik puca točno kroz središte, uz bljesak cijevi,
  tracer i čestice pogotka.
- **4 moda**: preciznost (60 s, 3 kugle, bonus na točnost), refleks (30
  pojedinačnih meta, statistika reakcijskog vremena), pokretne mete (putanje +
  množitelj kombinacija do x4) i chill (beskonačno, bez kazne, **E** završava
  sesiju).
- **3 teme** (u setupu, spremaju se): **svemir** sa sferom zvijezda, **crnom
  rupom sa svjetlećim prstenom** i planetom (zadano), neonska arena s mrežom na
  podu i synthwave suncem, te zatvoreno strelište.
- Osjetljivost se može mijenjati i usred igre s **+/-**; uz to **podesivi motion
  blur** (0-80%) za dodatno opušten izgled - oboje se sprema.

**Četiri u nizu**
- Ploča 7x6 s **animacijom padajućeg žetona**, pretpregledom pri prijelazu mišem i
  pulsirajućom pobjedničkom linijom; miš, strelice ili izravan odabir **1-7**.
- **3 razine AI-ja** (minimax s alfa-beta pretragom): Lako namjerno previđa
  prijetnje, Srednje pouzdano blokira, Teško planira duboko unaprijed - ili
  **2 igrača** lokalno na istom uređaju.
- Igrač koji počinje mijenja se svaku rundu; rekord broji vaše **pobjede protiv
  AI-ja** u jednoj sesiji.

**Tenkovski dvoboj**
- Dvoboj u 2D areni: **hici se jednom odbijaju od zidova** (rikošet) - pogađate iza
  uglova (ili sami sebe!). Prvi do 5 rundi uz odbrojavanje.
- **4 arene** (Otvorena, Križ, Stupovi, Labirint) ili nasumična rotacija;
  **power-ups**: brza paljba, štit, trostruki hitac.
- **AI s 3 razine** - teški gađa s predviđanjem i namjerno odbija hice od zidova -
  ili **2 igrača** na jednoj tipkovnici (I1 WASD+razmaknica, I2 strelice+Enter).

**Blackjack**
- Prava kasino pravila: **shoe od 4 špila**, djelitelj staje na 17, **blackjack
  plaća 3:2**, djelitelj viri kod asa/10; **udvostručavanje** i **jedno dijeljenje**
  (razdvojeni asovi dobivaju po jednu kartu).
- **Lama žetoni**: Blackjack igra s računom **Lama banke**, koji dijeli s Pokerom i
  Casinom (početak 1000, trajno spremljen u `mem.json`). Ulog se skida odmah pri
  dijeljenju; ispod 10 žetona Enter uzima **bankovni kredit** koji račun vraća na
  1000.
- **Rekord** = najviše stanje vaše **bilance u Blackjacku** (1000 plus sve što je
  u Blackjacku dobiveno i izgubljeno) - dobici na ruletu, automatu ili u pokeru
  ovdje se ne računaju, a ni krediti.
- Igra se preko gumba sa žetonima i tipki (**H**it/**S**tand/**D**ouble/dijeljenje
  **X**, **1-4** = ulog, Backspace = poništi ulog, Enter = dijeli) uz animacije
  karata; skrivena karta djelitelja pri otkrivanju sada se zaista okreće.

**Tunnel Racer**
- **Let kroz 3D neonsku cijev** (softverski renderer kao Aim Trainer): grede,
  blokovi i **prstenasti prolazi kroz koje se provlačite**, novčići na idealnoj
  liniji.
- **Dva moda**: beskonačni (brzina raste do gornje granice, rekord) i **30 razina
  generiranih sjemenom** s ciljem, vremenskim bonusom i označenim napretkom.
- **Upravljanje tipkama** (zadano) ili **izravno upravljanje mišem** (hvatanje
  pokazivača, tipka **C**); uz podesivi **motion blur** (tipka **B**, 0-80%) -
  sve se sprema.

**3D labirint**
- **Raycaster iz prvog lica u stilu Wolfensteina** (DDA, magla udaljenosti,
  spriteovi) uz mouselook + WASD, **minimapu** (tipka **M**) i zeleni pulsirajući
  izlaz - ili klasičan **2D pogled odozgo** (tipka **V** u setupu).
- **50 razina generiranih sjemenom** koje neprestano rastu; izlaz je uvijek na
  točki najudaljenijoj od početka, **orbovi** usput daju dodatne bodove.
- Bodovanje: 500 po razini + 100 po orbu + vremenski bonus; riješene razine se
  označavaju, a zbroj sesije postaje rekord.

**Reversi**
- **Othello na 8x8**: postavljate žetone koji zarobljavaju protivnikove redove i
  okrećete sve što je zatvoreno; nedopušteni potezi su blokirani, a potez bez
  ijednog legalnog poteza **automatski se preskače**.
- **Jedan igrač protiv AI-ja** (3 razine: negamax s alfa-betom, pozicijsko
  vrednovanje + pokretljivost) **ili lokalni dvoboj**, Crni protiv Bijelog.
- Legalna polja su istaknuta; igrate **mišem** ili okvirom za odabir (strelice +
  razmaknica/Enter). Svaka pobjeda protiv AI-ja vrijedi jedan bod za rekord.

**Yahtzee**
- **Klasik s kockicama**: 5 kockica, do 3 bacanja po potezu, kockice se
  **zadržavaju** pojedinačno, zatim upišete jednu od **13 kategorija** (uz živi
  pretpregled mogućih bodova).
- Cjelovit blok: gornji dio s **bonusom od 63 (+35)**, tri/četiri iste, full house,
  mala/velika skala, **Yahtzee (50)** i Chance.
- **Jedan igrač u lovu na rekord** za najviši ukupni zbroj ili **hotseat za 2
  igrača** s dva bloka jedan pored drugog; igra se mišem ili tipkama (razmaknica,
  1-5, strelice, Enter).

**Wordle**
- Pogodite skrivenu riječ; obojena povratna informacija (zeleno/žuto/sivo) uz
  ispravno **brojanje ponovljenih slova** i zaslonsku tipkovnicu koja se boji
  (QWERTZ za njemački, češki, slovenski i hrvatski, AZERTY za francuski, inače
  QWERTY).
- **Četiri načina**: *Beskonačno* (riječ za riječju, svaka sa 6 pokušaja; svaka
  riješena riječ donosi bodove, prva neriješena završava igru), *Riječ dana* (jedna
  riječ dnevno za svaki jezik i duljinu - ista na računalu i u pregledniku - s
  odbrojavanjem i nizom; započeta riječ dana se sprema), *Dordle* (2 riječi
  istodobno u 7 pokušaja) i *Quordle* (4 riječi u 9 pokušaja, tipke pokazuju boje
  svih ploča).
- **Postavke** prije svake igre: **duljina riječi od 4 do 7**, **teški način**
  (pronađeni savjeti moraju se dalje koristiti) i **paleta za daltoniste**
  (narančasta/plava); pokraj se vidi statistika.
- **Pravi popisi riječi na svih 14 jezika** (mapa `woordlistz/`, samo A-Z), za
  svaku duljinu posebni: samo za 5 slova gotovo **34.000 rješenja** i više od
  **213.000 dopuštenih riječi**, u sve četiri duljine ukupno oko 134.000 rješenja.
  Rješenja su uobičajene riječi bez imena, engleskih ostataka i uvredljivih riječi;
  svaki pokušaj provjerava se prema popisu - ostalo se odbija i redak se nakratko
  trese.
- **Statistika** po jeziku, duljini i načinu: igre, postotak pobjeda, trenutačni i
  najbolji niz te **raspodjela pokušaja kao stupčasti grafikon** (odjeljak
  `wordle` u `mem.json`). **Podijeli** (**C**) kopira mrežu emojija bez otkrivanja
  rješenja.
- Za rekord se broji samo *Beskonačno* s 5 slova; ostale duljine imaju vlastite
  najbolje rezultate. Postignuća **Vidovnjak** (najviše 2 pokušaja), **Navika
  riječi** (7 riječi dana zaredom) i **Četverostruki genij** (riješen Quordle).

**Poker**
- **3 varijante** na zaslonu pripreme: **Texas Hold'em** protiv 1–3 AI protivnika s
  gumbom djelitelja, blindovima i četiri runde klađenja, **5 Card Draw** (jedan na
  jedan protiv AI-ja, jedna zamjena karata) i **Video Poker** (*Jacks or Better*,
  solo protiv tablice isplata).
- Radnje gumbima ili tipkama: **F** = fold, **C** = check/call, **R** = raise,
  **A** = all-in; zadržavanje/zamjena karata klikom ili **1-5**, **Enter** vuče
  karte ili dijeli sljedeću ruku.
- **Lama žetoni** zajedničke **Lama banke**: na početku ruke vaš je račun na stolu
  kao hrpa, a ono što ide u pot odmah se skida - napustite li stol usred ruke,
  gubite samo svoj udio u potu. Bez novca (manje od big blinda od 20, u Video
  Pokeru manje od 10) = bankovni kredit do 1000.
- **Rekord** = najviše stanje **bilance u Pokeru** (1000 plus svi dobici i gubici u
  pokeru); postignuće **Chipleader** također broji samo tu bilancu.

**Šah**
- **Potpuni šah**: svi potezi figura uključujući **rokadu**, **en passant** i
  **promociju pješaka** (birate figuru); **šah, mat i pat** te remi po **pravilu
  50 poteza**, **trostrukom ponavljanju pozicije**, **nedostatnom materijalu** ili
  dogovoru.
- **Tri načina**: *partija* protiv AI-ja, *2 igrača* za istim računalom (ploča se
  može okretati nakon svakog poteza) i **zadaci**.
- **Jači AI bez trzanja** u 6 razina od *Početnika* do *Majstora*: iterativno
  produbljivanje, transpozicijska tablica, quiescence pretraga, knjiga otvaranja i
  procjena s pokretljivošću, strukturom pješaka i sigurnošću kralja. AI računa u
  malim obrocima po sličici - igra nikad ne trza.
- **Postavke**: izbor boje, **šahovski sat** (bez, 1+0, 3+2, 5+0, 10+5) i
  **Chess960** (svih 960 početnih pozicija, broj stoji iznad popisa poteza).
- **Bočna ploča** sa satovima, uzetim figurama, bilancom materijala i pomičnim
  **popisom poteza (SAN)**; povuci i ispusti, figure koje klize, koordinate.
  Tipke: **U** = poništi, **H** = strelica savjeta, **O** = ponudi remi, **X** =
  predaja, **F** = okreni ploču, nakon partije **P** = **izvoz PGN-a**.
- **Zadaci**: 200 zadataka u 5 stupnjeva (mat u 1/2/3, taktika I/II) iz
  **slobodne Lichess baze zadataka (CC0)**, provjerenih vlastitim motorom; u
  zadacima s matom vrijedi svaki potez koji matira. Napredak je u odjeljku `chess`
  u `mem.json`.
- Poništavanje i savjet čine partiju „potpomognutom": za rekord se broje samo
  pobjede protiv AI-ja bez pomoći (po sesiji).

**Mlin**
- **Mlin** sa sve tri faze: **postavljanje** (po 9 figura), **pomicanje** duž
  linija i **letenje** kad ostane samo 3 figure (može se isključiti).
- Zatvoreni **mlin** uklanja protivnikovu figuru (po mogućnosti onu izvan mlina);
  gubite kad padnete ispod 3 figure ili ne možete povući potez.
- **3 razine AI-ja** (minimax s alfa-betom, procjena prema fazi) ili **lokalni
  dvoboj**; uz savjete za poteze, isticanje mlinova i brojač figura.

**Simon**
- **Senso igra pamćenja**: osvijetljeni niz raste svaku rundu i mora se točno
  ponoviti.
- **Modovi**: *Klasično*, *Speed* (ubrzava), *Reverse* (unatrag), *Miješano* (mod
  se mijenja svaku rundu) i **Dvoboj** za dva igrača (naizmjence dodajete i
  ponavljate).
- **Zvuk** *isključen / uključen / miješan* (miješani trenira vizualno I slušno
  pamćenje), **4/6/9 polja** kao težina; **najbolji rezultat po modu** se sprema.
  Igra se mišem ili brojčanim tipkama 1-9.

**Biljar**
- **8-ball**, **9-ball** i **vježbovni** mod bez pravila, protiv AI-ja (uz pomoć
  pri ciljanju) ili **dva igrača lokalno**.
- **Tri slobodno birljiva pogleda**: klasičan **2D odozgo**, fiksna **3D kosa
  perspektiva** sa zasjenjenim kuglama i **slobodno rotirajuća 3D kamera** (desna
  tipka miša). Sve kretanje temelji se na vremenskim koracima i **glatko je
  prigušeno** (trenje, međukoraci protiv propadanja).
- **Udarac**: držite lijevu tipku miša za punjenje snage, otpustite za udarac;
  pomažu linija ciljanja i mjerač snage. **Ball in hand** nakon prekršaja. Pogled
  (V) pamti se u `settings.json`; dobiveni frameovi broje se za rekord.

**Klizna slagalica**
- Igra 15 u tri veličine: **3×3** (lako), **4×4** (klasično) i **5×5** (teško);
  klizite numerirane pločice u slobodnu prazninu.
- Uvijek rješiva (izmiješana mnogim nasumičnim potezima). Upravljanje **klikom** na
  pločicu u retku/stupcu praznine (klizi cijela linija) ili **strelicama**.
- Bodovi = osnovna vrijednost po veličini minus potezi i vrijeme; nakon rješavanja
  odmah kreće nova ploča.

**Mastermind**
- Razbijte skriveni **kôd boja**; nakon svakog pokušaja dobivate **crne** čavliće
  (točna boja + položaj) i **bijele** čavliće (točna boja, pogrešno mjesto).
- **3 moda**: Lako (4 čavlića / 6 boja / 12 redaka), Klasično (4/6/10) i Teško
  (5 čavlića / 8 boja); dopušteno je ponavljanje boja.
- Igra se preko palete boja (klik ili tipke **1–8**), OK/Enter ocjenjuje redak.
  **Beskonačni niz** kao u Wordleu: svaki razbijeni kôd donosi bodove.

**Bubble Shooter**
- **Puzzle Bobble** na saćastoj mreži: ciljate mišem, ispaljujete mjehuriće prema
  gore, **tri ili više iste boje** rasprskavaju skupinu.
- Mjehurići koji izgube vezu sa stropom **padaju** (bonus); hici se **odbijaju od
  zidova**, uz pretpregled sljedećeg mjehurića.
- **3 moda** (4/5/6 boja, neki s redovima koji se spuštaju); game over na crvenoj
  liniji.

**Vješala**
- Pogađajte riječ **slovo po slovo**; svaka pogreška crta dio vješala, gubitak
  nakon **6 pogrešaka**.
- **Popisi riječi po jeziku** (samo A–Z), **3 moda duljine** (kratke / miješano /
  duge); tipkajte ili klikajte zaslonsku tipkovnicu.
- **Beskonačni niz**: svaka pogođena riječ donosi bodove (više preostalih života +
  dulja riječ = više).

**Block Jump**
- **3D platformer u stilu Minecrafta** (softverski 3D kao Snakeov 3D mod): skačete
  preko lebdećeg **voxel svijeta** od blokova do svjetlećeg cilja.
- **Minecraft izgled**: svi blokovi imaju prave **pikselske teksture** (trava,
  zemlja, kamen, daske, dijamant, sluz, drvo); razina detalja ovisi
  o udaljenosti (**T** = visoko/nisko/isključeno).
- Uz to: lik **Steve** s animacijom hodanja (kamera iz trećeg lica), **ruka**
  u prvom licu, **zraka svjetionika** na cilju, rotirajuće **zlatne poluge**
  umjesto novčića, kvadratno **sunce**, **pikselni oblaci** i HUD sa **srcima**.
- Vrste blokova: čvrsti blokovi (trava/zemlja/kamen/drvo), **ljestve** (penjanje),
  **ograde** (preskakanje), **odskočni blokovi** (katapultiranje) i **novčići**.
- Kamera je **po zadanome iz prvog lica kao u Minecraftu**, **V** prebacuje na
  prateću kameru; **gledanje mišem** uz hvatanje pokazivača, podesivi **motion
  blur** (**B**) i osjetljivost (**+/-**).
- **Parkour razine generirane sjemenom** postaju sve teže; cilj = bodovi +
  vremenski bonus, novčići +50, pad stoji jedan život (počinjete s 3).
  Upravljanje: WASD/strelice, **razmaknica** za skok.

**Tower Defense**
- **Beskonačna obrana od valova** na **4 karte** (Livada, Kanjon, Raskrižje,
  Špalir), svaka s vlastitom stazom; zaključane karte otključava tvoj najbolji
  val, svaki **8. val** stiže **boss**.
- **3 načina**: Klasični (7 tornjeva, glavni način), Kompaktni (4 tornja,
  2 razine) i Maksimalni (**11 tornjeva**, **specijalizacija A/B** na najvišoj
  razini, posebni neprijatelji, aktivne sposobnosti **Meteor/Ledena nova/Zlatna
  groznica**).
- **11 vrsta tornjeva** od strijela do lasera i zlatne banke, svaki s do
  **3 razine nadogradnje**, prodaja vraća 70%; neprijatelji s oklopom,
  regeneracijom, dijeljenjem, kamuflažom, aurom liječenja i zračnom rutom.
- **Ekonomija**: zlato po obaranju, bonus za val + 5% kamata; bodovi po obaranju
  i valu. **F** = 2x brzina, **G** = dometi, desni klik odustaje.

**Minigolf**
- **360 staza na 40 terena**: *Classic* i *Pro* s po devet ručno izgrađenih
  staza, **Tour** s 38 terena po devet generiranih staza (ukupno 342) i rastućom
  težinom, uz to *Random* iz svega zajedno. Teren 7, staza 3 izgleda svugdje
  jednako - ništa se ne mora spremati.
- **Podloge i prepreke**: pijesak koči, rampe ubrzavaju, voda košta kazneni
  udarac, gumeni odbojnici vraćaju brzinu, a vjetrenjače i lutajući blokovi traže
  osjećaj za trenutak. Fizika radi u podkoracima s trenjem kao u biljaru - ništa
  ne poskakuje i ništa ne prolazi kroz bandu.
- **Upravljanje**: miš cilja, držanje lijeve tipke puni snagu, a otpuštanje
  udara (rade i strelice + razmaknica). **R** otkazuje napunjen udarac bez
  udaranja. **G** prebacuje liniju ciljanja, **Z** auto-ciljanje, **P**
  podizanje.
- **Zaključavanje snage (držana desna tipka)**: zamrzava traku punjenja točno
  ondje gdje jest - zlatna, s postotkom, lokotom i pulsirajućim prstenom oko
  loptice. Tako s napunjenim udarcem čekaš prolaz kroz mlin. Otpuštanje puni
  dalje; zaključana snaga preživi i udarac, a sljedeći lijevi klik udara točno
  tom vrijednošću.
- **Kartica rezultata** desno s parom i udarcima po stazi; u dvoje svatko igra
  istu stazu jedan za drugim. Bodovi: 600 po stazi, ±300 po udarcu ispod/iznad
  para, **500 dodatnih za hole in one**. Najmanji broj udaraca po terenu nalazi se
  u odjeljku `minigolf` datoteke `mem.json`.
- **Podizanje se može isključiti**: zadano staza završava nakon osam udaraca i
  broji se po najmanjoj vrijednosti. Tko radije igra do ubacivanja, u postavkama
  stavi *Podizanje* na ISKLJ (ili pritisne **P**).
- **Auto-ciljanje se može isključiti**: zadano se palica prije svakog udarca
  sama okrene prema rupi. Tko radije cilja sam na svakoj stazi, u postavkama
  stavi *Auto-ciljanje* na ISKLJ. (ali pritisne **Z**) - tada ostaje zadnji
  odabrani smjer, a na početku nove staze palica pokazuje neutralno prema gore.
- **F** poništava stazu u igri: udarci na 0, loptica na početak - ista staza, isti
  teren.
- **Dalje umjesto ponavljanja**: na kraju runde gumb **Dalje** vodi na sljedeći
  teren (*Classic* → *Pro* → *Tour 1* → *Tour 2* …), pa se isti komplet od devet
  staza ne ponavlja; pokraj njega **Ponovno** (isti teren) i **Postavke**. Tipke:
  Enter = dalje, R = ponovno, S = postavke.
- **Snimka runde**: na kraju **P** (ili gumb **Snimka**) prikazuje cijelu rundu
  udarac po udarac. Tipkom **S** ide u arhivu (gumb **Snimke** u bočnoj traci).
- **Gradnja i dijeljenje vlastitih staza**: kartica **MAPS** na pripremnom
  zaslonu vodi do tvoje zbirke - **Nova** otvara uređivač staza. Svaka staza
  dobiva naziv i **id** (mala slova, bez razmaka); id je ujedno predloženi
  naziv datoteke pri dijeljenju. Uz sedam klasičnih prepreka dolazi **osam
  novih**: cijev (premješta lopticu na drugi kraj), led, ljepljivo polje,
  ubrzivač, magnet, jednosmjerna vrata, okretna ploča i skakaonica. Veličina
  staze slobodno se postavlja (60x80 do 160x240), **12 predložaka** daje
  polazište, a poništi/ponovi i **Test** su tu. **Podijeli** zapisuje točno
  jednu stazu kao datoteku `.lamapgzmap` - preko dijaloga za spremanje ili
  ravno u mapu Preuzimanja, s tvojim imenom kao autora. **Uvezi** je učitava
  natrag i kod zauzetog id-a automatski prelazi na `-2`. Naziv, id i autor
  uvijek prolaze kroz **filtar riječi preko svih 14 jezika**. Staza se igra
  pojedinačno preko **Igraj**, a cijela zbirka preko petog izbora terena
  **Vlastite**.

**Pinball**
- **Tri stola**: *Classic* (tri odbojnika, jedan niz meta), *Space* (četiri
  odbojnika u rombu, dva niza) i *Lama* (otvoreno polje, šest meta u luku); 3 ili
  5 kugli po partiji, u dvoje naizmjence kuglu po kuglu.
- **Sve što flipper treba**: kanal za izbačaj s mjeračem snage (preslabo? kugla se
  vraća i smiješ ponovno), dva flippera, slingshotovi, nizovi meta, četiri staze
  **L-A-M-A**, zamka sa zaključavanjem kugle, **multiball s jackpotom**, šest
  sekundi **spašavanja kugle**, gurkanje i **TILT**.
- **Množitelj do x5** preko oborenih nizova i dovršenih staza; odbojnici 100,
  slingshotovi 50, mete 250 - tijekom multiballa odbojnici plaćaju jackpot od
  2.500.
- Flipperi rade na dodijeljenim tipkama lijevo/desno (te lijevi/desni [Shift]) ili
  mišem. Rekord svakog stola je u odjeljku `pinball` datoteke `mem.json`.

**Bowling**
- **Deset frameova po službenim pravilima**, uključujući strikeove, spareove i
  bonus bacanja u desetom frameu (najviše: 300). **Kartica** ispod zaglavlja
  prikazuje svaki frame s X, / i tekućim zbrojem.
- **Bacanje u četiri koraka**: položaj, kut, rotacija i snaga. Svaki klizač njiše
  se sam i fiksira ga tipka akcije - ili se namjesti ručno lijevo/desno, čime se
  njihanje zaustavlja.
- **Prava fizika čunjeva**: deset čunjeva kao krugovi s masom koji obaraju jedan
  drugoga; strike nastaje iz fizike, a ne iz sreće. Staza je sprijeda nauljena, pa
  **hook** hvata tek u zadnjoj trećini.
- Pogled na stazu u perspektivi s kanalima, strelicama i poljem čunjeva; tri
  težine (*Lako/Normalno/Pro*) mijenjaju brzinu klizača i rasipanje. Rekord za
  svaku težinu je u odjeljku `bowling` datoteke `mem.json`.
- **Snimka partije**: na kraju **P** ponovno prikazuje sva bacanja, a **S** ih
  sprema u arhivu (gumb **Snimke**).

**Crossy Road**
- **Beskonačno skakanje** preko livada (drveće i kamenje zatvaraju put), cesta s
  automobilima i kamionima, rijeka s deblima i lopočima te **tračnica** po kojima
  nakon svjetla upozorenja i zvona projuri vlak - dalje čekaju cijele stanice s do
  5 kolosijeka. Staza nastaje red po red, uvijek ima prohodan put, a tempo i
  promet rastu.
- **Izometrijski voxel stil**: likovi, vozila i drveće od sjenčanih kocaka
  (unaprijed iscrtani za svaku veličinu polja), kamera koja glatko prati, squash &
  stretch pri skokovima, prskanje vode, animacija spljoštenja, perje i svjetlucavi
  novčići; od reda 50 **izmjena dana i noći** s farovima.
- **Orao**: kamera polako puzi naprijed - tko predugo oklijeva ili se vrati za više
  od tri reda, zgrabi ga orao (crveni rub prije upozorava). Ako vas deblo odnese
  preko ruba slike, igri je također kraj.
- **Novčići i likovi**: skupljeni novčići (divovski novčić = 5) se spremaju i
  kupuju nove likove u kartici **Likovi**: žabu, svinju, pingvina, mačku, lisicu,
  lamu, robota, duha i jednoroga (25 do 250 novčića); pile je dostupno od početka.
- **Načini**: *Beskonačno* (bodovi = najdalji red, broji se za rekord) i *Dnevna
  staza* (danas ista za sve, i u pregledniku, s vlastitim dnevnim rekordom).
  Upravljanje: strelice/WASD, razmaknica/Enter/klik = skok naprijed; u postavkama
  **H** = sjene, **N** = dan/noć. Novčići, likovi i dnevni rekord nalaze se u
  odjeljku `crossy` u `mem.json`.

**Geometry Dash**
- **Ritmička platformska igra**: lik sam juri udesno - vi odlučujete samo kada
  skočiti ili letjeti. **Pet oblika** - kocka, brod, lopta, NLO i val - uz to
  portali oblika, gravitacije i brzine (0,5x do 3x), žuti/ružičasti/plavi
  **odskoci i kugle**, polublokovi, šiljci, jame i okidači boja.
- **8 ugrađenih razina** od *Lako* do *Demon* („Lama Inferno") s po **3 tajna
  novčića**. Svaka razina dokazano je prolazna: pri izradi ju je rješavač prešao
  pravim kodom igre - sa svim novčićima, čak i s pomakom od 1/240 sekunde.
- **Precizna fizika**: računanje s nepomičnim zarezom u stalnom koraku od 240 Hz;
  svaki pritisak djeluje točno u koraku u kojem se dogodio - jednako pri svakoj
  brzini sličica i bitovno jednako u pregledniku.
- **Način vježbe** (**P**) s automatskim i vlastitim kontrolnim točkama (**Z**
  postavlja, **X** briše), brojačem pokušaja, trakom napretka, eksplozijama i
  trenutačnim ponovnim pokretanjem (**R**). Svaka razina ima **vlastitu glazbu** -
  pozadina, tlo i kugle pulsiraju u ritmu (glazba se isključuje tipkom **M**).
- **Zvjezdice i novčići**: tko prijeđe razinu u normalnom načinu, dobiva njezine
  zvjezdice, a svaki novčić vrijedi još jednu; rekord je **ukupan broj
  zvjezdica** (najviše 65). Najbolji rezultati po razini, novčići, pokušaji i
  skokovi nalaze se u odjeljku `geodash` u `mem.json`.
- **Uređivač razina** na kartici **RAZINE**: platno s mrežom, paleta sa 6 skupina
  (blokovi, opasnosti, odskoci i kugle, portali, brzina, dodaci), okretanje,
  poništi/ponovi, pregledna traka, **test od početka ili odavde** i postavke
  razine (početna brzina i oblik, glazbeni stil, BPM, boje). Kvačica
  **„provjereno"** dolazi tek kada sami prijeđete svoju razinu. **Podijeli**
  zapisuje datoteku `.lamapgzlevel`, **Uvoz** je ponovno učitava; razine se
  spremaju u `ugc.json` uz vlastite minigolf staze.

**Battleship**
- **Pomorska bitka 10x10** s nosačem zrakoplova (5 polja), bojnim brodom (4),
  krstaricom (3), podmornicom (3) i razaračem (2) - pobjeđuje tko prvi potopi
  cijelu neprijateljsku flotu.
- **Raspoređivanje flote** povlačenjem iz doka: **R** ili desni klik okreće,
  pregled svijetli zeleno ili crveno, **X** sve raspoređuje nasumično, **C** prazni
  ploču; posljednji raspored ponovno se predlaže.
- **Pravila u postavkama** (spremaju se): *brodovi se smiju dodirivati*, *salva*
  (onoliko hitaca po potezu koliko vlastitih brodova još pluta) i *nakon pogotka
  pucaš ponovno*.
- **AI s 3 razine**: Lako puca nasumično, Srednje sustavno dovršava pogotke, Teško
  računa **kartu vjerojatnosti** s paritetom šahovnice (u prosjeku oko 70 / 60 / 45
  hitaca za cijelu flotu). Ili **2 igrača** za istim računalom - **zaslon za
  predaju** prije svakog poteza skriva obje flote.
- **Grafika**: radarsko skeniranje, animirani valovi, granate u luku, prskanje,
  eksplozije s dimom i zapaljena polja, otkrivanje „POTOPLJENO!" i sažetak runde s
  hicima, pogocima i preciznošću. Rekord broji vaše **pobjede protiv AI-ja** u
  jednoj sesiji.

**Casino**
- **Rulet** (europski, 37 polja): svi klasični ulozi klikom na broj, rub ili kut -
  **plein** (35:1), cheval, transversale, carré, sixain, stupac, desetica,
  crveno/crno, par/nepar i manque/passe. Žetoni 1/5/25/100/500, desni klik uklanja
  žetone; **Zavrti**, **Ponovi** (**R**), **Udvostruči** (**D**) i **Očisti**.
  Kuglica se spiralno skotrlja u unaprijed izvučeno polje, a gore se vidi
  posljednjih 12 brojeva.
- **Lama automat**: 5 valjaka x 3 reda, **10 dobitnih linija**, **lama = džoker**,
  **zlatnici = scatter** s 10 besplatnih vrtnji i dvostrukim dobicima, ulog po
  liniji 1/2/5/10, **automatska vrtnja** (10/25), **turbo** i tablica isplata.
  **Stopa povrata je 96,1 %** - točno izračunata iz traka valjaka.
- **Lama banka**: Casino, Blackjack i Poker dijele jedan račun **lama žetona**
  (početak 1000, odjeljak `casino` u `mem.json`); stara stanja žetona prenose se
  automatski. Ulozi se skidaju odmah, svaka igra vodi vlastitu bilancu za rekord, a
  pri bankrotu dobivate **bankovni kredit** do 1000.
- Konfeti, kiša novčića, natpisi big/mega/jackpot i animacije dobitnih linija;
  postignuća **Pun pogodak** (dobitni plein na ruletu) i **Lama jackpot** (5 lama na
  jednoj liniji).

Rekordi se spremaju u odjeljak `highscores` datoteke `mem.json` (uz kôd) – zajedno
s jezikom (odjeljak `mem`).

### Sučelje

Cijelo je sučelje nacrtano od nule (čisti Tkinter + Pygame, bez dodatnih paketa) i
dotjerano poput modernog pokretača igara:

- **Bočna traka s popisom igara**: svaki redak ima vlastiti **mini-piktogram** u
  naglasnoj boji igre, prikazuje trenutni **rekord (★)** i reagira glatko
  animiranim hover efektima. Igra koja je u tijeku ostaje istaknuta bojom; u malim
  prozorima popis se **pomiče** kotačićem miša.
- **Kartica stanja** dolje lijevo s **LED-om stanja** (sivo = izbornik, zeleno = u
  tijeku, zlatno = pauza, crveno = game over) i **prikazom FPS-a uživo**.
- **Početni zaslon** s aurorama, paralaksnim zvjezdanim poljem sa zvijezdama
  padalicama, lebdećim logotipom s iskrama u orbiti, **klikabilnom mrežom igara**
  odmah ispod logotipa (sve igre s hover efektom u svojoj naglasnoj boji) i
  **klizajućom trakom rekorda**.
- **Efekti posvuda**: meki prijelazi zaslona, iskre pri potvrdi u izborniku,
  **kiša konfeta pri novom rekordu** i pravi **blur** iza pauznog prekrivača.
- **Zaslon pripreme** svake igre pojavljuje se u njezinoj naglasnoj boji i
  prikazuje prethodni rekord kao čip. Uz mnogo načina i malu razlučivost postaje
  **kompaktan**: Opcije, Wiki i Natrag slažu se u jedan red, a font se
  prilagođava - ništa više ne izlazi iz slike.
- **Ujednačen izgled u igri**: svih 46 igara dijeli paletu teme i font izbornika -
  HUD-ovi, zasloni pripreme i prekrivači slijede dizajn odabran u opcijama
  (v4.1 / v4 / Classic), dok svako igralište zadržava svoje prepoznatljive boje.
  Svaka igra sada uredno obrađuje promjenu razlučivosti usred partije, a nazivi u
  izborniku prilagođavaju se jeziku (npr. „Schach" → „Chess" / « Échecs »).
- **Ugrađeni wiki** („LamaWiki"): detaljna pomoć za svaku igru (upravljanje,
  modovi, bodovanje, savjeti) uz opće stranice - s **poljem za pretraživanje**,
  kategorijama, člancima koji se pomiču i čipovima s tipkama, na svih 14 jezika.
  Dostupan preko gumba **„Wiki / Pomoć"** u bočnoj traci i sa zaslona pripreme
  svake igre (izravno otvara njezinu stranicu).
- **Postignuća i statistika**: **107 postignuća** u tri kategorije (23 cilja na
  razini kolekcije, 37 bodovnih prekretnica i 47 posebnih trenutaka poput
  šah-mata AI-ju, pločice 4096, T-Spin Doublea, 25 riješenih šahovskih zadataka,
  Killer Sudokua ili laminog jackpota; u 2048 i šahu partije s poništavanjem ili
  savjetom ne računaju se) sa **zlatnom obavijesti i fanfarom** pri
  otključavanju - čak i usred igre; stari rekordi priznaju se automatski. Uz
  to kartica **statistike**: ukupno vrijeme igranja, partije, pobjede,
  rekordi, najdraža igra i tablica igara poredana po vremenu. Dostupno preko
  gumba **„Postignuća i statistika"** u bočnoj traci.
- **Snimke**: minigolf i kuglanje snimaju svaku rundu. Na kraju **P** prikazuje
  snimku, a **S** je sprema u arhivu - dostupnu gumbom **Snimke** u bočnoj traci
  (kartica po igri, pauza, skokovi među sekvencama, brzina 0,5x do 4x). Može se
  isključiti pri prvom pokretanju i u opcijama.

### Upravljanje

- Igru odaberite gumbom u izborniku slijeva. Zatim se pojavljuje **zaslon
  pripreme**: odaberite **Jedan igrač** ili **Više igrača**, otiđite na **opcije**
  ili natrag. Strelice/miš za odabir, Enter za pokretanje.
- **ESC** = pauza / nastavak (u izbornicima: natrag).
- **F11** (ili gumb „Puni zaslon uklj./isklj.") = uključivanje/isključivanje punog
  zaslona. Pygame prikaz ostaje ugrađen i uvećava se uz zadržavanje omjera stranica
  (crne trake kad se omjer razlikuje). Prozor se može slobodno mijenjati veličinom.
- **„Natrag na izbornik"** završava igru i sprema rekord - isto vrijedi i za
  prelazak u drugu igru preko bočne trake.
- **Fiksne dodatne tipke**: osim pet radnji koje se mogu dodijeliti, neke igre
  imaju vlastite tipke (npr. spremište **C** i okretanje ulijevo **Z** u Tetrisu,
  poništavanje **U** u 2048, šahu i Sudokuu). Rade samo ako tipka u opcijama nije
  dodijeljena nijednoj radnji, a navedene su u savjetu postavki i u wikiju.
  Držane tipke ispravno se prepoznaju i otpuštaju pri pauzi ili Alt-Tabu - ništa
  više ne „zapinje".
- **„Izlaz"** uredno zatvara Pygame i Tkinter.

### Opcije, upravljanje i zvuk

Zaslon opcija otvara se gumbom **„Opcije / Upravljanje"** (slijeva) ili sa zaslona
pripreme. Organiziran je u **tri kartice** (**Općenito / Upravljanje / Izgled**;
prebacuje se klikom ili tipkom Tab):

- **Općenito**: **zvuk** uklj./isklj., **glasnoća** i **haptika** (vibracija
  gamepada, djeluje samo uz priključeni kontroler) te **automatska razlučivost**,
  **razlučivost**, **FPS** i **jezik** – svako se prebacuje tipkama Lijevo/Desno.
- **Upravljanje**: **predlošci** (*WASD + Strelice*, *WASD + IJKL*,
  *Strelice + WASD*) i **preslagivanje svake pojedine tipke** za igrača 1 i igrača
  2: odaberite redak, pritisnite Enter, pritisnite željenu tipku (Esc odustaje).
- **Izgled**: odaberite **dizajn sučelja** – **UI v4.2** (zadano: Midnight Glass –
  duboki ponoćni prijelaz s polako lebdećim mekim svjetlima u indigo, tirkiznoj i
  magenta boji, fino filmsko zrno, rijetke zvijezde i ploče poput matiranog stakla
  sa svjetlosnim rubom), **UI v4.1** (poput UI v4, ali življe – suptilne zvijezde
  te Saturn i crna rupa u pozadini početnog zaslona), **UI v4.1.1** (kao v4.1, ali
  umjesto zvjezdanog neba popločani **cik-cak uzorak** u crnoj i antracit boji),
  **UI v4.1.2** (isti uzorak u plavim tonovima palete – naglasna plava kao
  dominantna boja, tamnija plava kao podloga), **UI v4.1.3** (isti uzorak u indigo
  boji UI v4 na crnoj), **UI v4.1.4** (u grafitnom tonu UI v4 na crnoj), **UI v4**
  (posve smiren, plosnat grafitni izgled s jednim indigo naglaskom), **UI v3**
  (dosadašnje klasično sučelje sa zvjezdanim nebom, aurorama i sjajnim efektima),
  **UI v2** (prva preradba sučelja: tamnoplavi prijelaz, zvjezdano nebo i
  svijetleći gumbi, potpuno bez animacija) ili **UI v1** (izgled prije preradbe
  sučelja: jednobojna tamna pozadina, ravni gumbi, bez efekata). Sve kartice
  prikazuju malu pretpreglednu sličicu; odabir se odmah primjenjuje na cijelo
  sučelje (područje igre **i** bočnu traku) i sprema se.

Postavke se trajno spremaju u `settings.json`. U **jednom igraču** obje dodjele
upravljaju istim likom (zadano: WASD *i* strelice), a u **više igrača** svaka po
jednim. Sve igre imaju **zvučne efekte** (proceduralno generirane, bez potrebe za
dodatnim datotekama) koji se mogu globalno utišati.

### Struktura projekta

```
install-python.bat  Windows postavljanje: Python 3.13 + .venv + pygame
start.bat            Skripta za pokretanje (Windows)
start.sh             Skripta za pokretanje (Linux / macOS / Git Bash)
pyinstall.bat        Izrada EXE-a (Windows): pakira sve u builds\PyGameZ.exe
main.py              Tkinter sučelje, ugradnja Pygamea, središnja petlja igre
game_base.py         Osnovna klasa igre (update/draw/handle_event) + InputEvent + pomoćnici
settings.py          Učitavanje/spremanje postavki (zvuk/haptika/tipke/opcije igara s pravilima provjere) (JSON)
audio.py             Proceduralni zvučni efekti, glazbene petlje + vibracija gamepada
menu.py              Zasloni jezika, pripreme (mod) i opcija (zvuk/upravljanje)
highscore.py         Učitavanje/spremanje rekorda (odjeljak u mem.json)
store.py             Središnja datoteka spremanja mem.json (odjeljci: mem, highscores, stats, achievements + napredak igara), atomski sa sigurnosnom kopijom .bak
stats.py             Statistika igrača (partije, vrijeme, pobjede, rekordi) po igri
achievements.py      Postignuća: definicije, otključavanje, obavijest (toast)
progress.py          Zaslon postignuća i statistike (dvije kartice, pomični)
replay.py            Snimanje i arhiva snimaka (replay.json)
replayview.py        Zaslon snimaka: popis arhive i reprodukcija
ugc.py               Vlastiti sadržaj (minigolf staze, razine Geometry Dasha): pohrana, provjera, izvoz/uvoz (ugc.json)
swear.py             Filtar riječi za nazive i id (lang/swear/*.yml, svih 14 jezika)
filepick.py          Dijalozi datoteka ("Izvezi kao ...", "Uvezi")
prestige.py          Prestiž-sustav za Snake
competitive.py       Ugađanje Competitive moda za Snake (razine, slot machine, kockarske jabuke)
ngb.py               Vizualna personalizacija ("mods"): boja glave + koordinatna mreža + izbornik (mem-ngb.json)
lamabank.py          Lama banka: zajednički račun žetona za Blackjack, Poker i Casino (odjeljak casino u mem.json)
seedrand.py          Generator slučajnih brojeva s bitovno jednakim brojevima u Pythonu i pregledniku (dnevni načini, nove zagonetke)
i18n.py              Prevoditeljski mehanizam (učitava lang/*.json, t("ključ"))
lang/
  de.json  en.json  fr.json  es.json  pt.json   Jezični nizovi (jedan ključ po tekstu)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
  swear/                                    Popisi filtra riječi po jeziku (regex, .yml)
lamawiki/
  lamawiki.py          Ugrađeni wiki (pretraga, kategorije, prikaz članaka)
  de.json  en.json  fr.json  es.json  pt.json   Sadržaj wikija (jedna stranica po igri + opće stranice)
  lang.expansion/                           pl, tr, da, no, sv, fi, cs, sl, hr
woordlistz/
  build_wordlists.py   Ponovno gradi popise riječi za Wordle (rječnici + popisi učestalosti)
  de/ en/ fr/ ... hr/  answers.txt + allowed.txt (5 slova), answers4/6/7.txt + allowed4/6/7.txt (4, 6, 7 slova), 14 jezika
devtools/            Razvojni alati (ne pakiraju se u .exe)
  merge_staging.py           Unosi prijevode i wiki stranice iz devtools/staging/ u svih 14 jezičnih datoteka
  build_chess_puzzles.py     Gradi 200 šahovskih zadataka iz Lichess baze zadataka (CC0)
  build_sudoku_killer.py     Generira 400 Killer Sudokua s jedinstvenim rješenjem
  build_crossyroad_models.py Zapisuje voxel modele Crossy Roada za web-inačicu
  build_geodash_levels.py    Gradi 8 razina Geometry Dasha i rješavačem dokazuje da je svaka prolazna zajedno s novčićima
  build_geodash_solver.py    Rješavač sa stvarnim kodom koraka (rješenja u geodash_proofs.json)
games/
  snake.py  pong.py  airhockey.py  tictactoe.py  breakout.py
  tetris.py  tetris_core.py  tetris_ai.py
  invaders.py  asteroids.py  pacman.py  flappy.py  doodle.py
  game2048.py  minesweeper.py  sudoku.py  sudoku_gen.py  sudoku_draw.py
  frogger.py  memory.py  solitaire.py  cards.py  aimtrainer.py
  connect4.py  tanks.py  blackjack.py  tunnelracer.py
  labyrinth.py  maze_gen.py  reversi.py  kniffel.py  wordle.py  wordle_words.py
  trexrunner.py  dame.py  poker.py  muehle.py
  chess.py  chess_engine.py  chess_draw.py
  simon.py  billiard.py  slidepuzzle.py  mastermind.py
  bubbleshooter.py  hangman.py  hangman_words.py  blockjump.py
  lamatowerdefense.py  minigolf.py  minigolf_gen.py
  minigolf_draw.py  minigolf_edit.py  pinball.py  bowling.py
  crossyroad.py  crossyroad_world.py  crossyroad_draw.py
  geodash.py  geodash_core.py  geodash_draw.py  geodash_edit.py  geodash_music.py
  battleship.py  battleship_core.py
  casino.py  casino_logic.py  casino_roulette.py  casino_slots.py  casino_draw.py
  levels/              Podaci razina: snake-comp.json, chess-puzzles.json (+ README s izvorima), sudoku-killer.json, geodash.json
tests/
  arcade_casino_audit.py     Cjelovita provjera (unos, seedrand Python = JS, spremanje, jezične datoteke, zasloni pripreme) + svi audit_*.py
  audit_tetris.py  audit_2048.py  audit_chess.py  audit_sudoku.py  audit_wordle.py
  audit_crossyroad.py  audit_geodash.py  audit_battleship.py  audit_casino.py   Headless provjere po igri
  newgames_audit.py  blockjump_audit.py
```

Odabrani jezik sprema se u `mem.json` (u odjeljku `mem`, uz odjeljak `highscores` u
istoj datoteci) i automatski se učitava pri sljedećem pokretanju.

**Izvori i licence:** 200 šahovskih zadataka potječe iz
[Lichess baze zadataka](https://database.lichess.org/#puzzles) (licenca
**CC0 1.0**, javno dobro - hvala, lichess.org!); pojedinosti su u
`games/levels/chess-puzzles.README.md`. Izvori popisa riječi za Wordle navedeni su
u `woordlistz/README.md`.

### Napomene o platformi

Prikaz radi **off-screen**: pygame koristi dummy video upravljački program
(`SDL_VIDEODRIVER=dummy`), pa iscrtava u surface, a svaki se kadar crta kao slika u
Tkinter widget. Ne postoji **nativni SDL prozor** koji bi se s Tkinterom borio oko
veličine/položaja. Zbog toga se prozor svugdje ponaša jednako i stabilno:

- **Windows**: proces se dodatno označava kao DPI-aware kako bi prikaz ostao oštar
  na skaliranim zaslonima (125/150/200 %) i ne bi „podrhtavao".
- **Linux/X11 i Wayland**: radi bez posebnih slučajeva (bez `SDL_WINDOWID`).
- **macOS**: također radi (prije se ugrađeni prozor ovdje uopće nije prikazivao).

---

### Vodič za instalaciju

Preduvjet: **Python 3.9+** (preporučeno 3.12 ili 3.13) i **pygame ≥ 2.6**.

#### Windows (preporučeno: automatski)

1. Otvorite mapu projekta i dvokliknite **`install-python.bat`**. Skripta
   - provjerava je li prisutan **Python 3.13** i, ako nije, instalira ga putem
     **wingeta** (`winget install Python.Python.3.13`),
   - stvara virtualno okruženje **`.venv`**,
   - instalira **pygame** iz `requirements.txt`.
2. Zatim pokrenite zbirku pomoću **`start.bat`** (dvoklik).

> Napomena: ako skripta javi „još nije dostupno u ovom prozoru", Python je upravo
> instaliran – jednostavno otvorite **novi terminal/prozor** i ponovno pokrenite
> `install-python.bat`. Ako **winget** nije dostupan, instalirajte Python 3.13
> ručno s <https://www.python.org/downloads/> i označite
> **„Add python.exe to PATH"**.

#### Windows / Linux / macOS (ručno)

```bash
# 1. Provjeri Python (3.9+)
python --version

# 2. Stvori i aktiviraj virtualno okruženje
python -m venv .venv
#   Windows (cmd):        .venv\Scripts\activate
#   Windows (PowerShell): .venv\Scripts\Activate.ps1
#   Linux / macOS:        source .venv/bin/activate

# 3. Instaliraj ovisnosti
pip install -r requirements.txt
#   ili: pip install "pygame>=2.6" (ili pygame-ce)
#                                   pip install pygame-ce
# 4. Pokreni
python main.py
```

#### Linux / macOS sa start.sh

```bash
# Postavi Python + venv kao gore (koraci 2 i 3), zatim:
chmod +x start.sh      # jednom, ako još nije izvršno
./start.sh
```

Na Linuxu po potrebi instalirajte Python putem upravitelja paketa, npr.
`sudo apt install python3 python3-venv python3-pip` (Debian/Ubuntu); na macOS-u
npr. `brew install python`.

#### Korištenje druge verzije Pythona

`install-python.bat` po zadanome postavlja Python 3.13. Ako preferirate 3.12 (ili
neku drugu verziju), u datoteci promijenite redak `set "PYVER=3.13"` u željenu
verziju i winget ID u skladu s tim (`Python.Python.3.12`).

#### Izrada samostalne EXE datoteke (Windows)

```bat
pyinstall.bat         :: gradi builds\PyGameZ.exe (sve u jednoj datoteci)
```

`pyinstall.bat` koristi `.venv` (i stvara ga po potrebi), automatski instalira
**PyInstaller** i pakira cijelu igru - Python, pygame, sve igre, jezike, wiki i
logotipe - u **jedan `PyGameZ.exe`** u mapi **`builds\`**. Datoteka radi na svakom
Windows računalu bez instaliranog Pythona i može se slobodno kopirati. Postavke i
rekordi (`settings.json`, `mem.json`, `mem-ngb.json`) stvaraju se pokraj .exe
datoteke tijekom igranja.

#### Rješavanje problema

- **`pygame` nije pronađen** → je li venv aktiviran? Ponovite korak 3
  (`pip install -r requirements.txt`).
- **`python` nije prepoznat (Windows)** → Python je instaliran bez „Add to PATH";
  ponovno instalirajte i označite kućicu, ili koristite `py` umjesto `python`.
- **Nema zvuka** → provjerite „Sound" u opcijama; haptika radi samo uz kontroler.
- **Prozor/ugradnja na Linuxu** → vidi *Napomene o platformi* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ natrag na vrh / back to top</a></b></div>
