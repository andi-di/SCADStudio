# M0 – Architektur- und Integrationsnachweis

Prüfabschluss: **1. November 2026**. Die geforderten begrenzten technischen
M0-Durchstiche sind implementiert und auf Linux bestanden. Herkunft stammt aus
der Originalauswertung, nicht aus einer Zuordnung anhand des Gesamtnetzes.
Dies ist keine Gesamtprodukt-, Leistungs- oder Auslieferungsabnahme.

AGENTS.md und SPECIFICATION.md Version 1.1 wurden vollständig gelesen. Die
Spezifikation bleibt unverändert; keine Anforderung wird gestrichen oder als
stillschweigende Ausnahme behandelt.

## Referenz, Build und Architektur

- OpenSCAD `9ba0cee7c94785fac0ca8dfe040b965a12a25ed1`, Version 2026.10.31.
  Maßgebliches Backend: **Manifold**; CGAL zusätzlich einkompiliert. Version und
  Optionen sind unabhängig vom Bautag gesperrt.
- BOSL2 `e3d614aef7552121f993b30a6c3e87f170a2f545`, vollständiger Inhaltshash,
  Upstream-Submodule und Optionen: [reference.lock.json](../../engine/reference.lock.json).
- Zuerst wurde die unveränderte Referenz gebaut und separat gesichert, danach
  der Worker mit Herkunfts-Overlay. Ein unveränderter Upstream-Worktree ist
  erhalten; dessen `git status --short` ist leer. Der Patchgenerator wurde auf
  einem frischen Checkout erneut ausgeführt: alle acht erzeugten/geänderten
  Dateien stimmen bytegenau mit dem gebauten Worker-Quellstand überein.
- Rust 1.99.1, Cargo 1.99.1, eframe 0.37.0 mit wgpu 30.0.1. Cargo.lock ist
  vorhanden. Lokale Engine-Buildwerkzeuge: GCC 16.2.1, CMake 4.4.5, Eigen 5.0.1,
  Boost 1.92.0, CGAL 6.2.1, Manifold 3.5.4, nlohmann-json 3.12.0.
- Konkrete Modulgrenzen, privates Worker-Protokoll, Dateiansicht und offene
  Folgeverträge: [ARCHITECTURE.md](ARCHITECTURE.md). Das Setup ist in
  [README.md](../../README.md) beschrieben. Hashes der tatsächlich geprüften
  Binaries: [build-results.json](build-results.json).

Der Build ist über Quellen und Optionen nachvollziehbar wiederholbar, aber noch
nicht hermetisch oder bitidentisch über unterschiedliche Systembibliotheken.
Die CI für den Rust-Kern ist angelegt; der Workflow wurde noch nicht auf
GitHub ausgeführt. Engine-, GPU- und native Prüfungen werden darin
nicht stillschweigend als bestanden gewertet.

## Zuordnung der M0-Abnahme

| Szenario / Anforderungen | Tatsächlicher Nachweis | Ergebnis |
| --- | --- | --- |
| AT-001 begrenzt; COMP-004, EDIT-001, SEL-001 | Eigenständige Standard-SCAD mit beliebigem Modulnamen, Unicode und CRLF ohne Metadaten auswerten. Über den ausgewerteten Aufrufpfad eine Quellargumentstelle zuordnen, gezielt ändern und erneut in der Referenz vergleichen. | Bestanden |
| AT-002 begrenzt; EDIT-002, PARAM-001 | `base_height` 10 → 12; Kommentare, CRLF, Unicode und Formel bleiben erhalten. Der native Formularknopf verwendet dieselbe Änderungsverarbeitung. | Bestanden |
| AT-003 begrenzt; SEL-002, SEL-003, PARAM-004 | Drei Schleifeninstanzen mit Kontexten `i = 0, 1, 2`; verschachtelte `outer`-/`inner`-Aufrufe; zwei `children()`-Instanzen mit verschiedenen Ereignissen und Welttransformationen. Gemeinsame Parameterwirkung im Beispiel benannt. | Bestanden |
| BOSL2-Herkunft; SEL-001 bis SEL-003 | `cuboid` samt verschachtelten Bibliotheksaufrufen, Quellstellen und tatsächlicher Geometrie extrahieren. Auch die von BOSL2 überschriebenen Primitive bleiben bis zur Projektaufrufstelle nachvollziehbar. | Bestanden |
| Herkunftsteil AT-008; SEL-004, CAD-006 | Teilnetze abgezogener Zylinder behalten; Ergebnisansicht lehnt den Operanden ab, Ursprungsansicht erlaubt die Auswahl. Nativer Knopf-/Auswahlablauf und Hervorhebung geprüft. Sichtbarkeit verändert keinen SCAD-Code. | Bestanden |
| Ressourcenfall AT-015; EDIT-008, COMP-007 | Unicode-Include `nested/maße.scad`, weiteres verschachteltes Include mit ungespeichertem CRLF-Wert und relativer SVG-Import neben der Include-Datei. Berechnung verwendet Höhe 7 statt Quelldefault 2. | Bestanden |
| Bildteil AT-018; MCP-008 als Bildfähigkeit | Ohne DISPLAY und WAYLAND_DISPLAY: originales Engine-/EGL-PNG und unabhängiges Offscreen-wgpu-PNG mit Textur-Readback. Kein Fenster/Surface für den wgpu-Bildpfad. | Bestanden; noch keine MCP-Abnahme |
| EDIT-006, Teil RUN-003 | Veralteten Änderungsplan ablehnen, spätes Ergebnis verwerfen und Auswahl bei abweichender Revision sperren. | Bestanden |
| LIC-001, LIC-002, LIC-005, LIC-006 als M0-Bewertung | Konkrete GPL-Worker-Anpassungen, MIT-Host, Protokollkopplung, BOSL2 und mitgeführte Fonts getrennt bewertet; Lizenztexte erhalten. | Technische Bewertung dokumentiert; keine endgültige gemeinsame Release-Freigabe |

## Prüfungen und Zahlen

| Prüfung | Tatsächliches Ergebnis |
| --- | --- |
| `cargo fmt --check` | Bestanden |
| `cargo clippy --all-targets --all-features -- -D warnings` | Bestanden, keine Warnungen |
| `cargo test --all-targets` | 12 bestanden; der gesonderte Engine-Test ist hier ausdrücklich `ignored` |
| `cargo test --test engine_m0 -- --ignored --nocapture` mit gesperrten Komponenten | 1 bestanden; einschließlich echter Herkunfts-, Referenz- und GPU-/Bildprüfung |
| `cargo build --release` | Bestanden |
| `cargo run` | Native leere Anwendung gestartet, erfasst und visuell geprüft |
| `cargo run -- view-m0 ...` mit `SCADSTUDIO_UI_PROBE` | Reale egui-Rohereignisse betätigen Ursprungsansicht, Operandenwahl und Parameterknopf. Beide nativen Screenshots erfasst und visuell geprüft; Zustand und Quelltext geprüft. |
| Worker-Patch erneut aus sauberem Checkout erzeugen | Alle 8 Dateien identisch |
| Gesperrte Referenz und Worker bauen | Beide erfolgreich |

Der kombinierte Fall liefert **319 Knoten, 318 unterschiedliche Aufrufereignisse
und 106 unterschiedliche vollständige Kontexte**. Das normalisierte Manifest ist
etwa 1,2 MiB groß. Kein Kontext wurde dafür abgeschnitten.

| Modell | Dreiecke | Komponenten | Volumen Worker = Referenz |
| --- | ---: | ---: | ---: |
| Schleifen/Module/children/BOSL2 | 928 | 4 | 60982,50540857553 |
| Nach Parameteränderung | 928 | 4 | 72804,6220662009 |
| Ungespeicherte Includes + SVG | 12 | 1 | 700 |
| Unbekanntes Standardmodul | 252 | 1 | 106,03805797254283 |
| Dessen gezielt geändertes Argument | 252 | 1 | 251,34946332792057 |

Begrenzungen, Volumen, Komponenten und geschlossene Netzkanten stimmen in allen
fünf Fällen mit der unveränderten Referenz überein. Die Toleranzen aus 17.2 wurden
nicht erweitert. Hier sind die gemessenen Volumina sogar identisch; das ist keine
allgemeine Zusage binäridentischer Netze. Maschinenlesbare Ergebnisse:
[results.json](results.json), [native-results.json](native-results.json).

Beide fensterlosen Pfade wurden mit Software-Rendering geprüft: EGL 1.5,
Mesa 26.2.5, llvmpipe / LLVM 23.1.3. Der wgpu-Adapter verwendete das GL-Backend.
Der letzte CLI-Nachweis verwendet ausschließlich Komponenten unter `target/`,
mit entferntem DISPLAY/WAYLAND_DISPLAY und `PATH=/nonexistent`; eine separat über
PATH auffindbare OpenSCAD-Installation war damit nicht nutzbar. Dies ersetzt noch
keinen Test in einer frischen Umgebung mit gesperrtem Netzwerk.

## Sichtbare Nachweise

[Native Ursprungs-/Operandenauswahl](images/native-origin.png) und
[Parameteränderung mit veralteter Vorschau](images/native-edited.png) zeigen die
tatsächlich laufende eframe-Anwendung. Projektaufrufe und vollständig ausklappbare
Bibliothekspfade sind getrennt zugänglich; Zwischenknoten lassen sich einblenden.
[Engine/EGL-Bild](images/engine.png) und [Offscreen-wgpu-Bild](images/wgpu.png)
zeigen die beiden unabhängig ausgeführten Bildpfade. Kamera und Farben sind
unterschiedlich; das ist kein Pixelvergleich.

## Gefundene Grenzen und behobene Integrationsfehler

1. Ein isoliert verschobenes Engine-Binary meldete fehlende Shader trotz Exitcode
   0. Das Setup führt nun Shader, Farbschemata, Fonts einschließlich Lizenz und
   Lokalisierung mit. PNG-Läufe mit Engine-Fehlerdiagnosen werden zusätzlich zum
   Prozessstatus abgelehnt.
2. Eigen 5 unterstützt die ursprünglich verwendete Transformations-Kurzschreibweise
   nicht. Die unterstützte explizite Komposition ist umgesetzt und gebaut.
3. Vollständig wiederholte BOSL2-Kontexte überschritten die 32-MiB-Transportgrenze.
   Gemeinsame Frame-/Kontexttabellen lösen dies verlustfrei. Regressionstests
   prüfen einen wiederholten 700-KB-Kontext und ungültige Referenzen.
4. BOSL2 überschreibt Standardprimitive. Deshalb kann die unmittelbare Blattquelle
   korrekt in `builtins.scad` liegen. Die Benutzerquelle wird über den tatsächlichen
   vollständigen Aufrufpfad aufgelöst, nicht durch die Annahme „Blatt = Projektdatei“.
5. Dünn projizierte Dreiecke erzeugten bei Polygonkonturen lange Tessellierungsartefakte.
   Die Darstellung verwendet jetzt direkte Dreiecksmeshes; ein Regressionstest
   prüft begrenzte tessellierte Vertices. Die Bilder wurden erneut visuell geprüft.
6. Das native Quelltextfeld überlief seine Spalte. Ein begrenztes schreibgeschütztes
   Textfeld behebt dies; der Bediennachweis wurde danach erneut ausgeführt.

## Bewusst offene Anforderungen

Die folgenden Grenzen bleiben ausdrücklich sichtbar, ohne Produktanforderungen
zu streichen:

- Kein vollständiger Sprach-/Geometriekorpus und keine vollständige Herkunfts-
  oder Leistungsabnahme über jede mögliche Referenzkonstruktion. Die oben
  genannten schwierigen M0-Fälle sind nachgewiesen.
- Der technische Viewer ist keine vollständige M2-Arbeitsfläche: kein produktiver
  Tiefenpuffer, analytisches Picking, vollständiger Codeeditor oder allgemeiner
  Parameterabhängigkeitsgraph. Der Parameteradapter ist ausdrücklich ein Beispiel.
- Teilbaumexporte und C++-Kontextkopien bleiben teuer; Geometriededuplizierung,
  produktive GPU-Wiederverwendung, Warteschlangen und PERF-Messungen fehlen.
- Die kontrollierte explizite Snapshot-Dateiansicht belegt relative Ressourcen und
  ungespeicherte Includes. Absolute/externe Freigaben, Pfad-/Symlink-Sandboxing und
  vollständige Fontreproduzierbarkeit sind noch nicht abgenommen.
- Der Einmalworker unterstützt Fähigkeitenabfrage, Snapshot-/Revisionsbindung und
  Prozessabbruch. Kooperative IPC-Abbruchantworten, strukturierte Diagnosen und
  ein persistenter Worker-Vertrag sind noch offen. Kein fertiges MCP behauptet.
- Vollständige Syntaxbearbeitung, Undo/Redo, Persistenz/Wiederherstellung, DE/EN,
  Theme-Persistenz, umfassende Tastatur-/HiDPI-Prüfung sowie die übrigen Bibliotheken
  und deren Offline-Dokumentation stehen weiter im verbindlichen Zielbild.
- Frische Offline-Installation, vollständige Release-Komponentenliste und
  abschließende Lizenz-/Quellcodebereitstellungsprüfung sind noch offen.

Die konkrete [Lizenzbewertung](LICENSING.md) erklärt die enge Herkunftskopplung.
Weder ein MIT-only-Gesamtpaket noch eine abgeschlossene rechtliche Freigabe wird
behauptet. Eine spätere Änderung der Veröffentlichungsbedingungen benötigt die
in der Spezifikation verlangte ausdrückliche Produktentscheidung.
