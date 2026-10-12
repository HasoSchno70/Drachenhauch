# Duell

Ein Weltraum-Duell für zwei über das Netz. Einer ist Wirt, der andere
verbindet sich als Gast; jeder steuert ein Schiff, fünf Leben, wer zuerst
keins mehr hat, verliert.

- **Pfeile links/rechts** drehen, **Pfeil hoch** gibt Schub, **Leertaste**
  schießt
- Ohne Argumente fragt ein kleines Menü: **W** macht dich zum Wirt, eine
  Adresse eintippen und **Enter** verbindet als Gast
- Direkt: `dhrt run main.dh -- wirt [port]` bzw.
  `dhrt run main.dh -- gast 192.168.1.20 [port]` (Vorgabe Port 47110)

## Auf einem Rechner ausprobieren

Zwei Fenster nebeneinander genügen: eines als Wirt, eines als Gast mit
`127.0.0.1`. Gesteuert wird das Fenster, das gerade den Fokus hat.

Ein echtes Netz verliert Pakete und verzögert sie. `relay.dh` spielt das
nach:

```
dhrt run main.dh -- wirt 47110
dhrt run relay.dh -- 47111 127.0.0.1 47110 80 30 10
dhrt run main.dh -- gast 127.0.0.1 47110 --udp-port 47111
```

Das Relay hält jedes Paket 80 ms auf, dazu bis zu 30 ms Schwankung, und
verwirft 10 %. Das eigene Schiff bewegt sich trotzdem sofort, das fremde
gleitet; rechts oben stehen Ping und verlorene Pakete.

## Wie es gebaut ist

- **TCP** für alles, was ankommen muss: Hallo, Willkommen, Rundenende,
  Tschüss. TCP ist ein Strom ohne Nachrichtengrenzen, darum sammelt
  `zeileLesen` bis zum Zeilenende.
- **UDP** für das Spiel: der Gast schickt 30-mal je Sekunde seine Eingabe
  (ein Byte) und hängt die zwei vorigen an, falls ein Paket verloren geht. Der
  Wirt rechnet die Welt und schickt den Stand als Binärpaket zurück
  (`BUFFER_SET_F32`, `NET_UDP_SEND` mit einem `BUFFER`, `NET_UDP_RECV_BYTES`).
- **Vorhersage:** der Gast rechnet sein eigenes Schiff mit denselben
  Unterprogrammen wie der Wirt (`spiel.dh`) voraus. Kommt ein Stand, übernimmt
  er die Wahrheit des Wirts und wendet alle Eingaben danach noch einmal an.
  Das fremde Schiff zeigt er zwischen den letzten zwei Ständen.
- Bleibt der Gast drei Sekunden still oder bricht die Verbindung ab, wartet
  der Wirt auf den nächsten. Ein zweiter Gast bekommt eine Absage.

Geprüft wird es in `tests/pruef/beispiel_duell.dhtest`: Wirt und Gast als
zwei Prozesse ohne Fenster (`--bot N --ohne-fenster`), einmal über
`127.0.0.1`, einmal über das Relay, dazu ein hart beendeter Gast und eine
Absage. Am Ende muss das Schiff des Gastes bei beiden genau gleich stehen.
Ausprobiert ist es außerdem zwischen Windows und Linux (Wirt in der WSL, Gast
unter Windows über deren Adresse). Nicht ausprobiert: zwei echte Rechner
hinter Routern und Firewalls — dort muss der Port beim Wirt erreichbar sein
(TCP und UDP).

Entstanden beim Dogfooding am 2026-10-11; dabei kamen heraus: UDP konnte nur
Text senden und empfangen, der Absender ließ sich bei IPv6 nicht sauber
zerlegen, ein hart beendeter Gast ließ den Wirt mit einem Laufzeitfehler
abbrechen (TCP und UDP), die letzte Nachricht vor `NET_CLOSE` ging verloren,
und jedes UDP-Paket startete einen Faden für die Namensauflösung.
