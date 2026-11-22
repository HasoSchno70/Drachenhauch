const { stueck, zeilen } = require("../code_lesen");

const M = "kap23/mediathek/";

module.exports = (H) => [
  H.chapter("Weitergeben", 23),

  H.h2("Was wir bauen"),

  H.p("Bis hierher lief die Mediathek aus dem Studio, mit F5. Die Freundin in Dublin hat aber kein Studio, und sie soll auch keins brauchen. In diesem Kapitel wird aus dem Projekt ein Programm, das man weitergibt: eine Exe mit allem, was sie braucht, ein Installer, der sie unter „Programme“ legt und ins Startmenü einträgt, und eine Mediathek, die dort richtig arbeitet — ihre Daten und Einstellungen liegen beim Benutzer, nicht neben dem Programm. Dazu ein eigenes Symbol und eine Fassungsnummer."),

  H.h2("Der Plan"),

  H.bulletRich("Bauen ", "macht aus dem Projekt eine Exe in bau/."),
  H.bulletRich("Die Datenbank ", "zieht in den Datenordner des Benutzers — ein Haken im Studio."),
  H.bulletRich("Einstellungen ", "merkt sich die Mediathek ebenda, zuerst das helle Aussehen."),
  H.bulletRich("Ein Symbol und eine Fassung ", "zeigen, welches Programm das ist und welcher Stand."),
  H.bulletRich("Ein Installer ", "bringt alles auf einen anderen Rechner."),

  H.h2("Schritt 1: bauen"),

  H.p("Ausführen → Bauen (Strg+B) erzeugt wie F5 den Ordner _studio/ und schreibt dann mit dhrt --export eine eigenständige Exe nach bau/ im Projektordner. Daneben legt es, was das Programm zur Laufzeit liest: unter assets/ die Stildatei, die Sprachtafeln und die Bilder, und die Datei der Datenbank — die aus dem Projektordner, mit den Daten, mit denen man gerade arbeitet, wenn neben dem Programm noch keine liegt. Bauen nimmt dabei die kleinste Laufzeit, der keiner der Befehle fehlt, die das Programm braucht, und sagt in der Ausgabe, welche."),

  H.p("bau/Mediathek.exe läuft jetzt mit einem Doppelklick, auf jedem Windows-Rechner, ohne Drachenhauch. Den Ordner bau/ könnte man so, wie er ist, in eine ZIP-Datei packen und verschicken."),

  H.h2("Schritt 2: Daten in den Benutzerordner"),

  H.p("Liegt die Mediathek aber unter „Programme“, scheitert sie am ersten Eintrag: Dort darf ein gewöhnliches Programm nichts schreiben. Die Datenbank gehört in den Datenordner des Benutzers. Im Reiter der Datenbank steht neben der Datei der Haken „Im Benutzerordner“:"),

  H.figure("kap23_benutzerordner.png", "Der Reiter der Datenbank Mediathek: Mit „Im Benutzerordner“ hält das gebaute Programm seine Datei beim Benutzer, nicht neben sich.", 480, 300),

  H.p("Mit dem Haken öffnet das gebaute Programm die Datei in DIR_USER_DATA$(\"Mediathek\") — unter Windows %APPDATA%\\Mediathek, unter macOS ~/Library/Application Support/Mediathek, unter Linux ~/.local/share/Mediathek. Beim ersten Start gibt es dort noch keine; dann kopiert es die mit, die neben ihm liegt. Danach arbeitet es nur noch auf der beim Benutzer. Aus dem Studio gestartet bleibt alles, wie es war: F5 arbeitet auf der Datei im Projektordner."),

  H.h2("Schritt 3: Einstellungen merken"),

  H.p("Wer „Helles Aussehen“ wählt, will es beim nächsten Start wieder haben. Ein neues Modul Einstellungen legt eine kleine Datei daneben, mit dem Modul save — Schlüssel und Werte, ohne SQL:"),

  H.code(stueck(M + "Einstellungen.dh", "FUNCTION einstellungsDatei$", "END FUNCTION")),

  H.code(stueck(M + "Einstellungen.dh", "FUNCTION einstellung(", "END FUNCTION")),

  H.code(stueck(M + "Einstellungen.dh", "SUB einstellungMerken", "END SUB")),

  H.p("Den Wert liest App.Geoeffnet, bevor das erste Fenster aufgeht; der Menüeintrag schreibt ihn, sobald jemand den Haken setzt:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miHell_Aktion", "END SUB")),

  H.h2("Schritt 4: Symbol und Fassung"),

  H.p("Ein Programm, das man weitergibt, sollte man in der Taskleiste erkennen. Das Symbol ist ein Bild von 64 × 64 Punkten, bilder/symbol.png im Projektordner; App.Geoeffnet gibt es dem Fenster:"),

  H.code(stueck(M + "App.dh", "CLASS App", "END CLASS")),

  H.p("WINDOW_ICON setzt das Symbol des Fensters und der Taskleiste. Die Exe selbst zeigt im Explorer das gewöhnliche Symbol eines Programms; das Symbol einer Datei steckt in ihr selbst, und Drachenhauch schreibt keins hinein."),

  H.p("Die Fassung ist eine Konstante im Modul Einstellungen, CONST FASSUNG = \"1.0\", und Hilfe → „Über die Mediathek …“ zeigt sie. Wer einen Fehler meldet, kann sagen, mit welchem Stand:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miUeber_Aktion", "END SUB")),

  H.h2("Schritt 5: ein Installer"),

  H.p("Einen Installer baut man unter Windows am bequemsten mit Inno Setup, einem freien Werkzeug. Es liest ein Skript, das sagt, was wohin kommt — mediathek.iss im Projektordner:"),

  H.code(zeilen(M + "mediathek.iss"), { out: true }),

  H.p("ISCC mediathek.iss (oder ein Doppelklick auf die Datei und Compile im Programm Inno Setup) schreibt installer\\Mediathek-1.0-Setup.exe. {autopf} ist der Ordner „Programme“; PrivilegesRequiredOverridesAllowed=dialog lässt den Benutzer wählen, ob er für alle installiert — dann fragt Windows nach dem Administrator — oder nur für sich, ohne besondere Rechte. Der Installer legt alles aus bau/ dorthin, einen Eintrag ins Startmenü und auf Wunsch einen auf den Desktop, und unter „Apps“ steht die Mediathek zum Deinstallieren."),

  H.h2("Warum so?"),

  H.h3("Warum die Datenbank nicht neben dem Programm liegt"),

  H.p("Der Ordner „Programme“ ist für Programme da, nicht für Daten. Ein gewöhnlicher Benutzer darf dort nichts ändern — das schützt die Programme vor Viren und vor Versehen. Und er gehört allen Benutzern des Rechners gemeinsam: Lägen die Daten dort, sähe jeder die Sammlung des anderen."),

  H.p("Dazu kommt das nächste Update. Ein neuer Installer ersetzt, was im Programmordner liegt; läge dort die Datenbank, wäre die Sammlung danach wieder die vom Entwickler. Im Benutzerordner bleibt sie, wie sie ist, über jedes Update und jede Deinstallation hinweg, und eine Datensicherung des Benutzerordners erfasst sie mit."),

  H.h2("Was schiefgehen kann"),

  H.h3("Zwei Datenbanken"),

  H.p("„Ich habe doch eben einen Eintrag angelegt!“ — aber mit F5, und der steht in der Datei im Projektordner. Das gebaute Programm arbeitet auf der im Benutzerordner. Das ist gewollt: Die Entwicklung soll die Daten des echten Gebrauchs nicht anfassen. Die mitgelieferte Datei neben dem Programm zählt nur beim allerersten Start."),

  H.h3("Zwei Fassungsnummern"),

  H.p("Die Fassung steht zweimal, in Einstellungen.dh und in mediathek.iss. Wer eine ändert, ändert die andere mit — sonst sagt „Über die Mediathek“ 1.1, und der Installer heißt 1.0."),

  H.h3("Windows warnt"),

  H.p("Eine Exe aus dem Internet, die niemand signiert hat, hält Windows SmartScreen beim ersten Start an, mit einer Warnung vor einem unbekannten Herausgeber. Wer weitergibt, sagt das dazu; wer viel weitergibt, kauft ein Zertifikat und signiert Exe und Installer — mit dem eigenen Zertifikat, denn signieren kann nur, wer die Datei gebaut hat."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm den Haken „Im Benutzerordner“ weg, baue und leg in bau/Mediathek.exe einen Eintrag an. In welcher Datei steht er?"),
  H.bulletRich("Drehen 2. ", "Lösch einstellungen.save im Benutzerordner der Mediathek. Wie startet sie?"),
  H.bulletRich("Drehen 3. ", "Setz FASSUNG auf \"1.1\", baue und übersetze den Installer, ohne mediathek.iss anzufassen. Was steht wo?"),
  H.bulletRich("Erweitern 1. ", "Ansicht → „Große Schrift“: ein Haken, den sich die Mediathek merkt, mit GUI_SCALE aus Kapitel 21 — ab dem nächsten Start."),
  H.bulletRich("Erweitern 2. ", "Beim Start eine Mitteilung des Systems, wenn Ausleihen überfällig sind."),
  H.bulletRich("Erfinden. ", "Gib deine Sammlung aus Kapitel 13 weiter: bauen, Daten in den Benutzerordner, ein Installer. Was muss sie sich merken?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Starten und Bauen, Datenbanken: ", "was in bau/ landet, „Im Benutzerordner“."),
  H.bulletRich("Handbuch, Standard-Built-ins → Datei-I/O: ", "DIR_USER_DATA$; Modul save für die Einstellungen."),
  H.bulletRich("Handbuch, Native Rust-Runtime → Standalone-Export: ", "die schlanken Laufzeiten, Signieren."),
  H.bulletRich("Inno Setup: ", "jrsoftware.org, mit einer ausführlichen Hilfe zu jedem Abschnitt des Skripts."),
];
