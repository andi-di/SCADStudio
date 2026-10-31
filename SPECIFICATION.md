# SCADStudio – Produktspezifikation

| Eigenschaft | Festlegung |
| --- | --- |
| Dokumentversion | 1.1 |
| Stand | 31. Oktober 2026 |
| Status | Verbindliches Zielbild; Umsetzung in überprüfbaren Schritten |
| Dokumentensprache | Deutsch |
| Oberflächensprachen | Deutsch und Englisch |
| Implementierung und erzeugte Bezeichner | Englisch |
| Erste Zielplattform | Linux; anschließend Windows und macOS |
| Lizenz des eigenständig entwickelten Anwendungscodes | MIT; Fremdkomponenten behalten ihre Lizenzen |

## Inhaltsverzeichnis

1. [Zweck und Verbindlichkeit](#1-zweck-und-verbindlichkeit)
2. [Produktidee und Grenzen](#2-produktidee-und-grenzen)
3. [Begriffe und Dokumentmodell](#3-begriffe-und-dokumentmodell)
4. [OpenSCAD-Kompatibilität](#4-openscad-kompatibilität)
5. [Gemeinsame Bearbeitung durch Code und UI](#5-gemeinsame-bearbeitung-durch-code-und-ui)
6. [Strukturierte Sprachbearbeitung](#6-strukturierte-sprachbearbeitung)
7. [Grafisches Modellieren](#7-grafisches-modellieren)
8. [Editor und Sprachwerkzeuge](#8-editor-und-sprachwerkzeuge)
9. [Arbeitsfläche und Bedienung](#9-arbeitsfläche-und-bedienung)
10. [Bibliotheken und Offline-Dokumentation](#10-bibliotheken-und-offline-dokumentation)
11. [Customizer, Dateien und Persistenz](#11-customizer-dateien-und-persistenz)
12. [Vorschau, Berechnung und Export](#12-vorschau-berechnung-und-export)
13. [Architektur und interne Schnittstellen](#13-architektur-und-interne-schnittstellen)
14. [MCP und fensterloser Betrieb](#14-mcp-und-fensterloser-betrieb)
15. [Nichtfunktionale Anforderungen](#15-nichtfunktionale-anforderungen)
16. [Durchgehende Bedienbeispiele](#16-durchgehende-bedienbeispiele)
17. [Tests und Abnahme](#17-tests-und-abnahme)
18. [Umsetzungsreihenfolge](#18-umsetzungsreihenfolge)
19. [Lizenzen, Auslieferung und Nachweispflichten](#19-lizenzen-auslieferung-und-nachweispflichten)
20. [Entscheidungen, Annahmen und Quellen](#20-entscheidungen-annahmen-und-quellen)

## 1. Zweck und Verbindlichkeit

Dieses Dokument spezifiziert eine native Desktop-Anwendung, die die programmatische Modellierung von OpenSCAD mit einer grafischen 3D-Bearbeitung und modernen IDE-Werkzeugen verbindet. Es dient als gemeinsame Grundlage für Implementierung, Aufgabenplanung, Tests und Abnahme.

Die erste Zielgruppe sind mit OpenSCAD vertraute Anwender, die technische Bauteile für den 3D-Druck konstruieren: Gehäuse, Halterungen, Adapter und vergleichbare parametrische Einzelteile. Die Anwendung muss zwischen freier Codebearbeitung und grafischer Bearbeitung wechseln können, ohne das Modell in getrennte Darstellungen mit unterschiedlichem Funktionsumfang aufzuspalten.

**MUSS** bezeichnet eine verbindliche Anforderung des vollständigen Produkts. **SOLL** bezeichnet eine Empfehlung, von der nur mit dokumentierter Begründung abgewichen wird. **KANN** bezeichnet eine optionale Erweiterung. Stabile Anforderungs-IDs bleiben bei redaktionellen Änderungen erhalten. Entfernte Anforderungen werden als entfallen dokumentiert; ihre IDs werden nicht neu vergeben.

Die Umsetzungsschritte in Kapitel 18 bestimmen die Reihenfolge, nicht die Reduktion des Produktumfangs. Ein Zwischenstand darf als Prototyp veröffentlicht werden, erfüllt aber erst nach Gesamtprüfung die vollständige Spezifikation.

Das Repository enthält bei Erstellung dieses Dokuments noch keine Anwendung. Beschriebene Schnittstellen, Dateiformate und Leistungswerte sind Festlegungen beziehungsweise Abnahmeziele, keine Behauptungen über bereits vorhandene Funktionen. Insbesondere die vollständige Herkunftszuordnung von Geometrie muss früh technisch nachgewiesen werden.

## 2. Produktidee und Grenzen

### 2.1 Leitlinien

| ID | Verbindliche Anforderung |
| --- | --- |
| PROD-001 | Codeeditor, strukturierte UI und 3D-Werkzeuge MÜSSEN denselben maßgeblichen OpenSCAD-Quelldokumentzustand bearbeiten. |
| PROD-002 | Der normale Modellierungsbeginn MUSS unmittelbar in 3D möglich sein: Körper, Transformationen, Modulaufrufe und boolesche Operationen. |
| PROD-003 | Der Sprachumfang der festgelegten OpenSCAD-Referenz MUSS vollständig ausführbar und strukturiert grafisch bearbeitbar sein. |
| PROD-004 | Modellieren, Sprachwerkzeuge, mitgelieferte Bibliotheken und deren Dokumentation MÜSSEN ohne Internet und ohne Benutzerkonto funktionieren. |
| PROD-005 | Vollständige Referenzkompatibilität MUSS Vorrang vor einer eigenen Sprachauswertung oder unbelegten Performanceoptimierung haben. |
| PROD-006 | Externe KI MUSS das Produkt über MCP bedienen können; eine integrierte KI oder ein bestimmter KI-Anbieter wird nicht vorausgesetzt. |

Die vollständige grafische Bearbeitbarkeit bedeutet nicht, dass jede Funktion einen speziellen 3D-Griff erhält. Sprachkonstrukte sind vollständig über Baum, Formulare und Ausdruckseditor zugänglich. Für geeignete geometrische Operationen kommt die direkte Manipulation hinzu.

### 2.2 Vereinbarte Grenzen

Nicht zum verbindlichen Umfang gehören:

- Ein eigener OpenSCAD-Dialekt oder ein neu entwickelter Geometriekernel.
- Beliebiges direktes Verschieben einzelner Netzflächen oder Verrunden beliebiger ausgewählter Kanten.
- Ein allgemeiner CAD-Constraint-Solver, physikalische Baugruppensimulation oder eine FEM-Umgebung.
- STEP-Import/-Export, Zeichnungsableitung und Fertigungsplanung.
- Cloudspeicherung, gemeinsame Echtzeitbearbeitung, Benutzerkonten oder integrierte KI-Modelle.
- Ein allgemeines Erweiterungssystem, eine integrierte Git-Oberfläche oder ein Terminal.
- Eine Browserfassung oder mobile Oberfläche.

Bibliotheksbasierte Fasen, Rundungen, Gewinde und vergleichbare OpenSCAD-Operationen bleiben Bestandteil des Produkts. 2D-Modellierung, Projektion und Extrusion werden ebenfalls unterstützt, aber nicht als bevorzugter Einstieg erzwungen.

## 3. Begriffe und Dokumentmodell

| Begriff | Bedeutung |
| --- | --- |
| Quelldokument | Inhalt einer `.scad`-Datei einschließlich Kommentaren, Leerraum und Zeilenenden; auch ungespeichert vorhanden. |
| Projekt | Einstiegspunkt, Quelldokumente, Ressourcen und explizite Bibliotheksbindungen. |
| Programmbaum | Statische Struktur des Codes, beispielsweise Schleife, Moduldefinition oder Aufruf. |
| Ausgewertete Instanz | Durch eine konkrete Auswertung erzeugtes Element; ein Aufruf in einer Schleife kann viele Instanzen erzeugen. |
| Modellbaum | Navigation über ausgewertete Instanzen und erzeugende Aufrufe mit Verweisen auf den Programmbaum. |
| Herkunft / Provenienz | Verbindung einer ausgewerteten Instanz zu Quellstelle, Aufrufpfad, Auswertungskontext und Revision. |
| Revision | Monoton fortschreitender Zustand einer Sitzung nach einer übernommenen Dokument- oder Modelleinstellungsänderung. |
| Momentaufnahme | Unveränderlicher Berechnungsstand aus Dokumentinhalten, Einstiegspunkt, Parametern, Ressourcen und Engine-Konfiguration. |
| Transaktion | Zusammengehörige Änderungen, die gemeinsam übernommen und rückgängig gemacht werden. |
| Manipulationsvorschau | Vorläufige visuelle Rückmeldung während einer Interaktion, noch ohne Garantie einer vollständigen Neuberechnung. |
| Referenz-Engine | Mitgelieferter, exakt identifizierter OpenSCAD-Build als Kompatibilitätsmaßstab. |

Die Beziehung ist grundsätzlich:

```text
Quelldokumente + Parameter + Abhängigkeiten
              |
              +--> fehlertolerante Syntaxstruktur --> Editor und Formulare
              |
              +--> originale OpenSCAD-Auswertung --> Instanzen + Geometrie
                                                       |
                                             Herkunft und Auswahl
```

Ein Dreiecksnetz ist ein berechnetes Ergebnis, niemals das maßgebliche Quelldokument. Metadaten dürfen keine erforderliche Modellierungsoperation enthalten, die beim Öffnen der `.scad`-Datei in OpenSCAD verloren geht.

## 4. OpenSCAD-Kompatibilität

### 4.1 Referenz und Reproduzierbarkeit

| ID | Verbindliche Anforderung |
| --- | --- |
| COMP-001 | Eine originale OpenSCAD-Engine MUSS die maßgebliche Sprachsemantik und finale Geometrieberechnung liefern. |
| COMP-002 | Jeder Release MUSS genau einen verbindlichen Referenz-Build mit vollständiger Commit-/Versionskennung, Buildoptionen und Backend-Konfiguration benennen. |
| COMP-003 | Die Referenz MUSS mitgeliefert werden; eine separate OpenSCAD-Installation darf für den normalen Betrieb nicht erforderlich sein. |
| COMP-004 | Standardkonforme `.scad`-Dateien MÜSSEN ohne Migration, spezielle Kommentare oder Projektdatei geöffnet und ausgeführt werden können. |
| COMP-005 | Gespeicherte Modelle MÜSSEN in der unveränderten Referenz-Engine bei gleichen Ressourcen, Parametern und Einstellungen dasselbe Modell erzeugen. |
| COMP-006 | Engine- und Bibliotheksupdates MÜSSEN ausdrücklich erfolgen und die Kompatibilitätsprüfung erneut durchlaufen. |
| COMP-007 | Backend, Bibliotheken, Fonts, Importressourcen, Sondervariablen und relevante Vorschau-/Renderparameter MÜSSEN für reproduzierbare Ergebnisse nachvollziehbar sein. |

Die konkrete Engine-Revision wird im Architekturnachweis anhand eines zu diesem Zeitpunkt aktuellen, reproduzierbar baubaren Upstream-Stands festgeschrieben. Dieses Dokument erfindet keine bereits getestete Versionskombination. Der normale Nutzer wählt nicht zwischen beliebigen installierten Engines; zusätzliche Referenzen sind eine spätere Erweiterung.

Vollständige Kompatibilität bedeutet den vollständigen Sprach- und Funktionsumfang dieses expliziten Builds. Sie verspricht keine gleichzeitige Übereinstimmung mit allen historischen oder zukünftigen Versionen und keinen bitidentischen Netzexport über unterschiedliche Backends hinweg. Die offizielle Dokumentation bildet den Einstieg zur Sprachreferenz [Q1].

### 4.2 Prüfumfang

Die Kompatibilitätsmatrix MUSS alle Konstrukte und Built-ins der Referenz erfassen, einschließlich Funktionen als Werte, rekursiver Ausdrücke, spezieller Variablen, Bereichsregeln, Importverhalten und Modifikatoren, soweit vorhanden. Ein einfacher Geometriekatalog genügt nicht.

Für jede Fähigkeit werden dokumentiert: Ausführung, strukturiertes Bearbeiten, Signatur-/Dokumentationsunterstützung, Dateikontext, repräsentative Testfälle und bekannte Abweichungen. Ein fehlendes Sprachelement ist eine offene Produktanforderung und keine stillschweigende Ausnahme.

Die Referenz entscheidet über Sprachfehler und Geometrieverhalten. Eigene statische Diagnosen sind davon unterscheidbar. Nicht eindeutig statisch bestimmbare Eigenschaften werden als unbekannt behandelt, nicht als sicherer Fehler.

## 5. Gemeinsame Bearbeitung durch Code und UI

### 5.1 Änderungen und Historie

| ID | Verbindliche Anforderung |
| --- | --- |
| EDIT-001 | Jede grafische Modelländerung MUSS durch nachvollziehbare Änderungen gewöhnlichen OpenSCAD-Codes ausgedrückt werden. |
| EDIT-002 | Gezielte Änderungen MÜSSEN unberührte Quellbereiche einschließlich Kommentaren und Formatierung erhalten. Eine Gesamtformatierung ist ein eigener Befehl. |
| EDIT-003 | Dokumente, Programmbaum, Eigenschaften und Auswahl MÜSSEN nach einer übernommenen Änderung konsistent auf denselben aktuellen Zustand bezogen sein. |
| EDIT-004 | Code-, UI- und MCP-Änderungen MÜSSEN dieselbe transaktionale Undo-/Redo-Verarbeitung verwenden. |
| EDIT-005 | Ein abgeschlossener Ziehvorgang MUSS einen Undo-Schritt ergeben; `Esc` MUSS den laufenden Vorgang ohne Dokumentänderung abbrechen. |
| EDIT-006 | Änderungen MÜSSEN ihre Ausgangsrevision prüfen. Veraltete Befehle dürfen neuere Eingaben nicht überschreiben. |
| EDIT-007 | Transaktionen über mehrere Dokumente MÜSSEN im Arbeitsspeicher vollständig oder gar nicht übernommen werden. |
| EDIT-008 | Ungespeicherte Änderungen MÜSSEN für Berechnung und grafische Bearbeitung berücksichtigt werden, ohne zuerst die Originaldateien zu speichern. |

Sinnvoll zusammengehörige Texteingaben dürfen zu Undo-Schritten gebündelt werden. Eine zwischenzeitliche UI- oder MCP-Transaktion beendet die laufende Textgruppe. Jede Transaktion hält Ursprung, betroffene Dokumente und eine verständliche Beschreibung fest.

Wenn nach einer übernommenen, syntaktisch zulässigen Änderung die Engine einen semantischen Fehler meldet, bleibt die Änderung sichtbar und rückgängig machbar. Das Programm darf sie nicht ohne Erklärung automatisch zurücksetzen.

### 5.2 Parametrische Beziehungen

| ID | Verbindliche Anforderung |
| --- | --- |
| PARAM-001 | Berechnete Ausdrücke MÜSSEN bei grafischer Bearbeitung standardmäßig als Beziehungen erhalten bleiben. |
| PARAM-002 | Wenn mehrere Codeänderungen möglich sind, MUSS die UI die sinnvollen Ziele samt betroffener Instanzen zur Auswahl anbieten. |
| PARAM-003 | Das Ersetzen eines Ausdrucks durch einen festen Wert MUSS eine ausdrückliche Auswahl sein. |
| PARAM-004 | Änderungen am Ursprung eines mehrfach verwendeten Parameters oder Modulaufrufs MÜSSEN ihre gemeinsame Wirkung vor der Übernahme erkennbar machen. |
| PARAM-005 | Eine lokale Sonderbehandlung einer erzeugten Instanz darf nur angeboten werden, wenn sie als nachvollziehbarer kompatibler Code darstellbar ist. |

Es wird kein allgemeiner inverser Interpreter vorausgesetzt. Bei `height = calculate_height(config)` darf die Anwendung nicht behaupten, aus einer gewünschten Höhe eindeutig `config` berechnen zu können. Sie bietet die Eingangsparameter und den Ausdruck an. Automatisches algebraisches Umformen ist auf nachweisbar eindeutige unterstützte Fälle begrenzt.

### 5.3 Unvollständiger Code und externe Änderungen

| ID | Verbindliche Anforderung |
| --- | --- |
| REC-001 | Bei Syntaxfehlern MUSS die letzte gültige Vorschau erhalten und deutlich als veraltet markiert bleiben. |
| REC-002 | Grafische Änderungen MÜSSEN in sicher zuordenbaren, aktuell gültigen Bereichen weiter möglich sein. |
| REC-003 | Wenn die Zuordnung nach einer Codeänderung nicht mehr sicher ist, MUSS die betroffene Aktion mit nachvollziehbarem Grund gesperrt werden. |
| REC-004 | Externe Dateiänderungen MÜSSEN erkannt werden. Ungespeicherte Arbeit darf nicht stillschweigend überschrieben werden. |
| REC-005 | Für Dateien mit externen und lokalen Änderungen MUSS ein Vergleich mit Optionen zum Übernehmen, Beibehalten oder manuellen Zusammenführen angeboten werden. |

Die Anzeige alter Geometrie berechtigt nicht zur Anwendung alter Quellpositionen. Eine Auswahl wird nach jeder Revision neu zugeordnet oder ausdrücklich ungültig. Bei einem ungültigen Bereich zeigt der Eigenschaftenbereich den Grund und einen Sprung zur Problemstelle.

## 6. Strukturierte Sprachbearbeitung

### 6.1 Programmbaum und Ausdruckseditor

| ID | Verbindliche Anforderung |
| --- | --- |
| LANG-001 | Der Programmbaum MUSS jede Syntaxkategorie der Referenz anzeigen, einfügen, verändern und entfernen können. |
| LANG-002 | Formulare MÜSSEN benannte und positionale Argumente, ausgelassene Standardargumente, Ausdrücke sowie Kindanweisungen unterscheiden. |
| LANG-003 | Ein strukturierter Ausdruckseditor MUSS Literale, Variablenreferenzen, Operatoren, Indexzugriffe, Aufrufe und weitere Referenzkonstrukte ohne Pflicht zum Tippen von Code bearbeiten können. |
| LANG-004 | Verschachtelte Schleifen, Bedingungen, Definitionen, lokale Bindungen und Gültigkeitsbereiche MÜSSEN strukturiert bearbeitbar sein. |
| LANG-005 | Baumaktionen MÜSSEN ihre Zielposition und die resultierende Verschachtelung vor der Übernahme verständlich zeigen. |
| LANG-006 | Eigene Module und Funktionen MÜSSEN denselben allgemeinen Aufrufkomfort wie integrierte Symbole erhalten. |
| LANG-007 | Optionales direktes Eingeben von Code MUSS parallel möglich bleiben, darf aber die strukturierte Bearbeitung nicht ersetzen. |

Der Ausdruckseditor verwendet verschachtelte Felder: Ein Wertfeld kann beispielsweise in einen Funktionsaufruf umgewandelt werden; dessen Argumente erhalten ihrerseits Wertfelder. Vektoren bieten bearbeitbare Komponenten und variable Länge. Listen, Zeichenketten, boolesche Werte und `undef` dürfen nicht in numerische Schieberegler gezwungen werden.

Eine Strukturaktion erzeugt vollständige Syntax oder einen ausdrücklich sichtbaren, bearbeitbaren Entwurf. Temporäre Entwürfe in einem Formular werden erst mit gültiger Transaktion in das Quelldokument übernommen. Die freie Texteingabe darf jederzeit unvollständig sein.

### 6.2 Signaturen und dynamische Fälle

Aufrufformulare zeigen die deklarierte Signatur, Standardausdrücke, Dokumentation und die tatsächlich angegebenen Argumente. Ein ausgewerteter Wert ist eine zusätzliche Anzeige, kein Ersatz für den gespeicherten Ausdruck.

Bei dynamischen Funktionswerten oder nicht eindeutig auflösbaren Symbolen bleibt die allgemeine Argument-/Ausdrucksbearbeitung verfügbar. Spezialfelder werden nur angezeigt, wenn deren Zuordnung gesichert ist. Symbolnamen aus Bibliotheken oder Anwenderdateien werden nicht übersetzt.

## 7. Grafisches Modellieren

### 7.1 Erzeugen, kombinieren und transformieren

| ID | Verbindliche Anforderung |
| --- | --- |
| CAD-001 | 3D-Primitive, eigene Module und Bibliotheksmodule MÜSSEN über suchbare Einfügeaktionen erzeugt werden können. |
| CAD-002 | Verschieben, Drehen und Skalieren MÜSSEN über Griffe sowie präzise Zahleneingaben möglich sein. |
| CAD-003 | Unterstützte Primitive MÜSSEN Maßgriffe erhalten: mindestens Würfelmaße, Kugelradius sowie Zylinderhöhe und Radien. |
| CAD-004 | Transformationen MÜSSEN globale und lokale Achsen sowie ausdrücklich wählbare Bezugspunkte unterstützen. |
| CAD-005 | Mehrfachauswahl, Duplizieren, Gruppieren, Spiegeln und die booleschen Operationen Vereinigung, Differenz und Schnitt MÜSSEN grafisch verfügbar sein. |
| CAD-006 | Bei Differenzen MUSS die Reihenfolge von Grundkörper und abzuziehenden Operanden ausdrücklich erkennbar und veränderbar sein. |
| CAD-007 | Weitere OpenSCAD-Operationen wie Hülle, Minkowski-Summe, Extrusion und Projektion MÜSSEN über Baum und Formulare verfügbar sein. |
| CAD-008 | 2D-Objekte und -Operationen MÜSSEN ohne separate inkompatible Dokumentart bearbeitbar bleiben. |

Eine skalierende Transformation und eine Änderung ursprünglicher Maße sind verschiedene Aktionen. Der Nutzer muss erkennen, ob `scale(...)` erzeugt oder beispielsweise ein Würfelargument geändert wird. Wiederholte Manipulationen sollen passende vorhandene Transformationen bearbeiten statt unnötige Verschachtelungen anzuhäufen.

### 7.2 Auswahl und Herkunft

| ID | Verbindliche Anforderung |
| --- | --- |
| SEL-001 | 3D-Auswahl MUSS eine Verbindung zum erzeugenden Aufruf und zur Quelldatei anbieten. |
| SEL-002 | Verschachtelte Auswahl MUSS den Wechsel zwischen Ergebnis, Modulinstanz und enthaltenen erzeugenden Körpern ermöglichen. |
| SEL-003 | Mehrfach erzeugte Instanzen desselben Quellaufrufs MÜSSEN im aktuellen Berechnungsstand unterscheidbar sein. |
| SEL-004 | Abgezogene und verdeckte Körper MÜSSEN in einer ausdrücklich aktivierten Bearbeitungsansicht auswählbar sein. |
| SEL-005 | Mehrdeutige Treffer, beispielsweise überlagerte Körper, MÜSSEN durch Durchschalten oder eine Trefferliste auflösbar sein. |
| SEL-006 | Vorschau, Auswahlmarkierung und Ursprungshervorhebung MÜSSEN visuell unterscheidbar sein. |

Nach einer booleschen Operation muss nicht jede Netzfläche eine eindeutige einzelne Ursprungsinstanz besitzen. Wo mehrere Ursprünge beteiligt sind, bietet die Anwendung die nachvollziehbaren Kandidaten beziehungsweise den umschließenden Aufruf an. Sie darf keine erfundene eindeutige Herkunft anzeigen.

Die Bearbeitungsansicht stellt erzeugende Operanden zusätzlich dar. Sie verändert die Modellsemantik nicht. Eine vorübergehend ausgeblendete Darstellung ist ausdrücklich von den OpenSCAD-Modifikatoren `*`, `%`, `#` und `!` getrennt.

### 7.3 Messen, Einrasten und Ausrichten

| ID | Verbindliche Anforderung |
| --- | --- |
| ALIGN-001 | Die UI MUSS Koordinaten, Achsabstände und räumliche Abstände zwischen ausgewählten geeigneten Punkten messen können. |
| ALIGN-002 | Einrasten MUSS für ein konfigurierbares Raster, Achsen, Ursprünge, Mittelpunkte und geeignete Geometriepunkte angeboten werden. |
| ALIGN-003 | Körper MÜSSEN anhand von Begrenzungen, Mittelpunkten oder geeigneten expliziten Bezugspunkten ausrichtbar sein. |
| ALIGN-004 | Standardmäßig MUSS Ausrichten eine einmalig berechnete OpenSCAD-Transformation erzeugen. |
| ALIGN-005 | Optional MUSS eine Ausrichtung über verfügbare Parameter und kompatible Ausdrücke dauerhaft ausgedrückt werden können. |
| ALIGN-006 | Wenn eine gewünschte parametrische Beziehung nicht sicher darstellbar ist, MUSS die UI die einmalige Ausrichtung oder die ausdrückliche Formelbearbeitung anbieten. |

Aus Netzgeometrie ermittelte Maße sind als tessellierungsabhängig zu kennzeichnen. Eine gemessene Zylinderbegrenzung darf nicht als exakter analytischer Radius ausgegeben werden. Dauerhafte Referenzen auf beliebige Dreiecksindizes werden nicht angelegt.

## 8. Editor und Sprachwerkzeuge

| ID | Verbindliche Anforderung |
| --- | --- |
| IDE-001 | Der Editor MUSS mehrere Dateien, Tabs, Zeilennummern, Syntaxhervorhebung, Einrückung, Klammerzuordnung und Codefaltung unterstützen. |
| IDE-002 | Autocomplete MUSS Built-ins, eigene Symbole, sichtbare Variablen, Bibliothekssymbole und passende Argumentnamen berücksichtigen. |
| IDE-003 | Signaturhilfe und Dokumentation MÜSSEN während der Eingabe und über bewussten Aufruf erreichbar sein. |
| IDE-004 | Definitionen, Referenzen und Include-/Use-Ziele MÜSSEN navigierbar sein. |
| IDE-005 | Sicheres Umbenennen MUSS auflösbare Projektdefinitionen und zugehörige Referenzen über mehrere Dateien bearbeiten. |
| IDE-006 | Formatierung MUSS als ausdrückliche Aktion verfügbar sein und semantisch unveränderten Code erzeugen. |
| IDE-007 | Suche und Ersetzen MUSS im Dokument und im Projekt möglich sein; projektweite Ersetzungen benötigen eine überprüfbare Trefferübersicht. |
| IDE-008 | Diagnosen MÜSSEN Schweregrad, Quelle, Quellstelle und verständliche Nachricht anzeigen und navigierbar sein. |
| IDE-009 | Quick Fixes MÜSSEN für sicher behebbare Probleme angeboten werden; mehrdeutige Reparaturen dürfen nicht automatisch angewendet werden. |
| IDE-010 | Autocomplete, Navigation und Indizierung MÜSSEN ohne Netzwerk funktionieren. |

Schreibgeschützte Bibliotheksdefinitionen sind navigierbar. Ein Umbenennen darf Bibliotheksquellen nicht stillschweigend verändern. Wenn dynamische Spracheigenschaften eine vollständige Referenzsuche verhindern, wird der sichere Teil dargestellt und ein semantisch ungesichertes Refactoring gesperrt.

Als erste Quick Fixes sind auflösbare Bibliotheksimporte, eindeutig fehlende benannte Argumentzuordnungen und bekannte sichere Syntaxkorrekturen vorzusehen. Die konkrete Liste wird mit Verhaltenstests gepflegt; eine unsichere Namensähnlichkeit genügt nicht für automatische Korrektur.

## 9. Arbeitsfläche und Bedienung

### 9.1 Standardlayout

```text
+--------------------- Befehle / Suche / Status ---------------------+
| Projekt / Baum | Codeeditor       | 3D-Ansicht       | Eigenschaften |
|                |                  |                 | / Parameter   |
+----------------+------------------+-----------------+---------------+
| Diagnosen / Engine-Ausgabe / Berechnungsaufträge / Bibliotheken      |
+--------------------------------------------------------------------+
```

| ID | Verbindliche Anforderung |
| --- | --- |
| UI-001 | Bereiche MÜSSEN andockbar, verschiebbar, größenveränderbar und ausblendbar sein. Das Standardlayout MUSS wiederherstellbar sein. |
| UI-002 | Code und 3D MÜSSEN gleichzeitig nutzbar sein; kein Moduswechsel darf eine Seite schreibgeschützt machen. |
| UI-003 | Eine Befehlspalette MUSS zentrale Aktionen per Tastatur auffindbar machen. |
| UI-004 | Tastaturfokus, Auswahl, aktive Datei, ungespeicherter Zustand und laufende Berechnungen MÜSSEN deutlich sichtbar sein. |
| UI-005 | UI-Texte MÜSSEN auf Deutsch und Englisch verfügbar sein. Sprachwechsel darf Dokumente und Codebezeichner nicht verändern. |
| UI-006 | Die Oberfläche MUSS eine Theme-Auswahl in der Reihenfolge **System → Dark → Light** anbieten. Standard MUSS **System** sein; dabei folgt die Darstellung der Betriebssystemvorgabe. Helles und dunkles Farbschema sowie skalierbare Schrift und Abstände MÜSSEN unterstützt werden. |
| UI-007 | Alle zentralen Bearbeitungsaktionen MÜSSEN über Tastatur und Formulare erreichbar sein; ein Griff darf keine exklusive Eingabemöglichkeit sein. |
| UI-008 | Orbit, Schwenken, Zoom, orthografische/perspektivische Projektion, Standardansichten und Einpassen der Auswahl MÜSSEN verfügbar sein. |
| UI-009 | Leere Projekte, fehlende Auswahl, Ladevorgänge und Fehler MÜSSEN eigene verständliche Ansichten erhalten. |

Standardtasten sind `Ctrl+S` zum Speichern, `Ctrl+Z`/`Ctrl+Shift+Z` für Undo/Redo, `Ctrl+P` zum Dateiöffnen und `Ctrl+Shift+P` für die Befehlspalette. `F5` fordert eine Vorschau an, `F6` eine finale Berechnung. Die automatische Vorschau bleibt davon unabhängig aktiv. Editor- und 3D-Kontext dürfen keine unbemerkten konkurrierenden Tastaturaktionen auslösen.

Die erste Linux-Abnahme berücksichtigt 1280 × 800 logische Pixel sowie HiDPI mit 100 %, 150 % und 200 %. Auf kleinen Flächen dürfen Seitenbereiche eingeklappt werden. Farbe allein genügt nicht als Statusinformation.

Die Theme-Auswahl MUSS zur Laufzeit ohne Neustart zwischen **System**, **Dark** und **Light** umschaltbar sein. Im Modus **System** werden Änderungen der Betriebssystemvorgabe automatisch übernommen; ist keine Vorgabe ermittelbar, wird **Dark** verwendet. Eine ausdrücklich gewählte Einstellung **Dark** oder **Light** hat Vorrang vor der Systemvorgabe. Die Auswahl wird benutzerbezogen über Neustarts hinweg gespeichert und nicht in Projektmetadaten abgelegt. Das Farbschema gilt konsistent für Editor, Panels, Dialoge, Menüs und die Darstellung der 3D-Ansicht; Modellfarben und exportierte Modellgeometrie werden dadurch nicht verändert. Die sichtbaren Bezeichnungen werden passend zur gewählten Oberflächensprache übersetzt.

## 10. Bibliotheken und Offline-Dokumentation

### 10.1 Verbindlicher Katalog

| Bibliothek | Integrationsschwerpunkt |
| --- | --- |
| BOSL | Bestehende Modelle und allgemeine Form-/Transformationshilfen. |
| BOSL2 | Umfangreiche Formen, Befestigungen, Gewinde und Attachment-basierte Konstruktion. |
| MCAD | Etablierte mechanische und allgemeine Modellbausteine. |
| NopSCADlib | Technische Komponenten, Gehäuse und Bauteile. |
| dotSCAD | Mathematische und prozedurale Modellierung. |

Die Bibliotheken werden nicht nach einer behaupteten Popularitätsrangfolge gewählt, sondern sind der vereinbarte kuratierte Startumfang. OpenSCAD führt mehrere dieser Bibliotheken in seinem offiziellen Katalog [Q2]. BOSL und BOSL2 sind getrennte Bibliotheken; BOSL2 dokumentiert die fehlende Quellkompatibilität zu BOSL [Q3].

| ID | Verbindliche Anforderung |
| --- | --- |
| LIB-001 | Alle fünf Bibliotheken MÜSSEN in festgelegten Versionen offline bereitstehen. |
| LIB-002 | Der Katalog MUSS Symbole nach Name, Bibliothek und dokumentierter Kategorie durchsuchbar machen. |
| LIB-003 | Signaturen, Standardargumente und verfügbare Dokumentation MÜSSEN offline mit dem Aufruf erreichbar sein. |
| LIB-004 | Grafische Aufrufformulare MÜSSEN auch komplexe Ausdrücke, Listen, Funktionswerte und Kindgeometrie akzeptieren. |
| LIB-005 | Allgemeine Formulare MÜSSEN jedes öffentlich aufrufbare Symbol abdecken. Spezialisierte Komfortformulare dürfen sie ergänzen. |
| LIB-006 | Mitgelieferte Bibliotheksquellen MÜSSEN schreibgeschützt sein; eine ausdrückliche Projektkopie MUSS möglich sein. |
| LIB-007 | Weitere lokale Bibliotheken MÜSSEN eingebunden und mit allgemeinen Sprachwerkzeugen indiziert werden können. |
| LIB-008 | Updates MÜSSEN Version, Herkunft und Auswirkungen zeigen. Bestehende Projektbindungen dürfen nicht automatisch verändert werden. |
| LIB-009 | Namens- und Suchpfadkonflikte MÜSSEN erkennbar sein; die tatsächlich aufgelöste Datei MUSS angezeigt werden. |

Der Katalog speichert Quellrevision, Inhaltsprüfsumme, Lizenzhinweise und Dokumentationsherkunft. Dokumentationsinhalte behalten die Originalsprache, sofern keine lizenzierte Übersetzung vorliegt. Eine vollständige deutsche Übersetzung aller Fremddokumentation ist keine Voraussetzung.

Die Auflösung von `include` und `use` muss den Regeln der Referenz entsprechen. Projektgebundene Bibliotheken werden in die kontrollierte Engine-Umgebung eingebunden; es wird kein abweichender eigener Namensraum erfunden. Die effektiven Suchpfade sind einsehbar.

Vorschaubilder und Beispiele SOLLEN für typische Einstiegsbausteine vorhanden sein. Ihre Erzeugung darf den Start des Katalogs nicht blockieren. Aus Dokumentationsbeispielen darf nicht automatisch ungeprüfter Projektcode werden.

## 11. Customizer, Dateien und Persistenz

### 11.1 Projekte und Einstellungen

| ID | Verbindliche Anforderung |
| --- | --- |
| FILE-001 | Einzelne `.scad`-Dateien MÜSSEN ohne Projektmetadaten geöffnet, bearbeitet und gespeichert werden können. |
| FILE-002 | Ein Projekt MUSS einen Einstiegspunkt, mehrere Quelldateien und referenzierte Ressourcen unterstützen. |
| FILE-003 | Projektmetadaten MÜSSEN als versionierte JSON-Datei mit der Endung `.scadstudio.json` gespeichert werden. |
| FILE-004 | Projektdateien MÜSSEN relative Pfade bevorzugen. Maschinenabhängige absolute Pfade sind ausdrücklich als externe Abhängigkeit auszuweisen. |
| FILE-005 | Layout, Theme-Auswahl, Verlauf, Wiederherstellungsdaten und Freigabetoken MÜSSEN außerhalb der gemeinsam versionierten Projektdatei liegen. |
| FILE-006 | Beschädigte oder zukünftige unbekannte Metadatenformate dürfen nicht stillschweigend überschrieben werden. Die `.scad`-Dateien MÜSSEN weiterhin separat zugänglich sein. |
| FILE-007 | Speichern MUSS pro Datei über eine temporäre Datei im selben Dateisystem und anschließenden Austausch erfolgen. Fehler MÜSSEN den ungespeicherten Zustand erhalten. |
| FILE-008 | Wiederherstellungsstände MÜSSEN ungespeicherte Arbeit nach einem Absturz anbieten, ohne Originaldateien ungefragt zu ersetzen. |

Vorgesehenes Schema der ersten Projektdateiversion:

```json
{
  "schema_version": 1,
  "name": "Enclosure",
  "entrypoint": "src/enclosure.scad",
  "engine": { "reference_id": "<locked-engine-reference>" },
  "libraries": [
    {
      "name": "BOSL2",
      "revision": "<full-source-revision>",
      "content_hash": "sha256:<digest>"
    }
  ],
  "search_paths": ["vendor"],
  "external_resources": [],
  "customizer": { "file": "variants.json", "preset": "Default" }
}
```

Werte in Winkelklammern zeigen hier die Form von Sperreinträgen, keine gültigen Auslieferungsversionen. Ein ausgeliefertes Projekt verwendet tatsächliche Kennungen. Die Einstiegspfad- und Presetauflösung ist relativ zur Projektdatei. Ein Projekt ohne explizite Bindungen verwendet die mitgelieferten Standardversionen und zeigt diese an.

Mehrere atomar im Arbeitsspeicher geänderte Dateien sind nicht automatisch eine atomare Dateisystemtransaktion. Bei teilweisem Speicherfehler werden bereits geschriebene und noch ungespeicherte Dateien einzeln benannt. Über alle Dateien erfolgreich gespeichert wird erst nach dem letzten erfolgreichen Austausch gemeldet.

### 11.2 Customizer und Varianten

| ID | Verbindliche Anforderung |
| --- | --- |
| CUST-001 | Customizer-Kommentare, Gruppen, Wertebereiche und Auswahlmöglichkeiten der Referenz MÜSSEN erkannt werden. |
| CUST-002 | Kompatible OpenSCAD-Parameterdateien und benannte Parametersätze MÜSSEN gelesen und geschrieben werden können. |
| CUST-003 | Varianten MÜSSEN über die UI angelegt, ausgewählt, dupliziert, umbenannt und entfernt werden können. |
| CUST-004 | Die UI MUSS unterscheiden, ob ein Quelldefault oder ein Wert des aktiven Parametersatzes bearbeitet wird. |
| CUST-005 | Berechnung und Export MÜSSEN die gewählte Variante einschließlich ungespeicherter Änderungen verwenden. |

Eine Variante verändert nicht verdeckt die Quelldefinition. Beim Auswählen eines Parametersatzes zeigt der Eigenschaftenbereich sowohl den überschriebenen Wert als auch den Quellwert. Ungültige oder nicht mehr vorhandene Parameter werden diagnostiziert statt verworfen.

### 11.3 Ressourcen und Dateiaustausch

Standard-`.scad`-Dateien sind das primäre Austauschformat. STL und 3MF dienen dem 3D-Druck; SVG/DXF und weitere Formate stehen entsprechend der tatsächlichen Import-/Exportfähigkeiten der Referenz zur Verfügung. PNG dient dem Bildexport. Import und Export sind getrennte Fähigkeitslisten: Ein Exportformat muss nicht auch importierbar sein.

Fonts und andere systemabhängige Ressourcen werden im Projektstatus benannt. Auf einem anderen Rechner fehlende Fonts müssen erkennbar sein, weil eine Ersatzschrift die Geometrie verändern kann. Reproduzierbarkeit setzt dieselben relevanten Ressourcen voraus. Ein portables Projektarchiv mit automatisch gebündelten Ressourcen ist eine optionale spätere Erweiterung.

## 12. Vorschau, Berechnung und Export

| ID | Verbindliche Anforderung |
| --- | --- |
| RUN-001 | Vorschau MUSS standardmäßig nach 300 ms Eingabepause angefordert werden; der Wert ist konfigurierbar. |
| RUN-002 | Ziehen MUSS unmittelbar eine Manipulationsvorschau liefern; diese MUSS als vorläufig erkennbar sein. |
| RUN-003 | Eine neue Dokumentrevision MUSS ältere Vorschauaufträge ersetzen. Ergebnisse veralteter Momentaufnahmen dürfen den aktuellen Stand nicht überschreiben. |
| RUN-004 | Aktive Berechnungen MÜSSEN Fortschrittszustand, Abbruchmöglichkeit und nachvollziehbare Fehler liefern. Ein Prozentwert darf nur bei geeigneter Messgrundlage angezeigt werden. |
| RUN-005 | Finale Geometrieberechnung und Manipulationsvorschau MÜSSEN klar unterscheidbare Zustände haben. |
| RUN-006 | Exporte MÜSSEN die beim Start ausdrücklich angeforderte Momentaufnahme und eine erfolgreiche passende Berechnung verwenden. |
| RUN-007 | Ein Export darf ein bestehendes Ziel erst nach erfolgreicher Berechnung und ausdrücklicher Überschreibfreigabe ersetzen. |
| RUN-008 | Für fehlgeschlagene oder abgebrochene Exporte darf kein unvollständiges Ergebnis als erfolgreiches Ziel verbleiben. |

Ein Exportauftrag hält die Ausgangsrevision fest. Wird währenddessen weitergearbeitet, bleibt der Auftrag auf dieser Revision; die Fertigmeldung benennt die exportierte Variante und den Stand. Ein neuer aktueller Export ist ein neuer Auftrag. Das verhindert sowohl ein stilles Nachziehen späterer Änderungen als auch einen unbemerkten Export alter Vorschaugeometrie.

Nicht sichtbare oder minimierte Fenster dürfen keine permanenten Repaints auslösen. Geometrie und GPU-Ressourcen werden zwischen unveränderten Frames wiederverwendet. Fenstergrößenänderungen, Geräteverlust und nicht verfügbare GPU-Fähigkeiten erhalten eigene Behandlung.

## 13. Architektur und interne Schnittstellen

### 13.1 Module und Abhängigkeiten

Ein einzelnes Cargo-Anwendungsprojekt mit Bibliotheksziel ist der Ausgangspunkt. `src/main.rs` enthält nur Initialisierung, Argumentauswertung und Start. Zusätzliche Workspace-Crates benötigen einen konkreten Nutzen; ein separat gebauter OpenSCAD-Worker ist eine eigene Fremdkomponente.

| Bereich | Verantwortung | Darf nicht enthalten |
| --- | --- | --- |
| `document` | Text, Revisionen, Transaktionen, Undo/Redo. | egui- oder GPU-Typen. |
| `language` | Fehlertoleranter Syntaxbaum, Symbole, Quelländerungen, IDE-Abfragen. | Eigene maßgebliche Ausführung der OpenSCAD-Sprache. |
| `application` | Sitzungen, Befehle, Auswahlbezüge, Auftragskoordination. | Direkte Widget- oder Backend-Abhängigkeiten. |
| `engine` | Prozessverwaltung, Momentaufnahmen, Berechnungen, Herkunftsdaten. | Direkte UI-Zustandsänderungen aus Worker-Threads. |
| `libraries` | Katalog, Sperreinträge, Suchpfade und Dokumentation. | Unbemerkte Netzwerkupdates. |
| `persistence` | Speichern, Ressourcenauflösung, Migration, Wiederherstellung. | Verdeckte Modellierungssemantik. |
| `render` | GPU-Puffer, Kamera, Darstellung, Picking und Hervorhebung. | Maßgeblicher Dokumentzustand. |
| `ui` | egui-Widgets und flüchtiger Interaktionszustand. | Umgehung der gemeinsamen Befehlsverarbeitung. |
| `automation` | MCP, lokale IPC und fensterloser Start. | Eigene alternative Modellierungslogik. |

Abhängigkeiten verlaufen zur UI-unabhängigen Dokument- und Anwendungsschicht. Globale veränderliche Zustände und zyklische Modulabhängigkeiten sind zu vermeiden. Fehler erwartbarer externer Operationen werden mit `Result` und Kontext behandelt. `unwrap()`/`expect()` sind für externe Daten, Eingaben und Dateioperationen ausgeschlossen.

egui/eframe stellt die native UI bereit; wgpu wird über die zu eframe passende Integration verwendet. Die offizielle eframe-Dokumentation beschreibt das unterstützte Rendering-Backend [Q4]. Die konkrete kompatible Version einschließlich möglicher Docking-/Editorkomponenten wird beim Aufbau geprüft und in `Cargo.lock` fixiert, nicht aus unabhängigen neuesten Einzelversionen zusammengesetzt.

### 13.2 Zentrale Datentypen

Die folgende Tabelle legt die fachlichen Verträge fest, nicht die vollständige Rust-Implementierung:

| Typ | Mindestinhalt und Regeln |
| --- | --- |
| `SessionId`, `DocumentId` | Opake Identitäten; keine freien Dateipfade als Ersatz für Identitätsprüfung. |
| `Revision` | Monotoner Sitzungsstand; Undo erzeugt eine neue Revision mit altem Inhalt. |
| `SourceRange` | Dokumentidentität und halb offener UTF-8-Bytebereich, an eine Revision gebunden. |
| `SyntaxNodeId` | Identität im Syntaxstand; nur nach bestätigter Neuzuordnung revisionsübergreifend verwendbar. |
| `TextEdit` | Bereich und Ersatztext; mehrere Änderungen werden auf derselben Ausgangsrevision geprüft. |
| `ChangeSet` | Ausgangsrevision, Ursprung, Beschreibung und zusammengehörige Dokumentänderungen. |
| `ModelTarget` | Quellknoten oder ausgewertete Instanz mit Revisionsbindung; kein ungesicherter Netzindex. |
| `EvaluationSnapshot` | Unveränderliche Dokumentinhalte, Abhängigkeiten, Parameter, Ressourcenfingerprints und Engine-Referenz. |
| `InstanceOrigin` | Quellstelle, Aufruf-/Instanzpfad, Transformation, verfügbare ausgewertete Argumente und Berechnungskennung. |
| `EvaluationResult` | Momentaufnahme-ID, Status, Instanzstruktur, Renderdaten und Diagnosen. |
| `Diagnostic` | Quelle, Schweregrad, Code, Nachricht, optionale Quellstelle und optionale sichere Reparaturen. |
| `Job` | ID, Typ, Momentaufnahme, Status, Fortschritt, Ergebnis oder Fehler sowie Abbruchzustand. |
| `LibraryBinding` | Name, vollständige Revision, Inhaltsprüfsumme und aufgelöster Ort. |

Engine-Zeilen/-Spalten und Editorpositionen werden explizit auf diese Quellbereiche abgebildet. Unicode, CRLF und nicht-ASCII-Pfade gehören zur Prüfung. Eine Anzeige verwendet menschenlesbare Zeilen und Spalten; MCP-Textbereiche sind ausdrücklich als UTF-8-Bytes dokumentiert.

### 13.3 Verarbeitung einer grafischen Änderung

1. Die UI erfasst Ziel, aktuelle Revision und gewünschte Operation.
2. Die Anwendung prüft Zielgültigkeit, Schreibbarkeit und mögliche Mehrdeutigkeit.
3. Die Sprachschicht erzeugt einen Änderungsvorschlag mit betroffenen Quellbereichen und Auswirkungen.
4. Falls erforderlich, wählt der Nutzer einen Änderungsweg. Bei MCP liefert der Plan dieselben Optionen zurück.
5. Die Anwendung prüft die Ausgangsrevision erneut und übernimmt den Änderungssatz atomar im Arbeitsspeicher.
6. Historie und abhängige Ansichten werden aktualisiert; eine neue Momentaufnahme wird nach Bedarf angefordert.
7. Nur das Ergebnis der weiterhin passenden Momentaufnahme darf als aktueller Berechnungsstand erscheinen.

Ein Drag verwendet bis zum Abschluss flüchtigen UI-Zustand. Er schreibt nicht pro Frame Dokumenttext. Währenddessen eintreffende fremde Änderungen am selben Ziel führen zum Abbruch beziehungsweise zur erneuten Planung, nicht zu einer verlorenen Aktualisierung.

### 13.4 Engine-Worker und Herkunft

Die Engine läuft in einem getrennten Prozess. Bestehende Engine-Funktionen werden für Berechnung und Standardexport verwendet. Für fehlende Herkunftsdaten wird ein eng begrenzter, separat lizenzierter OpenSCAD-Worker beziehungsweise eine Upstream-Erweiterung vorgesehen. Sprach- und Geometriesemantik werden dabei nicht absichtlich verändert.

OpenSCAD besitzt intern bereits Verbindungen zwischen Knoten, Modulinstanziierungen und Quellpositionen [Q5]. Das ist ein Ansatzpunkt, kein Nachweis einer vollständigen öffentlichen API. Der erste Meilenstein muss zeigen, dass wiederholte Modulaufrufe, Schleifen, `children()` und Bibliotheksverschachtelungen ausreichend unterscheidbare Herkunft liefern.

Der Worker-Vertrag enthält Protokollversion, Fähigkeitenabfrage, Momentaufnahme-ID, Berechnung, Abbruch, Diagnosen, Instanzbaum und Geometriereferenzen. Große Geometrie wird über auftragsgebundene Binärartefakte übertragen; ein Parser darf dafür keine ungeprüften beliebigen Dateipfade übernehmen. Das Protokoll ist von MCP getrennt.

Engine-interne Knotenindizes werden nicht als dauerhaft stabile Dokument-IDs behandelt. Nach Neuberechnung wird Auswahl über bestätigte Quell-/Aufrufpfade neu zugeordnet oder aufgehoben. Herkunft wird während der Auswertung erfasst, nicht ausschließlich aus einem fertigen STL-Netz rekonstruiert.

Ungespeicherte Projekte werden über eine kontrollierte Dateiansicht für den Worker bereitgestellt. Relative Include-/Importpfade behalten ihre Bedeutung. Die Implementierung muss dabei auch Ressourcen neben ungespeicherten Quelldateien, verschachtelte Includes und externe freigegebene Ressourcen korrekt behandeln.

### 13.5 Nebenläufigkeit und Caches

Für jede Sitzung gibt es höchstens einen aktiven Vorschauauftrag und einen ersetzbaren wartenden Vorschaustand. Finale Berechnungen und Exporte verwenden eine begrenzte Warteschlange mit standardmäßig acht wartenden Aufträgen; bei voller Warteschlange wird ein verständlicher Fehler geliefert. Die globale Zahl rechenintensiver Worker ist konfigurierbar und standardmäßig zwei.

Sprachindizierung ist ebenfalls revisionsgebunden. Wiederholte Abfragen dürfen gemeinsam genutzte unveränderliche Ergebnisse verwenden. Cache-Schlüssel umfassen alle relevanten Dokument-, Parameter-, Bibliotheks-, Ressourcen- und Engine-Zustände. Ein Cache darf korrekte Invalidierung nicht durch vermeintlich ähnliche Dateinamen ersetzen.

Ein neuer Vorschauauftrag fordert den Abbruch des alten an. Reagiert ein isolierter Worker nicht innerhalb von zwei Sekunden, darf er beendet und neu gestartet werden. Finale Benutzeraufträge werden durch eine Codeänderung nicht automatisch abgebrochen, da ihre Momentaufnahme weiter gültig bleibt.

## 14. MCP und fensterloser Betrieb

### 14.1 Sitzung und Transport

| ID | Verbindliche Anforderung |
| --- | --- |
| MCP-001 | Ein lokaler MCP-Server MUSS lesenden und schreibenden Zugriff auf den vollständigen vereinbarten Modellierungsablauf anbieten. |
| MCP-002 | Dieselben Anwendungsbefehle MÜSSEN über UI und MCP verwendet werden. |
| MCP-003 | Die aktive GUI-Sitzung und eigene fensterlose Sitzungen MÜSSEN unterstützt werden. |
| MCP-004 | Nach ausdrücklicher Freigabe darf der Client im freigegebenen Projekt direkt arbeiten; eine zusätzliche Bestätigung jeder Codeänderung darf nicht erforderlich sein. |
| MCP-005 | Freigabe MUSS widerrufbar sein. Freigegebene Pfade, Sitzung und verbundener Client MÜSSEN in der GUI erkennbar sein. |
| MCP-006 | MCP-Schreibbefehle MÜSSEN eine erwartete Ausgangsrevision nennen und bei Konflikt ohne Teiländerung abbrechen. |
| MCP-007 | Lang laufende Aufträge MÜSSEN eine abrufbare Auftrags-ID und eine Abbruchmöglichkeit liefern. |
| MCP-008 | Modellansichten MÜSSEN auch im fensterlosen Betrieb als Bilder abrufbar sein. |

Der öffentliche lokale Transport ist stdio. Ein kleiner Start-/Brückenprozess verbindet sich über lokale, benutzergebundene IPC mit der GUI oder startet eine eigene fensterlose Sitzung. Logs gehen an stderr, nicht in den MCP-Nachrichtenstrom. Der Transport folgt dem offiziellen MCP-Modell [Q6]. Ein öffentlich erreichbarer HTTP-Server ist kein Bestandteil der ersten Umsetzung.

Vorgesehene CLI:

```text
scadstudio [project-or-scad-file]
scadstudio mcp --attach <session-id> --token-file <local-token-file>
scadstudio mcp --headless --project <path> --grant-root <path>
```

Die GUI stellt die Attach-Konfiguration bereit. Ein Token wird nicht als Klartext in der Kommandozeile oder Projektdatei gespeichert. Bei fensterlosem Start ist die explizite Startkonfiguration die Freigabe; es wird keine GUI-Bestätigung vorausgesetzt. Schreibzugriff ist auf freigegebene Wurzeln beschränkt. Abhängigkeiten außerhalb davon benötigen einen eigenen freigegebenen Lesebereich.

### 14.2 Werkzeuge und Ressourcen

Die Werkzeugnamen sind der vorgesehene öffentliche Vertrag der ersten Schnittstellenversion. Detaillierte JSON-Schemas werden daraus bei der Implementierung abgeleitet und gemeinsam mit Integrationstests versioniert.

| Werkzeug | Zweck / wesentliche Eingaben |
| --- | --- |
| `session_get` | Aktuelle Revision, Fähigkeiten, Freigaben und Berechnungsstatus. |
| `project_open` | Projekt oder `.scad` innerhalb freigegebener Pfade öffnen; ungespeicherte Arbeit nicht verwerfen. |
| `document_read` | Dokument, Revision und wahlweise Quellbereich lesen. |
| `document_apply_edits` | Textänderungen mit `expected_revision` übernehmen. |
| `model_inspect` | Programmbaum, Instanzen, Eigenschaften und Herkunft abfragen. |
| `model_plan_operation` | Strukturierte Operation am Ziel vorbereiten; Auswirkungen und Mehrdeutigkeiten zurückgeben. |
| `model_apply_operation` | Geplante Operation mit gewählter Alternative und passender Revision übernehmen. |
| `library_search` | Offline-Katalog und Symbole suchen. |
| `symbol_get_documentation` | Signatur, Dokumentation und Herkunft lesen. |
| `diagnostics_get` | Statische und Engine-Diagnosen samt Aktualitätsstand lesen. |
| `variant_list` / `variant_select` | Parametersätze abfragen und aktive Variante ändern. |
| `view_set` / `view_capture` | Kamera und Auswahl setzen; aktuelle oder angeforderte Ansicht als Bild erfassen. |
| `evaluation_start` | Vorschau oder finale Berechnung für eine Momentaufnahme starten. |
| `export_start` | Format, Ziel, Variante und ausdrückliche Überschreiboption für einen Export angeben. |
| `job_get` / `job_cancel` | Ergebnis, Fortschritt oder Abbruch eines Auftrags. |
| `project_save` | Benannte Dokumente und Metadaten speichern; Teilergebnisse ausweisen. |
| `history_undo` / `history_redo` | Historie unter Revisionsprüfung bedienen. |

Lesbare Projekt-, Dokument-, Diagnose- und Dokumentationsinhalte werden zusätzlich als MCP-Ressourcen angeboten. Bildantworten benennen Momentaufnahme und Vorschau-/Finalstatus. Ein Bild einer veralteten Geometrie darf nicht als aktuelle Modellprüfung erscheinen.

Alle Änderungsantworten enthalten die resultierende Revision und die Transaktionskennung. Fehler unterscheiden mindestens `revision_conflict`, `ambiguous_target`, `stale_target`, `read_only`, `permission_denied`, `invalid_input`, `evaluation_failed` und `queue_full`. Bei Mehrdeutigkeit werden verfügbare Optionen zurückgegeben; MCP darf nicht eigenmächtig einen anderen Standard als die UI verwenden.

### 14.3 Gleichzeitige Bedienung

Mehrere Verbindungen dürfen lesen. Schreibbefehle werden durch die Anwendung seriell übernommen. Nutzer- und KI-Eingaben erhalten keinen stillen Vorrang voreinander; die Revisionsprüfung verhindert Überschreiben. Ein KI-Plan wird ungültig, sobald sich sein Ausgangsstand geändert hat.

MCP-Aktionen dürfen weder beliebige Shellbefehle ausführen noch den freigegebenen Bereich durch symbolische Links oder relative Pfadbestandteile verlassen. Berechtigte lokale Ressourcenauflösung und Projekte mit externen Abhängigkeiten bleiben über ausdrückliche Freigaben möglich. Widerruf beendet die Verbindung und bricht zugehörige noch nicht abgeschlossene schreibende Aufträge vor ihrer finalen Übernahme ab; bereits abgeschlossene Änderungen bleiben in der Historie.

Fensterloser Bildexport muss ohne sichtbaren Displayserver funktionieren. Der Architekturnachweis prüft Offscreen-wgpu und eine geeignete Software-/Engine-Renderingmöglichkeit. Falls der erforderliche Grafikpfad fehlt, wird die Fähigkeit als nicht verfügbar gemeldet; ein Release darf die verbindliche fensterlose Bildfunktion damit nicht als abgenommen ausweisen.

## 15. Nichtfunktionale Anforderungen

### 15.1 Reaktionszeit und Leistungsprofil

Der Leistungsfokus sind technische Einzelteile mit Details und einigen hundert erzeugten Körpern, keine garantierte interaktive Bearbeitung beliebig großer prozeduraler Szenen.

Die folgenden Werte sind festgelegte Abnahmeziele für Release-Builds auf einem dokumentierten Linux-Referenzsystem: x86-64, mindestens acht CPU-Kerne, 16 GB RAM, SSD und eine von wgpu unterstützte GPU. Der Prüfbericht hält konkrete Hardware, Treiber, Betriebssystem, Engine, Modellprüfsummen und Kalt-/Warmzustand fest. Das spezifiziert eine Testklasse, keine gesammelten Benutzermetadaten.

| ID | Ziel und Messung |
| --- | --- |
| PERF-001 | Editor- und Befehlsrückmeldung: p95 unter 100 ms bei einer 10.000-Zeilen-Datei; Hintergrundberechnung darf die Eingabe nicht blockieren. |
| PERF-002 | Kamera und Manipulationsgriffe: mindestens 30 FPS bei einer bestehenden Ansicht bis 250.000 Dreiecke und 500 auswählbaren Instanzen; keine vollständige CSG-Neuberechnung pro Frame. |
| PERF-003 | Warme Autocomplete-/Signaturabfrage: p95 unter 150 ms nach abgeschlossener Indizierung des Referenzprojekts. |
| PERF-004 | Sichtbare Bestätigung eines Berechnungsstarts oder Abbruchs innerhalb von 100 ms; nicht reagierender Worker wird gemäß Kapitel 13 behandelt. |
| PERF-005 | Warmer Start bis zur bedienbaren leeren Arbeitsfläche unter drei Sekunden; Bibliotheksindizierung darf anschließend im Hintergrund fortfahren. |
| PERF-006 | Finale Berechnung ohne Herkunftszusatz: Median höchstens 25 % über demselben externen Referenz-Engine-Lauf für das Abnahmekorpus; Prozessstart und Cachezustand werden gleich behandelt. |

Geometrieberechnungen erhalten keine unrealistische universelle Sekundenobergrenze. Referenz- und SCADStudio-Zeiten werden für identische Qualität, Parameter und Ressourcen verglichen. Laufzeiten für Herkunftserfassung und Bearbeitungsgeometrie werden separat ausgewiesen. Mindestens 30 Wiederholungen dienen interaktiven p95-Messungen; für teure Finalberechnungen mindestens fünf vergleichbare Läufe.

### 15.2 Robustheit und Ressourcen

| ID | Verbindliche Anforderung |
| --- | --- |
| NFR-001 | Blockierende I/O und teure Berechnungen MÜSSEN außerhalb des UI-Threads stattfinden. |
| NFR-002 | Warteschlangen, Cachegrößen und parallele Berechnungen MÜSSEN begrenzt sein. |
| NFR-003 | Engine-Absturz, Speicherknappheit und ungültige externe Daten dürfen keine stillen Projektänderungen verursachen. |
| NFR-004 | Fehler MÜSSEN Kontext und, soweit möglich, eine sinnvolle nächste Aktion anbieten. |
| NFR-005 | Ohne aktive Interaktion, Berechnung oder Animation darf die Anwendung keine permanente Rendering-Schleife benötigen. |
| NFR-006 | Netzwerkzugriffe MÜSSEN ausdrücklich ausgelösten Bibliotheks-/Programmupdates zuordenbar sein; keine verpflichtende Telemetrie. |
| NFR-007 | Modelle, Quelldateien oder MCP-Inhalte dürfen nicht automatisch an externe Dienste übertragen werden. |
| NFR-008 | IDs für Widgets MÜSSEN stabil sein; temporärer Fokuszustand darf nicht mit persistenten Modellidentitäten vermischt werden. |

## 16. Durchgehende Bedienbeispiele

### 16.1 Maßänderung mit erhaltener Formel

Ausgangscode:

```scad
base_height = 10;

// Keep the relation to the base height.
cube([30, 20, base_height * 2]);
```

Der Nutzer zieht die Höhe auf 24. Die UI zeigt die Formel und bietet die Änderung von `base_height` oder des Ausdrucks an. Bei ausdrücklich gewählter Änderung des Eingangsparameters entsteht:

```scad
base_height = 12;

// Keep the relation to the base height.
cube([30, 20, base_height * 2]);
```

Nur der betroffene Wert wird ersetzt. Kommentar und übriger Code bleiben erhalten. Ohne ausdrückliche Auswahl entsteht nicht `cube([30, 20, 24]);`. Die gesamte Aktion ist ein Undo-Schritt.

### 16.2 Auswahl in einer Schleife

```scad
spacing = 20;

for (i = [0 : 3]) {
    translate([i * spacing, 0, 0])
        cylinder(h = 8, r = 3);
}
```

Die dritte sichtbare Instanz ist auswählbar und verweist auf den Zylinderaufruf sowie den Schleifendurchlauf `i = 2`. Eine Änderung des gemeinsamen Radius betrifft alle vier Zylinder und wird so angekündigt. Eine Änderung von `spacing` betrifft die gesamte Anordnung.

Möchte der Nutzer nur eine Instanz verschieben, wird dies nicht als Änderung von `spacing` getarnt. Ein lokaler Sonderfall darf beispielsweise über einen expliziten bedingten Ausdruck erzeugt werden, sofern die unterstützte Transformation ihn sicher einfügen kann. Andernfalls führt die UI zur strukturierten Formel-/Bedingungsbearbeitung.

### 16.3 Parametrisches Ausrichten

```scad
body_width = 40;
gap = 2;

cube([body_width, 20, 10]);
translate([body_width + gap, 0, 0])
    cube([8, 20, 10]);
```

Eine ausdrücklich parametrische Ausrichtung verwendet `body_width + gap`. Eine spätere Änderung der Breite bewegt den zweiten Körper entsprechend. Die einmalige Variante dürfte stattdessen eine feste Position erzeugen, muss aber als einmalig erkennbar sein.

### 16.4 Zentrales Gehäuseprojekt

Das Abnahmeprojekt enthält einen Gehäusekörper und Deckel, gemeinsam verwendete Maße für Breite, Tiefe, Höhe und Wandstärke sowie Schraubdome und ein per Schleife erzeugtes Lochmuster. Body und Deckel sind über einen Parameter einzeln exportierbar.

1. Neues Projekt erzeugen; Grundkörper und Aushöhlung grafisch anlegen.
2. Benannte Maße und Beziehungen im Code formulieren; Wandstärke und Außenmaße über die UI ändern.
3. Schraubdom als eigenes Modul erstellen; Instanzen über eine strukturiert bearbeitete Schleife platzieren.
4. Lochmuster im Code ergänzen; eine erzeugte Bohrung im 3D-Fenster zum Ursprung zurückverfolgen.
5. Eine passende BOSL2-Komponente mit benannten Parametern einsetzen; Dokumentation offline aufrufen.
6. Customizer-Varianten für mindestens zwei Größen und für Body/Deckel anlegen und speichern.
7. Einen unvollständigen Ausdruck tippen; alte Vorschau erkennen und einen unabhängigen gültigen Bereich weiterhin bearbeiten.
8. Fehler beheben; eine Code-, eine UI- und eine MCP-Änderung jeweils rückgängig machen und wiederherstellen.
9. Body und Deckel in einer ausdrücklich gewählten Variante als STL beziehungsweise 3MF exportieren.
10. `.scad` und kompatible Parameterdatei mit denselben Bibliotheken in der unveränderten Referenz-Engine öffnen und vergleichen.

Die Prüfvorrichtung verwendet sowohl ein vollständig vorbereitetes Beispiel als auch eine ohne SCADStudio-Metadaten übernommene Fassung. Eine spezielle Annotationspflicht darf nicht die Voraussetzung für Auswahl oder Bearbeitung sein.

## 17. Tests und Abnahme

### 17.1 Prüfschichten

Unit-Tests prüfen UI-unabhängige Syntax-, Dokument-, Revisions- und Änderungslogik. Integrationstests unter `tests/` prüfen Engine, Bibliotheken, Persistenz und MCP. UI-Prüfungen ergänzen diese Tests um sichtbare Bedienbarkeit und Rendering. Tests dürfen keine reine Kopie der Implementierungsstruktur sein.

| Test-ID | Szenario und erwartetes Ergebnis | Anforderungsbezug |
| --- | --- | --- |
| AT-001 | Unbekannte Standard-`.scad` ohne Metadaten öffnen, auswerten und gezielt ändern. | COMP-004, EDIT-001, SEL-001 |
| AT-002 | Formparameter ändern; Kommentar, Leerraum, Zeilenenden und fremde Bereiche bleiben erhalten. | EDIT-002, PARAM-001 |
| AT-003 | Schleifen-, Modul- und `children()`-Instanzen unterscheiden; gemeinsame Wirkung sichtbar machen. | SEL-002, SEL-003, PARAM-004 |
| AT-004 | Alle Referenz-Syntaxkategorien strukturiert erzeugen und wieder bearbeiten; Ergebnis in Referenz auswerten. | PROD-003, LANG-001, LANG-003 |
| AT-005 | Gleiche sichere Änderung über UI und MCP führt zum selben Dokumentinhalt und Historieneintrag. | EDIT-004, MCP-002 |
| AT-006 | Veralteter Änderungsplan wird nach neuer Texteingabe ohne Teiländerung abgelehnt. | EDIT-006, MCP-006 |
| AT-007 | Ungültiger Code zeigt letzte gültige Vorschau; unabhängiger sicherer Bereich bleibt bearbeitbar. | REC-001, REC-002, REC-003 |
| AT-008 | Abgezogenen Körper auswählen und bearbeiten, ohne Modellsemantik durch Sichtbarkeit zu verändern. | SEL-004, CAD-006 |
| AT-009 | Alle fünf Bibliotheken ohne Netz durchsuchen, dokumentieren, aufrufen und berechnen. | LIB-001, LIB-003, LIB-005 |
| AT-010 | Bibliotheksprojektkopie bearbeiten, ohne den installierten Stand zu verändern. | LIB-006 |
| AT-011 | Varianten laden/speichern; Quellwert und Überschreibung unterscheiden; Referenzvergleich erfolgreich. | CUST-001, CUST-002, CUST-004 |
| AT-012 | Überholter Worker beendet sich spät; aktuelles Modell bleibt unverändert. | RUN-003 |
| AT-013 | Export während weiterer Eingabe benennt seine ursprüngliche Revision und schreibt keinen falschen Stand. | RUN-006 |
| AT-014 | Abbruch, Engine-Absturz und Schreibfehler hinterlassen keine als erfolgreich gemeldete Teildatei. | RUN-007, RUN-008, FILE-007 |
| AT-015 | Unicode, CRLF, verschachtelte Includes und relative Ressourcen korrekt zuordnen. | EDIT-002, COMP-007 |
| AT-016 | Externe Dateiänderung und ungespeicherte Arbeit führen zu einer Konfliktansicht ohne Verlust. | REC-004, REC-005 |
| AT-017 | Beschädigte Metadaten und neuere Schemaversion nicht überschreiben; Quelldatei separat öffnen. | FILE-006 |
| AT-018 | Fenstersitzung und fensterlose MCP-Sitzung modellieren, Bild erfassen, speichern und exportieren. | MCP-001, MCP-003, MCP-008 |
| AT-019 | Freigabe widerrufen; neue Mutationen und ausstehende unautorisierte Übernahmen scheitern. | MCP-005 |
| AT-020 | Pfadtraversierung und Symlink außerhalb der Freigaben werden ohne Dateiänderung abgelehnt. | MCP-004, NFR-007 |
| AT-021 | Gehäuseablauf vollständig per wechselnder Code-/UI-/MCP-Bearbeitung ausführen. | PROD-001, CUST-005, RUN-006 |
| AT-022 | Tastatur, DE/EN, kleine Fenster und HiDPI ohne blockierende oder abgeschnittene Kernbedienung. | UI-005, UI-007, UI-009 |
| AT-023 | Leistungskorpus nach festgelegtem Protokoll messen; Ergebnisse und Einschränkungen protokollieren. | PERF-001, PERF-002, PERF-003, PERF-004, PERF-005, PERF-006 |
| AT-024 | IDE-Funktionen an einem Projekt mit mehreren Dateien und Bibliotheken prüfen; unsichere Refactorings sperren, sichere Änderungen in der Referenz auswerten. | IDE-001 bis IDE-010 |
| AT-025 | Primitive und Modulaufrufe einfügen; Transformationen, Maße, boolesche Reihenfolge, 2D, Messen, Einrasten und beide Ausrichtungsarten prüfen; überlagerte Auswahl auflösen. | CAD-001 bis CAD-008, SEL-005, SEL-006, ALIGN-001 bis ALIGN-006 |
| AT-026 | Layout ändern/zurücksetzen, Kamera und Anzeigezustände bedienen; Theme-Reihenfolge System → Dark → Light, Standard System, automatische Systemwechsel, Dark-Fallback bei fehlender Vorgabe, manuellen Vorrang und Speicherung über Neustarts prüfen; im Leerlauf und minimiert keine unnötigen Dauerrepaints beobachten. | UI-001 bis UI-009, FILE-005, NFR-005, NFR-008 |
| AT-027 | Warteschlangen bis zum Limit füllen, Abbruch und Worker-Neustart prüfen; UI bleibt bedienbar, kein Auftrag verschwindet ohne Status. | RUN-004, MCP-007, NFR-001, NFR-002 |
| AT-028 | Release-Komponentenliste gegen tatsächliche Artefakte, Lizenztexte, Kopplungsbewertung und passende Quellbereitstellung prüfen. | LIC-001 bis LIC-006 |
| AT-029 | Absturz mit ungespeicherten Dateien, beschädigter Wiederherstellung und Schreibfehlern simulieren; Originale bleiben erhalten, lokale Metadaten gelangen nicht in die Projektdatei. | FILE-003 bis FILE-008, NFR-003, NFR-004 |
| AT-030 | Lokale Bibliothek, Suchpfadkonflikt und ausdrücklich ausgelöstes Update prüfen; bestehende Sperreinträge bleiben unverändert und aufgelöste Dateien sind sichtbar. | COMP-006, LIB-007, LIB-008, LIB-009 |
| AT-031 | Frisches Linux-System ohne separates OpenSCAD und mit gesperrtem Netzwerk starten; Kern, Katalog, Dokumentation und MCP vollständig nutzen. | PROD-004, COMP-002, COMP-003, NFR-006, NFR-007 |
| AT-032 | Vollständiges Sprach-/Geometriekorpus gegen den unveränderten Referenz-Build vergleichen; Ressourcen und Backend protokollieren. | COMP-001 bis COMP-007, PROD-005 |
| AT-033 | Formeländerung mit mehreren Zielen prüfen; Konstantenersatz nur ausdrücklich; Griffziehen abbrechen/abschließen und automatische Vorschau nach Eingabepause beobachten. | PARAM-001 bis PARAM-005, EDIT-005, RUN-001, RUN-002, RUN-005 |
| AT-034 | Mehrdatei-Änderung mit ungültigem Teil ablehnen; gültige Änderung atomar übernehmen und teilweise fehlschlagendes Speichern ohne falsche Gesamt-Erfolgsmeldung behandeln. | EDIT-007, EDIT-008, FILE-007 |

Die folgende Abnahmematrix ordnet alle Anforderungen den verantwortlichen Szenarien zu. Bereiche sind einschließlich beider Grenzen gemeint. Bei Implementierung werden die Gruppen in einzelne Anforderungs-/Testzuordnungen überführt; Tests und Prüfergebnisse nennen die konkreten IDs. Szenarien mit sichtbarer Bedienung oder Lizenzbewertung enthalten ausdrücklich manuelle Prüfschritte. Für Fehlerkorrekturen werden Regressionstests ergänzt.

| Anforderungsgruppe | Verantwortliche Abnahmeszenarien |
| --- | --- |
| PROD-001 bis PROD-006 | AT-004, AT-018, AT-021, AT-025, AT-031, AT-032 |
| COMP-001 bis COMP-007 | AT-001, AT-015, AT-030, AT-031, AT-032 |
| EDIT-001 bis EDIT-008 | AT-002, AT-005, AT-006, AT-015, AT-021, AT-033, AT-034 |
| PARAM-001 bis PARAM-005 | AT-002, AT-003, AT-033 |
| REC-001 bis REC-005 | AT-007, AT-016 |
| LANG-001 bis LANG-007 | AT-004 |
| CAD-001 bis CAD-008 | AT-025 |
| SEL-001 bis SEL-006 | AT-001, AT-003, AT-008, AT-025 |
| ALIGN-001 bis ALIGN-006 | AT-025 |
| IDE-001 bis IDE-010 | AT-024 |
| UI-001 bis UI-009 | AT-022, AT-026 |
| LIB-001 bis LIB-009 | AT-009, AT-010, AT-030 |
| FILE-001 bis FILE-008 | AT-001, AT-014, AT-015, AT-017, AT-029, AT-034 |
| CUST-001 bis CUST-005 | AT-011, AT-021 |
| RUN-001 bis RUN-008 | AT-012, AT-013, AT-014, AT-027, AT-033 |
| MCP-001 bis MCP-008 | AT-005, AT-006, AT-018, AT-019, AT-020, AT-027 |
| PERF-001 bis PERF-006 | AT-023 |
| NFR-001 bis NFR-008 | AT-014, AT-022, AT-023, AT-026, AT-027, AT-029, AT-031 |
| LIC-001 bis LIC-006 | AT-028 |

### 17.2 Referenzkorpus und geometrischer Vergleich

Das Korpus enthält mindestens:

- Primitive und alle Sprachkategorien mit gültigen und ungültigen Eingaben.
- Verschachtelte Modulaufrufe, Schleifen, Bedingungen und Kindgeometrie.
- Ausgewählte öffentlich nutzbare Beispiele jeder mitgelieferten Bibliothek.
- 2D, Extrusion, Projektion, Text, relative Importe und mehrere Dateien.
- Das Gehäuseprojekt sowie ein Detailteil mit Gewinde und ein Muster mit 500 Instanzen.

Vergleiche prüfen Erfolg/Fehlerkategorie, Quellzuordnung soweit verfügbar, Begrenzungen, Volumen, Komponenten und gültige Netze entsprechend dem Referenzverhalten. Binäridentische Dateien und Dreiecksreihenfolgen sind keine allgemeine Voraussetzung.

Bei identischem Backend und identischen Parametern gilt als anfängliche numerische Prüftoleranz für Längen `max(1e-6, 1e-6 * Modellgröße)` und für Volumen `max(1e-9, 1e-6 * Referenzvolumen)` in den jeweiligen Modelleinheiten. Abweichungen benötigen Untersuchung; die Toleranz darf nicht pauschal vergrößert werden, um Unterschiede zu verdecken. Für ungültige oder volumenslose Ergebnisse werden passende Eigenschaften statt eines unbrauchbaren Volumenvergleichs gewählt.

### 17.3 Build- und Auslieferungsprüfungen

Nach Aufbau des Rust-Projekts sind erforderlich:

```text
cargo run
cargo build --release
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

`cargo run` dient der manuellen Sichtprüfung; die übrigen Befehle sind automatisierbare Prüfungen. README und CI beschreiben Toolchain, Systemabhängigkeiten, Engine-Build und Testdaten. GPU-/UI-Prüfungen werden auf einem geeigneten System ausgeführt und nicht durch bestandene reine Unit-Tests ersetzt.

Auslieferung setzt außerdem einen Offline-Test in einer frischen Umgebung ohne separat installiertes OpenSCAD, Lizenz-/Quellcodebeilagen und einen erfolgreichen Referenzvergleich voraus. Nicht ausgeführte Prüfungen werden als solche benannt.

## 18. Umsetzungsreihenfolge

### M0 – Architektur und Integrationsnachweis

**Ergebnis:** Reproduzierbare Referenz, isolierter Worker und belegte Herkunftszuordnung für schwierige Fälle.

- OpenSCAD-Revision, Backend und Buildoptionen sperren; originale Referenz unverändert als Vergleich verfügbar halten.
- Geometrie sowie Herkunft für Schleifen, verschachtelte Module, `children()` und einen BOSL2-Aufruf extrahieren.
- Ergebnis-/Ursprungsauswahl und Auswahl abgezogener Operanden als kleinen technischen Durchstich demonstrieren.
- Eine gezielte Parameteränderung einschließlich Kommentarerhalt und Revisionstest umsetzen.
- Ungespeicherte Includes und Importressourcen im Worker korrekt auflösen.
- Fensterlosen Bildpfad und erforderliche Systemkomponenten nachweisen.
- Lizenzgrenzen der konkreten Worker-Anbindung prüfen und dokumentieren.

**Abnahme:** AT-001 bis AT-003 als begrenzte Durchstiche, Herkunftsprüfung aus AT-008, Ressourcenfall aus AT-015 und Bildnachweis aus AT-018. Kein positiver Meilensteinbericht, wenn Herkunft lediglich aus dem Gesamtnetz geraten wird.

Falls der vollständige Herkunftsvertrag mit vertretbarer Erweiterung nicht erfüllt werden kann, wird die betroffene Architekturentscheidung mit konkretem Nachweis überarbeitet. Die Anforderung wird nicht stillschweigend auf vorbereitete Dateien oder einfache Primitive reduziert. Lizenzrelevante Änderungen der Veröffentlichungsbedingungen erfordern eine ausdrückliche Produktentscheidung.

### M1 – Dokumentkern und gemeinsame Befehle

**Ergebnis:** Revisionierte Quelldokumente, fehlertolerante Syntaxstruktur, sichere Änderungen, Undo/Redo, Speicher- und Konfliktbehandlung.

**Abnahme:** AT-002, AT-006, AT-007, AT-015 bis AT-017. Der Dokumentkern ist ohne egui testbar. Die Änderungsverträge stehen vor umfangreicher UI-Entwicklung fest.

### M2 – Nutzbarer 3D-Arbeitsablauf

**Ergebnis:** Andockbare Arbeitsfläche mit Editor, Vorschau, Auswahl, Primitiven, Transformationen, Maßgriffen und booleschen Operationen. Messen, Einrasten und Ausrichten sind enthalten.

**Abnahme:** Ein einfaches Gehäuse wird wechselnd in Code und UI erzeugt. AT-003, AT-008, AT-012 bis AT-014 und erste UI-/Leistungsmessungen bestehen. Manipulationsvorschau und finale Geometrie sind unterscheidbar.

### M3 – Vollständige Sprache, IDE und Bibliotheken

**Ergebnis:** Alle Referenz-Sprachkategorien grafisch bearbeitbar, vollständige allgemeine Aufrufformulare, Sprachwerkzeuge, fünf Offline-Bibliotheken und Customizer.

**Abnahme:** AT-004, AT-009 bis AT-011 sowie vollständige Sprach-/Bibliotheksmatrix. Statische Unsicherheit ist sichtbar, Refactorings sind gesichert.

### M4 – Automatisierung und vollständige Linux-Auslieferung

**Ergebnis:** MCP für GUI und fensterlose Sitzungen, komplette Projekt-/Exportabläufe, Offline-Auslieferung und Wiederherstellung.

**Abnahme:** AT-005, AT-018 bis AT-023 und sämtliche übrigen verbindlichen Anforderungen. Zentrales Gehäuseprojekt und vollständige Build-/Auslieferungsprüfung bestehen.

### M5 – Windows und macOS

**Ergebnis:** Plattformgerechte Pakete derselben Architektur mit Engine, Bibliotheken und Dokumentation.

**Abnahme:** Dasselbe fachliche Korpus, native UI-Prüfungen, HiDPI, Pfade/Dateirechte und fensterloser Betrieb je Plattform. Eine erfolgreiche Linux-Abnahme ersetzt diese Nachweise nicht.

## 19. Lizenzen, Auslieferung und Nachweispflichten

Die vorhandene `LICENSE` bleibt die MIT-Lizenz des eigenständig entwickelten SCADStudio-Codes. Das ist keine pauschale Umlizenzierung sämtlicher mitgelieferter Komponenten.

| ID | Verbindliche Anforderung |
| --- | --- |
| LIC-001 | Übernommener Fremdcode MUSS mit Herkunft und zugehörigen Lizenzbedingungen erfasst werden. |
| LIC-002 | OpenSCAD und etwaige darauf basierende Worker-Anpassungen MÜSSEN unter den zutreffenden ursprünglichen Bedingungen ausgeliefert werden. |
| LIC-003 | Jede mitgelieferte Bibliothek, Dokumentation, Schrift und Ressource MUSS vor Bündelung auf ihre konkreten Lizenzbedingungen geprüft werden. |
| LIC-004 | Release-Artefakte MÜSSEN erforderliche Copyright-/Lizenzhinweise und passende Quellcodebereitstellung für die tatsächlich ausgelieferten Versionen enthalten. |
| LIC-005 | Eine enge Engine-Anbindung darf nicht allein wegen getrennter Prozesse als abschließend lizenzrechtlich geklärt bezeichnet werden. |
| LIC-006 | Veröffentlichungen MÜSSEN Eigenlizenz und Fremdkomponenten klar unterscheiden; das Gesamtpaket darf nicht ausschließlich als MIT gekennzeichnet werden. |

OpenSCAD veröffentlicht seine Lizenz im eigenen Repository [Q7]. Die GNU-Erläuterungen unterscheiden eigenständige Programme und kombinierte Werke anhand technischer und inhaltlicher Kopplung [Q8]. Daraus wird hier keine allgemeine rechtliche Garantie für den geplanten Herkunftsdatenkanal abgeleitet.

Der Architekturnachweis dokumentiert die konkrete Kopplung, übernommenen Dateien, Buildgrenzen und erforderlichen Auslieferungspflichten. Wenn diese Konstruktion mit der gewünschten Lizenzaufteilung nicht vereinbar ist, muss die Kopplung überarbeitet oder die Veröffentlichungsentscheidung ausdrücklich angepasst werden. Eine Umbenennung von Komponenten genügt nicht.

Die Spezifikation sieht zunächst ein eigenständiges Linux-Paket einschließlich Engine, Bibliotheken und Dokumentation vor. Das konkrete Paketformat ist eine Auslieferungsentscheidung in M4; es darf keine Online-Nachinstallation wesentlicher Modellierfunktionen verlangen. Windows und macOS erhalten später entsprechende native Pakete.

## 20. Entscheidungen, Annahmen und Quellen

### 20.1 Festgelegte Entscheidungen

| Thema | Entscheidung |
| --- | --- |
| Maßgebliche Modellbeschreibung | Gewöhnlicher OpenSCAD-Quellcode. |
| Grafischer Sprachumfang | Vollständig über Baum, Formulare und Ausdruckseditor; passende Funktionen zusätzlich direkt in 3D. |
| Engine | Originale OpenSCAD-Auswertung, fest versioniert und mitgeliefert. |
| Modellierungsstil | Direkte 3D-Konstruktion; 2D bleibt vollständig verfügbar. |
| Mehrdeutige Änderung | Ziele und Auswirkungen zeigen, ausdrücklich auswählen lassen. |
| Parametrische Ausdrücke | Beziehungen erhalten; Konstantenersatz nur ausdrücklich. |
| Bibliotheksquellen | Mitgelieferte Quellen schreibgeschützt, Projektkopien bearbeitbar. |
| UI | Flexibles IDE-Layout, DE und EN. |
| Theme | Auswahl System → Dark → Light; Standard System, Dark als Fallback bei fehlender Systemvorgabe; manuelle Auswahl benutzerbezogen gespeichert. |
| Technische Bezeichner | Englisch für Implementierung und automatisch neu angelegte Namen; vorhandene Namen erhalten. |
| Dateiaustausch | `.scad` und Referenz-Engine-Formate; optionale Projektmetadaten. |
| Varianten | OpenSCAD-kompatibler Customizer und Parameterdateien. |
| Vorschau | Automatisch, abgestuft, revisionsgebunden. |
| MCP | Vollständiger Ablauf, GUI und fensterlos, direkte Änderungen nach Sitzungsfreigabe. |
| Zentrales Beispiel | Parametrisches 3D-Druck-Gehäuse. |
| Umsetzung | Vollständiges Zielbild mit abnehmbaren Schritten; Linux zuerst. |

### 20.2 Gesetzte Standardwerte und noch zu erbringende Nachweise

Die im Interview nicht einzeln festgelegten Standardwerte sind hier bewusst spezifiziert: 300 ms Vorschauverzögerung, begrenzte Auftragszahlen, JSON-Projektmetadaten, lokale stdio-/IPC-Anbindung, zwei Sekunden Worker-Abbruchfrist und die Leistungsbudgets in Kapitel 15. Sie sind keine nachträglich behaupteten Benutzerwünsche und können mit dokumentierter Änderung dieses Dokuments angepasst werden.

Folgende technische Nachweise sind offen, aber mit Verantwortungszeitpunkt und Prüfkriterium eingeplant:

| Nachweis | Zeitpunkt | Erforderliches Ergebnis |
| --- | --- | --- |
| Vollständige Instanzherkunft | M0 | Unterscheidbare erzeugende Pfade für die definierten schwierigen Fälle; keine Beschränkung auf annotierte Dateien. |
| Geeignete Referenz und Bibliotheksstände | M0, Bibliotheksabschluss M3 | Vollständige Kennungen, reproduzierbarer Build und bestandenes Kompatibilitätskorpus. |
| Lizenzfolgen des Herkunftskanals | M0, abschließend vor Auslieferung | Dokumentierte konkrete Kopplung und erfüllbare Lizenz-/Quellcodepflichten. |
| Vollständige grafische Sprachabdeckung | M3 | Jede Fähigkeit der Referenz einer bedienbaren Strukturaktion zugeordnet. |
| Fensterlose Bildfunktion | M0 und M4 | Erfolgreiche Bildaufnahme ohne sichtbaren Displayserver in der ausgelieferten Umgebung. |
| Reaktions- und Berechnungsbudgets | M2 bis M4 | Gemessene Ergebnisse unter dokumentierten Bedingungen. |

Diese Liste ist kein Freibrief, verbindliche Funktionen später entfallen zu lassen. Ein gescheiterter Nachweis führt zu einer begründeten Architektur- oder Spezifikationsänderung vor darauf aufbauender Umsetzung.

### 20.3 Primärquellen

Die Quellen wurden für die Spezifikation am 31. Oktober 2026 herangezogen. Verweise auf `master` oder `latest` erläutern den heutigen Ansatz und sind keine reproduzierbaren Versionspins. Bei Implementierung sind die tatsächlich verwendeten Stände zusätzlich unveränderlich festzuhalten.

- **Q1:** [Offizielle OpenSCAD-Dokumentation und Verweise auf Sprachreferenz, Customizer und Dateiverarbeitung](https://openscad.org/documentation.html).
- **Q2:** [Offizieller OpenSCAD-Bibliothekskatalog](https://openscad.org/libraries.html).
- **Q3:** [BOSL2-Projekt und Kompatibilitätshinweise](https://github.com/BelfrySCAD/BOSL2).
- **Q4:** [Offizielle eframe-API-Dokumentation](https://docs.rs/eframe/latest/eframe/).
- **Q5:** [OpenSCAD-Quellcode: Knoten und Quellzuordnung](https://github.com/openscad/openscad/blob/master/src/core/node.h).
- **Q6:** [MCP-Spezifikation: Transports, Version 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).
- **Q7:** [OpenSCAD-Lizenzdatei](https://github.com/openscad/openscad/blob/master/COPYING).
- **Q8:** [GNU GPL FAQ: Abgrenzung zusammen ausgelieferter und kombinierter Programme](https://www.gnu.org/licenses/gpl-faq.en.html#MereAggregation).
