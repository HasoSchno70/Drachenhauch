"""Hover-Tooltips fuer Drachenhauch-Built-ins.

Format: ``name_lowercase -> (signature, beschreibung)``. Wird von
``CodeEditor`` beim Hover ueber einem Identifier konsultiert. Inhalt
1:1 aus dem alten CTk-Editor uebernommen.
"""
from __future__ import annotations


BUILTIN_DOCS: dict[str, tuple[str, str]] = {
    # Datentypen-Helfer
    "str$":  ("STR$(value) AS STRING", "Wandelt einen Wert in einen STRING."),
    "val":   ("VAL(s$) AS INTEGER/FLOAT", "Liest eine Zahl aus dem STRING."),
    "int":   ("INT(zahl) AS INTEGER", "Schneidet die Nachkommastellen ab (Floor)."),
    "abs":   ("ABS(zahl)", "Absolutwert."),
    "sqr":   ("SQR(zahl) AS FLOAT", "Quadratwurzel."),
    "sqrt":  ("SQRT(zahl) AS FLOAT", "Quadratwurzel (Alias fuer SQR)."),
    "rnd":   ("RND([n])", "Zufallszahl. Ohne Argument 0..1.0; mit n: 0..n-1."),
    "len":   ("LEN(s/array)", "Laenge eines STRINGs oder erste Dim eines ARRAYs."),
    "chr$":  ("CHR$(code)", "Zeichen aus Unicode-Codepoint."),
    "asc":   ("ASC(s$)", "Codepoint des ersten Zeichens."),
    "upper$": ("UPPER$(s)", "Grossbuchstaben."),
    "lower$": ("LOWER$(s)", "Kleinbuchstaben."),
    "rgb":   ("RGB(r, g, b) AS INTEGER", "Farbe aus 3x INTEGER 0..255."),
    # Strings
    "left$":  ("LEFT$(s, n)", "Erste n Zeichen."),
    "right$": ("RIGHT$(s, n)", "Letzte n Zeichen."),
    "mid$":   ("MID$(s, start[, n])", "Substring ab start (0-basiert), optional n Zeichen."),
    "instr":  ("INSTR(haystack, needle[, start])", "0-basiert, -1 wenn nicht gefunden."),
    "replace$": ("REPLACE$(s, alt, neu)", "Ersetzt alle Vorkommen."),
    "trim$":  ("TRIM$(s)", "Entfernt fuehrende/nachstehende Whitespaces."),
    "split$": ("SPLIT$(s, trenner) AS ARRAY OF STRING", "Spaltet den STRING."),
    "join$":  ("JOIN$(arr, trenner) AS STRING", "Fuegt ein STRING-Array zu einem STRING zusammen."),
    "dimsize":  ("DIMSIZE(arr, n)", "Groesse der n-ten Dimension (0-basiert)."),
    "dimcount": ("DIMCOUNT(arr)", "Anzahl der Dimensionen."),
    # Datei-I/O
    "openfile":  ("OPENFILE(pfad, modus) AS FILE", 'Modus: "r", "w", "a".'),
    "closefile": ("CLOSEFILE(f)", "Schliesst eine geoeffnete Datei."),
    "readline":  ("READLINE(f) AS STRING", "Liest eine Zeile (ohne \\n)."),
    "readall$":  ("READALL$(f) AS STRING", "Gesamten Datei-Inhalt einlesen."),
    "endoffile": ("ENDOFFILE(f) AS BOOLEAN", "TRUE am Datei-Ende."),
    "writeline": ("WRITELINE(f, text)", "Schreibt eine Zeile mit Newline."),
    "write":     ("WRITE(f, text)", "Schreibt ohne Newline."),
    "fileexists": ("FILEEXISTS(pfad) AS BOOLEAN", "Existenz pruefen."),
    # Datei/Verzeichnis pfadbasiert (dhrt-only)
    "direxists": ("DIREXISTS(pfad) AS BOOLEAN", "Existiert das Verzeichnis?"),
    "dirlist":   ("DIRLIST(pfad) AS ARRAY OF STRING", "Eintragsnamen eines Verzeichnisses (sortiert)."),
    "mkdir":     ("MKDIR(pfad)", "Verzeichnis anlegen (inkl. Elternverzeichnisse)."),
    "deletefile": ("DELETEFILE(pfad)", "Datei loeschen."),
    "rename":    ("RENAME(alt, neu)", "Datei/Verzeichnis umbenennen oder verschieben."),
    "writeall":  ("WRITEALL(pfad, text)", "Text in Datei schreiben (ueberschreibt)."),
    "readlines": ("READLINES(pfad) AS ARRAY OF STRING", "Datei als Zeilen-Array lesen."),
    "filesize":  ("FILESIZE(pfad) AS INTEGER", "Dateigroesse in Bytes."),
    "pathjoin":  ("PATHJOIN(a, b, ...) AS STRING", "Pfadteile mit '/' verbinden."),
    # Betriebssystem (WP A): Argumente, Umgebung, Prozesse
    "argc":      ("ARGC() AS INTEGER", "Anzahl der Argumente, die dem Programm uebergeben wurden."),
    "arg$":      ("ARG$(nummer) AS STRING",
                  "Argument Nummer n (0-basiert). Ausserhalb: Leerstring, kein Fehler. "
                  "Beim Start ueber dhrt: alles hinter '--'; in der exportierten Exe: alle."),
    "getenv$":   ("GETENV$(name[, vorgabe]) AS STRING",
                  "Umgebungsvariable lesen. Fehlt sie, kommt die Vorgabe (sonst Leerstring)."),
    "setenv":    ("SETENV(name, wert)",
                  "Umgebungsvariable setzen -- gilt fuer dieses Programm und seine "
                  "Kindprozesse (SHELL), nicht fuer die Konsole des Aufrufers."),
    "cwd$":      ("CWD$() AS STRING",
                  "Aktuelles Arbeitsverzeichnis. Achtung: dhrt wechselt beim Start ins "
                  "Verzeichnis der .dh-Datei (fuer relative Asset-Pfade)."),
    "chdir":     ("CHDIR(pfad)", "Arbeitsverzeichnis wechseln."),
    "exit":      ("EXIT([code])",
                  "Programm sofort beenden, code wird der Rueckgabewert (0..255, "
                  "Vorgabe 0). Wird von TRY/CATCH NICHT gefangen."),
    "eprint":    ("EPRINT(text)",
                  "Zeile nach stderr statt stdout -- fuer Meldungen, die nicht zu den "
                  "Nutzdaten gehoeren. Anders als PRINT ein Builtin, also mit Klammern."),
    "shell":     ("SHELL(programm, ...) AS INTEGER",
                  "Programm starten, auf das Ende warten, Rueckgabewert liefern (-1 = "
                  "durch Signal beendet). Argumente EINZELN uebergeben, nicht als eine "
                  "Kommandozeile. Ausgabe geht direkt auf die Konsole."),
    "shell_out$": ("SHELL_OUT$(programm, ...) AS STRING",
                   "Wie SHELL, sammelt aber die stdout-Ausgabe ein und liefert sie. "
                   "stderr des Kindes bleibt stderr."),
    # Grafik
    "screen":   ("SCREEN(w, h[, titel[, skala]])",
                 "Fenster oeffnen. skala>1 macht Pixel groesser (Retro-Look)."),
    "cls":      ("CLS([farbe])", "Bildschirm loeschen."),
    "plot":     ("PLOT(x, y[, farbe])", "Pixel zeichnen."),
    "line":     ("LINE(x1, y1, x2, y2[, farbe])", "Linie zeichnen."),
    "box":      ("BOX(x1, y1, x2, y2[, farbe])", "Gefuelltes Rechteck."),
    "rect":     ("RECT(x1, y1, x2, y2[, farbe])", "Rechteck-Rahmen."),
    "circle":   ("CIRCLE(x, y, r[, farbe])", "Gefuellter Kreis."),
    "text":     ("TEXT(x, y, s[, farbe])", "Text rendern."),
    "textrot":  ("TEXTROT(x, y, s, winkel[, skala[, farbe]])",
                 "Text ZENTRIERT auf (x,y), um das Zentrum gedreht (Grad, wie "
                 "DRAWIMAGEROT) und skaliert -- Score-Popups, schraege Labels."),
    "flip":     ("FLIP()", "Doppelpuffer-Swap (anzeigen)."),
    "sleep":    ("SLEEP(ms)", "Pausiert das Programm."),
    "keypressed":   ("KEYPRESSED(code) AS BOOLEAN", "Pruefe Taste (KEY_*)."),
    "quitrequested": ("QUITREQUESTED() AS BOOLEAN", "Hat User Fenster geschlossen?"),
    "mousex":   ("MOUSEX() AS INTEGER", "Aktuelle X-Position."),
    "mousey":   ("MOUSEY() AS INTEGER", "Aktuelle Y-Position."),
    "mousebutton": ("MOUSEBUTTON(n) AS BOOLEAN", "n=0/1/2 (links/mitte/rechts)."),
    "mousewheel": ("MOUSEWHEEL() AS INTEGER", "Mausrad-Delta seit letztem Aufruf (+hoch/-runter)."),
    "mouse_visible": ("MOUSE_VISIBLE(an)",
                      "OS-Maus-Cursor zeigen/verstecken (fuer eigenes Fadenkreuz/Cursor-Sprite)."),
    "mouse_lock": ("MOUSE_LOCK(an)",
                   "Cursor fangen: verstecken + im Fenster einsperren (First-Person-Maussteuerung). "
                   "FALSE gibt frei."),
    "mouse_hidden": ("MOUSE_HIDDEN() AS BOOLEAN", "Ist der Cursor versteckt/gefangen?"),
    "screenwidth": ("SCREENWIDTH() AS INTEGER", "Logische Fensterbreite; 0 vor SCREEN."),
    "screenheight": ("SCREENHEIGHT() AS INTEGER", "Logische Fensterhoehe; 0 vor SCREEN."),
    "loadimage": ("LOADIMAGE(pfad) AS IMAGE", "Bild laden."),
    "drawimage": ("DRAWIMAGE(img, x, y)", "Bild zeichnen."),
    "drawimagepart": ("DRAWIMAGEPART(img, sx, sy, sw, sh, x, y)", "Bildausschnitt."),
    "drawimageflipped": ("DRAWIMAGEFLIPPED(img, x, y[, flipX[, flipY]])",
                         "Bild gespiegelt zeichnen (z.B. Sprite nach links/rechts)."),
    "collides": ("COLLIDES(x1, y1, w1, h1, x2, y2, w2, h2) AS BOOLEAN",
                 "AABB-Rechteck-Kollision."),
    "imagewidth": ("IMAGEWIDTH(img) AS INTEGER", "Breite eines IMAGE."),
    "imageheight": ("IMAGEHEIGHT(img) AS INTEGER", "Hoehe eines IMAGE."),
    "loadsound": ("LOADSOUND(pfad) AS SOUND", "Soundeffekt laden."),
    "playsound": ("PLAYSOUND(s[, loops, vol])", "Sound abspielen."),
    "stopsound": ("STOPSOUND(s)", "Sound stoppen."),
    "unloadsound": ("UNLOADSOUND(s)",
                    "Sound stoppen und seinen Puffer freigeben (gegen Puffer-"
                    "Akkumulation in langen Songs). Index bleibt gueltig, erneutes "
                    "Abspielen wirft."),
    "audio_sound_count": ("AUDIO_SOUND_COUNT() AS INTEGER",
                          "Anzahl lebender (nicht freigegebener) Sound-Slots -- Diagnose."),
    "playmusic": ("PLAYMUSIC(pfad[, loops, vol])", "Musik streamen."),
    "stopmusic": ("STOPMUSIC()", "Musik stoppen."),
    "drawtilemap": ("DRAWTILEMAP(tileset, map, tw, th, sx, sy)",
                    "2D-Tilemap rendern. -1 = transparent."),
    # Math
    "sin":   ("SIN(rad) AS FLOAT",   "Sinus, Eingabe in Radiant."),
    "cos":   ("COS(rad) AS FLOAT",   "Kosinus, Eingabe in Radiant."),
    "tan":   ("TAN(rad) AS FLOAT",   "Tangens."),
    "atan":  ("ATAN(x) AS FLOAT",    "Arcustangens."),
    "atan2": ("ATAN2(y, x) AS FLOAT", "Winkel von (x, y) zur X-Achse, in Radiant."),
    "floor": ("FLOOR(zahl) AS INTEGER", "Abrundung."),
    "ceil":  ("CEIL(zahl) AS INTEGER",  "Aufrundung."),
    "round": ("ROUND(zahl[, dezimalstellen]) AS INTEGER/FLOAT", "Banker-Rundung; mit dezimalstellen auf N Nachkommastellen (FLOAT)."),
    "log":   ("LOG(x[, basis]) AS FLOAT", "Natuerlicher Log oder Log zur Basis."),
    "exp":   ("EXP(x) AS FLOAT",     "e hoch x."),
    "pow":   ("POW(basis, exp) AS FLOAT", "basis hoch exp."),
    "min":   ("MIN(a, b, ...)",      "Minimum von 1+ Argumenten."),
    "max":   ("MAX(a, b, ...)",      "Maximum von 1+ Argumenten."),
    "clamp": ("CLAMP(wert, min, max)", "Wert in [min..max] beschraenken."),
    "sign":  ("SIGN(zahl) AS INTEGER", "-1, 0 oder 1."),
    "sgn":   ("SGN(zahl) AS INTEGER", "-1, 0 oder 1 (Alias fuer SIGN)."),
    "asin":  ("ASIN(x) AS FLOAT",    "Arcussinus (Radiant); x in [-1, 1]."),
    "acos":  ("ACOS(x) AS FLOAT",    "Arcuscosinus (Radiant); x in [-1, 1]."),
    "hypot": ("HYPOT(x, y) AS FLOAT", "SQR(x*x + y*y) ohne Overflow."),
    "deg":   ("DEG(rad) AS FLOAT",   "Radiant -> Grad."),
    "rad":   ("RAD(grad) AS FLOAT",  "Grad -> Radiant."),
    "lerp":  ("LERP(a, b, t) AS FLOAT", "Lineare Interpolation a..b bei t (nicht geklemmt)."),
    "remap": ("REMAP(v, in_lo, in_hi, out_lo, out_hi) AS FLOAT", "v aus [in_lo,in_hi] auf [out_lo,out_hi] abbilden."),
    "frac":  ("FRAC(x) AS FLOAT",    "Nachkommaanteil (vorzeichenbehaftet): x - TRUNC(x)."),
    # Farb-Helfer
    "red":   ("RED(farbe) AS INTEGER",   "Rot-Kanal 0..255 aus 0xRRGGBB."),
    "green": ("GREEN(farbe) AS INTEGER", "Gruen-Kanal 0..255."),
    "blue":  ("BLUE(farbe) AS INTEGER",  "Blau-Kanal 0..255."),
    "color_lerp": ("COLOR_LERP(c1, c2, t) AS INTEGER", "Zwei Farben kanalweise mischen (t 0..1)."),
    "hsv":   ("HSV(h, s, v) AS INTEGER", "HSV -> 0xRRGGBB. h in Grad, s/v in [0,1]."),
    # Zeit / Random
    "millis":    ("MILLIS() AS INTEGER", "Millisekunden seit Programmstart."),
    "randomize": ("RANDOMIZE([seed])",   "Zufallsgenerator setzen (deterministisch mit Seed)."),
    "randint":   ("RANDINT(lo, hi) AS INTEGER", "Zufalls-INTEGER in [lo, hi] (inklusiv)."),
    "randf":     ("RANDF(lo, hi) AS FLOAT", "Zufalls-FLOAT in [lo, hi)."),
    "choice":    ("CHOICE(array) AS T", "Zufaelliges Element eines 1D-Arrays."),
    "shuffle":   ("SHUFFLE(array)", "Mischt ein 1D-Array IN PLACE (Fisher-Yates)."),
    "time$":     ("TIME$() AS STRING",   'Aktuelle Uhrzeit "HH:MM:SS".'),
    "date$":     ("DATE$() AS STRING",   'Aktuelles Datum "YYYY-MM-DD".'),
    "sort": ("SORT(arr [, absteigend?-BOOL | comparator-FUNCREF])", "1D IN PLACE sortieren. BOOL-Flag fuer absteigend ODER FUNCREF-Comparator(a,b)->INT (<0/0/>0). Zweiarg-Formen: nur native Runtime."),
    # Array-Aggregate (dhrt-only -- nativ in der Runtime, nicht im Tree-Walker)
    "array_sum": ("ARRAY_SUM(arr) AS INTEGER/FLOAT", "Summe eines 1D-Zahl-Arrays."),
    "array_avg": ("ARRAY_AVG(arr) AS FLOAT", "Durchschnitt eines 1D-Zahl-Arrays (nicht leer)."),
    "array_min": ("ARRAY_MIN(arr) AS T", "Kleinstes Element eines 1D-Zahl-Arrays."),
    "array_max": ("ARRAY_MAX(arr) AS T", "Groesstes Element eines 1D-Zahl-Arrays."),
    "array_fill": ("ARRAY_FILL(arr, wert)", "Fuellt alle Elemente mit wert (IN PLACE)."),
    "array_copy": ("ARRAY_COPY(arr) AS ARRAY", "Liefert eine unabhaengige Kopie des Arrays."),
    # Dynamische 1D-Arrays (dhrt-only)
    "array_push": ("ARRAY_PUSH(arr, wert) AS INTEGER", "Element ans Ende; liefert die neue Laenge."),
    "array_pop": ("ARRAY_POP(arr) AS T", "Entfernt letztes Element und liefert es (Array nicht leer)."),
    "array_insert": ("ARRAY_INSERT(arr, idx, wert) AS INTEGER", "Element an Index einfuegen (0..len); neue Laenge."),
    "array_remove_at": ("ARRAY_REMOVE_AT(arr, idx) AS T", "Element an Index entfernen und liefern."),
    "redim": ("REDIM(arr, laenge)", "1D-Array auf laenge bringen (waechst mit Default, schrumpft schneidet ab; Bestand bleibt)."),
    # Strings extra
    "padl$":     ("PADL$(s, breite[, fueller]) AS STRING", "String linksbuendig auffuellen."),
    "padr$":     ("PADR$(s, breite[, fueller]) AS STRING", "String rechtsbuendig auffuellen."),
    "repeat$":   ("REPEAT$(s, n) AS STRING", "String n-mal wiederholen."),
    "space$":    ("SPACE$(n) AS STRING",  "String aus n Leerzeichen."),
    "hex$":      ("HEX$(n) AS STRING",    'INTEGER als Hex ("FF", "1A2B", ...).'),
    # String-Erweiterungen (dhrt-only)
    "ltrim$":    ("LTRIM$(s) AS STRING",  "Fuehrende Leerzeichen entfernen."),
    "rtrim$":    ("RTRIM$(s) AS STRING",  "Abschliessende Leerzeichen entfernen."),
    "reverse$":  ("REVERSE$(s) AS STRING", "Zeichen umkehren."),
    "startswith": ("STARTSWITH(s, praefix) AS BOOLEAN", "Beginnt s mit praefix?"),
    "endswith":  ("ENDSWITH(s, suffix) AS BOOLEAN", "Endet s mit suffix?"),
    "contains":  ("CONTAINS(s, teil) AS BOOLEAN", "Enthaelt s den Teilstring?"),
    "bin$":      ("BIN$(n) AS STRING",    "INTEGER als Binaerstring (mit Vorzeichen)."),
    "oct$":      ("OCT$(n) AS STRING",    "INTEGER als Oktalstring (mit Vorzeichen)."),
    "isnumeric": ("ISNUMERIC(s) AS BOOLEAN", "Ist s als Zahl (INT/FLOAT) parsebar?"),
    "tryval":    ("TRYVAL(s, default) AS INTEGER/FLOAT", "s zu Zahl parsen, sonst default (robustes VAL)."),
    # Maps
    "mapput":    ("MAPPUT(map, key$, value)", "Wert unter Schluessel speichern."),
    "mapget":    ("MAPGET(map, key$)", "Wert zum Schluessel - Fehler wenn nicht vorhanden."),
    "mapgetor":  ("MAPGETOR(map, key$, default)", "Wert oder Default."),
    "maphas":    ("MAPHAS(map, key$) AS BOOLEAN", "Schluessel existiert?"),
    "mapremove": ("MAPREMOVE(map, key$) AS BOOLEAN", "Eintrag entfernen, TRUE bei Erfolg."),
    "mapsize":   ("MAPSIZE(map) AS INTEGER", "Anzahl Eintraege."),
    "mapkeys":   ("MAPKEYS(map) AS ARRAY OF STRING", "Liste aller Schluessel."),
    "mapclear":  ("MAPCLEAR(map)", "Alle Eintraege entfernen."),
    # --- Modul: audio (IMPORT "audio") -----------------------------
    "audio_init":               ("AUDIO_INIT([freq[, channels[, buffer]]])",
                                  "Mixer initialisieren. Default 44100Hz, stereo, 512-Buffer."),
    "audio_set_num_channels":   ("AUDIO_SET_NUM_CHANNELS(n)",
                                  "Anzahl simultaner Channels setzen (Default 8)."),
    "audio_num_channels":       ("AUDIO_NUM_CHANNELS() AS INTEGER",
                                  "Aktuelle Channel-Anzahl."),
    "audio_busy_channels":      ("AUDIO_BUSY_CHANNELS() AS INTEGER",
                                  "Anzahl aktuell aktiver Channels."),
    "audio_pause_all":          ("AUDIO_PAUSE_ALL()", "Alle Channels pausieren."),
    "audio_resume_all":         ("AUDIO_RESUME_ALL()", "Alle Channels fortsetzen."),
    "audio_stop_all":           ("AUDIO_STOP_ALL()", "Alle Channels stoppen."),
    "audio_play":               ("AUDIO_PLAY(sound[, loops[, volume[, fade_in_ms]]]) AS AUDIO_CHANNEL",
                                  "Sound abspielen, Channel-Handle zurueckgeben. loops=0 einmal, N -> N+1 Durchlaeufe, -1 endlos; fade_in_ms blendet ein."),
    "audio_pause":              ("AUDIO_PAUSE(ch)", "Channel pausieren."),
    "audio_resume":             ("AUDIO_RESUME(ch)", "Channel fortsetzen."),
    "audio_stop":               ("AUDIO_STOP(ch[, fade_out_ms])", "Channel stoppen, optional ausblenden."),
    "audio_is_playing":         ("AUDIO_IS_PLAYING(ch) AS BOOLEAN", "Spielt der Channel?"),
    "audio_volume":             ("AUDIO_VOLUME(ch, v)", "Channel-Lautstaerke (0..1)."),
    "audio_set_volume":         ("AUDIO_SET_VOLUME(ch, v)", "Channel-Lautstaerke setzen (Alias fuer AUDIO_VOLUME)."),
    "audio_get_volume":         ("AUDIO_GET_VOLUME(ch) AS FLOAT", "Aktuelle Lautstaerke."),
    "audio_pan":                ("AUDIO_PAN(ch, left, right)", "Stereo-Pan -- left/right je 0..1."),
    "audio_pitch":              ("AUDIO_PITCH(ch, faktor)",
                                  "Tonhoehe/Geschwindigkeit (1.0=normal, 2.0=Oktave hoeher). "
                                  "Klassiker: pro Schuss leicht variieren (0.9 + RANDF()*0.2)."),
    "audio_music_pitch":        ("AUDIO_MUSIC_PITCH(faktor)",
                                  "Musik-Pitch (1.0=normal; ueberlebt LOAD/QUEUE). "
                                  "Slow-Motion: zusammen mit der Spielzeit absenken."),
    "audio_music_get_pitch":    ("AUDIO_MUSIC_GET_PITCH() AS FLOAT", "Aktueller Musik-Pitch."),
    "audio_pan_pos":            ("AUDIO_PAN_POS(ch, p)",
                                  "Stereo-Position direkt: 0=links, 0.5=Mitte, 1=rechts. "
                                  "Fasst nur das Pan an (Volume bleibt)."),
    "audio_pan_slide":          ("AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms)",
                                  "Einmalige Stereo-Wanderung von Position von nach nach "
                                  "(je 0=links..1=rechts), nicht-blockierend; bleibt am Ziel."),
    "audio_autopan":            ("AUDIO_AUTOPAN(ch, periode_s[, tiefe])",
                                  "Endloses Pendeln links<->rechts (startet links). periode_s = "
                                  "Dauer einer Runde, tiefe 0..1 = Auslenkung; periode_s <= 0 = aus."),
    "audio_music_load":         ("AUDIO_MUSIC_LOAD(path)", "Musik laden (streamt vom File)."),
    "audio_music_play":         ("AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])",
                                  "Musik starten. loops=-1 bedeutet endlos (Default), "
                                  "loops=N spielt N+1 Durchlaeufe; fade_in_ms blendet ein."),
    "audio_music_stop":         ("AUDIO_MUSIC_STOP([fade_out_ms])",
                                  "Musik stoppen, optional ausfaden (nicht-blockierend; "
                                  "AUDIO_MUSIC_BUSY() bleibt bis zum Fade-Ende TRUE)."),
    "audio_music_pause":        ("AUDIO_MUSIC_PAUSE()", "Musik pausieren."),
    "audio_music_resume":       ("AUDIO_MUSIC_RESUME()", "Musik fortsetzen."),
    "audio_music_volume":       ("AUDIO_MUSIC_VOLUME(v)", "Musik-Lautstaerke (0..1)."),
    "audio_music_set_volume":   ("AUDIO_MUSIC_SET_VOLUME(v)", "Musik-Lautstaerke setzen (Alias fuer AUDIO_MUSIC_VOLUME)."),
    "audio_music_get_volume":   ("AUDIO_MUSIC_GET_VOLUME() AS FLOAT", "Aktuelle Musik-Lautstaerke."),
    "audio_music_position":     ("AUDIO_MUSIC_POSITION() AS FLOAT", "Position in Sekunden."),
    "audio_music_busy":         ("AUDIO_MUSIC_BUSY() AS BOOLEAN", "Spielt Musik?"),
    "audio_music_queue":        ("AUDIO_MUSIC_QUEUE(path)", "Naechsten Track queuen."),
    "audio_tone":               ("AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, volume]]) AS SOUND",
                                  'Ton synthetisieren. Waveforms: "sine", "square", "saw", "triangle", "noise".'),
    "audio_noise":              ("AUDIO_NOISE(dauer_ms[, volume]) AS SOUND", "Weisses Rauschen."),
    # --- Modul: curves (IMPORT "curves") ---------------------------
    "curve_lerp":           ("CURVE_LERP(a, b, t) AS FLOAT", "Lineare Interpolation a..b mit t in [0,1]."),
    "curve_smoothstep":     ("CURVE_SMOOTHSTEP(edge0, edge1, x) AS FLOAT",
                              "GLSL-style smoothstep mit S-Kurven-Uebergang."),
    "curve_smootherstep":   ("CURVE_SMOOTHERSTEP(edge0, edge1, x) AS FLOAT",
                              "Wie smoothstep, aber mit kontinuierlicher 2. Ableitung."),
    "curve_bezier":         ("CURVE_BEZIER(t, p0, p1, p2, p3) AS FLOAT",
                              "Cubic Bezier 1D durch 4 Control-Points."),
    "curve_bezier2":        ("CURVE_BEZIER2(t, x0, y0, x1, y1, x2, y2, x3, y3) AS TUPLE",
                              "Cubic Bezier 2D -> Tupel (x, y)."),
    "curve_catmull":        ("CURVE_CATMULL(t, p0, p1, p2, p3) AS FLOAT",
                              "Catmull-Rom-Spline 1D zwischen p1 und p2."),
    "curve_catmull2":       ("CURVE_CATMULL2(t, x0, y0, x1, y1, x2, y2, x3, y3) AS TUPLE",
                              "Catmull-Rom 2D -> Tupel (x, y)."),
    "curve_hermite":        ("CURVE_HERMITE(t, p0, p1, m0, m1) AS FLOAT",
                              "Cubic-Hermite zwischen p0 und p1 mit Tangenten m0, m1."),
    # --- Modul: net (IMPORT "net") ---------------------------------
    "net_tcp_listen":       ("NET_TCP_LISTEN(port [, bind_addr$]) AS NET_LISTENER",
                              "TCP-Server auf Port. port=0 = OS-zugewiesener Port. "
                              "bind_addr$ optional: \"::\" fuer IPv6 statt IPv4."),
    "net_tcp_accept":       ("NET_TCP_ACCEPT(listener) AS NET_SOCKET",
                              "Wartende Verbindung annehmen, oder NIL wenn keine da ist."),
    "net_tcp_connect":      ("NET_TCP_CONNECT(host, port) AS NET_SOCKET",
                              "TCP-Verbindung zu host:port aufbauen (DNS + Connect je 5s Timeout)."),
    "net_send":             ("NET_SEND(sock, text) AS INTEGER",
                              "UTF-8-Bytes schicken. Rueckgabe: tatsaechlich gesendete Bytes."),
    "net_recv":             ("NET_RECV(sock, max_bytes) AS STRING",
                              "Bis zu max_bytes empfangen. Leerer STRING wenn nichts (non-blocking). "
                              "Ein an der Lesegrenze zerschnittenes Mehrbyte-Zeichen wird korrekt "
                              "erst im naechsten Aufruf vervollstaendigt."),
    "net_peer_addr":        ("NET_PEER_ADDR(sock) AS STRING", "Remote-IP-Adresse."),
    "net_peer_port":        ("NET_PEER_PORT(sock) AS INTEGER", "Remote-Port."),
    "net_is_connected":     ("NET_IS_CONNECTED(sock) AS BOOLEAN",
                              "FALSE sobald die Gegenseite geschlossen hat oder ein Recv/Send "
                              "fehlgeschlagen ist."),
    "net_close":            ("NET_CLOSE(sock)", "TCP-Socket schliessen."),
    "net_close_listener":   ("NET_CLOSE_LISTENER(listener)", "TCP-Listener schliessen."),
    "net_set_timeout":      ("NET_SET_TIMEOUT(sock, ms)",
                              "Timeout setzen. ms=0 non-blocking, ms<0 blocking, ms>0 Timeout in ms."),
    "net_udp_bind":         ("NET_UDP_BIND(port [, bind_addr$]) AS NET_UDP",
                              "UDP-Socket auf Port lauschen. bind_addr$ optional: \"::\" fuer IPv6."),
    "net_udp_open":         ("NET_UDP_OPEN() AS NET_UDP", "Ungebundener UDP-Socket (nur Senden)."),
    "net_udp_send":         ("NET_UDP_SEND(sock, host, port, text) AS INTEGER",
                              "UDP-Datagramm senden."),
    "net_udp_recv":         ("NET_UDP_RECV(sock, max_bytes) AS STRING",
                              "UDP-Datagramm empfangen."),
    "net_udp_last_from":    ("NET_UDP_LAST_FROM(sock) AS STRING",
                              "Absender des letzten RECV als 'host:port'."),
    "net_udp_set_timeout":  ("NET_UDP_SET_TIMEOUT(sock, ms)", "UDP-Timeout setzen."),
    "net_udp_close":        ("NET_UDP_CLOSE(sock)", "UDP-Socket schliessen."),
    # --- Modul: ecs (IMPORT "ecs") ---------------------------------
    "ecs_new_world":        ("ECS_NEW_WORLD() AS ECS_WORLD", "Neue ECS-Welt anlegen."),
    "ecs_new_entity":       ("ECS_NEW_ENTITY(world) AS INTEGER",
                              "Neue Entity, gibt eindeutige Entity-ID zurueck."),
    "ecs_destroy":          ("ECS_DESTROY(world, ent) AS BOOLEAN",
                              "Entity entfernen. TRUE wenn existierte."),
    "ecs_alive":            ("ECS_ALIVE(world, ent) AS BOOLEAN", "Lebt die Entity noch?"),
    "ecs_count":            ("ECS_COUNT(world) AS INTEGER", "Anzahl lebender Entities."),
    "ecs_add_int":          ("ECS_ADD_INT(world, ent, name, value)",
                              "INTEGER-Component an Entity haengen."),
    "ecs_add_float":        ("ECS_ADD_FLOAT(world, ent, name, value)", "FLOAT-Component."),
    "ecs_add_string":       ("ECS_ADD_STRING(world, ent, name, value)", "STRING-Component."),
    "ecs_add_bool":         ("ECS_ADD_BOOL(world, ent, name, value)", "BOOLEAN-Component."),
    "ecs_add_obj":          ("ECS_ADD_OBJ(world, ent, name, value)",
                              "Beliebiges Objekt als Component (User-Klasse, MAP, ...)."),
    "ecs_has":              ("ECS_HAS(world, ent, name) AS BOOLEAN", "Hat Entity diesen Component?"),
    "ecs_remove":           ("ECS_REMOVE(world, ent, name) AS BOOLEAN", "Component entfernen."),
    "ecs_get":              ("ECS_GET(world, ent, name)",
                              "Beliebiger Component-Wert. Wirft wenn fehlt."),
    "ecs_get_int":          ("ECS_GET_INT(world, ent, name) AS INTEGER",
                              "INTEGER-Component lesen. Wirft wenn fehlt oder falscher Typ."),
    "ecs_get_float":        ("ECS_GET_FLOAT(world, ent, name) AS FLOAT", "FLOAT-Component lesen."),
    "ecs_get_string":       ("ECS_GET_STRING(world, ent, name) AS STRING", "STRING-Component lesen."),
    "ecs_get_bool":         ("ECS_GET_BOOL(world, ent, name) AS BOOLEAN", "BOOLEAN-Component lesen."),
    "ecs_get_or_int":       ("ECS_GET_OR_INT(world, ent, name, default) AS INTEGER",
                              "INTEGER-Component oder default."),
    "ecs_get_or_float":     ("ECS_GET_OR_FLOAT(world, ent, name, default) AS FLOAT",
                              "FLOAT-Component oder default."),
    "ecs_get_or_string":    ("ECS_GET_OR_STRING(world, ent, name, default) AS STRING",
                              "STRING-Component oder default."),
    "ecs_get_or_bool":      ("ECS_GET_OR_BOOL(world, ent, name, default) AS BOOLEAN",
                              "BOOLEAN-Component oder default."),
    "ecs_query":            ("ECS_QUERY(world, name) AS ARRAY OF INTEGER",
                              "Entities mit dem Component."),
    "ecs_query2":           ("ECS_QUERY2(world, n1, n2) AS ARRAY OF INTEGER",
                              "Entities mit beiden Components (Intersection)."),
    "ecs_query3":           ("ECS_QUERY3(world, n1, n2, n3) AS ARRAY OF INTEGER",
                              "Entities mit allen drei Components."),
    # Modul timer (geplante Aktionen + Cooldowns)
    "camera_shake":         ("CAMERA_SHAKE(staerke[, dauer_ms])",
                              "Screen-Shake: zufaelliger Kamera-Ruckel (Welt-Pixel), klingt "
                              "linear ueber dauer_ms ab (Default 300). Laeuft selbststaendig; "
                              "staerke=0 stoppt sofort. Der Juice-Klassiker bei Explosionen."),
    "timer_after":          ("TIMER_AFTER(ms, fn) AS INTEGER",
                              "Ruft die FUNCREF einmalig nach ms Millisekunden auf "
                              "(beim naechsten TIMER_UPDATE danach). Liefert eine Timer-ID."),
    "timer_every":          ("TIMER_EVERY(ms, fn) AS INTEGER",
                              "Ruft die FUNCREF alle ms Millisekunden auf (max. 1x pro "
                              "TIMER_UPDATE). Liefert eine Timer-ID."),
    "timer_cancel":         ("TIMER_CANCEL(id)",
                              "Timer abbrechen. Bereits gefeuerte/unbekannte IDs sind ein No-Op."),
    "timer_active":         ("TIMER_ACTIVE(id) AS BOOLEAN", "Laeuft der Timer noch?"),
    "timer_count":          ("TIMER_COUNT() AS INTEGER", "Anzahl aktiver Timer."),
    "timer_clear":          ("TIMER_CLEAR()", "Alle Timer und Cooldowns verwerfen (z.B. Scene-Wechsel)."),
    "timer_update":         ("TIMER_UPDATE()",
                              "Pro Frame aufrufen: feuert faellige TIMER_AFTER/EVERY-Callbacks "
                              "(wie INPUT_UPDATE/GUI_UPDATE)."),
    "cooldown":             ("COOLDOWN(id, ms) AS BOOLEAN",
                              "TRUE wenn die String-ID frei ist -- startet dann die ms-Sperre. "
                              "FALSE solange gesperrt. Ideal fuer Schuss-Raten."),
}


def get_doc(name: str) -> tuple[str, str] | None:
    """Liefert (Signatur, Beschreibung) zu einem Built-in oder None.

    `name` kommt haeufig OHNE trailing `$` an -- `lsp.features.word_at()`
    strippt ihn z.B. per Konvention ("Wort ohne trailing $"), waehrend
    BUILTIN_DOCS $-Builtins MIT `$` als Key speichert (z.B. "str$"). Review-
    Fund: dieser Lookup versuchte bisher nur den Namen wie uebergeben --
    Hover fuer JEDES $-Builtin (STR$, LEFT$, MID$, CHR$, ...) lief dadurch
    ueber die LSP immer ins Leere. Der Qt-Editor selbst war unbetroffen
    (sein eigenes _word_at_cursor haelt das `$` im Identifier), daher hier:
    erst den Namen wie uebergeben versuchen, dann zusaetzlich mit
    angehaengtem `$`.
    """
    key = name.lower()
    doc = BUILTIN_DOCS.get(key)
    if doc is not None:
        return doc
    return BUILTIN_DOCS.get(key + "$")
