<a name="other-languages"></a>

# PyGameZ - Otros idiomas / Outros idiomas

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](README.md#-deutsch)** · **🇬🇧 [English](README.md#-english)** · **🇪🇸 [Español](#-espanol)** · **🇵🇹 [Português](#-portugues)**

---

<a name="-espanol"></a>

## 🇪🇸 Español

Una colección de juegos de escritorio en Python: **Tkinter** aporta la ventana y
el menú, **Pygame** va incrustado como pantalla de juego dentro de la ventana de
Tkinter. Veintitrés juegos con opciones compartidas, controles totalmente
reasignables, récords, efectos de sonido procedurales y, en varios títulos, modo
multijugador. La interfaz es **multilingüe** (alemán / inglés / francés /
español / portugués); el idioma se elige en el primer arranque (el español y el
portugués están allí tras el discreto botón **«Otros idiomas»** abajo del todo)
y se puede cambiar en cualquier momento en las opciones.

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

**El multijugador (2 jugadores en local)** está disponible en **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Cuatro en raya** y **Duelo de tanques**.
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
izquierda) o desde la pantalla previa:

- **Sonido** sí/no, **volumen** y **vibración** (vibración del gamepad, solo
  efectiva con mando conectado) – cada uno con Izq/Der.
- **Plantillas** de controles: *WASD + Flechas*, *WASD + IJKL*, *Flechas + WASD*.
- **Cada tecla individual** de los jugadores 1 y 2 es reasignable: elegir fila,
  pulsar Enter, pulsar la tecla deseada (Esc cancela).

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
  labyrinth.py  maze_gen.py
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
Vinte e três jogos com opções partilhadas, controlos totalmente reatribuíveis,
recordes, efeitos sonoros procedurais e, em vários títulos, modo multijogador.
A interface é **multilingue** (alemão / inglês / francês / espanhol /
português); o idioma escolhe-se no primeiro arranque (o espanhol e o português
estão lá atrás do discreto botão **«Outros idiomas»** em baixo de tudo) e pode
ser mudado a qualquer momento nas opções.

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

**O multijogador (2 jogadores em local)** está disponível em **Snake**, **Pong**,
**Air Hockey**, **Tic-Tac-Toe**, **Tetris (Versus)**, **Asteroids (duelo
cooperativo)**, **Memory (duelo)**, **Quatro em linha** e **Duelo de tanques**.
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
partir do ecrã de preparação:

- **Som** sim/não, **volume** e **vibração** (vibração do gamepad, só com
  comando ligado) – cada um com Esq/Dir.
- **Modelos** de controlos: *WASD + Setas*, *WASD + IJKL*, *Setas + WASD*.
- **Cada tecla individual** dos jogadores 1 e 2 é reatribuível: escolher a
  linha, premir Enter, premir a tecla desejada (Esc cancela).

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
  labyrinth.py  maze_gen.py
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

#### Resolução de problemas

- **`pygame` não encontrado** → venv ativado? Repete o passo 3
  (`pip install -r requirements.txt`).
- **`python` não é reconhecido (Windows)** → o Python foi instalado sem "Add to
  PATH"; reinstala com a opção marcada, ou usa `py` em vez de `python`.
- **Sem som** → verifica "Som" nas opções; a vibração só funciona com comando.
- **Janela/incorporação no Linux** → ver *Notas de plataforma* (Wayland/XWayland).

<div align="right"><b><a href="#other-languages">↑ voltar ao topo / back to top</a></b></div>
