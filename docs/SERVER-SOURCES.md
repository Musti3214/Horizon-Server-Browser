# Serverquellen, Stand 3. November 2026

## Ergänzung aus der Serverify-Recherche

[Serverify](https://serverify.louchat.site/) ist eine eigenständige Mod. Ihre
[öffentliche Horizon-Anleitung](https://serverify.louchat.site/docs/horizon)
nennt Modrinth und mehrere weitere Verzeichnis-Websites als Quellen. Es wurde
nur diese Nutzerdokumentation als Quellenhinweis verwendet, kein fremder
Mod-Quellcode und keine fremden Mod-Bilder.

Die öffentliche Modrinth-API wurde am 3. November 2026 direkt abgefragt:
`GET https://api.modrinth.com/v3/search`, Filter
`[["project_types:minecraft_java_server"]]`, 50 Ergebnisse pro Seite. 49 Seiten
enthielten 2.412 Einträge. Nach Java-Prüfung, Host/Port-Normalisierung,
Ausschluss lokaler Adressen und Abgleich mit unserem bisherigen Katalog bleiben
**2.347 neue eindeutige Adressen**. Der gebündelte Katalog wächst damit von
488 auf **2.835 eindeutige Adressen**. Erreichbarkeit oder Kompatibilität wurde
daraus nicht abgeleitet. Es fand kein Portscan und keine Serveranmeldung statt.

Die neue Rust-Anbindung nutzt genau diese öffentliche Quelle mit eigenem
Projekt-User-Agent. JSON wird auf 8 MiB und 50 Einträge pro Seite begrenzt.
Nur Adresse, Name, Kategorien, Sprache und Region werden übernommen. Die
Quell-Statuswerte gelten nicht als eigene aktuelle Ping- oder Spielerwerte.
Modpack-Server werden markiert; Modpacks und externe Bilder werden nicht geladen.
Privat gespeicherte Listen und Suchtexte werden nicht an Modrinth geschickt.

Jede Quelle hat einen eigenen persistenten Minutenabstand und Fünf-Minuten-Cache.
Ein fehlgeschlagener Abruf einer Quelle lässt die andere nutzbar; ein partielles
Update behält vorherige gute Einträge. Duplikate werden nach Host und Port
entfernt. Ist eine Quelle ausgeschöpft, werden keine weiteren Seiten außerhalb
ihrer gemeldeten Seitengrenze angefordert. Neue erste Seiten erneuern die Grenze.

Das Entwicklerwerkzeug `tools/update_modrinth_snapshot.py` kann den Datenstand
erneuern. Der vollständige lokale Abrufbericht liegt unter
`horizon-server-browser/.local/native-browser/modrinth-20261103/snapshot.json`.
Der Bericht enthält Quellenzeitpunkt, abgefragte Seiten und einzelne öffentliche
Quellverweise. Die bisherigen Quellen und ihre Einschränkungen stehen unten.

## Vorheriger Abgleich vom 2. November

Die Quellen wurden im Chrome-Browser geprüft. Es gibt keine vollständige,
verlässliche weltweite Serverliste: private, neue und nicht eingetragene Server
fehlen in öffentlichen Verzeichnissen.

| Quelle | Beobachtung | Verwendung |
| --- | --- | --- |
| [minecraftserve.rs API](https://minecraftserve.rs/api-docs) | Öffentliche Java-API ohne Schlüssel, 50 Einträge pro Seite. Sieben Seiten meldeten insgesamt 341 Server. 322 Adressen fehlten nach Host/Port-Abgleich im bisherigen Katalog. | Automatische Live-Abfrage, seitenweise beim Scrollen. Alle sieben Seiten wurden mit mindestens einer Minute Abstand abgerufen. Zusätzliche Adressen sind als Ausfallreserve enthalten. |
| [MinecraftBuzz](https://minecraft.buzz/) | Die Startseite nennt über 9.400 Server. Auf den geprüften Seiten wurde keine dokumentierte öffentliche API für den Gesamtkatalog gefunden. | Recherche und manuelle Quellenprüfung; kein unkontrolliertes Scraping. |
| [Minecraft-MP](https://minecraft-mp.com/servers/latest/) | 9.093 aktuell erfasste Server auf der geprüften Liste. Historische Gesamtzahlen sind keine Zahl aktuell erreichbarer Server. | Zusätzliche öffentliche Recherchequelle. |
| [Minecraft-MP API](https://minecraft-mp.com/help/api/) | Dokumentierte Detail- und Vote-Endpunkte brauchen den Schlüssel des jeweiligen Serverbesitzers. | Kein geeigneter freier Export des vollständigen Verzeichnisses. |

## Aktualität im Spiel

- Neue Einstellungen aktivieren das Online-Verzeichnis und erneuern direkte
  Statusabfragen alle 60 Sekunden. Ein ausdrücklich gespeichertes Aus bleibt Aus.
- Verzeichnisseiten werden fünf Minuten gecacht. Nach fünf Minuten ohne neue
  Seite erneuert Discover die erste Seite; Scrollen lädt weitere Seiten.
- Ein persistiertes Limit erlaubt höchstens eine öffentliche API-Anfrage je
  Minute. HTTP 429 und Retry-After werden berücksichtigt; Fehler führen zu einer
  Wartezeit. Filterwechsel und Neustarts umgehen das Limit nicht.
- Fehlgeschlagene Aktualisierungen behalten die zuletzt geladene Liste. Bei
  einem Kaltstart ohne erreichbare API stehen die eingebetteten Adressen bereit.
  Die Ausfallreserve enthält keine vermeintlich aktuellen Spielerzahlen.
- Adressen werden nach normalisiertem Host und Port zusammengeführt.
  Erreichbarkeit, Ping und kompatible Version prüft der vorhandene begrenzte
  Statusdienst; unbekannte Informationen bleiben unbekannt.
- Private gespeicherte Listen, Notizen und Suchtexte werden nicht an den
  Verzeichnisdienst geschickt. Öffentliche Tag- und Versionsfilter werden an
  dessen API übertragen. Externe Icon-URLs werden nicht nachgeladen.

Die zusätzliche Adressreserve kann mit `tools/update_directory_snapshot.py`
erneuert werden. Das Entwicklerwerkzeug respektiert den Minutenabstand und
protokolliert Quelle, Zeitpunkt und Seitenzahlen; es führt keine Portscans aus.

## Automatisches Feedback

[Static Forms](https://www.staticforms.dev/docs/api-reference) unterstützt
JSON-POSTs mit einem öffentlichen Formularschlüssel. Dieser Schlüssel erlaubt
nur Einreichungen; er ist kein Verwaltungsschlüssel und kein Mailpasswort.
Der kostenlose Tarif nennt 250 Einreichungen je 30 Tage für das gesamte Formular.
Eine positive API-Antwort bestätigt den Empfang durch den Dienst, nicht die
tatsächliche Zustellung bei Outlook. Bei Kontingentüberschreitung kann der Dienst
Einreichungen annehmen und die Mail zurückhalten.

Die Mod sendet ausschließlich nach dem ausdrücklichen Senden-Klick:
Nachricht, Kategorie, Mod-Version und den festen Projektnamen. Keine privaten
Serverlisten, Konten oder Diagnosedaten werden angehängt. Die Empfängeradresse
erscheint nicht in der Spieloberfläche. Ein fehlgeschlagener oder unklarer
Versand wird nicht automatisch wiederholt, um Doppelzustellungen zu vermeiden.
Ein Entwurf bleibt lokal; ein lokales Speichern behauptet niemals einen Versand.

Die kostenlose Registrierung wurde mit Zustimmung des Nutzers vorbereitet und
abgesendet, aber Cloudflare verhinderte den Abschluss. Auch die manuelle
Bestätigung funktionierte beim Nutzer nicht. Es gibt noch keinen verifizierten
Formularschlüssel; echter Mailversand ist deshalb noch nicht aktiviert.
