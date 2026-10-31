# SCADStudio

Nativer OpenSCAD-Arbeitsplatz in Rust mit egui/eframe. Der aktuelle Stand ist ein
**M1-Dokumentarbeitsplatz** auf dem M0-Architekturdurchstich, keine vollständige Modellierungsanwendung. Verbindliches
Zielbild: [SPECIFICATION.md](SPECIFICATION.md). Nachweise und Grenzen:
[M1-Bericht](docs/m1/REPORT.md) und [M0-Bericht](docs/m0/REPORT.md).

## Einrichtung unter Linux

Stabiles Rust ab 1.99 (Edition 2024), Cargo, C++17-Compiler, Python 3, Git,
CMake und Ninja. Rust-Abhängigkeiten einschließlich eframe 0.37.0 und dessen
wgpu 30.0.1 sind in `Cargo.lock` gesperrt. Keine unabhängige wgpu-Version.

OpenSCAD benötigt außerdem Eigen, Boost (program_options, regex), CGAL/GMP/MPFR,
Clipper2 und Manifold (gesperrte Submodule), TBB, OpenCSG, OpenGL/EGL/GLU,
Fontconfig, FreeType, HarfBuzz, GLib, double-conversion, libzip, libxml2, lib3mf,
OpenSSL, Flex und Bison. Der Herkunftsworker benötigt nlohmann-json.
Auf Arch/CachyOS sind insbesondere `cmake ninja eigen cgal boost tbb opencsg
libglvnd glu fontconfig freetype2 harfbuzz glib2 double-conversion libzip libxml2
lib3mf openssl flex bison nlohmann-json` erforderlich. CMake prüft die tatsächlichen
Abhängigkeiten. Der reine Engine-Build benötigt kein Qt. Für fensterlose Bilder
müssen ein funktionierender EGL-Treiber und ein wgpu-Adapter verfügbar sein;
Mesa llvmpipe/lavapipe ist eine mögliche Softwareumgebung.

Der explizit gestartete Setup-Schritt lädt Fremdquellen und baut zuerst die
unveränderte Referenz, danach den Worker mit derselben Konfiguration:

```bash
python3 tools/build_engine.py target/m0-engine --jobs 2
cargo build --release
cargo run --release -- prove-m0 \
  "$PWD/target/m0-engine/worker/openscad" \
  "$PWD/target/m0-engine/reference/openscad" \
  "$PWD/target/m0-engine/libraries" \
  "$PWD/target/m0-proof"
cargo run -- view-m0 target/m0-proof
```

`cargo run` öffnet den letzten Nachweis unter `target/m0-proof`, sonst eine leere Arbeitsfläche.

`tools/build_engine.py` verlangt ein neues Zielverzeichnis und überschreibt keine
bestehenden Builds. Quellstände, Submodule, Backend und Optionen stehen in
[engine/reference.lock.json](engine/reference.lock.json). Ein unverändertes
Referenzbinary und ein unveränderter Quell-Worktree bleiben erhalten. Lokale
Buildprodukte, Prüflogs und Hostinformationen bleiben unter `target/`.
Die Versionszeichenfolge ist ebenfalls gesperrt und hängt nicht vom Bautag ab.
Der Build ist über Quellen und Optionen reproduzierbar, **kein hermetischer oder
bitidentischer Build**: Systembibliotheken müssen ebenfalls zusammenpassen.

Der technische Viewer zeigt Ergebnisgeometrie und Herkunftspfade. Die
Ursprungsansicht erlaubt die Auswahl abgezogener Operanden, ohne den SCAD-Code zu
verändern. Der Parameterknopf bearbeitet ausdrücklich den `base_height`-Literal
im mitgelieferten Durchstich; Kommentare und die Formel bleiben erhalten. Die
Vorschau wird danach als veraltet markiert und Auswahl gesperrt. Der M1-Dokumentarbeitsplatz ergänzt Undo/Redo und sichere Literaländerungen;
vollständige grafische Sprache und automatische Neuberechnung folgen in M2/M3.

## Dokumente bearbeiten (M1)

```bash
cargo run -- path/to/model.scad
cargo run -- edit path/to/main.scad path/to/nested/maße.scad
cargo run -- path/to/project.scadstudio.json
```

Code- und Literaländerungen, Undo und Redo verwenden denselben UI-unabhängigen
Dokumentkern. Die Syntaxstruktur erhält alle Bytes; ihre aufklappbaren Zahlenknoten
können als ausdrücklich gewählte Quellziele dienen. Veraltete Ziele werden gesperrt.
`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+S` und `F5` bedienen Historie, Speichern und Vorschau.
Die Vorschau benötigt die oben gebaute Engine an `target/m0-engine` und bleibt bei
Fehlern sichtbar als veraltet markiert. M1 führt noch keine automatische Vorschau aus.

Dateiänderungen werden jede Sekunde im Hintergrund geprüft. Der Konfliktvergleich
zeigt Basis, lokalen Text und Datenträger mit den Optionen Übernehmen und Beibehalten.
Für manuelles Zusammenführen lokal behalten, den Editor verwenden und speichern.
Speichern tauscht jede Datei einzeln über eine temporäre Geschwisterdatei aus;
Fehler erhalten den Dirty-Status. Gleichzeitige Eingabe bleibt während I/O erhalten.

Metadaten verwenden Schema 1; beschädigte, unbekannte und zukünftige Formate
werden nicht überschrieben. Quelldateien lassen sich weiter einzeln öffnen.
Das M1-Fenster öffnet Metadaten mit der gesperrten Engine und ohne zusätzliche
Bibliotheks-/Variantenbindungen. Andere Konfigurationen werden ausdrücklich
abgelehnt und können separat als `.scad` geöffnet werden; ihre vollständige
Anwendung gehört zu M3/M4. Es gibt noch keine Wiederherstellung nach Absturz.

Der native M1-Nachweis verwendet ausschließlich eine **Wegwerfdatei**, da er
auch externe Dateiänderungen simuliert:

```bash
mkdir -p target/m1-fixture
python3 - <<'PYCODE'
from pathlib import Path
Path('target/m1-fixture/main.scad').write_bytes(
    'h = 10;\r\n// Größe und Formel erhalten\r\ncube([h * 2, 20, 10]);\r\n'.encode())
PYCODE
SCADSTUDIO_M1_PROBE="$PWD/target/m1-native-new" \
  cargo run -- edit target/m1-fixture/main.scad
```

Das neue Ausgabe-Verzeichnis enthält drei echte Fensterbilder und `report.json`.
Ein vollständiger Durchlauf prüft Fehler/alte Vorschau, sichere Literaländerung,
Undo/Redo, Konfliktvergleich, beide Übernahmeoptionen und Speichern.

## Prüfungen

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
cargo run -- view-m0 target/m0-proof
```

Bei knappem Speicher die Cargo-Parallelität beispielsweise mit
`CARGO_BUILD_JOBS=2` begrenzen. Der separate echte Engine-Test ist bewusst als
`ignored` gekennzeichnet und wird ausdrücklich gestartet:

```bash
SCADSTUDIO_WORKER="$PWD/target/m0-engine/worker/openscad" \
SCADSTUDIO_REFERENCE="$PWD/target/m0-engine/reference/openscad" \
SCADSTUDIO_LIBRARIES="$PWD/target/m0-engine/libraries" \
cargo test --test engine_m0 -- --ignored --nocapture
```

Die CI unter `.github/workflows/ci.yml` prüft den Rust-Kern mit Rust 1.99.0,
Linux-Buildabhängigkeiten und Cargo.lock. Engine-/GPU-/Fensterprüfungen sind darin
explizit nicht enthalten; sie werden mit den obigen Befehlen separat ausgeführt.
Der Workflow wurde lokal erstellt, noch nicht auf GitHub ausgeführt.

Der native Bediennachweis erzeugt in einem **neuen** Verzeichnis zwei Screenshots
und `native-report.json`; er betätigt reale Widgets über eframe-Rohereignisse:

```bash
SCADSTUDIO_UI_PROBE="$PWD/target/m0-native-proof" \
cargo run -- view-m0 target/m0-proof
```

Nicht mit `SCADSTUDIO_SCREENSHOT` kombinieren. Der Nachweis prüft Ursprungsansicht,
Auswahl eines abgezogenen Körpers, Parameteränderung, Kommentar-/Formelerhalt und
Sperren der veralteten Auswahl. Er ersetzt keine vollständige Tastatur-/HiDPI-Abnahme.

`prove-m0` liefert bei jeder fehlgeschlagenen Teilprüfung einen Fehlerstatus.
Es schreibt `report.json`, Herkunftsmanifest, Teilnetze und Bilder ins gewählte
Prüfverzeichnis. Der Bildpfad der Engine entfernt `DISPLAY` und `WAYLAND_DISPLAY`;
wgpu erzeugt weder Fenster noch Surface. `SCADSTUDIO_SCREENSHOT=<png-path>` lässt
`view-m0` einmal das tatsächliche native Fenster erfassen und danach schließen.
Dies ist eine explizite lokale Prüfoption, keine Telemetrie.

## Architektur und Lizenzen

`document`, `language`, `application`, `engine` und `persistence` sind von egui-
und GPU-Typen unabhängig; `ui`, `render` und `offscreen` bilden die Darstellung.
`src/main.rs` enthält nur CLI und Start. Die Anwendung führt keine eigene
OpenSCAD-Semantik aus. Engine-Aufträge gehören auf Hintergrundthreads;
der CLI-Durchstich darf synchron warten. Das M0-Protokoll ist keine MCP-API.

Eigenständiger Rust-Code: MIT, siehe [LICENSE](LICENSE).
OpenSCAD und die Worker-Erweiterung: GPL-2.0-or-later mit den zutreffenden
Upstream-Ausnahmen. BOSL2: BSD-2-Clause. Lizenztexte befinden sich unter
[engine/licenses](engine/licenses); Kopplungsbewertung und noch offene
Auslieferungspflichten stehen in [docs/m0/LICENSING.md](docs/m0/LICENSING.md).
Das Gesamtpaket wird nicht als ausschließlich MIT bezeichnet.
