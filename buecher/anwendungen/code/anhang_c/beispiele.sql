-- anlegen
CREATE TABLE eintrag (
    id      INTEGER PRIMARY KEY,
    titel   TEXT NOT NULL,
    urheber TEXT,
    art     TEXT,
    jahr    INTEGER
);
CREATE TABLE freund (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
CREATE TABLE ausleihe (
    id      INTEGER PRIMARY KEY,
    eintrag INTEGER REFERENCES eintrag(id),
    freund  INTEGER REFERENCES freund(id),
    von     TEXT,
    zurueck TEXT
);

-- einfuegen
INSERT INTO eintrag (titel, urheber, art, jahr) VALUES
    ('Momo', 'Michael Ende', 'Buch', 1973),
    ('Metropolis', 'Fritz Lang', 'Film', 1927),
    ('Kind of Blue', 'Miles Davis', 'Musik', 1959),
    ('Tetris', 'Alexei Paschitnow', 'Spiel', 1984),
    ('Emil und die Detektive', 'Erich Kästner', 'Buch', 1929);
INSERT INTO freund (name) VALUES ('Lena'), ('Jonas');
INSERT INTO ausleihe (eintrag, freund, von, zurueck) VALUES
    (4, 1, '2026-10-11', NULL),
    (1, 2, '2026-09-02', '2026-09-20'),
    (2, 2, '2026-11-01', NULL);

-- auswaehlen
SELECT titel, jahr FROM eintrag
WHERE art = 'Buch'
ORDER BY jahr;

-- suchen
SELECT titel FROM eintrag
WHERE titel LIKE 'm%' OR urheber LIKE '%Kästner%'
ORDER BY titel;

-- verbinden
SELECT e.titel, f.name, a.von
FROM ausleihe AS a
JOIN eintrag AS e ON e.id = a.eintrag
JOIN freund AS f ON f.id = a.freund
WHERE a.zurueck IS NULL
ORDER BY a.von;

-- alle
SELECT e.titel, COALESCE(f.name, '-') AS bei
FROM eintrag AS e
LEFT JOIN ausleihe AS a ON a.eintrag = e.id AND a.zurueck IS NULL
LEFT JOIN freund AS f ON f.id = a.freund
ORDER BY e.titel;

-- zaehlen
SELECT art, COUNT(*) AS anzahl FROM eintrag
GROUP BY art
ORDER BY anzahl DESC, art;

-- tage
SELECT e.titel,
       CAST(julianday('2026-11-20') - julianday(a.von) AS INTEGER) AS tage
FROM ausleihe AS a
JOIN eintrag AS e ON e.id = a.eintrag
WHERE a.zurueck IS NULL
ORDER BY tage DESC;

-- monate
SELECT strftime('%Y-%m', von) AS monat, COUNT(*) AS ausleihen
FROM ausleihe
GROUP BY monat
ORDER BY monat;

-- aendern
UPDATE ausleihe SET zurueck = '2026-11-20'
WHERE eintrag = 2 AND zurueck IS NULL;

-- loeschen
DELETE FROM eintrag
WHERE art = 'Musik' AND id NOT IN (SELECT eintrag FROM ausleihe);

-- danach
SELECT titel FROM eintrag
WHERE id NOT IN (SELECT eintrag FROM ausleihe WHERE zurueck IS NULL)
ORDER BY titel;
