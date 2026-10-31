# M0 – Architekturentscheidung und Worker-Vertrag

## Maßgeblicher Zustand und Grenzen

Das Einzelprojekt mit Bibliotheksziel folgt Kapitel 13. Quelldokumente sind
UTF-8-Text; Textänderungen beziehen sich auf eine monotone Sitzungsrevision und
halb offene Bytebereiche. Alle Bereiche eines Änderungssatzes werden geprüft,
bevor ein Dokument verändert wird. Unberührte Bytes bleiben erhalten. Ein
explizit gewählter numerischer Quellliteral kann geändert werden; die
Sprachschicht wertet keine OpenSCAD-Ausdrücke aus. Der M0-Parameterknopf ist ein
klar benannter Beispieladapter, kein allgemeiner Parser oder inverser Interpreter.

Eine Berechnung verwendet eine private temporäre Dateiansicht mit vollständig
expliziten Quelldateien und Ressourcenbytes. Ungespeicherte Include-Dateien werden
an ihrer richtigen relativen Position bereitgestellt. Originale werden nicht
beschrieben. Traversierung, absolute Snapshotpfade und fehlende Einstiegspunkte
werden abgelehnt. Das ersetzt noch keine vollständige Dateifreigabe-Sandbox:
OpenSCAD-Code kann selbst absolute Pfade referenzieren. M0 ist nur für vertrauenswürdige
lokale Prüfmodelle; die MCP-Freigabegrenze bleibt in M4 zwingend umzusetzen.
Externe freigegebene Ressourcen und vollständige Referenz-Suchpfadkonflikte sind
noch kein abgenommener Bestandteil der Dateiansicht.

## Entscheidung: während der Originalauswertung instrumentieren

Ein fertiges STL enthält weder Schleifendurchläufe noch Quellstellen oder
abgezogene Operanden. Auch ein gewöhnlicher CSG-Export enthält nicht den
vollständigen dynamischen Aufrufkontext. Diese Schnittstellen allein werden
als Herkunftsarchitektur verworfen. Ein eng begrenztes OpenSCAD-Overlay erfasst
stattdessen `ModuleInstantiation::evaluate` und verwendet die ursprünglichen
Kontexte. Es verändert weder SCAD-Dateien noch Aufrufargumente und wertet
Argumentausdrücke nicht erneut aus.

Pro `EvaluationSession` existieren ein Aufrufstapel und ein Ereigniszähler.
Jeder ausgewertete Aufruf erhält Namen, Quellbereich und die zu diesem Zeitpunkt
verfügbaren lexikalischen Kontextwerte. Näherer Scope hat Vorrang vor äußerem
Scope. Schleifenwerte stehen im passenden dynamischen Frame; wegen lexikalischer
Modulscopes müssen sie nicht im innersten Frame sichtbar sein. Ein RAII-Guard
entfernt Frames auch bei Auswertungsfehlern. Der zurückgegebene Knoten erhält eine
Kopie des Pfads. Engine-Indizes und Ereignisnummern gelten nur in dieser Berechnung.

Die Ausgabe erfolgt aus dem ursprünglichen ausgewerteten Knotenbaum vor dem
normalen Export. Pro Knoten liefert derselbe `GeometryEvaluator` ein Teilnetz;
so bleiben Differenzoperanden erhalten. Übergeordnete Transformationen werden
auf eine Geometriekopie angewandt. Das Original bleibt unverändert. Die normale
finale Ausgabe wird zusätzlich mit einer unveränderten Referenz desselben Builds
verglichen. Übereinstimmung im M0-Korpus ersetzt kein vollständiges Sprachkorpus.

Die boolesche Herkunft wird absichtlich nicht als eindeutige Flächenherkunft
behauptet: `subtracted` markiert einen Pfad unter einem abzuziehenden Operanden.
Das ist keine allgemeine Vorzeichenalgebra für beliebig verschachtelte CSGs.
Die Auswahl betrifft Teilbäume, nicht dauerhaft referenzierbare Dreiecksnummern.

## M0-Protokoll v1

Ein Auftrag ist ein eigener isolierter OpenSCAD-Prozess. Eingabe sind die
unveränderliche Dateiansicht, expliziter Backendparameter `Manifold`, der gesperrte
Bibliothekssuchpfad und ein ausschließlich vom Host gesetztes privates
`SCADSTUDIO_TRACE_DIR`. Ausgabe sind ein Manifest und auftragsgebundene
`<node-id>.stl`-Dateien sowie der normale finale Export. Standardfehler wird in
ein Auftragslog umgeleitet; stdout bleibt für den CLI-Durchstich frei.

Manifest:

- `protocol_version: 1`, `nodes` mit maximal 4096 Knoten.
- Knoten: `id`, `parent`, `kind`, `evaluated`, `source`, `origin`, `world_matrix`,
  `subtracted`, `mesh`.
- `frames`: Tabelle `event → {event, call, source, context}`; `context` referenziert
  die separate `contexts`-Tabelle mit vollständigen OpenSCAD-Wertdarstellungen.
- `origin` am Knoten ist eine geordnete Liste von Ereignisreferenzen. Identische
  Kontexte werden genau einmal serialisiert und im Rust-Host über `Arc` geteilt.
  Fehlende Referenzen werden abgelehnt. Maximal 262144 Pfadreferenzen pro Ergebnis;
  Überschreitung ist ein Fehler, keine Kürzung.
- Quelle: Dateiname sowie einbasierte Zeilen und UTF-8-Bytespalten; Ende exklusiv.
- `evaluated` ist die tatsächliche Engine-Knotendarstellung mit verfügbaren
  ausgewerteten Werten; kein vorgetäuschter vollständiger Modulargument-Dump.
- Meshnamen werden nur als exakt gebundene `<id>.stl` akzeptiert. Manifest maximal
  32 MiB, jedes Mesh maximal 64 MiB, gesamte Teilgeometrie maximal 256 MiB; STL-Parser maximal eine Million Dreiecke.

Der Host ordnet Ergebnis und Snapshotfingerprint seiner Revision zu.
Die ID umfasst Quelle, Ressourcenbytes, Einstieg, Revision, tatsächliche Engine-
Binary, Lockkonfiguration und tatsächliche BOSL2-Dateien sowie die mitgeführten Shader, Farbschemata und Fonts. Das M0-Korpus enthält
keinen Text und keine externen Fonts; vollständige Font-Fingerprints bleiben
für entsprechende Modelle erforderlich. Eine spätere
Antwort darf neuere Dokumente nicht überschreiben. Auswahl in einer anderen
Revision wird abgelehnt. Der Host kann den Prozess abbrechen; im Durchstich wird
sofort beendet und gereapt, spätestens nach dem konfigurierten Auftragsdeadline.
Die Zwei-Sekunden-Frist für kooperativen Abbruch und Neustart ist ein späterer
Koordinatorvertrag, kein bereits implementierter persistenter Worker.

`--scadstudio-capabilities` liefert Protokollversion, vollständige Engine-Revision,
Herkunfts-/Teilnetzfähigkeiten und Grenzen. `request.json` im privaten
Auftragsverzeichnis benennt Snapshot-ID und Revision; beide müssen im Manifest
unverändert zurückkehren. Noch fehlen strukturierte Diagnosen und kooperativer
Abbruch des endgültigen Protokolls aus 13.4. M0 nutzt eine Host-API plus
Einmalprozess und darf nicht als fertiger persistenter Worker- oder MCP-Vertrag
veröffentlicht werden.

## Leistung und Darstellungsgrenzen

Die Teilbaumexporte können quadratische Zusatzarbeit und viele doppelte Netze
erzeugen. Vollständige Kontextkopien während der C++-Auswertung sind ebenfalls teuer; der Transport und der Rust-Host teilen die Kontexte bereits. M0 misst Machbarkeit,
keine PERF-Abnahme. Knoten- und Artefaktlimits geben verständliche Fehler statt
stiller Kürzung; die Engine-Auswertung selbst ist nicht speicherkontingentiert.
Für M2 sind deduplizierte Geometriereferenzen und begrenzte Hintergrundwarteschlangen
erforderlich. Eigene globale veränderliche Zustände werden nicht eingeführt.

Die native Darstellung verwendet eframe/wgpu und eine orthografische Projektion
mit Painter-Sortierung. Sie besitzt noch keinen produktionsreifen Tiefenpuffer
oder exaktes CSG-Flächenpicking. Das Offscreen-wgpu-Bild verwendet dieselben
projizierten Dreiecke. Das EGL-Bild kommt unabhängig aus der Engine und prüft
auch deren Software-Renderingpfad. Fehlende Adapter sind Fehler, keine bestandenen
Bildnachweise. 2D-Teilnetze und deren Rendering sind weiterhin offene Produktanforderungen.
