// Saetze mit eingesetzten Zahlen ("Die 41 Programme ... benutzen 312
// Befehle"). Stuende der fertige Satz im Katalog, aenderte sich sein
// Schluessel mit jeder neuen Zahl, und das uebersetzte Buch fiele still ins
// Deutsche zurueck. Darum schreibt ein Kapitel
//
//     H.p(mitZahlen("Die {0} Programme benutzen {1} Befehle.", [p, b]))
//
// Das liefert den fertigen deutschen Satz (das deutsche Buch merkt nichts)
// und merkt sich die Vorlage. i18n.js schlaegt dann die VORLAGE im Katalog
// nach und setzt die Zahlen erst in den uebersetzten Satz ein; der Katalog
// enthaelt "Die {0} Programme ..." -> "The {0} programs ...".
"use strict";

const vorlagen = new Map();

function fuellen(vorlage, werte) {
  return vorlage.replace(/\{(\d+)\}/g, (m, i) => String(werte[Number(i)]));
}

function mitZahlen(vorlage, werte) {
  const text = fuellen(vorlage, werte);
  vorlagen.set(text, { vorlage, werte });
  return text;
}

module.exports = { mitZahlen, vorlagen, fuellen };
