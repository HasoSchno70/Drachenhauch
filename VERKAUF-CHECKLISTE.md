# Drachenhauch verkaufen – Checkliste

Stand der Vorbereitung, damit Drachenhauch als **kommerzielles Windows-Produkt**
verkauft werden kann. Aufgeteilt in „technisch erledigt" (im Repo gebaut) und
„noch von dir zu erledigen" (kein Code – Zertifikat, Recht, Texte).

> ⚠️ Keine Rechtsberatung. EULA, LGPL-Detailauflagen und Verbraucherrecht vor
> einem ernsthaften Verkauf einmal juristisch gegenlesen lassen.

---

## ✅ Technisch erledigt (im Repo)

| Baustein | Wo | Zweck |
|---|---|---|
| **Windows-Installer** (IDE + Runtime + Werkzeuge, **ohne Python**) | `installer/` (`bauen.dh`, `Drachenhauch-IDE.iss`) | Endkunden installieren per Doppelklick |
| **Beispiele am beschreibbaren Ort** | `{commondocs}\Drachenhauch\examples` | Editor findet Beispiele + Showcase-Bilder, Demos können schreiben |
| **IDE startet direkt** aus dem Startmenü | `installer/Drachenhauch-IDE.iss` (`dhrt.exe run ide\ide.dh`) | „Ausführen" (F5) funktioniert in der Installation |
| **EULA** (Endbenutzer-Lizenzvertrag, Vorlage) | `installer/EULA.txt` | Zustimmungsseite im Setup; Nutzer besitzen + verkaufen ihre Spiele |
| **Drittanbieter-Lizenzen** (auto-generiert) | `installer/lizenzen.dh` → `THIRD-PARTY-NOTICES-IDE.txt` | Pflicht-Beilage für MIT/BSD/Apache/MPL |
| **Asset-Lizenzen geprüft + dokumentiert** | `examples/ASSET-CREDITS.md` | alles eigen/CC0/CC-BY (mit Attribution) |
| **„Mario" → IP-sicherer Plattformer-Satz** | `examples/platformer/` | kein Nintendo-Marken-/Urheberrechtsrisiko |
| **Code-Signing-Hook** (inert bis Zertifikat) | `installer/bauen.dh` (`signieren`) | signiert Runtime und Installer automatisch, sobald Zertifikat da |

**Installer bauen:** `dhrt run installer\bauen.dh`
→ `installer/output/Drachenhauch-IDE-Setup-<fassung>.exe`. Details: `installer/README.md`.

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
  set DH_SIGN_CERT=C:\keys\meincert.pfx     (oder SHA1-Thumbprint bei EV-USB-Token)
  set DH_SIGN_PASS=geheim
  dhrt run installer\bauen.dh
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
- Bei jedem Release `dhrt run installer/bauen.dh` laufen lassen – Notices werden frisch
  generiert, Signatur (falls konfiguriert) automatisch angewandt.

---

## Schnell-Status
**Verkaufsfertig, sobald:** Zertifikat gekauft + EULA-Platzhalter gefüllt +
(bei DE-Endkunden) Impressum/Widerruf/Steuer geklärt. Alles Technische steht.
