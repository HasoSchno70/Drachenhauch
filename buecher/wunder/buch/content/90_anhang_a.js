const { neu, zeilen, stueck: stueckLoesung } = require("../code_lesen");

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

  H.h2("Kapitel 7 · Fallender Sand"),

  H.h3("Drehen 1"),

  H.p("Das Wasser wird zäh wie Honig. Unter der Düse wächst ein Buckel, der nur langsam auseinanderläuft, und auf dem Brett staut es sich, statt hinunterzufließen. Eine Zelle je Schritt ist zu langsam gegen den Nachschub; vier sind schnell genug, dass der See eben aussieht. Echtes Wasser hat keine Zellen, aber dieselbe Frage — wie schnell es nachgibt — unterscheidet auch dort Wasser von Honig."),

  H.h3("Drehen 2"),

  H.p("Die Sandhaufen bleiben, wie sie waren: Ist links besetzt, rutscht ein Korn eben nach rechts, und 45 Grad ist auf beiden Seiten die einzige Schräge, die hält. Das Wasser aber fließt nur noch nach links. Es läuft am linken Ende vom Brett, füllt den Boden links von der Düse, und rechts davon bleibt es trocken. Der Unterschied: Sand bewegt sich nur, wenn er fallen kann, und das Fallen ist auf beiden Seiten gleich. Wasser bewegt sich auch waagrecht, und dort entscheidet allein s, wohin."),

  H.h3("Drehen 3"),

  H.p("Alles läuft viermal so schnell, und weil die Düsen je Schritt ein Korn und einen Tropfen geben, kommt in den zehn Sekunden viermal so viel heraus: ein großer Haufen, ein tiefer See. Die Formen bleiben — 45 Grad, ebenes Wasser —, denn die Regeln wissen nichts von Sekunden, nur von Zellen. TAKT bestimmt allein, wie oft sie gelten."),

  H.h3("Erweitern 1: Pinselgröße"),

  H.p("Der Radius wird eine Variable, und das Mausrad verstellt ihn (code/loesungen/kap07_pinsel.dh):"),

  ...neu("loesungen/kap07_pinsel.dh").map((t) => H.code(t)),

  H.p("MOUSE_WHEEL liefert, wie weit das Rad seit dem letzten Aufruf gedreht wurde, meist 0. CLAMP hält den Radius zwischen 1 und 12. Statt 9 steht im Vergleich jetzt radius ^ 2 — die 9 war der Radius 3 zum Quadrat, nur ohne es zu sagen."),

  H.h3("Erweitern 2: Schwerkraft umdrehen"),

  H.p("Aus jedem y + 1 in den Regeln wird y + g, und die Taste G kehrt g um (code/loesungen/kap07_schwerkraft.dh):"),

  ...neu("loesungen/kap07_schwerkraft.dh").map((t) => H.code(t)),

  H.p("Was sich außer dem Fallen noch umdrehen muss, ist die Reihenfolge des Schritts. Er muss immer beim „Boden“ anfangen, sonst fiele ein Korn in einem Schritt bis ans Ende — der Fehler aus „Warum so?“, nur nach oben. Und die Zeile, die er überspringt, wechselt mit: Fällt alles nach oben, kann in der obersten Zeile nichts mehr fallen, und die unterste muss mitrechnen."),

  H.h3("Erfinden: Holz und Feuer"),

  H.p("Eine Möglichkeit (code/loesungen/kap07_feuer.dh): zwei neue Stoffe im ENUM, eine neue Regel für Feuer, eine Zeile im SELECT CASE. Holz braucht keine Regel; es liegt, bis Feuer es erreicht:"),

  ...neu("loesungen/kap07_feuer.dh").map((t) => H.code(t)),

  H.p("Die Zahlen 0.02 und 0.01 sind ausprobiert. Mit 0.2 statt 0.02 brennt der Stapel in einem Augenblick ab; mit 0.1 statt 0.01 geht das Feuer aus, bevor es das nächste Holz erreicht. Die Flammen haben eine Zufallsfarbe in jedem Bild — darum flackern sie, ohne dass es dafür eigenen Code braucht."),

  H.h2("Kapitel 8 · Schwarm"),

  H.h3("Drehen 1"),

  H.p("Nur noch eine Handvoll — acht, zehn Dreiecke. Die Vögel sind alle noch da, aber ohne Abstand ziehen Zusammenhalt und Ausrichtung sie aufeinander, bis mehrere genau übereinander fliegen, gleich schnell, in dieselbe Richtung. Die Regel, die am wenigsten nach Schwarm aussieht, hält ihn erst auseinander."),

  H.h3("Drehen 2"),

  H.p("Kein Schwarm mehr, sondern ein Gewimmel wie bei Mücken. Vögel finden sich zu kleinen Klumpen, aber jeder fliegt in seine eigene Richtung, und die Klumpen zerfallen wieder. Zusammenhalt allein macht einen Haufen; erst die Ausrichtung macht daraus einen Schwarm, der irgendwohin fliegt."),

  H.h3("Drehen 3"),

  H.p("Mit SICHT 20 sieht ein Vogel kaum jemanden. Es dauert lange, bis sich kleine Gruppen finden, und große Schwärme entstehen nicht. Mit 150 sieht jeder fast den halben Himmel: Nach wenigen Sekunden gibt es einen oder zwei große Schwärme, breite Bänder, die quer über das Bild ziehen. Rechnen muss das Programm in beiden Fällen gleich viel, denn umschauen geht immer alle Vögel durch — 150 mal 150, gut zwanzigtausend Abstände in jedem Bild."),

  H.h3("Erweitern 1: zur Maus"),

  H.p("Eine vierte Regel, gebaut wie der Zusammenhalt, nur mit der Maus als Mitte (code/loesungen/kap08_maus.dh):"),

  ...neu("loesungen/kap08_maus.dh").map((t) => H.code(t)),

  H.p("Der Aufruf steht nach dem IF nachbarn > 0: Auch ein Vogel, der allein fliegt, soll zur Maus. Hält man die Taste lange gedrückt, kreist der Schwarm um die Maus, statt auf ihr zu landen — tempoHalten lässt ihn nicht langsamer werden, also schießt er immer wieder vorbei."),

  H.h3("Erweitern 2: Felsen"),

  H.p("Drei Felsen als drei Felder mit Mitte und Radius, und eine Regel, die von jedem wegschiebt, dem der Vogel zu nah kommt (code/loesungen/kap08_hindernisse.dh):"),

  ...neu("loesungen/kap08_hindernisse.dh").map((t) => H.code(t)),

  H.p("rand ist der Abstand vom Rand des Felsens, nicht von seiner Mitte. st wächst von 0, VORSICHT Punkte vor dem Rand, bis 1 am Rand. So weicht ein Vogel sanft aus, wenn er von weitem kommt, und scharf, wenn er schon fast dran ist. Die beiden Zahlen sind ausprobiert: Mit KRAFT_FELS = 900 und VORSICHT = 30 steckte in jedem fünften Bild irgendwo ein Vogel im Felsen, gedrängt vom Schwarm hinter ihm. Jetzt kommt das noch vor, aber selten und nur für einen Augenblick. Ganz verhindern könnte es erst bewegen, indem es einen Vogel gar nicht hineinlässt."),

  H.h3("Erfinden: ein Raubvogel"),

  H.p("Eine Möglichkeit (code/loesungen/kap08_raubvogel.dh). Der Raubvogel ist auch ein Vogel — er hat einen Ort und eine Geschwindigkeit, er fliegt und kommt am Rand wieder herein. Nur entscheidet er anders und sieht anders aus. Genau das sagt EXTENDS: Raubvogel bekommt alles von Vogel und ersetzt, was er anders macht. Die Vögel bekommen eine Regel dazu, die Flucht:"),

  ...neu("loesungen/kap08_raubvogel.dh").map((t) => H.code(t)),

  H.pmix(["Bewegen und Init schreibt der Raubvogel nicht selbst; ", ["NEW Raubvogel()", true], " ruft das Init von Vogel, und ", ["jaeger.bewegen(dt)", true], " das bewegen von Vogel. ", ["jaeger.entscheiden(…)", true], " dagegen ruft seine eigene Fassung. In ihr beginnt ", ["ziel", true], " als NIL und wird der Vogel mit dem kleinsten Abstand. Der Raubvogel ist mit 170 etwas langsamer als die schnellsten Vögel — sonst wäre die Jagd zu schnell vorbei."]),

  H.h2("Kapitel 9 · Federn und Seile"),

  H.h3("Drehen 1"),

  H.p("Mit einer Runde ist das Seil mal 300, mal fast 390 lang statt 285, und es wippt wie Gummi: Die Glieder rücken ihre Punkte zurecht, verschieben dabei die Nachbarn und haben keine zweite Gelegenheit. Mit hundert Runden ist es genau 285 lang — und sieht trotzdem aus wie mit zwanzig, denn dort war es 286 bis 288, weniger als ein Zehntel Bildpunkt je Glied. Hundert Runden kosten aber fünfmal so viel Rechenzeit. Wie viele Runden man braucht, ist eine Abwägung, und das Auge ist der Maßstab."),

  H.h3("Drehen 2"),

  H.p("Mit 1.0 verliert das Seil keine Bewegung: Nach zehn Sekunden schwingt es noch fast so weit wie am Anfang, und es hört nie auf. Mit 0.95 hängt es nach drei Sekunden still, als hinge es in Honig. 0.995 liegt dazwischen und schwingt lange aus wie ein echtes Seil. Eine Zahl so nah an 1 wirkt klein, aber sie gilt sechzigmal je Sekunde: 0.995 hoch 60 ist etwa 0.74 — jede Sekunde ein Viertel weniger Schwung."),

  H.h3("Drehen 3"),

  H.p("Die Fahrbahn hängt durch wie eine Hängematte. Gemessen sinkt ihre Mitte ohne Hänger um 49 Punkte, mit Hängern um 6. Dabei ist die Fahrbahn ohne Hänger nur um 10 Punkte länger geworden, 610 statt 600 — jedes Glied gibt ein wenig nach, weil zwanzig Runden es nie ganz genau machen. Ein Seil, das zwischen zwei Punkten gespannt ist, verwandelt ein kleines Zuviel an Länge in viel Durchhang. Darum tragen Hängebrücken ihre Fahrbahn an einem Tragseil: Es hängt ohnehin durch, und die Fahrbahn darf gerade bleiben."),

  H.h3("Erweitern 1: ein Vorhang"),

  H.p("ziehen, fliegen, die Maus — alles bleibt. Neu ist nur, welche Punkte und welche Glieder es gibt (code/loesungen/kap09_vorhang.dh):"),

  ...neu("loesungen/kap09_vorhang.dh").map((t) => H.code(t)),

  H.p("netz merkt sich, welche Nummer der Punkt in Spalte sp und Zeile ze bekommen hat; so lassen sich die Nachbarn finden, ohne nachzurechnen, in welcher Reihenfolge punkt sie angelegt hat. Sechshundert Punkte und gut tausend Glieder, zwanzigmal je Schritt — das ist das Meiste, was dieses Kapitel rechnet, und es läuft noch flüssig."),

  H.h3("Erweitern 2: ein Seil, das reißt"),

  H.p("Jedes Glied bekommt ein Feld, ob es noch heil ist. Wird es mehr als 1,6-mal so lang, wie es sein soll, reißt es: Es zieht nicht mehr und wird nicht mehr gezeichnet (code/loesungen/kap09_reissen.dh):"),

  ...neu("loesungen/kap09_reissen.dh").map((t) => H.code(t)),

  H.figure("kap09_reissen.png", "Wer die Mitte der Fahrbahn weit genug nach unten zieht, reißt sie heraus: Das Stück fällt ins Wasser, die beiden Hälften hängen weiter an ihren Hängern.", 420, 262),

  H.p("Gerissen wird nur in ziehen, und nur, wenn das Glied zu lang ist. Gedehnt wird es meist von der Maus: Ein Punkt, den sie hält, gibt nicht nach, und das Glied daneben muss alles allein tragen."),

  H.h3("Erfinden: eine Fahne im Wind"),

  H.p("Eine Möglichkeit (code/loesungen/kap09_wind.dh): ein Netz wie der Vorhang, aber an der linken Spalte fest statt an der oberen Reihe, und in fliegen eine zweite Kraft neben der Schwerkraft:"),

  ...neu("loesungen/kap09_wind.dh").map((t) => H.code(t)),

  H.pmix(["Der Wind hat zwei Teile. ", ["boe", true], " schwillt langsam an und ab, für alle Punkte gleich — dann hebt und senkt sich die ganze Fahne. Die ", ["welle", true], " hängt von der Zeit UND vom Ort ab: ", ["SIN(zeit * 7 - x / 25)", true], " hat an jeder Stelle einen anderen Wert und wandert mit der Zeit nach rechts, vom Mast weg. So läuft eine Welle durch den Stoff. Einen Vorhang von vorn könnte der Wind übrigens nur zur Seite schieben: Flattern geht aus dem Bild heraus, und dieses Bild hat nur zwei Richtungen."]),

  H.h2("Kapitel 10 · Bälle in der Kiste"),

  H.h3("Drehen 1"),

  H.p("Der große Ball räumt den mittleren und den rechten Turm ab; übrig bleibt ein Haufen Klötze, nur der linke Turm steht. Ein Ball mit dem Radius 30 hat gut viermal so viel Fläche wie einer mit 14, und die Physik gibt ihm entsprechend mehr Masse. Bei gleichem Tempo trägt er viermal so viel Schwung in die Türme."),

  H.h3("Drehen 2"),

  H.p("Die Säulen biegen sich, als wären sie aus Gummi, und Klötze stecken ein Stück ineinander: In großen Schritten merkt die Physik eine Berührung erst, wenn die Körper schon überlappen, und schiebt sie dann ruckartig auseinander. Außerdem läuft alles langsamer. PHYS2D_STEP nimmt höchstens 0,05 Sekunden auf einmal; fünfzehn Schritte je Sekunde sind also nur drei Viertel einer Sekunde Physik."),

  H.h3("Drehen 3"),

  H.p("Mit demselben Zug wie vorher fliegt der Ball fast gerade und weit über die Türme hinweg. Auf dem Mond zielt man direkt auf das Ziel und zieht dafür weniger nach unten. Fällt dann etwas um, fällt es gemächlich, wie in Zeitlupe."),

  H.h3("Erweitern 1: Domino"),

  H.p("Nur aufbauen ändert sich (code/loesungen/kap10_domino.dh):"),

  ...neu("loesungen/kap10_domino.dh").map((t) => H.code(t)),

  H.p("Die Steine sind 60 hoch und 34 auseinander. Kippt einer, trifft er den nächsten etwa auf halber Höhe. Ein Abstand über 60 hieße: Er fällt flach auf den Boden, und die Kette reißt ab. Zielen muss man flach, denn ein Ball im hohen Bogen fliegt über die niedrigen Steine hinweg."),

  H.h3("Erweitern 2: die Abrissbirne"),

  H.p("Ein fester Haken, eine schwere Kugel, ein Seil dazwischen — und eine Linie, die das Seil zeichnet, denn die Welt kennt es, aber das Bild nicht (code/loesungen/kap10_abrissbirne.dh):"),

  ...neu("loesungen/kap10_abrissbirne.dh").map((t) => H.code(t)),

  H.figure("kap10_abrissbirne.png", "Die Abrissbirne ist in die Türme geschwungen und pendelt zurück; der mittlere Turm ist eingeknickt.", 420, 262),

  H.p("Am Anfang ist die Kugel 276 vom Haken entfernt, das Seil aber 280 lang: Es hängt lose, und die Kugel fällt erst ein Stück, bis es sich strafft. Von da an zieht das Seil sie auf einen Bogen um den Haken. Ein Seil zieht nur, es drückt nie — darum PHYS2D_ADD_ROPE, „höchstens so weit“."),

  H.h3("Erfinden: eine Brücke aus Brettern"),

  H.p("Eine Möglichkeit (code/loesungen/kap10_bruecke.dh): zwei feste Pfosten, dazwischen zwölf Bretter, und an jeder Stelle, an der sich zwei berühren, ein Scharnier:"),

  ...neu("loesungen/kap10_bruecke.dh").map((t) => H.code(t)),

  H.pmix(["Das Scharnier sitzt bei ", ["x - 380 / 24", true], ", einer halben Brettlänge links von der Brettmitte — dort, wo das Brett das vorige berührt. Das erste Brett hängt am linken Pfosten, das letzte am rechten. Vergleiche mit Kapitel 9: Dort waren es Punkte und Glieder, und wir haben jede Zeile der Rechnung selbst geschrieben. Hier sind es echte Bretter mit Breite, auf die Klötze fallen und liegen bleiben — mit einer Handvoll Befehlen."]),

  H.h2("Kapitel 11 · Sanft ankommen"),

  H.h3("Drehen 1"),

  H.p("in_back holt Schwung: Der Punkt fährt erst ein Stück nach links, hinter den Start, und dann immer schneller nach rechts — das Gegenstück zu out_back. inout_elastic schwingt an beiden Enden: am Anfang zittert er um den Start, schießt dann in wenigen weiten Sprüngen hinüber und zittert am Ende um das Ziel, ein Stück darüber hinaus. in_bounce hüpft am Anfang ein paar kleine Sprünge auf der Stelle und fährt dann los. Die Regel dahinter: in_x ist out_x rückwärts abgespielt."),

  H.h3("Drehen 2"),

  H.p("Mit spaeter(0) kommen alle vier Knöpfe gleichzeitig, als ein Block — die Staffelung ist weg, und das Menü wirkt sofort steifer. Mit spaeter(i * 0.3) wartet der letzte Knopf 90 Prozent der Zeit und muss den ganzen Weg in den restlichen 0,12 Sekunden fahren: Er schießt herein. Die Verzüge sollten so klein sein, dass alle Knöpfe noch genug Zeit für ihren Weg haben."),

  H.h3("Drehen 3"),

  H.p("s bestimmt, wie weit die Kurve über das Ziel hinausschießt. Mit s = 0 wird aus zurueck die Formel 1 + q³ — genau out_cubic: sanft ankommen, ohne Überschießen. Mit s = 1,70158 sind es etwa zehn Prozent des Wegs, rund 53 Punkte. Mit s = 4 sind es fast vierzig Prozent, über 200 Punkte: Die Knöpfe fliegen weit nach rechts und kommen dann zurück, als wären sie an einem Gummiband."),

  H.h3("Erweitern 1: eigene Kurven"),

  H.p("Zwei Funktionen, die p bekommen und den Anteil des Wegs liefern, und zwei Tweens mehr (code/loesungen/kap11_eigene.dh):"),

  ...neu("loesungen/kap11_eigene.dh").map((t) => H.code(t)),

  H.pmix(["treppe rundet p auf Viertel auf: Bis p = 0,25 ist der Punkt schon ein Viertel weit, dann steht er, bis p über 0,25 kommt, und so fort. Ein Tween, dessen Kurve springt, springt eben auch. zittern ist p plus eine kleine Welle, ", ["(1 - p)", true], " lässt sie zum Ende hin ausklingen, damit der Punkt genau am Ziel ankommt. Eine Kurve darf alles, solange sie bei p = 1 den Wert 1 liefert — sonst endet der Tween woanders als bei seinem Ende."]),

  H.h3("Erweitern 2: eine Kette von Bewegungen"),

  H.p("Ein eigenes kleines Programm (code/loesungen/kap11_kette.dh). Eine Strecke sind zwei Tweens, einer für x, einer für y, mit derselben Kurve; an jeder Ecke steht die Kurve, mit der es von dort weitergeht:"),

  H.code(zeilen("loesungen/kap11_kette.dh")),

  H.p("Der Trick ist TWEEN_DONE: Sobald die laufende Strecke fertig ist, legt strecke die nächste an, beginnend genau an der Ecke, an der die vorige endete. Darum gibt es keinen Sprung, obwohl jede Strecke ein neuer Tween ist."),

  H.h3("Erfinden: ein Titelbildschirm"),

  H.p("Eine Möglichkeit (code/loesungen/kap11_titel.dh): Jeder Buchstabe des Titels bekommt einen Tween für seine Höhe, mit einer Hüpfkurve, die wie spaeter um einen kleinen Verzug versetzt ist:"),

  ...neu("loesungen/kap11_titel.dh").map((t) => H.code(t)),

  H.p("huepfen ist aus drei Bögen gebaut: erst ein Fall bis ganz unten (4 · p² erreicht bei p = 0,5 die 1), dann ein kleiner Sprung auf 88 Prozent und zurück, dann ein winziger. Das Menü fährt jetzt doppelt so lange und beginnt erst nach 45 Prozent der Zeit — wenn der Titel fast steht. Alles hängt am selben Werkzeug: eine Funktion, die eine Kurve herstellt."),

  H.h2("Kapitel 12 · Ein Garten wächst"),

  H.h3("Drehen 1"),

  H.p("Mit 10 Grad wird der Baum schmal und hoch, die Krone ein Fächer wie bei einem Besen. Mit 60 Grad biegen sich die Äste nach drei Gabeln wieder nach unten, und es entsteht ein Muster aus Sechsecken wie Bienenwaben. Mit 0.5 bleibt ein kleines Bäumchen: Die Äste schrumpfen so schnell, dass alles über dem Stamm zusammen kürzer ist als der Stamm selbst — 110 · (0,5 + 0,25 + …) bleibt unter 110. Mit 0.85 wird er riesig, wächst oben aus dem Fenster, und die Äste kreuzen sich: Die Längen über dem Stamm ergeben zusammen gut 470 Punkte."),

  H.h3("Drehen 2"),

  H.p("Bei Tiefe 14 sind es 32 767 Linien statt 1023 — jede Tiefe mehr verdoppelt sie. Zu sehen ist kaum ein Unterschied: Die neuen Ästchen sind 110 · 0,72¹⁴ ≈ 1 Punkt lang, die Krone wird am Rand nur etwas dichter. Bezahlt wird mit Zeit: Auf meinem Rechner fiel das Programm von 60 auf 50 Bilder je Sekunde. Mehr Tiefe, als das Auge auflöst, ist verschwendet."),

  H.h3("Drehen 3"),

  H.p("Der Farn wird schlanker und steht steiler, mehr wie eine Feder oder ein Grashalm. Ein einziger Winkel im Bauplan verändert die ganze Pflanze, weil er in jeder Generation an jeder Verzweigung wirkt."),

  H.h3("Erweitern 1: Wind im Baum"),

  H.p("Eine Neigung, die in jedem Bild aus der Uhr kommt, und ast addiert sie zu beiden Zweigen — umso mehr, je weiter außen die Gabel sitzt (code/loesungen/kap12_wind.dh):"),

  ...neu("loesungen/kap12_wind.dh").map((t) => H.code(t)),

  H.p("Weil jede Gabel ihre Neigung zur Richtung ihres Astes dazuzählt, summiert sie sich nach außen: Der Stamm bewegt sich gar nicht, die Spitzen am meisten — wie bei einem echten Baum. Zwei Sinuswellen mit verschiedenem Tempo übereinander ergeben ein unregelmäßiges Wiegen statt eines gleichmäßigen Pendelns."),

  H.h3("Erweitern 2: Jahreszeiten"),

  H.p("Die Jahreszeit ist eine Zahl von 0 bis 4, die mit der Uhr wandert, und blatt macht daraus eine Farbe — oder -1 für „kein Blatt“ (code/loesungen/kap12_jahreszeiten.dh):"),

  ...neu("loesungen/kap12_jahreszeiten.dh").map((t) => H.code(t)),

  H.pmix(["Blätter wachsen nur bei ", ["tiefe = 0", true], ", an den äußersten Spitzen — dort, wo die Rekursion endet. Im SELECT CASE nach INT(j) ist j selbst der genaue Stand im Jahr: Im Herbst wandert der Farbton mit ", ["55 - (j - 2) * 50", true], " von Gelb (55) nach Rot (5)."]),

  H.h3("Erfinden: die Drachenkurve"),

  H.p("Ein eigenes kleines Programm mit derselben Art Bauplan, aber zwei Regeln für X und Y, rechten Winkeln und ohne Klammern (code/loesungen/kap12_drache.dh):"),

  H.code(zeilen("loesungen/kap12_drache.dh")),

  H.p("Faltet man einen Papierstreifen immer wieder in der Mitte und klappt ihn danach so auf, dass jede Falte einen rechten Winkel bildet, entsteht genau diese Kurve — Generation 12 sind zwölf Faltungen. Jede Generation macht den Drachen um den Faktor √2 größer; darum schrumpft der Schritt um denselben Faktor, und das Bild bleibt gleich groß. Die Farbe zählt die Striche mit, so sieht man, wie die eine Linie durch das ganze Bild läuft."),

  H.h2("Kapitel 13 · Die Mandelbrotmenge"),

  H.h3("Drehen 1"),

  H.p("Mit 20 Schritten sieht die Menge aufgeblasen aus: Die kleinen runden Knospen verschmelzen mit der großen Form, weil viele Punkte am Rand mehr als zwanzig Schritte brauchen, um davonzulaufen, und darum fälschlich schwarz werden. Mit 1000 Schritten ändert sich in diesem Ausschnitt fast nichts — der Rand wird nur ein wenig feiner. Gerechnet werden dann 97 statt 11 Millionen Schritte; mit Maschinencode sind das immer noch nur 0,19 Sekunden. Länger dauert das Zeichnen der 400 000 Punkte."),

  H.h3("Drehen 2"),

  H.p("Mit 2 wandert der Farbton langsam: Fast das ganze Bild bleibt im selben Blau-Violett, nur der Rand wird heller. Mit 30 springt die Farbe bei jedem Schritt um ein Zwölftel des Farbkreises, und um die Menge legen sich schmale bunte Ringe. Die Zahl sagt, wie viel Farbe ein Schritt wert ist."),

  H.h3("Drehen 3"),

  H.p("Die Übergänge bleiben weich, aber im Hintergrund erscheinen feine Bögen, wo die Farbe einen kleinen Sprung macht. Die Formel für den Bruchteil stimmt nur ungefähr, und umso genauer, je weiter z beim Überschreiten der Grenze schon draußen ist. Bei Radius 16 ist der Fehler unsichtbar, bei Radius 2 nicht."),

  H.h3("Erweitern 1: das Mausrad"),

  H.p("Der Punkt unter der Maus soll stehen bleiben. Darum rückt alles um denselben Faktor näher an ihn heran — auch die Mitte (code/loesungen/kap13_mausrad.dh):"),

  ...neu("loesungen/kap13_mausrad.dh").map((t) => H.code(t)),

  H.pmix(["Ein Schritt am Rad vergrößert anderthalbfach. ", ["mx + (mitteX - mx) * f", true], " ist die Mitte, um den Faktor f näher an mx gerückt; der Punkt mx selbst rückt nirgendwohin. Dieselbe Rechnung braucht jedes Programm, das an einer Stelle heranzoomt, ob Karte, Zeichenprogramm oder Spiel."]),

  H.h3("Erweitern 2: als PNG sichern"),

  H.p("Das Bild liegt schon fertig in einem Render-Ziel; RENDERTARGET_IMAGE macht daraus ein Bild, IMAGE_SAVE schreibt es (code/loesungen/kap13_png.dh):"),

  ...neu("loesungen/kap13_png.dh").map((t) => H.code(t)),

  H.p("IMAGE_SAVE gehört zum Modul imgfx, darum das IMPORT. Die Datei landet im Ordner des Programms — dhrt run wechselt dorthin, bevor es startet. Drei Sekunden lang steht ihr Name im Bild, damit man weiß, dass etwas geschehen ist."),

  H.h3("Erfinden: Julia-Mengen"),

  H.p("Eine Möglichkeit (code/loesungen/kap13_julia.dh): dieselbe Rechnung, aber z beginnt beim Bildpunkt und c steht fest. Ein Klick in die Mandelbrotmenge nimmt den Punkt unter der Maus als c:"),

  ...neu("loesungen/kap13_julia.dh").map((t) => H.code(t)),

  H.p("Die Mandelbrotmenge ist eine Landkarte der Julia-Mengen. Klickt man in ihr Inneres, ist die Julia-Menge ein zusammenhängendes Gebilde; klickt man weit draußen, zerfällt sie in Staub; und klickt man an den Rand, sieht die Julia-Menge aus wie die Mandelbrotmenge an genau dieser Stelle — Spiralen im Seepferdchental, Rüssel im Elefantental."),

  H.h2("Kapitel 14 · Wolken und Landschaften"),

  H.h3("Drehen 1"),

  H.p("Mit 0.03 ist das Bild ein Inselmeer: viele kleine Inseln, jede mit einem kleinen Gipfel, weil man sich viermal so schnell über die Rauschfläche bewegt und auf dieselbe Bildbreite viermal so viele Hügel kommen. Mit 0.002 sieht man von oben auf ein Stück eines großen Kontinents mit einer Küste am Rand — und alles wirkt unscharf, denn auch die feinste der sechs Lagen ist jetzt viel größer als ein Bildpunkt."),

  H.h3("Drehen 2"),

  H.p("Mit einer Lage wird die Karte glatt wie Knetmasse: runde Kontinente, glatte Küsten, Berge als weiche Kuppen. Mit zehn Lagen sieht sie fast genauso aus wie mit sechs — die Lagen sieben bis zehn sind kleiner als ein Bildpunkt und haben zusammen nur wenige Prozent der Stärke. Mehr Lagen kosten Zeit und zeigen nichts."),

  H.h3("Drehen 3"),

  H.p("Fast alles wird Land, und aus den Meeren werden Seen mit Stränden ringsum. Die Berge werden höher und bekommen mehr Schnee. Der Meeresspiegel ist nur die Höhe, an der der Farbverlauf von Blau zu Sand wechselt — wer ihn verschiebt, verschiebt die ganze Welt."),

  H.h3("Erweitern 1: eine Insel"),

  H.p("Zum Rauschen kommt eine Schale: in der Mitte hoch, zum Rand hin tief (code/loesungen/kap14_inseln.dh):"),

  ...neu("loesungen/kap14_inseln.dh").map((t) => H.code(t)),

  H.figure("kap14_inseln.png", "Die Insel: Das Rauschen bestimmt die Küste und die Berge, die Schale sorgt dafür, dass ringsum Meer ist.", 420, 262),

  H.p("d ist 0 in der Mitte und 1 an den Rändern, 2 in den Ecken. + 0.25 hebt alles an, − 0.6 · d senkt es nach außen ab. Wo die Schale das Land unter 0 drückt, ist Meer — das Rauschen entscheidet nur noch, wie die Küste verläuft und wo die Berge stehen."),

  H.h3("Erweitern 2: Höhenlinien"),

  H.p("Eine Höhenlinie liegt dort, wo ein Punkt und sein Nachbar auf verschiedenen Stufen liegen — FLOOR(h / LINIE) zählt die Stufen (code/loesungen/kap14_hoehenlinien.dh):"),

  ...neu("loesungen/kap14_hoehenlinien.dh").map((t) => H.code(t)),

  H.figure("kap14_hoehenlinien.png", "Höhenlinien alle 0.04: Wo sie dicht liegen, ist der Hang steil.", 420, 262),

  H.p("Hier zahlt sich das Feld gelaende aus: Die Linie braucht die Höhe des Nachbarn, und die steht schon da. Unter Wasser gibt es keine Linien (hier > 0). Wie auf einer echten Karte liegen die Linien an steilen Hängen dicht und auf flachen Wiesen weit auseinander."),

  H.h3("Erfinden: Wolken, die ziehen"),

  H.p("Eine Möglichkeit (code/loesungen/kap14_wolken.dh): ein zweites Rauschen, in jedem Bild neu gerechnet, auf einem kleinen Bild, das durchsichtig über der Karte liegt:"),

  ...neu("loesungen/kap14_wolken.dh").map((t) => H.code(t)),

  H.pmix(["Die Wolken ziehen, weil die Zeit in die x-Koordinate eingeht: ", ["x * 0.025 - zeit * 0.15", true], " schaut jede Sekunde an einer anderen Stelle der Rauschfläche nach. Wo das Rauschen klein ist, bleibt der Himmel frei; darüber wird die Wolke dichter, bis höchstens 85 Prozent Deckkraft. Das Wolkenbild ist nur 200 mal 125 Punkte groß und wird viermal vergrößert — Wolken brauchen keine Schärfe, und so sind es pro Bild 25 000 statt 400 000 Rechnungen."]),

  H.h2("Kapitel 15 · Leuchten"),

  H.h3("Drehen 1"),

  H.p("Mit 0.9 verblasst fast nichts: Die Lichter ziehen lange Bänder, und nach einer halben Minute ist der Schirm voller Schleifen, die sich kreuzen. Mit 0.05 bleibt von jeder Spur nach einer Sekunde ein Zwanzigstel — die Lichter werden zu Kometen mit kurzem Schweif. REST ist ein Anteil je Sekunde, keine Länge; wie lang die Spur aussieht, hängt auch davon ab, wie schnell das Licht fliegt."),

  H.h3("Drehen 2"),

  H.p("Mit staerke 0 kommt nichts dazu: Das Bild sieht aus wie ohne Shader (Taste G). Mit 8 werden die Linien zu dicken, grellen Röhren, und wo zwei sich kreuzen, wird es fast weiß. Die 3.0 in glow.fs ist der Abstand der Nachbarn: Mit 1.0 liest der Shader nur 4 Bildpunkte weit, der Schein wird eng wie ein Rand; mit 8.0 reicht er 32 Bildpunkte weit, ist aber gitterig — zwischen den gelesenen Punkten liegen jetzt Lücken von 8 Bildpunkten, und das Muster der 81 Proben wird sichtbar. Ein weiterer Schein braucht mehr Proben, nicht nur größere Abstände."),

  H.h3("Drehen 3"),

  H.p("Das Plasma steht auf dem Kopf. Auf einem Standbild fällt das kaum auf, denn ein Plasma hat kein Oben. Man sieht es an der Bewegung: Die Wellen von sin(y / 30.0 − zeit) wandern jetzt nach oben statt nach unten, und die schrägen Bänder von sin((x + y) / 60.0 + zeit) laufen von links unten nach rechts oben. Wer ein Bild mit Schrift durch einen Shader schickt, sieht es sofort — dort ist der Fehler nicht zu übersehen."),

  H.h3("Erweitern 1: ein Röhrenbildschirm"),

  H.p("Der Shader (code/loesungen/kap15_roehre.fs) macht drei Dinge nacheinander:"),

  H.code(zeilen("loesungen/kap15_roehre.fs")),

  H.p("Im Programm wird er geladen wie glow.fs, und G wechselt zwischen beiden (code/loesungen/kap15_roehre.dh):"),

  ...neu("loesungen/kap15_roehre.dh").map((t) => H.code(t)),

  H.figure("kap15_roehre.png", "Die Leuchtspuren auf einer alten Röhre: dunkle Zeilen zwischen den hellen, das Bild leicht gewölbt.", 420, 262),

  H.p("Die Wölbung ist ein Trick beim Lesen, nicht beim Schreiben: Jeder Punkt holt seine Farbe von einer Stelle, die etwas weiter außen liegt — je weiter vom Mittelpunkt, desto mehr. Dadurch rückt der Inhalt zur Mitte zusammen, und die Ränder wirken nach hinten gebogen. Ein Shader kann nicht bestimmen, wohin sein Punkt geht; er kann nur bestimmen, woher er seine Farbe nimmt."),

  H.h3("Erweitern 2: Glühen im Takt"),

  H.p("Eine Möglichkeit (code/loesungen/kap15_takt.dh):"),

  ...neu("loesungen/kap15_takt.dh").map((t) => H.code(t)),

  H.pmix(["AUDIO_NOTE baut einen Ton aus Welle, Tonhöhe, Dauer und Hüllkurve: 55 Hertz sind ein tiefes A, 2 Millisekunden Anstieg machen den Schlag hart, 180 Millisekunden Abfall auf 0 lassen ihn verklingen. ", ["AUDIO_PEAK()", true], " liefert den lautesten Ausschlag seit der letzten Frage; beim Schlag gemessen 0,66, eine halbe Sekunde später fast 0. Weil es zwischen zwei Bildern auch einmal nichts zu messen gibt, nimmt das Programm den größeren Wert von neuem Ausschlag und 90 Prozent des alten — so fällt das Glühen weich ab, statt zu flackern. Kapitel 18 und 19 machen aus Tönen Musik und aus Musik Bilder."]),

  H.h3("Erfinden: Licht malt Plasma"),

  H.p("Eine Möglichkeit: Die Lichter malen weiß, und ein Shader verbindet glow.fs mit plasma.fs (code/loesungen/kap15_plasmaspur.fs):"),

  H.code(stueckLoesung("loesungen/kap15_plasmaspur.fs", "void main", "}", 3)),

  H.pmix(["Der Shader liest nur den roten Anteil des Bildes — bei Weiß sind alle drei gleich — und nimmt ihn als Helligkeit: ", ["licht", true], ". Die Farbe kommt aus der Funktion plasma, die dieselbe Formel rechnet wie in Schritt 1. Ein Shader muss also nicht entweder ein Bild erzeugen oder eines verändern — er kann beides zugleich. Im Programm ändert sich nur, welche Datei geladen wird, dass die Lichter weiß sind und dass der Shader die Zeit bekommt (code/loesungen/kap15_plasmaspur.dh)."]),
];
