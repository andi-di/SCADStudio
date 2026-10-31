# M1 – Dokumentkern und gemeinsame Änderungen

Prüfstand: 1. November 2026. Der M1-Dokumentkern, seine Dateiverträge und die
zugehörige kleine native Prüfoberfläche sind implementiert. Die in Kapitel 18
für M1 festgelegten Szenarien AT-002, AT-006, AT-007 und AT-015 bis AT-017 wurden
mit den unten genannten Prüfungen abgenommen. Dies ist keine M2-/M3-/M4- oder
Gesamtproduktabnahme.

AGENTS.md, SPECIFICATION.md, README.md sowie alle drei M0-Dokumente wurden
vollständig gelesen. Der tatsächliche Ausgangsstand enthielt die uncommittierte
M0-Implementierung. Vor der ersten Implementierungsänderung wurde dieser Stand
vollständig mit **`127652a` – `chore: preserve M0 architecture and integration baseline`**
gesichert. Bestehende M0-Dateien, Engine-Lock, Fremdlizenzen und Nachweise bleiben
erhalten. Die Spezifikation wurde nicht geändert.

## Abnahmen

| Szenario | Ausgeführter Nachweis | Ergebnis |
| --- | --- | --- |
| AT-002; EDIT-002, PARAM-001 | Bestehender Byteerhalt-Test und nativer Wechsel `h = 10` → `h = 12` bei Unicode, CRLF, Kommentar und `h * 2`; tatsächlicher Text wird bytegenau mit der erwarteten Fassung verglichen. | Bestanden |
| AT-006; EDIT-006 | Alter ChangeSet nach neuer Eingabe abgelehnt; altes SourceRange-Ziel trotz zufällig numerischer neuer Bytes abgelehnt; veraltetes Undo ohne Mutation abgelehnt. | Bestanden |
| AT-007; REC-001 bis REC-003 | Reale Vorschau berechnen, per egui-Rohereignis unvollständiges `cube([` tippen, Enginefehler auslösen; alte Geometrie bleibt sichtbar als veraltet. Unabhängigen bestätigten Literalbereich ändern, mit Undo/Redo prüfen. Fehlerbereiche und Teil-Tokens sind gesperrt, alte Ziele werden nicht neu verwendet. | Bestanden |
| AT-015; EDIT-002, COMP-007 | Unicode-/CRLF-Quellkoordinaten und Dateikonflikte; separaten echten Engine-Durchstich erneut ausführen: verschachtelte ungespeicherte Includes und relative SVG-Ressource ergeben Höhe 7 und Volumen 700 wie die unveränderte Referenz. | Bestanden |
| AT-016; REC-004, REC-005 | Native externe Änderung bei lokaler Arbeit, sichtbarer Dreiwegevergleich, Beibehalten, Speichern, zweite externe Änderung und Übernehmen. Kernprüfungen prüfen geänderte Datenträgerstände, Undo einer Übernahme und gleichzeitige neue Eingabe. | Bestanden |
| AT-017; FILE-006 | Beschädigte und zukünftige Metadaten lassen sich weder öffnen noch durch Speichern überschreiben; Bytes bleiben unverändert und `.scad` öffnet separat. | Bestanden |

Zusätzlich geprüft: atomare Mehrdateiänderungen im Speicher, zusammengehöriges
Mehrdatei-Undo/Redo, monotone Revisionen und Historienzweige; teilweise
fehlschlagendes Speichern mit richtigem Dirty-Zustand; schreibgeschützte Dateien;
Hintergrundspeichern während neuer Eingabe und Konfliktübernahme während neuer
Eingabe. Die Syntaxstruktur erhält sämtliche Bytes und begrenzt übermäßig tiefe
Verschachtelung. Ein Regressionstest deckt eine fehlende äußere Klammer ab, wenn
das letzte Zeichen bereits die innere Klammer schließt.

## Tatsächliche Prüfungen

| Prüfung | Ergebnis |
| --- | --- |
| `cargo fmt --check` | Bestanden |
| `cargo clippy --all-targets --all-features -- -D warnings` | Bestanden, keine Warnungen |
| `cargo test --all-targets` | 26 bestanden; 1 Engine-Test bewusst ignored |
| Separater `engine_m0` mit gesperrtem Worker, Referenz und BOSL2 | 1 bestanden; inklusive echter Ressourcen-, Herkunfts-, Referenz- und fensterloser Bildprüfungen |
| `cargo build --release` | Bestanden |
| `cargo run -- edit ...` mit `SCADSTUDIO_M1_PROBE` | Nativer Ablauf bestanden; Bilder visuell geprüft |
| `git diff --check` | Bestanden |

Die eframe-/egui-APIs wurden gegen die lokal installierten offiziellen
Crate-Quellen 0.37.0 geprüft. Der zusätzliche docs.rs-Abruf war nicht verfügbar.
Es wurden keine neuen Abhängigkeiten oder Engineänderungen benötigt. Der
bestehende CI-Workflow wurde nicht auf GitHub ausgeführt; er führt den um die
M1-Tests erweiterten Cargo-Testlauf aus. Native und Engine-Prüfungen bleiben
separate explizite Prüfungen.

## Sichtbare Nachweise

- [Veraltete Vorschau und sichere Literaländerung](images/stale-preview.png)
- [Basis, lokaler Stand und Datenträger mit Übernahmeoptionen](images/conflict.png)
- [Übernommener Datenträgerstand](images/resolved.png)
- [Maschinenlesbarer nativer Ablauf](native-results.json)

Die erste Sichtprüfung fand eine durch die Vorschau verdrängte Konfliktbedienung.
Die Geometriefläche ist jetzt begrenzt und der Inhalt scrollbar; der komplette
native Ablauf wurde anschließend erneut ausgeführt. Die letzte gültige Geometrie
ist auch nach der Konfliktübernahme ausdrücklich veraltet. Die vorherige
Engine-Diagnose bleibt mit ihrer berechneten Revision erkennbar.

## Technische Grenzen und Folgearbeit

Die [Architektur](ARCHITECTURE.md) dokumentiert Verträge und Grenzen im Detail.
Die Syntaxstruktur ist verlustfrei und fehlertolerant; sie ist noch keine
vollständige Referenzgrammatik mit strukturierten Ausdrucksformularen,
Symbolauflösung und Refactorings. Diese M3-Anforderungen bleiben unverändert.
Die sichere M1-Formularmutation betrifft ausdrücklich bestätigte numerische
Tokens, keine inverse Formelauswertung und keine allgemeine Maßbearbeitung.

Die Prüfoberfläche verwendet weiterhin den M0-Renderer. Produktives 3D-Picking,
Tiefenpuffer, andockbares Layout, automatische Vorschau und Griffoperationen
gehören zu M2; vollständige Bibliotheks-/Variantenkonfiguration zu M3/M4.
Metadaten mit noch nicht ausführbaren Bindungen werden ausdrücklich abgelehnt,
statt mit anderen Einstellungen zu rechnen. Das M1-Fenster benötigt die lokale
Engine unter `target/m0-engine`; ein fertiges Auslieferungspaket wird nicht
behauptet.

Dateien werden pro Datei atomar ersetzt, nicht gemeinsam über alle Dateien.
Gegen fremde Prozesse gibt es keinen betriebssystemweiten atomaren
Vergleich-und-Austausch-Vertrag. Polling erkennt Änderungen im Sekundentakt.
Wiederherstellung nach Absturz und garantierte Stromausfalldauerhaftigkeit sind
noch nicht abgenommen. Die Historie verwendet vollständige Inhaltssnapshots und
hat noch keine Speicherbudget-/Leistungsabnahme. HiDPI-, Sprach-, Theme-,
Layout- und Leistungsprüfungen werden durch diesen M1-Bediennachweis nicht
ersetzt. Die Lizenz- und Auslieferungsgrenzen aus M0 gelten unverändert.
