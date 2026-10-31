# Konkrete Lizenzgrenzen des M0-Durchstichs

Dies dokumentiert die technische Kopplung und den Prüfstand. Es ist keine
abschließende rechtliche Freigabe eines zukünftigen Gesamtpakets. Es wird keine
Änderung der Produktlizenz oder Veröffentlichungsentscheidung vorgenommen.

## Tatsächlich verwendete Komponenten

| Komponente | Herkunft / unveränderliche Kennung | Lizenz und Buildgrenze |
| --- | --- | --- |
| Eigenständiger Rust-Host | Dieses Repository, `src/`, `tests/` | MIT; kein OpenSCAD-Header und kein GPL-Quellcode eingebunden. |
| OpenSCAD-Referenz | `openscad/openscad`, `9ba0cee7c94785fac0ca8dfe040b965a12a25ed1` | GPL v2 oder später, zutreffende CGAL-Ausnahme; separates unverändertes Executable. |
| Herkunftsworker | Dieselbe OpenSCAD-Revision plus `engine/overlay/` und Änderungen aus `tools/prepare_worker.py` | Abgeleiteter OpenSCAD-Build; Overlay ausdrücklich GPL-2.0-or-later. Keine MIT-Umlizenzierung des Workers. |
| BOSL2 | `BelfrySCAD/BOSL2`, `e3d614aef7552121f993b30a6c3e87f170a2f545` | BSD-2-Clause laut zugehöriger LICENSE; SCAD-Quelldateien, im Durchstich explizit eingebunden. |
| egui/eframe, egui-wgpu, wgpu | Cargo.lock; eframe 0.37.0 / wgpu 30.0.1 | MIT/Apache-2.0 laut Crate-Metadaten; Rust-Build. Transitive Komponenten separat zu erfassen. |
| Liberation 2.00.1 | OpenSCAD `fonts/Liberation-2.00.1`, gesperrter OpenSCAD-Stand | SIL OFL 1.1, Hinweise und reservierte Namen bleiben mit den unveränderten Fonts erhalten. |
| nlohmann-json | Systempaket, über CMake als Abhängigkeit geprüft | MIT; Header wird nur im GPL-Worker eingebunden. |

Die genauen Submodule von OpenSCAD stehen im Lock. Manifold, Clipper2, OpenCSG,
CGAL und dynamische Systembibliotheken sind zusätzliche Fremdkomponenten;
deren vollständige Release-Liste und Lizenzbeilagen sind noch nicht erstellt.
Das Repository enthält weder Engine-Binaries noch BOSL2-Quellen. Die lokalen
Builds unter `target/` sind kein fertiges Auslieferungspaket.

## Konkrete Kommunikation und Bewertung

Der MIT-Host startet einen eigenen Prozess über `std::process::Command`. Es gibt
keinen Link zu OpenSCAD, keine FFI, keine gemeinsamen C++-Objekte und keine
übernommenen OpenSCAD-Datenstrukturen im Rust-Adressraum. Standardberechnungen
verwenden SCAD-Eingabe, originale CLI und STL/PNG-Ausgabe.

Der zusätzliche Herkunftskanal ist jedoch speziell auf beide Seiten zugeschnitten:
eine private Umgebungseinstellung aktiviert ihn; eine JSON-Anfrage benennt
Snapshot und Revision; das Manifest enthält dynamische Engine-Aufrufpfade,
Quellpositionen, Kontextwerte und Teilgeometrien. Die fachliche Bedienung ist auf
diese Informationen angewiesen. **Die Prozessgrenze allein belegt deshalb keine
rechtlich unabhängigen Programme.** Die GNU-Erläuterung zu Aggregation und
kombinierten Werken nennt auch Art und Bedeutung der Kommunikation als relevant.
Diese enge Kopplung bleibt vor Veröffentlichung fachkundig zu beurteilen.

Technische Schlussfolgerung: Der Worker wird eindeutig unter den zutreffenden
OpenSCAD-Bedingungen behandelt; Rust-Quellen bleiben mit MIT gekennzeichnet.
Eine Garantie, dass jede denkbare gemeinsame Binärauslieferung bei dieser
Kopplung genau diese Trennung rechtlich einhalten kann, wird nicht gegeben.
Erfordert eine konkrete Veröffentlichung andere Bedingungen, ist die in der
Spezifikation verlangte ausdrückliche Produktentscheidung nötig. Es wird hier
keine solche Entscheidung vorweggenommen.

## Vor Auslieferung erforderlich

- Passende vollständige OpenSCAD-/Worker-Quellen einschließlich Overlay,
  Buildskripten, Patches und erforderlicher Hinweise zur tatsächlich gelieferten
  Binary-Version bereitstellen; ursprüngliche Hinweise und Ausnahmen bewahren.
- Für BOSL2 Quellen-/Binary-Hinweise gemäß Lizenz beilegen; Dokumentations- und
  sonstige Ressourcenrechte separat erfassen. Eine README-Lizenzangabe ersetzt
  keine Prüfung aller tatsächlich gebündelten Dateien.
- Vollständige Komponentenliste aus Cargo.lock, Engine-Submodulen, statischen und
  dynamischen Bibliotheken sowie Treiber-/Paketanteilen erzeugen und gegen das
  fertige Artefakt prüfen (AT-028). Für vier weitere Produktbibliotheken ist hier
  keine Freigabe behauptet.
- Veröffentlichungsmodell und enge Herkunftskopplung abschließend bewerten;
  erforderliche Quellbereitstellung, Lizenztexte und Rechte der ausgelieferten
  Schriften, Bilder und Dokumentation nachweisen.

## Primärquellen

- [OpenSCAD COPYING, gesperrte Revision](https://github.com/openscad/openscad/blob/9ba0cee7c94785fac0ca8dfe040b965a12a25ed1/COPYING),
  lokal geprüft und unter `engine/licenses/OpenSCAD-COPYING` erhalten.
- [BOSL2 LICENSE, gesperrte Revision](https://github.com/BelfrySCAD/BOSL2/blob/e3d614aef7552121f993b30a6c3e87f170a2f545/LICENSE),
  lokal geprüft und unter `engine/licenses/BOSL2-LICENSE` erhalten.
- [GNU GPL FAQ zur Aggregation](https://www.gnu.org/licenses/gpl-faq.en.html#MereAggregation).
  Der Webabruf war in dieser Sitzung nicht erfolgreich; die Schlussfolgerung oben
  ist bewusst keine abschließende rechtliche Zusicherung.
- [eframe 0.37.0](https://docs.rs/eframe/0.37.0/eframe/), tatsächliche API zusätzlich
  in den heruntergeladenen Crate-Quellen geprüft.
