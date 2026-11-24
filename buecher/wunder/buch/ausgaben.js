// Holt die Meldungen und Werkzeug-Ausgaben, die DIESES Buch abdruckt, von
// dhrt in einer anderen Sprache nach i18n/ausgaben.<sprache>.json. Das
// Werkzeug liegt fuer alle Buecher in buecher/_i18n/ausgaben.js.
process.env.DH_BUCH = __dirname;
require("../../_i18n/ausgaben");
