# Repository Guidelines

## Projektstruktur und Techstack

Das Repository enthält noch kein Rust-Projekt; folgende Vorgaben gelten für dessen Aufbau. Verwende stabiles Rust, Cargo und egui/eframe. Integriere eigenes GPU-Rendering bei Bedarf mit wgpu über eframe. Halte Bibliotheksversionen kompatibel und prüfe APIs anhand offizieller Dokumentation.

Halte `src/main.rs` schlank. Trenne Anwendungslogik, UI, Persistenz und Rendering in Modulen. Fachlogik muss ohne egui- oder GPU-Typen testbar sein. Nutze `tests/` für Integrationstests und `assets/` für Ressourcen. Zusätzliche Workspace-Crates benötigen einen konkreten Nutzen. Vermeide globale veränderliche Zustände und zyklische Abhängigkeiten.

## Build, Tests und lokale Entwicklung

Dokumentiere Einrichtung und Toolchain in `README.md`; committe `Cargo.lock`. Nach Einrichtung des Cargo-Projekts gelten:

- `cargo run`: Anwendung lokal starten.
- `cargo build --release`: optimierten Build erstellen.
- `cargo fmt --check`: Formatierung prüfen.
- `cargo clippy --all-targets --all-features -- -D warnings`: Lints prüfen.
- `cargo test --all-targets`: automatisierte Tests ausführen.

## Coding-Stil und Fehlerbehandlung

Verwende `rustfmt`, vier Leerzeichen, `snake_case` für Funktionen und Module sowie `PascalCase` für Typen. Bevorzuge kleine Funktionen, klare Besitzverhältnisse und aussagekräftige Typen. Vermeide unnötige Klone und `unsafe`; dokumentiere unvermeidbare Sicherheitsannahmen.

Behandle erwartbare Fehler mit `Result` und Kontext. Kein `unwrap()` oder `expect()` für externe Daten, Benutzereingaben oder Dateioperationen. Dokumentiere öffentliche Schnittstellen und nicht offensichtliche Entscheidungen. Ergänze Abhängigkeiten gezielt.

## egui, Rendering und Nebenläufigkeit

Trenne Anwendungszustand, temporären UI-Zustand und Renderdaten. Verwende stabile Widget-IDs. Verlege blockierende Ein-/Ausgabe und teure Berechnungen in Hintergrundaufgaben. Übergib Ergebnisse über Nachrichten; verwerfe veraltete Ergebnisse und begrenze Warteschlangen. Führe Async-Runtimes nur bei Bedarf ein.

Vermeide teure Neuberechnungen pro Frame. Verwende GPU-Ressourcen wieder, beachte Thread-Anforderungen und behandle Größenänderungen explizit. Fordere Repaints bedarfsgerecht an.

Verwende konsistente Farben, Typografie und Abstände. Berücksichtige Tastaturbedienung, Fokus, HiDPI, Fenstergrößen sowie Lade-, Leer- und Fehlerzustände. Prüfe sichtbare Änderungen in der laufenden Anwendung.

## Testrichtlinien

Nutze Rusts Testframework: Unit-Tests in `#[cfg(test)]`-Modulen, Integrationstests unter `tests/`. Benenne Tests nach geprüftem Verhalten, etwa `rejects_invalid_input`. Ein Coverage-Schwellenwert ist noch nicht festgelegt.

Teste Randfälle und Fehlerpfade; ergänze Regressionstests für Fehlerkorrekturen. Vermeide Tests reiner Implementierungsdetails. Führe passende Prüfungen aus und berichte tatsächliche Ergebnisse sowie Einschränkungen.

## Commits, Pull Requests und Konfiguration

Git-Historie ist hier nicht auswertbar. Verwende kurze imperative Commit-Titel, etwa `Add settings panel`. Bewahre fremde Änderungen und halte Änderungen thematisch zusammenhängend.

Pull Requests beschreiben Zweck, Verhalten und Validierung, verlinken relevante Issues und enthalten bei UI-Änderungen möglichst Screenshots. Dokumentiere Konfigurationsänderungen. Committe keine Zugangsdaten, Umgebungsmetadaten oder `target/`-Artefakte. Versioniere persistierte Datenformate bei Bedarf und behandle beschädigte Eingaben kontrolliert.
