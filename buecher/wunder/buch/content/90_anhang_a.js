const { neu, stueck: stueckLoesung } = require("../code_lesen");

// Anhang A: die Loesungen. Der Code kommt aus code/loesungen/ -- nur die
// Stellen zwischen ' >>> neu und ' <<< neu, der Rest ist das Programm des
// Kapitels. Die Dateien selbst laufen in der Pruefsammlung.
module.exports = (H) => [
  H.chapter("A · Lösungen der Übungen"),

  H.p("Zu den Drehen-Übungen steht hier die Antwort mit ihrer Begründung. Zu Erweitern und Erfinden steht eine Lösung als Datei in code/loesungen; abgedruckt sind nur die Stellen, die gegenüber dem Programm des Kapitels neu sind. In den Dateien sind sie mit >>> und <<< markiert. Eine andere Lösung ist nicht falsch, nur weil sie anders aussieht — wenn sie tut, was die Aufgabe verlangt, ist sie richtig."),

  H.h2("Kapitel 0 · Die Werkstatt"),

  H.h3("Drehen 1"),

  H.p("Mit -40 dreht sich der Ring andersherum. Weil y auf dem Bildschirm nach unten wächst, läuft ein wachsender Winkel im Uhrzeigersinn; ein schrumpfender gegen ihn. Mit 400 dreht er sich zehnmal so schnell, eine volle Runde in 0,9 Sekunden — und auch das Atmen wird zehnmal so schnell, denn der Radius hängt am selben Winkel (drehung * 3). Ein Programm, das zwei Bewegungen an eine Größe hängt, verändert beide, wenn man an ihr dreht."),

  H.h3("Drehen 2"),

  H.p("Die Farbtöne reichen jetzt nur noch von 0 bis 165 Grad: Rot, Orange, Gelb, Grün bis zu einem grünlichen Türkis. Blau, Violett und Pink fehlen, und zwischen dem letzten und dem ersten Kreis springt die Farbe von Türkis zurück nach Rot. Erst mit zwölf mal dreißig sind es die vollen 360 Grad des Farbkreises."),

  H.h3("Drehen 3"),

  H.p("i * 30 steht dreimal im Programm: im Winkel, im Atmen und in der Farbe. Für 24 Kreise muss es überall i * 15 heißen, denn 24 mal 15 sind wieder 360. Vergisst man den Winkel, liegen die Kreise 13 bis 24 auf denselben Stellen wie 1 bis 12 — man sieht nur zwölf. Vergisst man die Farbe, läuft der Regenbogen zweimal herum. Vergisst man das Atmen, läuft die Welle zweimal um den Ring; das ist kein Fehler, nur ein anderes Muster. Damit sich 24 Kreise nicht überlappen, darf der Radius kleiner werden, etwa 16 + SIN(…) * 6."),

  H.h3("Erweitern 1: ein zweiter Ring"),

  H.p("Eine zweite Schleife nach der ersten, mit kleinerem Abstand zur Mitte, kleineren Kreisen und einem Winkel, der doppelt so schnell und andersherum läuft (code/loesungen/kap00_zwei_ringe.dh):"),

  ...neu("loesungen/kap00_zwei_ringe.dh").map((t) => H.code(t)),

  H.p("Die Farbe ist um 180 Grad verschoben: Jeder innere Kreis hat die Gegenfarbe des äußeren Kreises mit derselben Nummer. Die Variablen heißen wi, xi und yi statt w, x und y — nötig wäre das nicht, denn ein DIM in einer Schleife fängt jede Runde neu an, aber so sieht man beim Lesen, welche Rechnung zu welchem Ring gehört."),

  H.h3("Erweitern 2"),

  H.p("Mit DELTA() dreht sich der Ring genauso schnell wie vorher, nur ruckeliger: Er springt jetzt fünfzehnmal je Sekunde um je 2,7 Grad statt sechzigmal um 0,67. Die Fassung mit dem festen Schritt dreht sich viermal so langsam, denn sie macht weiter 0,67 Grad je Bild, und es gibt nur noch ein Viertel der Bilder. Genau das passiert, wenn ein Rechner zu langsam ist, um sechzig Bilder zu schaffen."),

  H.h2("Kapitel 1 · Sternenflug"),

  H.h3("Drehen 1"),

  H.p("Mit 100 rückt alles zur Mitte: Die Sterne bleiben lange kleine Punkte und schießen erst ganz am Ende nach außen, das Bild wirkt weit und tief. Mit 900 sieht man nur noch den mittleren Teil des Sternenfelds, stark vergrößert; die Sterne verlassen das Bild schnell, der Flug wirkt rasant und flach. Das ist der Unterschied zwischen einem Weitwinkel und einem Teleobjektiv — die Brennweite eines Fotoapparats ist genau die Zahl f aus der Zeichnung."),

  H.h3("Drehen 2"),

  H.p("Viertausend Sterne machen aus dem Nebel in der Mitte eine helle Wolke; ob das schöner ist, ist Geschmack. Langsamer wird das Programm nicht spürbar: Auf dem Rechner, mit dem dieses Buch entstand, lief es selbst mit zwanzigtausend Sternen viel schneller, als der Bildschirm Bilder zeigen kann. Eine Schleife über ein paar tausend Zahlen ist für einen Rechner keine Arbeit."),

  H.h3("Drehen 3"),

  H.p("Das Programm bricht nach einem Augenblick ab, mit der Meldung „RGB-Werte muessen 0..255 sein“. Der erste Stern, der an uns vorbeifliegt, bekommt ein negatives z, damit wird naehe = 1 - z / TIEFE größer als 1 und die Helligkeit größer als 255. Die eine Zeile schützt also drei Dinge zugleich: die Division vor der 0, das Bild vor gespiegelten Sternen und die Farbe vor Werten über 255. Wer eine solche Schutzzeile herausnimmt, sollte vorher überlegen, wer sich auf sie verlässt."),

  H.h3("Erweitern 1: blaue Ferne"),

  H.p("HSV mit dem festen Farbton 220 (Blau) und einer Sättigung, die mit der Nähe abnimmt (code/loesungen/kap01_farbe.dh):"),

  ...neu("loesungen/kap01_farbe.dh").map((t) => H.code(t)),

  H.p("Die Helligkeit hell, die bisher in alle drei Farbanteile ging, wird zum dritten Wert von HSV, geteilt durch 255, weil HSV dort eine Zahl von 0 bis 1 erwartet. So bleiben ferne Sterne dunkel und blau, nahe hell und fast weiß — wie in einer klaren Nacht, in der das Licht ferner Dinge bläulich wirkt."),

  H.h3("Erweitern 2: Warp"),

  H.p("Statt der Konstante TEMPO fliegen die Sterne jetzt mit einer Variablen flug, die dem gewünschten Tempo nachläuft (code/loesungen/kap01_warp.dh):"),

  ...neu("loesungen/kap01_warp.dh").map((t) => H.code(t)),

  H.p("Die Zeile flug = flug + (ziel - flug) * MIN(1.0, 4 * dt) holt in jeder Sekunde einen großen Teil des Abstands zum Ziel auf, zuerst schnell, dann immer langsamer; das ist der Anlauf. Das MIN verhindert, dass bei einem sehr langen Bild (dt groß) übers Ziel hinausgeschossen wird. Die Variable heißt flug und nicht tempo: Weil Drachenhauch Groß- und Kleinschreibung nicht unterscheidet, wäre tempo dasselbe wie die Konstante TEMPO, und das Programm würde nicht übersetzt. Kapitel 4 kommt darauf zurück."),

  H.h3("Erfinden: ein Wirbel"),

  H.p("Eine Möglichkeit (code/loesungen/kap01_wirbel.dh): Bevor ein Stern geteilt wird, dreht das Programm ihn um die Blickachse — um so mehr, je näher er ist. So scheinen die Sterne in einem Strudel auf uns zuzukommen:"),

  ...neu("loesungen/kap01_wirbel.dh").map((t) => H.code(t)),

  H.p("Die beiden Zeilen mit COS und SIN sind die Drehung eines Punkts um den Nullpunkt; sie kommen in Kapitel 3 und 4 wieder. Der Schweif braucht dieselbe Drehung für sein z — sonst zeigte er nicht dorthin, woher der Stern kam."),

  H.h2("Kapitel 2 · Plasma"),

  H.h3("Drehen 1"),

  H.p("Mit MASSSTAB 2 rechnet das Programm 400 × 250 Punkte, viermal so viele wie vorher; die Stufen sind kaum noch zu sehen. Auf dem Rechner, mit dem dieses Buch entstand, lief es mit Maschinencode weiter flüssig (60 Bilder je Sekunde), ohne ihn nur noch mit 17. Mit 8 sind es 100 × 62 Punkte: Das Plasma zerfällt sichtbar in Klötzchen, wie auf einem sehr alten Rechner. Mit 1 ist man wieder beim ersten Programm, 16 Bilder je Sekunde mit Maschinencode, 4 ohne. Welche Zahl die richtige ist, hängt davon ab, wie schnell der Rechner ist, auf dem das Programm läuft — 4 läuft überall."),

  H.h3("Drehen 2"),

  H.p("Es ändert sich nur die erste Welle, die von links nach rechts läuft: Sie wird viermal so schmal, eine volle Schwingung dauert jetzt etwa 63 statt 250 Punkte. Das Muster bekommt dichte, senkrechte Streifen, durch die die beiden anderen Wellen hindurchwabern. Je kleiner die Zahl unter dem Bruchstrich, desto feiner die Welle."),

  H.h3("Drehen 3"),

  H.p("Die Farben werden blass, wie Pastellkreide oder ein Kinderzimmer. Die Sättigung sagt, wie viel Farbe in einer Farbe steckt; bei 0 bleibt nur Grau in der Helligkeit, die der dritte Wert angibt."),

  H.h3("Erweitern 1: Feuer"),

  H.p("v wird erst in eine Zahl w von 0 bis 1 umgerechnet. Die bestimmt dann den Farbton (0 bis 55 Grad, also Rot bis Gelb) und zugleich die Helligkeit (code/loesungen/kap02_feuer.dh):"),

  ...neu("loesungen/kap02_feuer.dh").map((t) => H.code(t)),

  H.p("Dass der Farbton und die Helligkeit an derselben Zahl hängen, macht das Feuer: Wo es hell ist, ist es gelb, wo es dunkel ist, rot. Die Zeit t steckt nur noch in den Wellen, nicht mehr in der Farbe — sonst liefe der Farbton aus dem Feuer heraus."),

  H.h3("Erweitern 2: im Kreis"),

  H.p("Für jeden Punkt den Abstand zur Mitte prüfen; innen wie bisher, außen dunkel (code/loesungen/kap02_kreis.dh):"),

  ...neu("loesungen/kap02_kreis.dh").map((t) => H.code(t)),

  H.p("dx * dx + dy * dy < 220 * 220 ist der Satz des Pythagoras ohne Wurzel: Der Abstand ist kleiner als 220, wenn sein Quadrat kleiner ist als 220 zum Quadrat. Die Wurzel wegzulassen spart bei 25 000 Punkten je Bild ein wenig Arbeit und ändert nichts am Ergebnis."),

  H.h3("Erfinden: Wellen um die Maus"),

  H.p("Eine Möglichkeit (code/loesungen/kap02_maus.dh): ein vierter Summand, der vom Abstand zur Maus abhängt. Weil t abgezogen wird, laufen die Ringe nach außen:"),

  ...neu("loesungen/kap02_maus.dh").map((t) => H.code(t)),

  H.p("Hier steht die Wurzel (SQR) doch: Der Abstand selbst geht in den SIN, nicht nur sein Vergleich mit einer Zahl. ^ 2 ist das Quadrat. Die Lage der Maus wird einmal je Bild gelesen, vor der Schleife — 25 000-mal MOUSE_X() zu fragen, brächte nichts außer Arbeit."),

  H.h2("Kapitel 3 · Spirograph"),

  H.h3("Drehen 1"),

  H.p("Mit Stift 120 sitzt der Stift außerhalb des Rads, wie an einem Arm. Die Bögen der Kurve werden zu Schleifen, die sich auf dem Weg nach innen umeinander winden; die Kurve reicht weiter nach außen und weiter nach innen als vorher. Mit Stift 0 sitzt der Stift genau in der Mitte des Rads und macht dessen Drehung nicht mit: Er zeichnet einen einfachen Kreis mit dem Radius 220 - 84 = 136, den Weg der Radmitte."),

  H.h3("Drehen 2"),

  H.p("Mit Rad 110 und Stift 60 entsteht eine Ellipse, liegend in der Mitte, nach einer einzigen Runde geschlossen. Mit Stift 110 wird die Ellipse ganz flach: Der Stift fährt auf einer geraden Linie hin und her, quer durch den Ring. Ein Rad, das in einem doppelt so großen Ring rollt, macht aus einer Drehung eine gerade Bewegung — das ist als Kardankreis bekannt und wurde in Maschinen benutzt, lange bevor es Rechner gab. Rechnerisch: innen / rad ist dann 1, und mit stift = 110 heben sich in kurveY die beiden SIN genau auf."),

  H.h3("Drehen 3"),

  H.p("Die Kurve besteht aus sichtbar geraden Stücken, ein Stern aus Ecken statt einer Rosette aus Bögen. Geschlossen ist sie trotzdem: neu = MIN(t + 0.5, bis) kürzt den letzten Schritt so, dass er genau bei ende aufhört. Im ersten Programm mit FOR … STEP wäre das anders; dort endet die Schleife beim letzten Schritt, der noch vor dem Ende liegt, und an der Naht bliebe eine Lücke."),

  H.h3("Erweitern 1: Farbverlauf"),

  H.p("Der Farbton wächst mit t (code/loesungen/kap03_farbverlauf.dh):"),

  ...neu("loesungen/kap03_farbverlauf.dh").map((t) => H.code(t)),

  H.p("Bei 21 Runden, also t bis etwa 132, wandert der Farbton um rund 790 Grad, gut zweimal durch den Farbkreis. Weil die Kurve dabei Runde um Runde neben der vorigen liegt, sieht man die Farben als breite Bänder."),

  H.h3("Erweitern 2: Regler"),

  H.p("Ein kleines gui-Fenster mit zwei Reglern; neueKurve nimmt Rad und Stift von dort statt vom Zufall (code/loesungen/kap03_regler.dh):"),

  ...neu("loesungen/kap03_regler.dh").map((t) => H.code(t)),

  H.p("GUI_CHANGED ist genau in dem Bild wahr, in dem jemand den Regler bewegt hat — wie GUI_CLICKED bei einem Knopf. Der Regler gibt eine Kommazahl, INT macht eine ganze daraus, denn ggt rechnet mit ganzen Zahlen. Mehr über das gui-Modul steht im Buch „Fenster, Formulare, Daten“."),

  H.h3("Erfinden: drei Räder"),

  H.p("Eine Möglichkeit ist ein eigenes kleines Programm (code/loesungen/kap03_drei_raeder.dh). Statt Ring und Rad gibt es drei Arme, jeder dreht sich am Ende des vorigen, mit eigener Länge und eigenem Tempo:"),

  H.code(stueckLoesung("loesungen/kap03_drei_raeder.dh", "FUNCTION armX", "END FUNCTION")),
  H.code(stueckLoesung("loesungen/kap03_drei_raeder.dh", "FUNCTION stiftX", "END FUNCTION")),

  H.figure("kap03_drei_raeder.png", "Drei Räder, Arme 140, 70 und 30 lang, mit dem Tempo 1, -6 und 15: ein Stern mit sechs Schlaufen.", 420, 262),

  H.p("armX ist die Rechnung für ein Rad, so allgemein, dass sie für alle drei passt; stiftX zählt die drei Arme zusammen. Hier zeigen benannte Argumente, wozu sie da sind: armX(t, laenge: 70, tempo: -6) liest sich ohne Nachschlagen. Mit den Tempi 1, -6 und 15 schließt sich die Kurve nach einer Runde; andere ganze Zahlen geben andere Sterne, Kommazahlen Kurven, die sich nie schließen."),

  H.h2("Kapitel 4 · Das Ziffernblatt"),

  H.h3("Drehen 1"),

  H.p("Mit SPRUNG = 1.0 ist der Zeiger immer unterwegs: Er braucht die ganze Sekunde, um zur nächsten zu kommen, und gleitet ohne Pause — aber nicht gleichmäßig, denn weich bremst zum Ende jeder Sekunde ab und fährt am Anfang der nächsten wieder an. Man sieht ein leichtes Pulsieren. Mit 0.02 ist der Sprung so kurz, dass er bei 60 Bildern je Sekunde in ein einziges Bild fällt: Der Zeiger springt hart wie bei einer billigen Quarzuhr."),

  H.h3("Drehen 2"),

  H.p("Ohne die Vierteldrehung zeigt jeder Zeiger um 90 Grad zu weit im Uhrzeigersinn. Um zwölf Uhr zeigen alle drei auf die 3, um drei Uhr auf die 6. Das Zifferblatt selbst stimmt weiter, denn zifferblatt hat sein eigenes - 90. Die Uhr geht also scheinbar drei Stunden, fünfzehn Minuten und fünfzehn Sekunden vor."),

  H.h3("Drehen 3"),

  H.p("Die Uhr wird hell mit dunklen Strichen, Zahlen und Zeigern, und sie bleibt gut lesbar, denn hell und dunkel tauschen nur die Plätze. Eine Änderung genügt, weil jede Stelle, die das Zifferblatt malt, BLATT benutzt, und jede, die darauf schreibt, TINTE. Hätte das Programm die Farbzahlen an jede Stelle geschrieben, müsste man sie einzeln suchen — und übersähe sicher eine."),

  H.h3("Erweitern 1: Weltzeit"),

  H.p("Drei Uhren in einer Schleife, jede mit eigener Mitte und kleinerem Radius; die Zeit ist dieselbe Unix-Sekunde, um einige Stunden verschoben (code/loesungen/kap04_weltzeit.dh):"),

  H.code(stueckLoesung("loesungen/kap04_weltzeit.dh", "DIM staedte", "DIM abstand")),
  H.code(stueckLoesung("loesungen/kap04_weltzeit.dh", "FOR i = 0 TO 2", "NEXT")),

  H.p("Hier zahlen sich die Vorgaben von uhrZeichnen aus: mx: mx, my: 230, r: 115 setzt genau die drei Werte, die bei jeder Uhr anders sind. Dass die Verschiebung fest ist, ist eine Vereinfachung: New York und Berlin stellen ihre Uhren an verschiedenen Tagen auf Sommerzeit um, und Tokio gar nicht. Eine richtige Weltzeituhr bräuchte die Zeitzonen-Regeln, und die stehen nicht im Modul time."),

  H.h3("Erweitern 2: Stoppuhr"),

  H.p("Die Stoppuhr zählt ihre eigene Zeit mit DELTA() und gibt sie als Minuten und Sekunden an uhrZeichnen (code/loesungen/kap04_stoppuhr.dh):"),

  H.code(stueckLoesung("loesungen/kap04_stoppuhr.dh", "IF KEY_HIT(KEY_SPACE)", "anzeige = anzeige")),

  H.p("Die Uhr bekommt die Stunde 0, also steht der kurze Zeiger oben; der lange zeigt die Minuten, der rote die Sekunden mit Bruchteil — er gleitet, statt zu springen. FORMAT$ arbeitet mit Masken nach dem Muster der Sprache C: %02d ist eine ganze Zahl mit mindestens zwei Stellen und Nullen davor, %04.1f eine Kommazahl mit einer Nachkommastelle und mindestens vier Zeichen."),

  H.h3("Erfinden: Mondphase"),

  H.p("Eine Möglichkeit (code/loesungen/kap04_mond.dh): Der Mond braucht 29,53 Tage von Neumond zu Neumond. Kennt man einen Neumond, sagt die Zeit seither, wie weit er im laufenden Monat ist:"),

  H.code(stueckLoesung("loesungen/kap04_mond.dh", "CONST NEUMOND", "END FUNCTION")),

  H.p("runden - INT(runden) ist der Bruchteil, 0 bei Neumond, 0,5 bei Vollmond. Gezeichnet wird eine helle Scheibe, über die sich eine dunkle in der Farbe des Zifferblatts schiebt, und damit der Schatten nicht über das Fenster hinausragt, beschneidet SCISSOR alles Zeichnen auf ein Rechteck, bis SCISSOR_END. Das ist eine Näherung — der echte Schatten ist eine halbe Ellipse, keine Kreisscheibe —, aber auf 24 Punkten Radius sieht man den Unterschied kaum. Hier zahlt sich auch die Aufteilung von uhrZeichnen in zifferblatt und alleZeiger aus: Der Mond gehört zwischen das Blatt und die Zeiger."),

  H.h2("Kapitel 5 · Feuerwerk"),

  H.h3("Drehen 1"),

  H.p("Die Steighöhe ist v² / (2 · SCHWERE). Mit SCHWERE = 30 steigen die Raketen 230² / 60 ≈ 880 bis 300² / 60 = 1500 Pixel — weit über den oberen Rand eines 500 hohen Fensters. Man sieht Raketen aufsteigen, aber keine Explosion, nur sehr viel später ein paar Funken, die langsam von oben hereinsinken (die Schwere ist ja auch für sie schwach)."),

  H.p("Mit SCHWERE = 400 sind es 230² / 800 ≈ 66 bis 300² / 800 ≈ 112 Pixel. Die Raketen zerplatzen nach gut einer halben Sekunde knapp über dem Boden, und die Funken fallen schwer und schnell nach unten. Das sieht weniger nach Feuerwerk aus als nach Funkenflug aus einem Schleifstein."),

  H.h3("Drehen 2"),

  H.p("Die Explosionen werden dichter und heller, weil sich mehr durchsichtige Scheine überlagern; die Mitte einer frischen Explosion wird fast weiß. Bei fünf Raketen kurz hintereinander leben etwa fünftausend Funken gleichzeitig, und je nach Rechner beginnt das Bild dann zu ruckeln. Hier zeigt sich, warum Zeit in Sekunden gerechnet wird: Auch wenn weniger Bilder pro Sekunde kommen, fliegen die Funken gleich schnell, nur in größeren Sprüngen."),

  H.h3("Drehen 3"),

  H.p("Jeder Funke bekommt eine völlig zufällige Farbe, jede Explosion wird bunt gesprenkelt. Verloren geht, dass man die Raketen auseinanderhalten kann: Zwei Explosionen, die sich überlagern, verschwimmen zu einer. Die Farbe einer Rakete ist Information, nicht nur Schmuck."),

  H.h3("Erweitern 1: zünden mit der Maus"),

  H.pmix(["Aus h = v² / (2 · SCHWERE) wird v = √(2 · SCHWERE · h). Die Höhe h ist der Abstand vom unteren Rand bis zum Klick, also ", ["HOEHE - y", true], ". Das Tempo ist negativ, weil die Rakete nach oben fliegt. Neu ist eine SUB neben ", ["raketeStarten", true], " (Datei code/loesungen/kap05_klick.dh):"]),

  ...neu("loesungen/kap05_klick.dh").map((t) => H.code(t)),

  H.pmix(["Die zweite Stelle steht in der Hauptschleife, direkt unter der Abfrage der Leertaste. Die Rakete zerplatzt nicht auf das Pixel genau an der Klickstelle — sie fliegt in Schritten von einem Bild, und der letzte Schritt kann ein wenig über den höchsten Punkt hinausgehen. Bei 60 Bildern pro Sekunde sind das höchstens ein, zwei Pixel."]),

  H.h3("Erweitern 2: die Spur"),

  H.pmix(["Ein neuer STRUCT ist nicht nötig: Ein Spurfunke ist ein Funke wie jeder andere, nur langsamer, gelblich und kurzlebig. Darum genügt es, in ", ["raketenBewegen", true], " in jedem Bild einen Funken unter die Rakete zu setzen (code/loesungen/kap05_spur.dh):"]),

  ...neu("loesungen/kap05_spur.dh").map((t) => H.code(t)),

  H.p("Das kleine Tempo nach unten lässt die Spur ein Stück zurückfallen, so wirkt sie wie ein Ausstoß. Bei 60 Bildern pro Sekunde und drei Raketen in der Luft sind das 180 Funken pro Sekunde zusätzlich — für das Feld kein Problem, es wächst und schrumpft einfach mit."),

  H.h3("Erfinden: das Feuerwerk in zwei Stufen"),

  H.pmix(["Eine Möglichkeit (code/loesungen/kap05_kaskade.dh): Der Funke bekommt ein Feld ", ["knaller", true], ", jeder zehnte ist einer. Wenn ein Knaller verlischt, ruft das Aufräumen ", ["nachknallen", true], " auf, bevor es ihn aus dem Feld nimmt:"]),

  ...neu("loesungen/kap05_kaskade.dh").map((t) => H.code(t)),

  H.pmix(["Zwei Dinge sind daran lehrreich. Erstens nehmen die neuen Funken das Tempo des Knallers mit (", ["k.vx + …", true], "), so fliegt die kleine Explosion in seine Richtung weiter, wie bei einer echten. Zweitens hängt ", ["nachknallen", true], " neue Funken an das Feld an, WÄHREND die Schleife es rückwärts durchläuft. Das ist sicher: Die neuen landen hinten, an Stellen, die die Schleife schon hinter sich hat — sie werden erst im nächsten Bild bewegt."]),

  H.h2("Kapitel 6 · Das Spiel des Lebens"),

  H.h3("Drehen 1"),

  H.p("Mit 0.05 sind zu wenige Zellen da, um einander am Leben zu halten: Fast alles stirbt in den ersten Generationen, übrig bleiben ein paar einsame Blöcke und Blinker. Mit 0.7 ist es zu voll: Fast alles stirbt an Überfüllung, und aus den Resten wächst ein ähnliches Bild wie bei 0.25, nur später. Leben braucht die mittlere Dichte — irgendwo zwischen einem Fünftel und der Hälfte entsteht das meiste."),

  H.h3("Drehen 2"),

  H.p("Mit Geburt auch bei sechs Nachbarn heißt die Variante HighLife. Das meiste sieht aus wie vorher, aber hin und wieder taucht ein Muster auf, das sich selbst kopiert: der Replikator. Er hinterlässt eine Spur von Abbildern seiner selbst. Eine einzige Zahl in den Regeln ändert, was in der Welt möglich ist."),

  H.h3("Drehen 3"),

  H.p("Drei Zellen waagrecht nebeneinander: Die mittlere hat zwei Nachbarn und überlebt, die beiden äußeren haben nur einen und sterben. Über und unter der mittleren liegen je eine tote Zelle mit genau drei lebenden Nachbarn — sie werden geboren. Aus der waagrechten Reihe wird eine senkrechte, in der nächsten Generation wieder eine waagrechte: Der Blinker blinkt, für immer."),

  H.h3("Erweitern 1: die Maus"),

  H.p("Die Zelle unter der Maus ist die Mausstelle, ganzzahlig durch die Zellgröße geteilt, plus eins, weil das Feld bei 1 beginnt (code/loesungen/kap06_maus.dh):"),

  ...neu("loesungen/kap06_maus.dh").map((t) => H.code(t)),

  H.p("MOUSE_DOWN statt MOUSE_HIT: Solange die Taste gedrückt ist, zeichnet man, wie mit einem Stift. Die Prüfung der Grenzen schützt vor einem Index außerhalb, wenn die Maus am Rand des Fensters steht."),

  H.h3("Erweitern 2: herumlaufende Ränder"),

  H.p("Nur lebt ändert sich (code/loesungen/kap06_torus.dh):"),

  ...neu("loesungen/kap06_torus.dh").map((t) => H.code(t)),

  H.p("(x - 1) MOD B rechnet die Spalte auf 0 bis B - 1 um, + 1 wieder auf 1 bis B. Für x = 0 ist (x - 1) MOD B in Drachenhauch B - 1, denn MOD rundet wie \ nach unten, auch bei negativen Zahlen — in vielen anderen Sprachen käme -1 heraus, und man bräuchte noch ein + B. Ein Gleiter, der rechts unten hinausfliegt, kommt jetzt links oben wieder herein."),

  H.h3("Erfinden: Farbe nach Alter"),

  H.p("Eine Möglichkeit (code/loesungen/kap06_alter.dh): Statt 0 und 1 steht im Feld das Alter der Zelle. lebt antwortet weiter mit 0 oder 1, damit die Regeln dieselben bleiben; nur die Geburt setzt 1, das Überleben zählt hoch:"),

  ...neu("loesungen/kap06_alter.dh").map((t) => H.code(t)),

  H.p("Hier zahlt sich die Funktion lebt ein zweites Mal aus: nachbarn musste fast nicht geändert werden, weil es nie selbst auf das Feld geschaut hat. Nur das Abziehen der Zelle selbst fragt jetzt lebt statt das Feld — sonst zöge es ihr Alter ab statt einer 1."),
];
