# Module `xml`

Reading XML — invoices, export lists, GPX tracks, SVG, the response of an
older web interface.

```basic
IMPORT "xml"
```

## Read-only — and why

With JSON, **writing** was the gap (item 2 of the audit): a program builds
JSON bodies for REST interfaces all the time. With XML it is the other way
round. The case is almost always "data comes from a foreign system"; building
an XML tree yourself is the exception.

If you have to write XML anyway, glue it together and protect the values with
`XML_ESCAPE$` — it replaces exactly the five characters on which hand-made
XML otherwise breaks:

```basic
text = "<name>" + XML_ESCAPE$(kunde) + "</name>"
```

## Overview

| Function | Purpose |
|---|---|
| `XML_PARSE(text$)` → XML_HANDLE | from a string |
| `XML_LOAD(pfad$[, kodierung$])` → XML_HANDLE | from a file |
| `XML_NAME$(k)` → STRING | the name of the element |
| `XML_TEXT$(k[, pfad$])` → STRING | the text (including that of the children) |
| `XML_ATTR$(k, name$[, vorgabe$])` → STRING | an attribute |
| `XML_HAS(k, pfad$)` → BOOLEAN | does the path exist? |
| `XML_FIND(k, pfad$)` → XML_HANDLE | jump there |
| `XML_COUNT(k, pfad$)` → INTEGER | how many with the same name? |
| `XML_AT(k, pfad$, i)` → XML_HANDLE | the i-th of them (0-based) |
| `XML_CHILD_COUNT(k)` / `XML_CHILD(k, i)` | walk through an unknown tree |
| `XML_ATTR_NAMES(k)` → ARRAY OF STRING | all attribute names |
| `XML_ESCAPE$(text$)` → STRING | replace the five special characters |

## Paths

As with JSON, only with `/` instead of `.` — that is how it is written in every XML example:

```basic
IMPORT "xml"

DIM d AS XML_HANDLE
DIM p AS XML_HANDLE
DIM i AS INTEGER

d = XML_LOAD("rechnung.xml")
PRINT XML_ATTR$(d, "nr")                  ' attribute of the root
PRINT XML_TEXT$(d, "kunde")               ' text of a child

FOR i = 0 TO XML_COUNT(d, "posten/p") - 1
    p = XML_AT(d, "posten/p", i)
    PRINT XML_ATTR$(p, "menge") + "x " + XML_TEXT$(p)
NEXT
```

**If a name occurs more than once, `XML_FIND`/`XML_TEXT$` take the first.** For
all the others there are `XML_COUNT` and `XML_AT`. A path that guessed would
be the surest way to get something different on the second record.

**A missing path is an error for `XML_FIND`**, and for `XML_HAS` the answer
`FALSE` — ask first, then jump. A missing **attribute**, on the other hand, is
not an error but returns the default: a foreign file leaves out what it does
not need, and that is the normal case.

## What is read

* elements, attributes (both kinds of quotes), text
* self-closing elements `<b/>`
* `<?xml …?>`, `<!-- … -->`, `<!DOCTYPE …>` are skipped
* `<![CDATA[…]]>` literally (nothing is resolved **there**)
* the five entities `&lt; &gt; &amp; &quot; &apos;` as well as `&#65;` and `&#x41;`

**Mixed content keeps its order.** `<p>Hallo <b>schöne</b>
Welt</p>` yields exactly `Hallo schöne Welt` with `XML_TEXT$`. That sounds
obvious, but it is not: anyone who stores text and child elements separately
gets `Hallo  Weltschöne` — the bug never shows up with data XML and
immediately with running text.

**Namespaces stay in the name.** `<ns:titel>` is called `ns:titel` here, the
`xmlns` is an ordinary attribute. Real namespace resolution needs a scope per
element and answers a question nobody asks when reading out a known file.

**Reading is strict** — unlike the [`ini`](module-ini.md) module next door,
which skips broken lines. An INI file is edited by a human; an XML file comes
from another program. An unclosed element there is not a typo but usually a
sign that the transfer was cut off — and then silently reading on is the
worst answer. The message names the line.

The **encoding** from [text encoding](builtins-core.md#text-encoding) applies
here too (`XML_LOAD(pfad, "cp1252")`); the `encoding=` attribute in the
XML declaration is **not** evaluated.

Example: [examples/175_xml.dh](../../examples/175_xml.dh).
