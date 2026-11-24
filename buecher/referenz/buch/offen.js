// Gibt die noch nicht uebersetzten Texte bestimmter Kapiteldateien DIESES Buchs aus.
// Das Werkzeug selbst liegt fuer alle Buecher in buecher/_i18n/offen.js;
// Aufruf und Argumente wie dort beschrieben.
process.env.DH_BUCH = __dirname;
require("../../_i18n/offen");
