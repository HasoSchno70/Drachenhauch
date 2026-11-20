const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Sanft ankommen", 11),

  H.h2("Was wir bauen"),

  H.figure("kap11_menue.png", "Das Menü im Hereinkommen: Der oberste Knopf ist über sein Ziel hinausgeschossen und federt zurück, die anderen folgen versetzt.", 470, 294),

  H.p("Ein Titelmenü, wie es am Anfang vieler Spiele steht. Die Knöpfe kommen nicht einfach herein; sie gleiten nacheinander von links, schießen ein Stück über ihren Platz hinaus und federn zurück. Fährt die Maus über einen Knopf, rückt er ein wenig vor. Ein Klick schickt alle wieder hinaus. Nichts davon ist schwer zu rechnen, aber die Art, wie etwas ankommt, entscheidet, ob es sich lebendig oder hölzern anfühlt."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Tween ", "ist ein Wert, der in einer bestimmten Zeit von einem Anfang zu einem Ende wandert. Das Programm legt ihn einmal an und fragt dann in jedem Bild, wo er gerade ist."),
  H.bulletRich("Eine Kurve ", "sagt, wie er wandert: gleichmäßig, langsam anfahrend, über das Ziel hinaus, hüpfend."),
  H.bulletRich("Eine eigene Kurve ", "ist eine Funktion, und eine Funktion lässt sich wie jeder andere Wert weitergeben."),

  H.h2("Schritt 1: Kurven ansehen"),

  H.p("Das Modul tween hat neunzehn Kurven eingebaut. Um sie zu vergleichen, fahren sieben Punkte gleichzeitig von links nach rechts, jeder mit seiner Kurve, alle in 1,5 Sekunden:"),

  H.code(stueck("kap11/1_kurven.dh", "DIM namen AS ARRAY OF STRING", "NEXT")),

  H.pmix([["TWEEN_NEW(LINKS, RECHTS, 1500, name)", true], " legt einen Tween an: Anfang, Ende, Dauer in Millisekunden, Kurve. Von da an liefert ", ["TWEEN_VALUE(t)", true], " in jedem Bild den Wert, der gerade dran ist, und ", ["TWEEN_DONE(t)", true], " sagt, ob er angekommen ist. Mitzählen muss das Programm nichts; der Tween kennt die Uhr. Alle zwanzigstel Sekunde lässt das Programm einen Abdruck liegen:"]),

  H.figure("kap11_1_kurven.png", "Sieben Kurven. Wo die Abdrücke dicht liegen, war der Punkt langsam, wo sie weit auseinander liegen, schnell. out_back und out_elastic schießen über das Ziel hinaus, out_bounce hüpft am Ende.", 470, 294),

  H.p("Das Bild liest sich wie ein Fahrplan. linear: gleiche Abstände, gleiches Tempo. in_quad fährt langsam an und wird schneller — in heißt „am Anfang sanft“. out_quad macht es umgekehrt: schnell los, sanft ans Ziel — out heißt „am Ende sanft“, und das ist für Dinge, die ankommen, fast immer die richtige Wahl. inout_sine ist an beiden Enden sanft. Die letzten drei tun mehr als bremsen: out_back schießt hinaus und kommt zurück, out_elastic schwingt wie eine Feder, out_bounce hüpft wie ein Ball, der auf den Boden fällt."),

  H.h2("Schritt 2: das Menü"),

  H.p("Die Knöpfe sollen ankommen wie out_back. Diese Kurve ist eine kleine Formel, und hier schreiben wir sie selbst — gleich wird klar, warum:"),

  H.code(stueck("kap11/2_menue.dh", "FUNCTION zurueck", "END FUNCTION")),

  H.p("Eine Kurve bekommt den Fortschritt p von 0 bis 1 und liefert, welcher Anteil des Wegs dann zurückgelegt ist. Bei p = 0 ist es 0, bei p = 1 ist es 1, und dazwischen kommt zurueck kurz über 1 — das ist das Überschießen. Statt eines Namens nimmt TWEEN_NEW auch so eine Funktion."),

  H.p("Jetzt sollen die Knöpfe nacheinander kommen. Der zweite soll etwas später losfahren als der erste, der dritte noch etwas später. Das lässt sich in die Kurve legen: Eine Kurve, die die ersten 15 Prozent der Zeit stillsteht und dann erst wie zurueck fährt. Und weil jeder Knopf einen anderen Verzug braucht, schreiben wir eine Funktion, die solche Kurven HERSTELLT:"),

  H.code(stueck("kap11/2_menue.dh", "' Eine Kurve, die erst", "END FUNCTION")),

  H.pmix([["AS FUNCREF", true], " heißt: spaeter liefert eine Funktion. ", ["FUNCTION(p) …", true], " in der Mitte ist eine Funktion ohne Namen, ein Lambda, und es merkt sich den Verzug, mit dem es angelegt wurde. ", ["spaeter(0.3)", true], " ist also eine Kurve, die bis p = 0,3 bei 0 bleibt und dann in der restlichen Zeit den ganzen Weg fährt. Damit bekommt jeder Knopf seine eigene:"]),

  H.code(stueck("kap11/2_menue.dh", "DIM fahrt AS ARRAY OF TWEEN", "NEXT")),

  H.p("Der Schub unter der Maus ist ein zweiter Tween je Knopf, von 0 nach 18. Wichtig ist, wovon er losfährt, wenn sich etwas ändert: vom Wert, den er gerade hat."),

  H.code(stueck("kap11/2_menue.dh", "SUB mausPruefen", "END SUB")),

  H.p("Ein Klick schickt alle hinaus. TWEEN_REVERSE vertauscht Anfang und Ende und fängt von vorn an — die Knöpfe fahren mit derselben Kurve zurück, also wieder versetzt:"),

  H.code(stueck("kap11/2_menue.dh", "IF MOUSE_HIT(0) THEN", "END IF")),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap11/2_menue.dh")),

  H.h2("Warum so?"),

  H.h3("Eine Funktion als Wert"),

  H.p("Ohne Funktionen als Werte hätte das Modul tween für jeden Wunsch einen eigenen Befehl gebraucht: einen mit Verzug, einen mit Verzug und Überschießen, einen, der zweimal ansetzt … oder das Programm hätte für jeden Knopf von Hand ausgerechnet, ob seine Zeit schon gekommen ist. So fragt der Tween nur eines: „Wie weit bist du bei p?“ Was die Antwort ist, entscheidet die Funktion, die man ihm gibt. Und weil Funktionen Werte sind, kann eine Funktion wie spaeter neue herstellen. Dieselbe Idee steckt in SORT mit einer eigenen Vergleichsfunktion und in ARRAY_MAP."),

  H.h3("Nach der Uhr, nicht nach Bildern"),

  H.p("Ein Tween rechnet in Millisekunden. Auf einem Rechner, der dreißig Bilder in der Sekunde schafft, kommt ein Knopf nach 1,2 Sekunden an, auf einem mit hundertzwanzig auch — nur mit mehr Zwischenbildern. Darum gibt es kein TWEEN_UPDATE, das man in jedem Bild aufrufen müsste: Der Wert ergibt sich jedes Mal neu aus der Uhr."),

  H.h3("Vom aktuellen Wert aus"),

  H.pmix(["Ein Tween springt nie, wenn er dort anfängt, wo der alte gerade war. Darum steht in mausPruefen ", ["jetzt = TWEEN_VALUE(schub[k])", true], " und dann ", ["TWEEN_NEW(jetzt, ziel, …)", true], ": Verlässt die Maus einen Knopf, der erst halb vorgerückt ist, fährt er von genau dort zurück. Das ist die wichtigste Regel für alles, was auf Bedienung reagiert."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Sprung"),

  H.p("Schreibt man den Schub mit festem Anfang,"),

  H.code(["schub[k] = TWEEN_NEW(0.0, ziel, 180, \"out_quad\")"]),

  H.p("dann geht das Vorrücken gut — es fängt ja bei 0 an. Aber verlässt die Maus den Knopf, ist das Ziel 0 und der Anfang auch: Der Knopf springt in einem Bild zurück. Kein Fehler, keine Meldung, nur ein Ruck, den man sieht und lange nicht findet."),

  H.h3("Ein Tween, der schon läuft"),

  H.p("Ein Tween läuft ab dem Moment, in dem er angelegt wird. Die Kurvenschau legt ihre Tweens vor SCREEN an, und ein Fenster aufzumachen dauert einen Moment. Ohne den Neustart nach SCREEN lag der erste Abdruck der linearen Fahrt bei x ≈ 270 statt bei 160 — ein Fünftel der Fahrt war vorbei, bevor das erste Bild kam. Darum:"),

  H.code(stueck("kap11/1_kurven.dh", "' Ein Tween laeuft ab dem Anlegen", "NEXT")),

  H.h3("Ein vertippter Kurvenname"),

  H.code(zeilen("kap11/fehler_kurve.dh")),
  H.code(["Laufzeitfehler in fehler_kurve.dh:3: TWEEN: unbekanntes easing",
          "'out_bonce'. Meintest du out_bounce, inout_bounce, out_back? Alle",
          "Kurven nennt TWEEN_EASINGS()"], { out: true }),

  H.p("Kurven sind Namen in einer Zeichenkette, und eine Zeichenkette prüft Drachenhauch erst, wenn die Zeile läuft. Die Meldung schlägt die nächsten Namen vor. (Bis zur Fassung 2026.52 sagte sie nur „unbekanntes easing“ — ohne Vorschlag, und die Liste der Namen musste man im Handbuch suchen. Und eine eigene Funktion statt eines Namens ging gar nicht. Beides ist beim Schreiben dieses Kapitels entstanden.)"),

  H.h3("Eine Kurve mit zwei Werten"),

  H.pmix(["Eine Kurve bekommt genau einen Wert, den Fortschritt. ", ["TWEEN_NEW(0, 1, 500, FUNCTION(p, q) p)", true], " läuft noch an, aber das erste TWEEN_VALUE bricht ab: „die Kurve nimmt 2 Wert(e), TWEEN_VALUE gibt ihr 1 (den Fortschritt 0..1)“. Ebenso, wenn die Kurve statt einer Zahl etwas anderes liefert."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze in der Kurvenschau in_back, inout_elastic und in_bounce an die Stelle der ersten drei Namen. Sage vorher, wie ihre Spuren aussehen."),
  H.bulletRich("Drehen 2. ", "Ersetze im Menü spaeter(i * 0.15) durch spaeter(0) und dann durch spaeter(i * 0.3)."),
  H.bulletRich("Drehen 3. ", "Setze in zurueck die Zahl s auf 0 und dann auf 4. Was macht s?"),
  H.bulletRich("Erweitern 1. ", "Eigene Kurven in der Kurvenschau: eine Treppe, die in vier Sprüngen ankommt, und eine, die unterwegs zittert."),
  H.bulletRich("Erweitern 2. ", "Eine Kette von Bewegungen: Ein Ball fährt die Ecken eines Rechtecks ab, jede Strecke mit einer anderen Kurve. Ist eine fertig, beginnt die nächste."),
  H.bulletRich("Erfinden. ", "Ein Titelbildschirm: Die Buchstaben des Titels fallen einzeln von oben und hüpfen auf, danach kommt das Menü. Oder eine Anzeige, deren Zahl hochzählt; Karten, die sich auffächern; ein Fenster, das aufploppt. Anhang A zeigt den Titel."),

  H.figure("kap11_titel.png", "Eine Möglichkeit für die Erfinden-Übung: Die Buchstaben fallen versetzt — vorn sind sie schon gelandet und hüpfen noch, hinten fallen sie noch.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul tween: ", "alle Kurven, Pause, Endlos-Tweens mit TWEEN_NEW_LOOP und TWEEN_NEW_PINGPONG."),
  H.bulletRich("Handbuch, Sprache → Funktionen als Werte: ", "FUNCREF, Lambdas, gebundene Methoden, und was ein Lambda von den Variablen um es herum sieht."),
];
