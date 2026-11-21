module.exports = (H) => [
  H.chapter("Vorwort"),

  H.p("Dieses Buch lehrt eine Programmiersprache, ohne je ein Programm zu zeigen, das nur dazu da ist, eine Sprachregel vorzuführen. Jedes Kapitel baut etwas, das man gern ansieht: einen Sternenflug, ein Feuerwerk, ein Fraktal, einen Schwarm, ein Instrument, eine Landschaft, über die man fliegt. Die Sprache lernt man unterwegs — genau dort, wo sie gebraucht wird, und mit dem Grund, warum man sie an dieser Stelle so und nicht anders einsetzt."),

  H.p("Das „Warum“ ist der eigentliche Gegenstand des Buchs. Ein Programm abzutippen und laufen zu sehen ist schön, aber danach weiß man oft nur, dass es geht. Wer weiß, warum der Funke ein STRUCT ist und keine fünf Felder, warum die Schleife rückwärts zählt und warum die Schwere in Pixeln pro Sekunde gerechnet wird, kann das nächste Programm selbst schreiben. Darum erklärt jedes Kapitel seine Entscheidungen, nennt die naheliegende Alternative und sagt, warum sie hier schlechter wäre."),

  H.h2("Für wen"),

  H.p("Für alle, die die Grundlagen schon haben: Variablen, Schleifen, Entscheidungen, ein Fenster mit SCREEN. Das ist der Stoff der ersten zehn Kapitel des Buchs „Der Einstieg“, und es ist auch das, was man aus fast jeder anderen Programmiersprache mitbringt. Mathematik braucht es wenig — wo ein Sinus oder eine Wurzel vorkommt, erklärt das Kapitel, was sie tut, und zeigt es an einem Bild."),

  H.h2("Wie ein Kapitel aufgebaut ist"),

  H.bulletRich("Was wir bauen: ", "ein Bild des Ergebnisses und zwei Sätze dazu."),
  H.bulletRich("Der Plan: ", "was das Programm tun muss, in Worten, bevor eine Zeile Code dasteht."),
  H.bulletRich("Schritt für Schritt: ", "der Code in kleinen Stücken, nach jedem Schritt ein Zwischenstand, der schon läuft."),
  H.bulletRich("Warum so? ", "die Entscheidungen, die der Code trifft, jeweils mit der Alternative und dem Grund."),
  H.bulletRich("Was schiefgehen kann: ", "echte Fehler mit den echten Meldungen von dhrt, wörtlich, und wie man aus ihnen herausfindet."),
  H.bulletRich("Übungen ", "in drei Stufen: Drehen (eine Zahl ändern und vorhersagen, was passiert), Erweitern (etwas dazubauen), Erfinden (etwas Eigenes). Lösungen in Anhang A."),
  H.bulletRich("Zum Weiterlesen: ", "die Stellen im Handbuch, die das Thema vollständig beschreiben."),

  H.h2("Wie man es liest"),

  H.p("Am meisten bringt das Buch, wenn man die Programme abtippt, statt sie zu kopieren. Das klingt mühsam und ist es ein wenig, aber beim Tippen liest man jede Zeile, und die Tippfehler, die man dabei macht, sind die besten Lehrer: Sie führen genau zu den Meldungen, die dieses Buch erklärt."),

  H.p("Und bevor man ein Programm startet, lohnt eine Vorhersage: Was wird gleich passieren? Wer richtig lag, hat verstanden. Wer danebenlag, hat gerade etwas gelernt, und meist mehr als beim richtigen Raten."),

  H.h2("Die Programme"),

  H.p("Jedes Programm und jeder Zwischenstand liegt als Datei bei, im Ordner buecher/wunder/code des Drachenhauch-Repositorys, die Lösungen der Übungen unter code/loesungen. Das Buch druckt sie aus diesen Dateien ab, und eine Prüfsammlung lässt sie bei jeder neuen Fassung von Drachenhauch laufen: Jedes Programm übersetzt, jede abgedruckte Ausgabe stimmt, jede Fehlermeldung hat den Wortlaut, den dhrt wirklich ausgibt. Auch die Bilder entstehen aus den Programmen, nicht von Hand."),

  H.note("Dieser Stand enthält das Vorwort, Kapitel 0 (die Werkstatt), Teil I mit den Kapiteln 1 bis 4, Teil II mit den Kapiteln 5 bis 8, Teil III mit den Kapiteln 9 bis 12, Teil IV mit den Kapiteln 13 bis 16 und Teil V mit den Kapiteln 17 bis 19, dazu die Lösungen all dieser Kapitel. Die übrigen Kapitel folgen nach der Gliederung in docs/intern/entwurf-buecher.md.", "Arbeitsstand"),
];
