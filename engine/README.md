# Gesperrte Fremdengine

`reference.lock.json` sperrt Upstream, Submodule, BOSL2-Inhalte, Backend und
Buildoptionen. `tools/build_engine.py` erzeugt zwei getrennte Executables:
`reference/openscad` unverändert, `worker/openscad` mit Herkunftserweiterung.
Der unveränderte Referenz-Quellstand bleibt als Worktree erhalten.

`overlay/` ist GPL-2.0-or-later lizenzierte Worker-Erweiterung. Die daraus
angepassten OpenSCAD-Dateien behalten ihre ursprünglichen Lizenzhinweise und
die zutreffenden Ausnahmen. Der Patchgenerator prüft die vollständige Revision,
einen sauberen Ausgangsstand und eindeutige Textanker; ein abweichender Stand
wird nicht stillschweigend angepasst. Der Worker erweitert nur Herkunftsausgabe
und die private Fähigkeitenabfrage; normale Exporte bleiben original.

Die Anwendung startet weder Download noch Build automatisch. Setup benötigt
Netzwerk, fertig bereitgestellte Modelle und Bibliotheken werden lokal verarbeitet.
Ein Offline-Releasepaket folgt erst nach M4-Abnahme. Siehe
[Architektur](../docs/m0/ARCHITECTURE.md) und
[Lizenzbewertung](../docs/m0/LICENSING.md).
