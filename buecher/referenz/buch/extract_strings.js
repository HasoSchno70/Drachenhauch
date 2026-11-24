// Zieht alle uebersetzbaren Texte DIESES Buchs in den Katalog i18n/<sprache>.json.
// Das Werkzeug selbst liegt fuer alle Buecher in buecher/_i18n/extract_strings.js;
// Aufruf und Argumente wie dort beschrieben.
process.env.DH_BUCH = __dirname;
require("../../_i18n/extract_strings");
