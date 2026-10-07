# Packages: fetching and sharing libraries

A library is a folder of `.dh` files. `dhrt paket` fetches it into your
project, remembers which version it was, and fetches exactly the same one
again on another computer.

```bash
dhrt paket hole github:hans/spielkiste@1.2
```

```basic
IMPORT "spielkiste/vektor.dh"
```

## Where packages come from

There is no dedicated server. A package comes from wherever it already lives:

| Source | Meaning |
|---|---|
| `github:nutzer/repo@stand` | the ZIP that GitHub delivers for a tag, branch or commit |
| `https://beispiel.de/kiste.zip` | any ZIP |
| `https://beispiel.de/hilfe.dh` | a single file |
| `../libs/werkzeug` | a local folder (relative to the project) |
| `../libs/werkzeug.zip` | a local ZIP |

**The version is mandatory.** `github:hans/spielkiste` without `@…` is an
error: “the latest version” would be a different one tomorrow. Best use a tag
(`@1.2`); a branch (`@main`) works, but keeps moving.

If everything in the ZIP sits inside a single top-level folder (that is how
GitHub delivers, `spielkiste-1.2/`), that folder is dropped.

## Where they end up

In the project, in `pakete/<name>/` next to the file `paket.json`. Every
project thus has its own versions. `<name>` is the last part of the source;
`--als` picks a different one:

```bash
dhrt paket hole ../libs/werkzeug --als wz
```

After looking next to the program, `IMPORT` searches `pakete/`, going upwards
from the importing file. The exact order is in the
[language reference](sprache.md#where-it-searches); in short:

1. next to the importing file (your own copy always wins),
2. `pakete/`, going upwards from there,
3. `DH_PATH`, then the library in the user folder.

## The two files

**`paket.json`** says what the project needs. You can read and change it by
hand:

```json
{
  "name": "mein-spiel",
  "pakete": {
    "spielkiste": "github:hans/spielkiste@1.2",
    "wz": "../libs/werkzeug"
  }
}
```

**`paket.lock.json`** records what was actually fetched: the source, the
address and a **SHA-256 checksum over the contents**. If something different
arrives on the next fetch (a tag was moved, a server delivers something
foreign, a local library was changed), that is an error, and the old package
stays in place. If you want the new contents:

```bash
dhrt paket hole --erneuern
```

The checksum covers the files, not the bytes of the ZIP. So if a server
repacks the same thing, it does not trip.

Both files belong in the repository. `pakete/` can be included or left out;
with the lock file, `dhrt paket hole` fetches exactly the same thing again.

## Commands

| Command | Effect |
|---|---|
| `dhrt paket hole` | fetch everything in `paket.json`, checked against the lock file |
| `dhrt paket hole --erneuern` | the same, accepting new contents |
| `dhrt paket hole <quelle> [--als name]` | add a package |
| `dhrt paket entferne <name>` | remove it from both files and from `pakete/` |
| `dhrt paket liste` | what the project has, with checksum |
| `dhrt paket neu [name]` | create an empty `paket.json` |

The project is the nearest folder with a `paket.json`, searched upwards from
the working folder. With `dhrt paket -C <ordner> ...`, `<ordner>` counts as
the working folder (as with git) -- that is how the IDE calls it.

## In the IDE

**File → Project packages** (Ctrl+Alt+P) shows what the project has, and
fetches, renews and removes packages through the same commands. Refactorings
across the whole project (rename, replace) and the project search leave
`pakete/` out: whatever lies there is overwritten by the next fetch. Details
in the [IDE manual](ide.md).

## Offering a package of your own

A folder of `.dh` files is enough, ideally a repository with tags. If the
package needs packages itself, it gets a `paket.json` of its own.
`dhrt paket hole` fetches them into **its** `pakete/`, and there the package
finds them first, even if the project has a different version of the same
package.

A local path in this `paket.json` is relative to the folder the package came
from. A downloaded package cannot refer to local paths. A cycle (A needs B, B
needs A) is an error.

## Security

Fetching **runs nothing**: no installation script, no build. Archives are
unpacked so that no entry gets out of its folder (no `..`, no absolute
paths).

A package is still foreign code, though: when running it may do everything a
Drachenhauch program may do, that is, write files and go onto the network.
The checksum protects against a package changing between two fetches, not
against a malicious original. Fetch packages from sources you trust.

## Export

`dhrt --export` compiles the program together with all imports. Packages are
thus inside the finished file, and the target computer needs neither
`pakete/` nor `dhrt paket`.

## Why it is built this way

The decisions (no server, packages in the project, checksum over the
contents) and what is deliberately missing are in the
[design document](../entwurf-pakete.md) (German).
