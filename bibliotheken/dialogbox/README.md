# dialogbox

A text box that types its text letter by letter, as in a role-playing game.
Enter or Space shows the whole page at once, then turns to the next one.
Lines wrap at spaces to fit the box.

```bash
dhrt paket hole github:HasoSchno70/Drachenhauch@main#bibliotheken/dialogbox
```

```basic
IMPORT "dialogbox/dialogbox.dh"
DIM box AS DialogBox = NEW DialogBox(20, 200, 440, 100)
box.Say("The dragon wakes up.")
box.Say("It is hungry.")
' in the loop:
box.Update(DELTA())
box.Draw()
```

| Member | What it does |
|---|---|
| `NEW DialogBox(x, y, w, h)` | a box at (x, y) of size w × h |
| `Say(text)` | queue a page |
| `Update(dt)` | once per frame with `DELTA()`; types on and reacts to Enter/Space |
| `Advance()` | what Enter does: show the whole page, or turn to the next one |
| `Done()` | nothing left to say? |
| `Visible$()` | the part of the current page that is visible right now |
| `SetSpeed(lettersPerSecond)`, `SetColors(text, back)` | speed (40 by default) and colours |
| `Draw()` | the box with its text |
