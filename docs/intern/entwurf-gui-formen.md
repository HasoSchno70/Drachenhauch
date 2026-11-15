# Entwurf: eigene Formen, Bildhäute und Animationen in der gui

> **Stand 14.11.2026: Stufe 1 gebaut** (Formen, Bildhaut, Animationen für
> Knöpfe bzw. jedes Widget). Stufen 2 und 3 offen.

Frage des Nutzers: Kann man Knöpfe anders aussehen lassen und animieren --
eine eigene Form, Animationen --, damit man GUIs bauen kann, die nicht nach
Standard aussehen (Spielmenüs, Kiosk, Markenoberflächen)?

## Was es vorher gab

Die Stildatei (`GUI_STYLESHEET`) setzt je Art, Klasse, Name und Zustand
Farben, `radius`, `randbreite`, `schatten`, `verlauf`, `glanz`, `fase` und
weiche Übergänge (`uebergang`). Dazu Sinnbilder und Bilder auf Knöpfen,
Knopfarten, Glas-Themen und der Lichtstreif über ein Fenster. Alles andere
ging nur, indem das Programm selbst auf eine Zeichenfläche malte -- ohne
Fokus, Tastatur und Bildschirmleser der gui.

## Stufe 1 (gebaut)

Alles über die Stildatei, damit ein Aussehen an EINER Stelle steht und für
viele Widgets gilt; dazu zwei Befehle für das Programm.

| Eigenschaft | Werte | gilt |
|---|---|---|
| `form` | `rechteck`, `pille`, `kreis`, `raute`, `bild` | Knöpfe; nur in Regeln ohne Zustand |
| `bild` | Pfad einer Bilddatei | Knöpfe; auch je Zustand (`:hover`, `:active`, `:disabled`, `:checked`) |
| `bildrand` | 0..200 Pixel des Bildes (9-Slice) | wie `bild`; 0 = gestreckt |
| `animation` | `puls`, `wackeln`, `schweben`, `huepfen`, `drehen` [Dauer ms] [`einmal`] | jedes Widget; auch je Zustand |

- **Form = Treffer.** Eine Form gilt für Zeichnen UND Treffertest (eine
  Quelle, `form_trifft`): ein Klick neben dem Kreis trifft nicht. `form:
  bild` nimmt die Deckkraft der Bildhaut: wo das Bild durchsichtig ist,
  geht der Klick vorbei -- so wird jede Form möglich, auch ein Stern.
- **Bildhaut.** Statt Fläche und Rand zeichnet der Knopf das Bild (mit
  `bildrand` als 9-Slice), Sinnbild und Text liegen darüber. Je Zustand ein
  eigenes Bild; geladen wird beim ersten Gebrauch.
- **Animation** ist eine Verschiebung, Drehung oder Vergrößerung des ganzen
  Widgets beim Zeichnen (eine Matrix um seine Mitte); Treffer und Lage
  bleiben, wo sie sind. Die Zeit läuft mit `DELTA()`. `button:hover {
  animation: puls 600ms }` pulsiert nur beim Überfahren.
- **Befehle:** `GUI_SET_SHAPE(wdg, form$)` / `GUI_GET_SHAPE$(wdg)` (eigene
  Form schlägt die Stildatei), `GUI_ANIMATE(wdg, art$ [, dauer_ms [,
  anzahl]])` -- etwa einmal wackeln bei falscher Eingabe; `""` hält an --
  und `GUI_ANIMATING(wdg)`.

## Stufe 2 (gebaut 2026-11-15)

- Bildfolgen aus einem Atlas: `bild: atlas.png 8 12fps` (als Bildhaut,
  nicht als Animation -- es ist dieselbe Mechanik wie das animierte GIF).
- Eigene Abläufe flach statt geschachtelt: `@ablauf name { 50%: y -8,
  drehen 10 }` -- verschieben, drehen, Größe; seit 2026-11-15 auch
  `deckkraft` und `farbe` über eine Schicht (Renderziel, mit Deckkraft und
  Ton aufgelegt) statt durch jeden Zeichenbefehl.
- Bildhäute für Kästchen, Optionen und Schalter (je Zustand). Formen
  (`form`) bleiben bei Knöpfen; Regler offen.

## Stufe 3 (gebaut 2026-11-15: `GUI_CUSTOM`)

- Selbst gezeichnete Widgets: das Programm malt (wie in eine Zeichenfläche,
  `GUI_CANVAS_BEGIN`), die gui führt Fokus, Tastatur, Klick und
  Bildschirmleser.
