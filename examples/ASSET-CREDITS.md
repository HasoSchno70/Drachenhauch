# Drachenhauch – Lizenz & Quellen der Beispiel-Assets

Diese Datei dokumentiert Herkunft und Lizenz aller mit den Drachenhauch-Beispielen
ausgelieferten Medien (Audio, 3D-Modelle, HDRIs, Grafiken). Sie dient zugleich
als **Namensnennung (Attribution)** für die CC-BY-lizenzierten Inhalte – bei
Weitergabe bitte beibehalten.

## Eigene / selbst erzeugte Inhalte
Selbst erstellt (mit den Drachenhauch-Tools `dhsprites`/`dhtracker`/`dhsfx` bzw.
Generator-Skripten) und damit frei – auch kommerziell – verwendbar:
- Alle Pixel-Grafiken/Sprites: `*.dhsprite`, `*.gif`, `*.png`, Atlanten/`*.json`
  – darunter der eigenständige Plattformer-Satz in `platformer/` und der Spieler
  `assets/player.*` („Twilight"-Thema, bewusst nicht an Marken/Werke angelehnt).
- Shader: `assets/shaders/*.fs` (bloom, crt, vignette).
- Audio: `assets/demo.mod` (ProTracker-MOD, `make_demo_mod.py`),
  `assets/*.wav` (`make_pluck_sample.py` u. a.), `assets/sfx_*.ogg`,
  `assets/ambient.ogg` – über die eingebauten Sound-Werkzeuge erzeugt.

## Gemeinfrei (CC0 1.0) – keine Pflichten, Nennung aus Höflichkeit
| Datei | Werk / Autor | Quelle |
|---|---|---|
| `assets/amiga_action_level1.ogg`, `assets/amiga_title.ogg` | „5 Action Chiptunes" – Juhani Junkala (SubspaceAudio) | opengameart.org/content/5-chiptunes-action |
| `assets/techno_messup.ogg` | „Technological Messup" – josepharaoh99 | opengameart.org/content/cc0-upbeat-electronic-music |
| `assets/robot.glb` | raylib-Beispielmodell – raysan5 | github.com/raysan5/raylib |
| `assets/ibl_env.hdr` | HDRI „kloofendal_43d_clear" – Poly Haven | polyhaven.com/a/kloofendal_43d_clear |

CC0 erlaubt jede Nutzung inkl. kommerziell, ohne Namensnennung.

## Namensnennung erforderlich (CC-BY 4.0)
| Datei | Werk / Autor | Quelle / Lizenz |
|---|---|---|
| `assets/cybermatic_pulse.ogg` | **„Cybermatic pulse" – Alexandr Zhelanov** (soundcloud.com/alexandr-zhelanov) | opengameart.org/content/cybermatic-pulse · CC-BY 4.0 (creativecommons.org/licenses/by/4.0/) |

> Bei Weitergabe eines Produkts, das `cybermatic_pulse.ogg` enthält, **muss** der
> Autor genannt werden (z. B. in dieser Datei oder im Abspann). Keine Änderungen
> an der Datei vorgenommen.

## Plattformer-Sprites (`platformer/`)
Eigenständiger, prozedural erzeugter Sprite-/Tile-Satz im „Twilight"-Thema
(violetter Spieler, schiefer-violette Gegner, Stahl-Cyan-Röhren, Kristall-Boxen),
**bewusst nicht an Nintendo-Marken/-Figuren angelehnt** → frei, auch kommerziell,
verwendbar. Generiert über `platformer/make_sprites.py`. Siehe `platformer/README.md`.
