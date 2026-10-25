# Class diagram

`examples/210_klassendiagramm.dh` shows all classes and structs of a
project as boxes: the name on top, below it fields, constants, properties
and methods with their type. Inheritance is an arrow with a hollow triangle
to the ancestor (as in UML), a field whose type is another class (also
`ARRAY OF` or `MAP OF`) a dashed line to it.

```
dhrt run examples/210_klassendiagramm.dh -- <projektordner oder datei>
```

In the IDE: **Tools → Class diagram** (takes the project folder).

## Usage

| What | How |
|---|---|
| Move a box | drag it -- the position is remembered (`.klassendiagramm.json` in the project folder) |
| Move the view | drag on empty space, or the mouse wheel |
| Jump to the code | double-click a box: on a member its line, otherwise the header line of the class |
| Read again | F5 |
| Arrange again | L -- levels by inheritance, the remembered positions are overwritten |
| Save as image | P -- the whole diagram as `klassendiagramm.png` in the project folder |

The jump goes through the tool's output: it writes a line
`DH_IDE_OEFFNEN|pfad|zeile`, and the IDE that started it opens the file
there. Every tool can do that; started on its own the line only shows in
the console.

## What it reads

The classes come from `CODE_CLASSES$`, without compiling -- a file with an
error still makes it into the picture. Folders starting with `.` or `_`, as
well as `pakete/` and `target/`, stay out, as with the IDE's refactorings.
If the same class name appears in two files, the first one counts. An
ancestor the project does not know is shown small above the box.

Tested in `tests/pruef/werkzeug_klassendiagramm.dhtest`.
