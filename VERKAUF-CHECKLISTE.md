# GameBasic verkaufen – Checkliste

Stand der Vorbereitung, damit GameBasic als **kommerzielles Windows-Produkt**
verkauft werden kann. Aufgeteilt in „technisch erledigt" (im Repo gebaut) und
„noch von dir zu erledigen" (kein Code – Zertifikat, Recht, Texte).

> ⚠️ Keine Rechtsberatung. EULA, LGPL-Detailauflagen und Verbraucherrecht vor
> einem ernsthaften Verkauf einmal juristisch gegenlesen lassen.

---

## ✅ Technisch erledigt (im Repo)

| Baustein | Wo | Zweck |
|---|---|---|
| **Windows-Installer** (komplette IDE + Runtime, **ohne Python**) | `installer/` (`build_installer.py`, `GameBasic.spec`, `GameBasic.iss`) | Endkunden installieren per Doppelklick |
| **Beispiele am beschreibbaren Ort** | `{commondocs}\GameBasic\examples` | Editor findet Beispiele + Showcase-Bilder, Demos können schreiben |
| **Editor startet direkt** (kein Auswahlfenster) + findet `dhrt` | `dhrun.py`, `editor_qt/dhrt_locate.py` | „Ausführen" funktioniert in der Installation |
| **EULA** (Endbenutzer-Lizenzvertrag, Vorlage) | `installer/EULA.txt` | Zustimmungsseite im Setup; Nutzer besitzen + verkaufen ihre Spiele |
| **Drittanbieter-Lizenzen** (auto-generiert) | `installer/gen_notices.py` → `THIRD-PARTY-NOTICES.txt` | Pflicht-Beilage für MIT/BSD/Apache/LGPL |
| **Asset-Lizenzen geprüft + dokumentiert** | `examples/ASSET-CREDITS.md` | alles eigen/CC0/CC-BY (mit Attribution) |
| **„Mario" → IP-sicherer Plattformer-Satz** | `examples/platformer/` | kein Nintendo-Marken-/Urheberrechtsrisiko |
| **Code-Signing-Hook** (inert bis Zertifikat) | `installer/build_installer.py` (`sign()`) | signiert App/Runtime/Installer automatisch, sobald Zertifikat da |

**Installer bauen:** `\.venv\Scripts\python.exe installer\build_installer.py`
→ `installer/output/GameBasic-Setup-<version>.exe`. Details: `installer/README.md`.

---

## ⬜ Noch von dir zu erledigen (kein Code)

### 1. Code-Signing-Zertifikat kaufen  ⭐ wichtigster Punkt
Ohne Signatur zeigt Windows bei **jedem** Käufer „Unbekannter Herausgeber"
(SmartScreen) – wirkt unseriös und schreckt ab.
- Anbieter z. B. Sectigo, DigiCert, GlobalSign. **OV** = günstiger; **EV** =
  teurer, baut aber **sofort** SmartScreen-Reputation auf (sonst dauert das, bis
  genug Downloads gesammelt sind).
- Danach signieren (Hook ist fertig, `signtool` ist auf dem Rechner):
  ```
  set GB_SIGN_CERT=C:\keys\meincert.pfx     (oder SHA1-Thumbprint bei EV-USB-Token)
  set GB_SIGN_PASS=geheim
  \.venv\Scripts\python.exe installer\build_installer.py
  ```

### 2. EULA fertigstellen
`installer/EULA.txt` ist eine **Vorlage** – ersetze die `[PLATZHALTER]`:
Anbietername/Firma, Anschrift, E-Mail, Jahr, Gerichtsstand, Lizenzmodell
(pro Person / pro Gerät / …). Danach juristisch prüfen lassen.

### 3. Beim Verkauf an Verbraucher in DE/EU
- **Impressum** + **Datenschutzerklärung** (DSGVO) auf der Verkaufsseite.
- **Widerrufsbelehrung** (bei digitalen Produkten i. d. R. mit Verzichts-
  Zustimmung auf das Widerrufsrecht beim Download).
- Korrekte **Umsatzsteuer** (Kleinunternehmer §19 UStG oder regulär; bei
  EU-Endkunden OSS-Verfahren). Mit Steuerberater klären.

### 4. Vertriebsweg wählen
- Eigene Seite (z. B. mit Lemon Squeezy / Paddle / Gumroad → die übernehmen oft
  USt./Rechnungen als „Merchant of Record"), oder
- Plattform (itch.io, Microsoft Store – Store verlangt eigene Zertifizierung).

### 5. Vor dem ersten Verkauf einmal testen
- Setup auf einem **frischen** Windows (ohne Python/Dev-Tools) installieren.
- Editor starten, ein Beispiel laufen lassen, ein eigenes Spiel exportieren
  (`dhrt --export`) und die exportierte `.exe` auf einem anderen PC starten.

---

## Wichtige Dauer-Auflagen (nicht vergessen)
- **Nicht `--onefile` bauen** – die Qt-DLLs müssen austauschbar bleiben (LGPL).
  Die `onedir`-Konfiguration erfüllt das bereits.
- **`THIRD-PARTY-NOTICES.txt` + `examples/ASSET-CREDITS.md` mitliefern** (tut der
  Installer). Bei `cybermatic_pulse.ogg` (CC-BY) bleibt die Autorennennung Pflicht.
- Bei jedem Release `build_installer.py` laufen lassen – Notices werden frisch
  generiert, Signatur (falls konfiguriert) automatisch angewandt.

---

## Schnell-Status
**Verkaufsfertig, sobald:** Zertifikat gekauft + EULA-Platzhalter gefüllt +
(bei DE-Endkunden) Impressum/Widerruf/Steuer geklärt. Alles Technische steht.
