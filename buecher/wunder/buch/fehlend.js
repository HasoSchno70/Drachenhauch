// Meldet je Kapiteldatei DIESES Buchs, wie viele Texte noch nicht uebersetzt sind.
// Das Werkzeug selbst liegt fuer alle Buecher in buecher/_i18n/fehlend.js;
// Aufruf und Argumente wie dort beschrieben.
process.env.DH_BUCH = __dirname;
require("../../_i18n/fehlend");
