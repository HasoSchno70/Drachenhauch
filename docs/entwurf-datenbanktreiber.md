# Untersuchung: PostgreSQL und MySQL als Module

*Untersuchung, keine Umsetzung.* Drachenhauch spricht heute genau eine
Datenbank: SQLite, über das Modul [`db`](module-db.md). Für ein Programm auf
einem Rechner reicht das vollständig. Sobald aber **zwei Leute gleichzeitig**
mit denselben Daten arbeiten sollen — die Kassiererin vorn, die Buchhaltung
hinten —, endet SQLite: es ist eine Datei, kein Server.

Die Frage lautet also nicht „ist SQLite gut genug" (ist es, für seinen Fall),
sondern: **was kostet ein zweiter Treiber?**

## Gemessen, nicht geschätzt

Alle Zahlen von diesem Rechner, 24.08.2026, jeweils frischer Bau
(`rm -rf target && cargo build --release`, `lto = true`):

| | Kisten | Bauzeit | Programmgröße |
|---|---|---|---|
| leeres Rust-Programm | 0 | — | 110 KB |
| `postgres` 0.19 (synchron) | 61 | 7 s | **1,09 MB** |
| `mysql` 26 (`minimal-rust`) | 90 | 23 s | **2,23 MB** |
| `dhrt` heute (voller Bau) | — | ~90 s | 16,3 MB |

Ein Postgres-Treiber wüchse `dhrt` also um rund **1 MB (+6 %)**, MySQL um
rund **2 MB (+13 %)**. Beide bringen **keine C-Toolchain** mit — kein cmake,
kein `cc`, kein bindgen. Das ist die Hürde, an der `rusqlite` nur deshalb
vorbeikommt, weil es seinen SQLite-Quelltext mitliefert.

## Verschlüsselung: hier trennen sich die beiden

Eine Datenbank im Netz ohne TLS ist keine Option — dort gingen Kennwort und
Daten im Klartext über die Leitung, dieselbe Überlegung wie beim Modul
[`smtp`](module-smtp.md).

* **PostgreSQL:** `tokio-postgres-rustls` 0.13 löst sich auf **rustls 0.23 +
  ring** auf — Fassung für Fassung dieselben Kisten, die über `ureq` (Modul
  `html`) und `smtp` **ohnehin schon im Baum liegen**. Kostet also nichts
  Zusätzliches und bleibt reines Rust.
* **MySQL:** die Kiste `mysql` zieht mit ihrem `rustls-tls`-Schalter
  **`aws-lc-sys`** nach — eine C- und Assembler-Bibliothek, die cmake und
  einen C-Übersetzer verlangt. Ohne TLS bleibt `mysql` reines Rust, mit TLS
  nicht. Damit steht MySQL vor genau der Wand, um die dieses Projekt seit
  jeher herumbaut.

Das ist der deutlichste Unterschied zwischen beiden, und er kommt nicht aus
dem Geschmack, sondern aus dem Abhängigkeitsbaum.

## Was ein Modul `pg` bräuchte

Die vorhandene `db`-API ist der Bauplan — sie ist bewusst schlank und passt
fast unverändert:

```basic
IMPORT "pg"
DIM c AS PG_CONN
c = PG_OPEN("host=server user=hans password=... dbname=laden")
PG_EXEC(c, "INSERT INTO kunde (name, ort) VALUES ($1, $2)", "Meier", "Köln")
DIM r AS PG_RESULT
r = PG_QUERY(c, "SELECT name, umsatz FROM kunde WHERE ort = $1", "Köln")
WHILE PG_NEXT(r)
    PRINT PG_GET_STRING(r, 0), PG_GET_FLOAT(r, 1)
WEND
PG_CLOSE(c)
```

Vier Dinge, die dabei **nicht** wie bei SQLite sind und die Arbeit ausmachen:

1. **Platzhalter heißen `$1`, `$2`**, nicht `?`. Eine Übersetzung wäre
   möglich, aber sie würde in Zeichenketten mit `?` hineinschneiden; besser
   die echte Schreibweise, dokumentiert.
2. **Typen sind streng.** SQLite nimmt fast alles entgegen; Postgres will
   wissen, ob `int4`, `numeric` oder `text` gebunden wird. Die Zuordnung
   Drachenhauch↔Postgres ist die eigentliche Fleißarbeit — samt der Frage,
   was mit `numeric` (Geld!) passiert, siehe
   [Entwurf: Geld](entwurf-geldtyp.md).
3. **Die Verbindung kann wegbrechen.** Eine Datei tut das nicht. Jeder Aufruf
   braucht eine verständliche Fehlermeldung, und es braucht eine Antwort auf
   „ist die Verbindung noch da".
4. **Es blockiert.** `postgres` ist synchron (mit `tokio` innen drin); eine
   Abfrage über das Netz hält die Hauptschleife an. Dafür gibt es das Muster
   schon: `DB_QUERY_START`/`DB_QUERY_READY` (Modul `db`, Hintergrundaufträge)
   müsste es entsprechend geben.

Aufwand geschätzt: in der Größenordnung des `db`-Moduls plus Punkt 2 und 4 —
also deutlich mehr als `ini`, deutlich weniger als `gui`.

## Was dagegen spricht

* **Zielgruppe.** Drachenhauch ist für Bastler und für Software auf dem
  eigenen Rechner gedacht ([Leitbild](../README.md)). Wer einen
  PostgreSQL-Server betreibt, hat meist auch schon Werkzeuge dafür.
* **Ein Treiber ist ein Versprechen.** Fassungen, Typen, TLS, Zeitzonen — das
  wird gepflegt oder es verrottet. `db` gegen SQLite ist stabil, weil SQLite
  mitgeliefert wird; ein Netz-Treiber hängt an einem fremden Server.
* **Es gibt einen Umweg**, der heute schon funktioniert: das Modul
  [`html`](module-html.md) spricht mit einer HTTP-Schnittstelle vor der
  Datenbank (PostgREST, ein eigener kleiner Dienst, `cloudserver/`). Für
  „mehrere Arbeitsplätze auf denselben Datenbestand" ist das oft ohnehin die
  bessere Bauform, weil die Regeln dann an einer Stelle liegen.

## Was dafür spricht

* **Der Schritt vom Einzelplatz zum Netz ist heute eine Wand**, nicht eine
  Stufe. Wer mit `db` anfängt und dann zu zweit arbeiten will, muss die
  Sprache verlassen.
* **Postgres kostet erstaunlich wenig**: 1 MB, 61 Kisten, reines Rust, TLS
  aus dem, was schon da ist. Das ist billiger als `physics3d` (Rapier) und
  billiger als das Audio-Backend.
* **Die API ist schon entworfen** — `db` ist der Bauplan, und ihre
  Hintergrundaufträge lösen das Blockierproblem bereits.

## Empfehlung

**PostgreSQL: ja, wenn ein konkreter Anlass da ist. MySQL: nein.**

* Für **MySQL** ist der Preis falsch herum: doppelt so groß wie Postgres, und
  mit TLS bricht es die Zusage „kein C-Übersetzer nötig". Ohne TLS wäre es
  ein Modul, das man im Netz nicht benutzen darf — dann lieber keins.
* Für **PostgreSQL** stimmen die Zahlen. Es fehlt nicht die Machbarkeit,
  sondern der Anlass: bisher hat niemand danach gefragt. Ein Modul zu bauen,
  das keiner braucht, kostet nichts an Größe, aber dauerhaft an Pflege.

Vorschlag deshalb: **liegen lassen, bis jemand es braucht** — und diesen
Zettel behalten, damit die Frage dann nicht neu untersucht werden muss. Die
Messungen oben sind der eigentliche Ertrag dieser Untersuchung.
