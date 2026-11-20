#version 330
// Loesung zu Kapitel 15, Erweitern 1: ein alter Roehrenbildschirm --
// gewoelbtes Glas, dunkle Zeilen dazwischen und Ecken, die ins Dunkle
// laufen.
in vec2 fragTexCoord;
out vec4 finalColor;

uniform sampler2D texture0;
uniform vec2 groesse;

void main() {
    // Die Woelbung: je weiter vom Mittelpunkt, desto weiter nach aussen
    // gelesen. Das Bild wirkt dadurch nach vorn gewoelbt.
    vec2 mitte = fragTexCoord - 0.5;
    float abstand = dot(mitte, mitte);
    vec2 ort = 0.5 + mitte * (1.0 + abstand * 0.25);
    // Was ueber den Rand hinaus gelesen wuerde, ist das Gehaeuse.
    if (ort.x < 0.0 || ort.x > 1.0 || ort.y < 0.0 || ort.y > 1.0) {
        finalColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    vec3 farbe = texture(texture0, ort).rgb;
    // Jede zweite Bildzeile dunkler: die Zeilen des Elektronenstrahls.
    float zeile = ort.y * groesse.y;
    farbe *= 0.75 + 0.25 * sin(zeile * 3.14159);
    // Zum Rand hin dunkler.
    farbe *= 1.0 - abstand * 1.6;
    finalColor = vec4(farbe * 1.3, 1.0);
}
