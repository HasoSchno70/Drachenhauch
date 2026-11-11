# Drachenhauch 2026.46

*The notes for this version. It is the version **for tidying up and handing
on**: three large programs have their own repositories now, the repository has
a clear layout, three security holes in dependencies are fixed, and a release
is built from a single tag. Plus a start screen that lets you choose between
the IDE and the Studio. What existing programs may notice comes first.*

## What changes for existing programs

* **Glutfaden, Funkenflug and the fridge app are no longer in this repository
  and no longer in the installer.** They have their own repositories:
  [Glutfaden](https://github.com/HasoSchno70/Glutfaden) (cross-stitch editor),
  [Funkenflug](https://github.com/HasoSchno70/Funkenflug) (schematic editor) and
  [Kuehlschrank-Drachenhauch](https://github.com/HasoSchno70/Kuehlschrank-Drachenhauch)
  (Studio project), each with its full history and under the same licence.
  They run with this version as before.
* **The form designer (`examples/197_form_designer.dh`) is gone**; Drachenhauch
  Studio replaces it. A `.dhform` in the IDE's project tree now opens in the
  Studio, and the installer removes the old shortcut.
* **Games and books have folders of their own:** `spiele/` (Pyramid Pusher,
  Circuit Runner, Voidreaver, Restaurant, Fireplace) and `buecher/`
  (`referenz`, `einstieg`, `galaga`, `tippspiel` instead of `buch-*`). Starting
  them from the source tree takes the new path, for example
  `dhrt run spiele/pyramid_pusher/pyramid_pusher.dh`. The vocabulary lists of
  the beginner's book stay under `buch-einstieg/vokabellisten/`, so the address
  printed in books already handed out keeps working.
* **Physics (`PHYS2D_*`, `PHYS3D_*`) runs on Rapier 0.36 instead of 0.22.**
  Trajectories may differ in the last digits. A stack created asleep still
  rests until the first push, but settles by a little over a millimetre in the
  very first step, where it used to stay exactly in place.

## New in this version

### IDE and Studio

* **Start screen:** after the intro the IDE asks whether to continue the
  *classic* way in the IDE or the *modern* way in the Studio (mouse, arrows,
  Enter, 1 and 2). "Don't ask again" remembers the choice; change it back in
  the settings or by holding Shift at start.
* **Open menus and drop-down lists sit above all windows**, including a design
  window in the Studio. Before, the "View" menu lay underneath it, and a click
  on the hidden part still picked the entry.
* **`PROCESS_START_DETACHED(program$, ...)`** starts a program that keeps
  running when the caller ends (a child from `PROCESS_START` ends with it). The
  IDE uses it to start the Studio.

### Security

* **Three holes in dependencies fixed**, found by the newly set up
  `cargo audit`:
  * the bundled **SQLite** (`libsqlite3-sys`, RUSTSEC-2026-0459) -- via
    `rusqlite` 0.41,
  * the bundled **root certificates** (`webpki-roots`, RUSTSEC-2026-0381)
    trusted intermediate certificates that browsers now distrust,
  * `ctutils` (RUSTSEC-2026-0385), comparisons not in constant time.
* **Security problems can be reported privately**: via "Report a
  vulnerability" on the repository's *Security* tab (see `SECURITY.md`).

### Downloading and contributing

* **Every release comes with a `SHA256SUMS.txt`** with the checksums of all
  files, and the three big books (German textbook, English handbook, beginner's
  book as `.docx` and `.epub`) as downloads of their own.
* `CONTRIBUTING.md`, `SECURITY.md`, a code of conduct and forms for bug reports
  and wishes; the README shows a hello world without extra files, how to build
  on Windows, macOS and Linux, and all four books. Origin and licence of intro,
  logo and fonts are in `daten/HERKUNFT.md`.

## Under the hood

**Numbers.** 2433 commands (previously 2432) in 57 modules, 233 examples at the
top level of `examples/` (234; the form designer is gone),
**5849 cases in 480 test collections** (6068 in 518 -- the 40 collections of
Glutfaden and the fridge app moved with them) and 670 Rust test functions (670).

* **A release is built from a tag:** `.github/workflows/release.yml` builds the
  Windows installer, the packages for macOS and Linux and the checksums and
  creates a draft release. Until now the installer was only built on the
  author's computer.
* **Dependencies up to date**: Rapier 0.36, the hash crates together on
  `digest` 0.11 (`sha2`, `sha1`, `md-5`, `hmac`), `gif` 0.14, `libloading` 0.9,
  `tokio-postgres-rustls` 0.14, `webpki-roots` 1.0 and 16 smaller ones.
  Dependabot now proposes updates monthly in bundles, `cargo audit` checks
  every Monday.
* **The website and its playground rebuild** whenever the runtime changes --
  before, only when `Cargo.toml` changed.
* **`docs/` is just the manual now**: design notes and roadmaps live in
  `docs/intern/`, the release notes in `docs/releases/`.
* Tidied up: a private sales checklist, leftover save games, a duplicated
  4.9 MB image, the last `gb` names and traces of Python.

## Still open

* **Real devices are largely untested** (Raspberry Pi, SPI, camera,
  microphone, Arduino, Modbus) -- feedback with hardware is welcome.
* **Full text shaping** (ligatures of other fonts, Indic scripts) is not yet
  available in the gui.
* **raylib 6.2** is out, but raylib-rs has no version for it yet.
* The macOS package is not notarised and the Windows installer not signed
  (SmartScreen warns when downloading).
* The finished books and the intro videos are still in the repository (about
  40 MB); in future they should only be attached to the release.
