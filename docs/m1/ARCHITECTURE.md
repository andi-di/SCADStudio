# M1 – Dokument- und Dateiverträge

`document` besitzt den maßgeblichen UTF-8-Text. Alle Änderungen laufen durch
`ChangeSet` mit Ausgangsrevision, Ursprung, Beschreibung und dokumentgebundenen
Bytebereichen. Die Prüfung aller Bereiche erfolgt vor der ersten Mutation.
Undo und Redo stellen alle gemeinsam betroffenen Dokumente wieder her und
vergeben jeweils eine neue monotone Revision. Eine neue Änderung nach Undo
verwirft den Redo-Zweig. Auch Historienaktionen werden mit betroffenen
Dokumenten protokolliert. `SourceRange` bindet ein bestätigtes Quellziel zusätzlich
an seine Revision; `plan_numeric_target` lehnt alte Ziele ausdrücklich ab.

Dokumentzustände und Historienstapel verwenden `Arc` und Copy-on-write.
Hintergrundaufträge erhalten damit unveränderliche Ausgangsstände, ohne beim
Start die gesamte Historie zu kopieren. Inhaltssnapshots bleiben zunächst
vollständige Textkarten; die Historie ist noch kein Rope-/Delta-Speicher.

`syntax` stellt einen verlustfreien Token-/Klammerbaum mit Fehlerbereichen und
zusätzlichen groben Anweisungshüllen bereit. Kommentare, Leerraum, CRLF und
unvollständige Tokens bleiben durch Quellbereiche erhalten. Die maximale
Klammerrekursion beträgt 128; tiefere Eingaben werden sichtbar als Fehlerbereich
behalten. Die Struktur wird im Fenster revisions- und dokumentgebunden
zwischengespeichert. Zahlen in unsicheren Fehlerbereichen, Teilbereiche eines
Tokens, Kommentar-/Stringinhalte und alte Ziele können nicht als Literalziel
übernommen werden. Vorzeichen werden in eindeutig unären Positionen erkannt.

Dies ist die fehlertolerante M1-Grundstruktur, kein vollständiger struktureller
OpenSCAD-Editor oder eigener Interpreter. Ausführbare Sprachsemantik und
vollständige Parserdiagnosen kommen weiterhin von der originalen Engine.
Vollständige Referenzgrammatik, Ausdrucksformulare, Symbolauflösung und
Refactorings bleiben die ausdrücklich in M3 vorgesehene Arbeit. Ein aktuell
bestätigtes Literalziel benötigt keine Zuordnung zu alter Geometrie.

`files::FileSession` trennt Quelldokumente, kanonische Dateipfade und zuletzt
bekannte Datenträgerinhalte. Dirty wird durch Inhaltsvergleich ermittelt,
nicht durch die Revisionsnummer; Undo bis zum gespeicherten Inhalt wird sauber.
Eine externe Änderung erzeugt einen Vergleich aus Basis, lokalem Stand und
Datenträger. Erkennen ersetzt keinen Text. Übernehmen ist eine rückgängig
machbare normale Transaktion. Beibehalten aktualisiert die bestätigte Basis;
manuelles Zusammenführen erfolgt danach im gleichen Editor und mit derselben
Historie. Vor einer Auflösung werden alle drei Vergleichsstände erneut geprüft.

I/O läuft auf einer Kopie der Sitzung mit genau einem begrenzten Antwortkanal.
Währenddessen bleibt der Editor nutzbar. `merge_io` übernimmt neue Dateibasen,
erhält aber zwischenzeitlich eingegebenen Text. Eine im Hintergrund vorbereitete
Textübernahme darf nur bei unveränderter Ausgangsrevision übernommen werden.
Ein weiterer gleichzeitiger Dateiauftrag wird mit einer sichtbaren Meldung
abgelehnt. Pro Datei werden temporäre Geschwisterdatei, Schreiben, `sync_all`,
erneuter Basisvergleich und anschließender Austausch verwendet. Schreibgeschützte
Dateien werden abgelehnt. Bei mehreren Dateien liefert jede ein eigenes
Ergebnis; Fehler lassen die betroffene Datei dirty. Dies ist keine atomare
Mehrdatei-Dateisystemtransaktion.

Projektmetadaten sind typisiertes JSON mit Schema 1. Unbekannte Felder,
beschädigte Daten und andere Versionen werden abgelehnt; der Speicherpfad prüft
auch eine vorhandene Datei vor einem Austausch. Es werden keine lokalen Themes,
Verläufe, Tokens oder Wiederherstellungsdaten in das Projekt geschrieben.
Das M1-Fenster akzeptiert eine Einstiegspunktbindung an die gesperrte Engine.
Zusätzliche Bibliotheks-, Suchpfad-, externe Ressourcen- und Variantenbindungen
werden im Fenster ausdrücklich abgelehnt, bis ihre Anwendung in M3/M4 umgesetzt
ist; `.scad` bleibt separat zugänglich. Der Datenvertrag liest und schreibt diese
Schemafelder bereits ohne versteckte Modellierungssemantik.

Die kleine Arbeitsfläche in `workbench` ist eine M1-Prüfoberfläche, nicht das
andockbare M2-Layout. Code und Formulare verwenden denselben Dokumentkern.
Ein F5-Auftrag baut außerhalb des UI-Threads eine vollständige Dateiansicht unter
dem Verzeichnis des Einstiegspunkts, überlagert offene ungespeicherte Dateien
und berechnet mit dem vorhandenen M0-Worker. Symlinks werden in dieser einfachen
Ansicht abgelehnt. Grenzen: 4096 Dateien, 64 MiB je Ressource, 128 MiB insgesamt;
Quelldateien beim Öffnen 16 MiB, Metadaten 1 MiB. Ein Fehler oder ein spätes
Ergebnis ersetzt die letzte gültige Geometrie nicht. Die Anzeige nennt deren
Veraltung; eine alte Geometrie liefert keine bearbeitbaren Auswahlpositionen.
Engine-Fehler tragen die berechnete Revision.

Das Dateipolling läuft im Sekundentakt. Es gibt noch keinen plattformspezifischen
Dateiwächter. Die letzte Vergleichsprüfung vor dem Austausch verkleinert das
Rennfenster gegenüber fremden Schreibern, ist aber kein Dateisystem-CAS und
keine Kooperation mit fremden Editoren. Änderungen eines anderen Prozesses
unmittelbar zwischen letzter Prüfung und `rename` können ohne gemeinsame
Sperrkonvention nicht ausgeschlossen werden. `sync_all` betrifft die temporäre
Datei; eine Stromausfallgarantie für den Verzeichniseintrag wird nicht behauptet.
Absturz-Wiederherstellung und die umfassende AT-029-Abnahme bleiben M4.
