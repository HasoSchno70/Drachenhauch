const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil VI · Die Sprache für Fortgeschrittene"),

  H.p("Die Programme werden größer, und damit wird wichtig, wie man sie ordnet. Dieser Teil zeigt die Mittel, mit denen man ein großes Programm in Stücke teilt, die einzeln verständlich bleiben: Klassen, die voneinander erben, Funktionen als Werte, Abläufe, die warten können, Daten in einer Zeile umgeformt, Fehler, die man auffängt. Jedes Kapitel stellt dabei die Frage, wann ein Mittel hilft und wann es nur schwerer zu lesen macht."),

  H.chapter("Die Partikelmaschine", 20),

  H.h2("Was wir bauen"),

  H.figure("kap20_2_maschine.png", "Ein Feuer mit Rauch, Schnee und — der Maus nach — ein Funkenregen. Vier Effekte aus einem Baukasten.", 470, 294),

  H.p("Ein Baukasten für Effekte. Ein Feuer brennt, darüber steigt Rauch auf und treibt im Wind, Schnee fällt über das ganze Bild, und wo die Maus ist, sprüht Magie — mit der Leertaste wechselt sie zwischen Funkenregen, Wirbel und Blasen. Alle bestehen aus Teilchen, kleinen Kreisen, die entstehen, sich bewegen, ihre Farbe ändern und vergehen. Was sie unterscheidet, ist nur, wie sie beginnen, wie sie sich bewegen und welche Farbe sie haben. Dieses Kapitel zeigt zwei Arten, solche Unterschiede zu schreiben: als Unterklassen, die erben und überschreiben, und als Rezepte aus Lambdas. Und es fragt, wann welche besser ist."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Teilchen ist nur Daten: ", "Ort, Tempo, Alter, Lebensdauer, Größe."),
  H.bulletRich("Ein Effekt kann fast alles: ", "Teilchen auswerfen, bewegen, zeichnen, entfernen. Nur wie ein Teilchen beginnt, kündigt er an, ohne es zu sagen."),
  H.bulletRich("Feuer, Rauch, Schnee ", "erben von Effekt und schreiben nur hin, was bei ihnen anders ist."),
  H.bulletRich("Ein Rezept ", "ist ein Effekt, dem man drei Lambdas mitgibt — die Magie, ohne neue Klasse."),

  H.h2("Schritt 1: ein Teilchen"),

  H.code(stueck("kap20/2_maschine.dh", "CLASS Teilchen", "END CLASS")),

  H.p("Klassen kennen wir aus Kapitel 8. Ein Teilchen hat hier keine Methode, die etwas tut — nur anteil, das sagt, wie weit es in seinem Leben ist. Was es tut, entscheidet der Effekt, zu dem es gehört. So kann ein Feuerteilchen dieselbe Klasse sein wie eine Schneeflocke."),

  H.h2("Schritt 2: ein Effekt, der fast alles kann"),

  H.code(stueck("kap20/2_maschine.dh", "CLASS Effekt", "END CLASS")),

  H.pmix(["schritt ist der Ablauf, den alle Effekte teilen. ", ["vorrat", true], " sammelt angefangene Teilchen: Bei 160 je Sekunde kommen in einem Bild 2,7 hinzu; zwei werden ausgeworfen, der Rest wartet aufs nächste Bild. Entfernt wird von hinten nach vorn, wie bei den Funken in Kapitel 19. Für das Bewegen und die Farbe gibt es eine einfache Fassung — geradeaus, weiß und verblassend —, die Unterklassen ändern dürfen. Und eine Methode hat gar keinen Rumpf: ", ["ABSTRACT FUNCTION neues() AS Teilchen", true], ". Sie ist nur angekündigt. Einen Effekt kann man darum nicht selbst erzeugen, nur eine Unterklasse, die sagt, wie ihr Teilchen beginnt. schritt ruft neues trotzdem auf — und landet beim Feuer, beim Rauch oder beim Schnee, je nachdem, welcher Effekt es ist."]),

  H.h2("Schritt 3: Feuer, eine Unterklasse"),

  H.code(stueck("kap20/2_maschine.dh", "CLASS Feuer", "END CLASS")),

  H.pmix([["CLASS Feuer EXTENDS Effekt", true], " heißt: Ein Feuer ist ein Effekt. Es hat alle Felder und Methoden eines Effekts, ohne sie noch einmal hinzuschreiben, und schreibt neu, was bei ihm anders ist. Init ruft zuerst ", ["SUPER.Init", true], ", die Init des Effekts, und schaltet danach das Leuchten an. neues sagt, wo ein Flammenteilchen beginnt: unten, breit gestreut, schnell nach oben. bewegen ", ["überschreibt", true], " die Fassung des Effekts — aber nicht ganz: ", ["SUPER.bewegen(t, dt)", true], " lässt erst den Effekt geradeaus bewegen, danach zieht das Feuer das Teilchen zur Mitte und macht es kleiner. So wird die Flamme oben spitz. farbe geht von Gelb über Orange nach Rot."]),

  H.h2("Schritt 4: Rauch und Schnee"),

  H.code(stueck("kap20/2_maschine.dh", "CLASS Rauch", "END CLASS")),

  H.p("Der Rauch ist noch kürzer: wenige Teilchen, langsam, groß. Er bewegt sich wie jeder Effekt, wächst dabei und treibt nach rechts. Der Schnee (im ganzen Programm unten) beginnt am oberen Rand über die ganze Breite und schaukelt beim Fallen mit SIN. Jede Unterklasse ist eine Handvoll Zeilen, weil der Effekt die Arbeit tut."),

  H.h2("Schritt 5: ein Rezept aus Lambdas"),

  H.code(stueck("kap20/2_maschine.dh", "CLASS Rezept", "END CLASS")),

  H.pmix(["Ein Rezept ist auch ein Effekt, aber eins, das nichts Eigenes weiß. Was es tun soll, bekommt es mit drei Lambdas, Funktionen ohne Namen wie in Kapitel 11: ", ["anfang", true], " baut ein Teilchen, ", ["zug", true], " bewegt es zusätzlich, ", ["faerben", true], " gibt die Farbe. Die Lambdas stehen in Feldern und werden aufgerufen wie Methoden: ", ["zug(t, dt)", true], ". Beim Schreiben dieses Kapitels ging das noch nicht — dhrt suchte einen Befehl ZUG, und von außen hieß es „Methode existiert nicht“. Jetzt ruft ein Feld, das eine Funktion hält, diese Funktion. Die drei Rezepte der Magie:"]),

  H.code(stueck("kap20/2_maschine.dh", "SUB kreisen", "FUNCTION(t) RGBA(120")),

  H.p("Ein Lambda ist eine Zeile. Der Funkenregen lässt die Teilchen in alle Richtungen fliegen und zieht sie mit 320 nach unten; seine Farbe wandert mit dem Alter durch den Farbkreis. Für den Wirbel reicht eine Zeile nicht, also ruft sein Lambda die benannte SUB kreisen. Die Blasen steigen langsam und schaukeln."),

  H.figure("kap20_wirbel.png", "Das zweite Rezept, der Wirbel: Jedes Teilchen kreist, während die Maus weiterzieht, und hinterlässt eine Kette aus Schleifen.", 420, 262),

  H.h2("Schritt 6: alle zusammen"),

  H.code(stueck("kap20/2_maschine.dh", "DIM e AS Effekt", "NEXT")),

  H.pmix(["Die Hauptschleife weiß nicht, welcher Effekt welcher ist. Sie ruft ", ["e.schritt", true], " und ", ["e.zeichnen", true], " — und jeder Effekt nimmt dabei seine eigenen Fassungen von neues, bewegen und farbe. Welche Methode läuft, entscheidet das Objekt, nicht die Stelle, an der sie aufgerufen wird. Ein neuer Effekt braucht darum keine Zeile in der Hauptschleife. Das ganze Programm:"]),

  H.code(zeilen("kap20/2_maschine.dh")),

  H.h2("Warum so?"),

  H.h3("Vererbung für „ist ein“, Lambda für „verhält sich so“"),

  H.p("Ein Feuer ist ein Effekt — der Satz stimmt, und darum passt EXTENDS: Das Feuer bekommt alles, was ein Effekt kann, und dazu einen eigenen Namen, eigene Felder, eigene Init. Wer im Programm „ein Feuer“ sagen will, schreibt NEW Feuer. Ein Rezept dagegen ist eine Sammlung von Verhalten: wie es beginnt, wie es zieht, wie es aussieht. Dafür braucht es keine Klasse, nur drei Funktionen."),

  H.h3("Warum man meistens das Lambda will"),

  H.p("Drei Anfänge, drei Arten zu ziehen und drei Färbungen ergeben 27 Effekte. Als Klassen wären das 27 Unterklassen, und jede neue Färbung brauchte neun neue. Als Rezepte sind es drei Zeilen je Teil, die man frei zusammensteckt — die zweite Übung tut genau das. Vererbung bindet zusammen, was zusammen bleiben soll; Lambdas lassen getrennt, was man mischen will. Darum steht heute in vielen Programmen eine kleine Klasse, die das Gerüst gibt, und Lambdas, die das Verhalten liefern. Die Partikelmaschine hat beides: Effekt ist das Gerüst, Rezept das Steckbrett."),

  H.h3("Warum ABSTRACT"),

  H.p("neues könnte im Effekt auch einfach ein weißes Teilchen in der Mitte liefern. Dann aber würde ein vertippter Name in einer Unterklasse — neue statt neues — nicht auffallen: Das Feuer hätte eine zusätzliche Methode, die niemand aufruft, und würfe weiße Teilchen aus statt Flammen. Mit ABSTRACT bricht schon das Übersetzen ab, weil das Feuer die angekündigte Methode nicht ausfüllt. Ein Fehler beim Übersetzen ist der billigste, den es gibt."),

  H.h2("Was schiefgehen kann"),

  H.h3("Die angekündigte Methode vertippt"),

  H.code(zeilen("kap20/fehler_abstract.dh")),
  H.code(["fehler_abstract.dh:12: Compile-Fehler: NEW FUNKEN: die Klasse kuendigt",
          "eine Methode an, ohne sie auszufuellen (neues). Entweder eine",
          "Unterklasse nehmen, die sie ausschreibt, oder die Methode in FUNKEN mit",
          "einem Rumpf versehen."], { out: true }),

  H.p("Funken hat neue statt neues. Das Programm startet gar nicht erst, und die Meldung nennt die Methode, die fehlt."),

  H.h3("SUPER vergessen"),

  H.pmix(["Schreibt man im Rauch bewegen ohne die Zeile ", ["SUPER.bewegen(t, dt)", true], ", ersetzt die neue Fassung die alte ganz: Die Teilchen wachsen, aber niemand bewegt sie mehr. Der Rauch steht als grauer Ball über dem Feuer, und das Tempo, das neues ihm gab, ist vergessen. Überschreiben heißt ersetzen; wer erweitern will, ruft die alte Fassung selbst auf."]),

  H.h3("Lambdas in einer Schleife"),

  H.code(zeilen("kap20/fehler_schleife.dh")),
  H.code(["40 40 40"], { out: true }),

  H.p("Gemeint waren 10, 20 und 30. Ein Lambda, das im Hauptprogramm entsteht, sieht die Variablen des Hauptprogramms so, wie sie sind, wenn es aufgerufen wird — und nach der Schleife ist i 4. In einer Funktion wäre es anders: Dort kopiert ein Lambda die lokalen Variablen beim Anlegen. Die Rezepte der Magie stehen darum nicht in einer Schleife, sondern jedes für sich."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze die Rate des Feuers auf 1000 und auf 20."),
  H.bulletRich("Drehen 2. ", "Lass das Feuer nicht leuchten (leuchten = FALSE). Was ändert sich?"),
  H.bulletRich("Drehen 3. ", "Füge in Effekt.bewegen eine Schwerkraft hinzu: t.vy = t.vy + 100 * dt. Welche Effekte merken es?"),
  H.bulletRich("Erweitern 1. ", "Ein eigener Effekt: Glühwürmchen, die über dem Boden schwirren und jedes in seinem Takt blinken."),
  H.bulletRich("Erweitern 2. ", "Effekte kombinieren: Die Taste K baut ein neues Rezept aus Anfang, Zug und Farbe der vorhandenen, zufällig gemischt."),
  H.bulletRich("Erfinden. ", "Ein Editor mit Reglern: Feuer, Wind und Schnee lassen sich einstellen, während alles läuft. Oder: ein Effekt, der andere enthält (ein Lagerfeuer aus Feuer, Rauch und Funken); Teilchen, die vom Boden abprallen; ein Effekt, der der Musik aus Kapitel 19 folgt. Anhang A zeigt den Editor."),

  H.figure("kap20_regler.png", "Erfinden: drei Regler aus dem gui-Modul stellen Feuer, Wind und Schnee — während die Maschine weiterläuft.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Klassen und Strukturen: ", "EXTENDS, SUPER über mehrere Ebenen, ABSTRACT, und welche Typen ein Feld vom Typ einer Klasse annimmt."),
  H.bulletRich("Handbuch, Sprache → Funktionen als Werte: ", "FUNCREF, Lambdas, gebundene Methoden, ein FUNCREF im Feld eines Objekts."),
  H.bulletRich("examples/167_vererbung.dh: ", "Vererbung, die sich nicht wiederholt: SUPER und ABSTRACT an einem kleinen Beispiel."),
];
