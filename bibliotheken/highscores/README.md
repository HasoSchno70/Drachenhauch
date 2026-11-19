# highscores

The best results of a game, kept in a text file (one line per entry,
`points<TAB>name`, best first). A missing or broken file is an empty table.

```bash
dhrt paket hole github:HasoSchno70/Drachenhauch@main#bibliotheken/highscores
```

```basic
IMPORT "highscores/highscores.dh"
DIM best AS Highscores = NEW Highscores("best.txt", 10)
IF best.Qualifies(score) THEN best.Add("Ada", score)
best.Draw(20, 40)
```

| Member | What it does |
|---|---|
| `NEW Highscores(path, size = 10)` | open the table in `path`, keeping at most `size` entries |
| `Qualifies(points)` | would these points make it into the table? |
| `Add(name, points)` | enter and save; returns the place (1 = best), 0 if it did not make it |
| `Count()` | how many entries there are |
| `NameAt$(place)`, `PointsAt(place)` | name and points of a place (1 = best) |
| `Draw(x, y, color = &HFFFFFF)` | draw the table as text |
| `Load()`, `Save()` | read and write the file again |

Equal points: the older entry stays ahead.
