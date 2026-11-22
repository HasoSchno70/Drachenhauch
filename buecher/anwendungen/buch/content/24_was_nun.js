const { stueck } = require("../code_lesen");

const M = "kap24/mediathek/";

module.exports = (H) => [
  H.chapter("Was nun?", 24),

  H.p("Die Mediathek ist fertig: Fenster, Datenbank, Berichte, Diagramme, zwei Sprachen, ein Installer. Dieses letzte Kapitel baut noch ein kleines Stück dazu — die Mediathek als Web-Seite im Heimnetz — und zeigt dann, wohin es von hier aus geht: mehrere Benutzer, eigene Bausteine für das nächste Projekt, und wo man zeigt, was man gebaut hat."),

  H.h2("Die Mediathek im Heimnetz"),

  H.p("Jonas will wissen, ob Metropolis gerade da ist, bevor er fragt. Die Mediathek kann ihm eine Seite zeigen, die jeder Browser im selben Netz öffnet. Das Modul httpd ist ein kleiner Webserver, der im Takt des Programms läuft; ein neues Modul Netzseite benutzt ihn:"),

  H.code(stueck(M + "Netzseite.dh", "SUB netzStarten", "END SUB")),

  H.code(stueck(M + "Netzseite.dh", "SUB netzBedienen", "END SUB")),

  H.p("HTTPD_ACCEPT kehrt sofort zurück, wenn niemand anklopft — die Mediathek bleibt bedienbar. Kommt eine Anfrage, antwortet HTTPD_SEND mit der Seite, und die ist nur Text: HTML, Zeile für Zeile zusammengesetzt, aus einer Abfrage wie der des Hauptfensters, nach dem Titel sortiert:"),

  H.code(stueck(M + "Netzseite.dh", "FUNCTION netzSeite$", "END FUNCTION")),

  H.p("Was aus der Datenbank kommt, geht durch html$: Ein Titel mit < oder & darf die Seite nicht zerbrechen — und niemand soll über einen Titel eigenes HTML in die Seite schmuggeln."),

  H.code(stueck(M + "Netzseite.dh", "FUNCTION html$", "END FUNCTION")),

  H.p("Im Menü Datei schaltet „Im Heimnetz zeigen“ die Seite an und aus. Ein Timer im Hauptfenster fragt alle 50 ms nach, ob ein Browser anklopft — ein Timer, kein Takt, damit es auch dann geht, wenn das Programm im Leerlauf schläft:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB miNetz_Aktion", "END SUB")),

  H.p("Jonas öffnet dann http:// und den Namen des Rechners, auf dem die Mediathek läuft, mit :8080 dahinter — etwa http://wohnzimmer:8080. Auf dem Rechner selbst geht http://localhost:8080."),

  H.h3("Was dabei schiefgehen kann"),

  H.p("Beim ersten Start fragt die Firewall von Windows meist, ob das Programm im Netz erreichbar sein darf — „Private Netzwerke“ genügt fürs Heimnetz. Belegt schon ein anderes Programm den Port 8080, geht der Server nicht auf; dann nimmt man eine andere Zahl. Und der Server ist für das Heimnetz gebaut: ohne Verschlüsselung und ohne Anmeldung. Ins offene Internet gehört er nicht."),

  H.h2("Mehrere Benutzer"),

  H.p("Die Seite zeigt allen, was da ist; ändern kann nur, wer an dem einen Rechner sitzt. Sollen mehrere an mehreren Rechnern mit denselben Daten arbeiten, braucht es eine Datenbank auf einem Server. Das Modul db spricht PostgreSQL und MySQL mit demselben SQL und denselben Platzhaltern wie SQLite; nur das Ziel in DB_OPEN ändert sich:"),

  H.code(['DIM c AS DB_CONN', 'c = DB_OPEN("postgres://lena:geheim@server:5432/mediathek")']),

  H.p("Das Datenbank-Element des Studios arbeitet heute mit einer SQLite-Datei. Wer auf einen Server umzieht, öffnet die Verbindung im eigenen Code und schreibt die Abfragen selbst — das Handbuch zeigt im Modul db, wie. Bis dahin trägt die Datei weit: SQLite hält Hunderttausende Einträge, und eine Mediathek für eine Familie ist davon weit entfernt."),

  H.h2("Eigene Bausteine"),

  H.p("Manches aus diesem Buch braucht das nächste Projekt wieder: die Einstellungen im Benutzerordner, das Einlesen einer CSV mit einer Liste der Fehler, die Web-Seite. Eine Bibliothek ist in Drachenhauch ein Ordner mit .dh-Dateien; dhrt paket holt sie in ein Projekt und merkt sich, welcher Stand es war:"),

  H.code(["dhrt paket hole ../bausteine/einstellungen", "dhrt paket hole github:lena/bausteine@1.0#einstellungen"], { out: true }),

  H.p("Danach steht im Code IMPORT \"einstellungen/einstellungen.dh\". Damit ein Baustein in jedes Projekt passt, darf er nichts über die Mediathek wissen: Der Name des Programms für DIR_USER_DATA$ etwa wird ein Parameter, statt fest „Mediathek“ zu heißen. Wie man ein Paket anbietet, steht im Handbuch unter Pakete."),

  H.h2("Zeigen, was man gebaut hat"),

  H.p("Die Galerie im Handbuch sammelt Programme, die mit Drachenhauch gebaut sind — Spiele, Demos und Anwendungen, darunter ein Studio-Projekt wie dieses, eine Vorratsverwaltung für den Kühlschrank mit Einkaufsliste, Rezepten und Barcodes. Es lohnt, dort hineinzusehen: Ein fremdes Projekt zeigt oft am schnellsten, wie man etwas anders machen kann. Und wer selbst etwas gebaut hat, zeigt es in den Discussions des Projekts unter „Show and tell“; ein Programm, das andere ausprobieren können, kommt auf Wunsch in die Galerie."),

  H.h2("Und dann?"),

  H.p("Dieses Buch hat einen Weg genommen — ein Projekt, Kapitel für Kapitel. Das Handbuch nimmt alle anderen. Die Kapitel zum Modul gui und zum Studio sagen zu jedem Widget, jedem Ereignis und jeder Eigenschaft, was es kann; die anderen Module zeigen, was es außer Fenstern noch gibt: Ton, Grafik bis zu 3D, Geräte am USB-Kabel, Netz. Das Buch „Kleine Wunder“ geht den Weg der Spiele und Bilder."),

  H.p("Und das beste nächste Projekt ist eins, das du selbst brauchst."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul httpd: ", "der Webserver, Dateien ausliefern, Grenzen."),
  H.bulletRich("Handbuch, Modul db → Server: PostgreSQL und MySQL: ", "Ziele, Verschlüsselung, Platzhalter."),
  H.bulletRich("Handbuch, Pakete: ", "Bibliotheken holen und teilen."),
  H.bulletRich("Handbuch, Galerie: ", "was andere gebaut haben, und wie man ein eigenes Programm zeigt."),
];
