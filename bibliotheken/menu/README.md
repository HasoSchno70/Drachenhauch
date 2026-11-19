# menu

A title or pause menu: arrow keys or W/S move, Enter or Space choose,
Escape backs out.

```bash
dhrt paket hole github:HasoSchno70/Drachenhauch@main#bibliotheken/menu
```

```basic
IMPORT "menu/menu.dh"
DIM m AS Menu = NEW Menu("Dragon flight", ["Start", "Options", "Quit"])
' in the loop:
DIM choice AS INTEGER : choice = m.Update()
IF choice = 2 OR m.Cancelled() THEN EXIT(0)
m.Draw(160, 80)
```

| Member | What it does |
|---|---|
| `NEW Menu(title, items)` | a menu with a title and its entries |
| `Update()` | once per frame; the chosen entry (0 = the first) in the frame it is chosen, otherwise -1 |
| `Cancelled()` | was Escape pressed in this frame? |
| `Selected()`, `SetSelected(index)` | the marked entry |
| `SetColors(normal, active)` | colours of the entries and of the marked one |
| `Draw(x, y)` | title at (x, y), the entries below it |
