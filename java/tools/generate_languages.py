"""English and German UI resources; Minecraft supplies other vanilla text and widgets."""
from pathlib import Path
import json
PAIRS={
'import_json':('Import JSON','JSON importieren'),'import_nbt':('Import servers.dat','servers.dat importieren'),'export_json':('Export JSON','JSON exportieren'),'export_nbt':('Export servers.dat','servers.dat exportieren'),'file_path':('File path','Dateipfad'),'transfer_note':('Import merges entries. Existing server data is preserved.','Import ergänzt Einträge. Vorhandene Serverdaten bleiben erhalten.'),'import_done':('Import complete.','Import abgeschlossen.'),'export_done':('Export complete.','Export abgeschlossen.'),'overwrite':('Replace this exported file?','Diese Exportdatei ersetzen?'),
'feedback_icon':('✉','✉'),'tags_short':('Tags','Tags'),'move_up_short':('↑','↑'),'move_down_short':('↓','↓'),'info':('Info','Info'),'reset':('Reset','Zurücksetzen'),'undo_short':('Undo','Rückgängig'),
'server':('Server','Server'),'details':('Details','Details'),'status':('Status','Status'),'values':('Values','Werte'),'tags_region':('Tags / Region','Tags / Region'),'resource_packs':('Server resource packs','Server-Ressourcenpakete'),
'compatible_short':('Compatible','Kompatibel'),'incompatible':('Different version','Andere Version'),'ping_history':('Ping history (local)','Ping-Verlauf (lokal)'),
'settings':('Settings','Einstellungen'),'classic':('Classic','Klassisch'),'search':('Search names, addresses, tags, notes…','Name, Adresse, Tags, Notizen suchen…'),'clear':('Clear','Leeren'),
'filters':('Filters','Filter'),'sort':('Sort','Sortieren'),'profiles':('Profiles','Profile'),'refresh':('Refresh','Aktualisieren'),'join':('Join server','Beitreten'),'direct':('Direct connect','Direkt verbinden'),
'add':('Add server','Hinzufügen'),'edit':('Edit','Bearbeiten'),'delete':('Delete','Entfernen'),'favorite':('Favorite','Favorit'),'more':('More…','Mehr…'),'back':('Back','Zurück'),
'save':('Save','Speichern'),'cancel':('Cancel','Abbrechen'),'name':('Server name','Servername'),'address':('Server address','Serveradresse'),'tags':('Tags (comma separated)','Tags (durch Komma getrennt)'),
'notes':('Personal notes','Persönliche Notizen'),'group':('Group','Gruppe'),'language':('Language','Sprache'),'region':('Region','Region'),'auth':('Authentication (your own label)','Anmeldung (eigene Angabe)'),
'count':('%s of %s servers','%s von %s Servern'),'players_value':('Players: %s / %s','Spieler: %s / %s'),
'online':('Server status','Serverstatus'),'compatible':('Compatible version only','Nur kompatible Version'),'version':('Exact server version (optional)','Genaue Serverversion (optional)'),
'min_players':('Minimum players (optional)','Spieler mindestens (optional)'),'max_players':('Maximum players (optional)','Spieler höchstens (optional)'),
'max_ping':('Maximum ping in ms (optional)','Ping höchstens in ms (optional)'),'not_empty':('Not empty','Nicht leer'),'not_full':('Free slots','Freie Plätze'),'favorites_only':('Favorites only','Nur Favoriten'),
'enabled':('Open browser with Multiplayer','Browser unter Mehrspieler öffnen'),'online_directory':('Online discovery','Online-Verzeichnis'),'show_offline':('Show offline servers','Offline-Server anzeigen'),
'refresh_seconds':('Status refresh seconds (0 = manual)','Status-Intervall in Sekunden (0 = manuell)'),'concurrent_pings':('Concurrent status checks (1–16)','Gleichzeitige Statusabfragen (1–16)'),
'loading':('Loading public directory…','Öffentliches Verzeichnis lädt…'),'partial':('Cached list · some sources unavailable','Zwischengespeichert · Quelle teils nicht erreichbar'),
'sources':('Online · minecraftserve.rs + Modrinth','Online · minecraftserve.rs + Modrinth'),'directory_off':('Online discovery is off','Online-Verzeichnis ist aus'),
'local_error':('Local data could not be read or saved. Original retained.','Lokale Daten konnten nicht gelesen/gespeichert werden. Original erhalten.'),
'select_server':('Select a server for details.','Wähle einen Server für Details.'),'empty':('No matching servers. Try Discover or add your first server.','Keine passenden Server. Öffne Entdecken oder füge einen Server hinzu.'),
'lan_wait':('Searching for games on your local network…','Suche Spiele im lokalen Netzwerk…'),'lan_error':('LAN discovery could not start.','LAN-Suche konnte nicht gestartet werden.'),
'save_failed':('Could not save. Original files have been retained.','Speichern fehlgeschlagen. Originaldateien bleiben erhalten.'),'invalid_value':('Please check the address and field values.','Bitte prüfe die Adresse und die Eingaben.'),
'local_note':('Saved only in this game profile.','Wird nur in diesem Spielprofil gespeichert.'),'and_filters':('All filters apply together. Empty fields impose no limit.','Alle Filter gelten gemeinsam. Leere Felder begrenzen nicht.'),
'unknown_last':('Unknown values stay last in either direction.','Unbekannte Werte bleiben in beiden Richtungen zuletzt.'),
'directory_privacy':('Public APIs receive no search text or personal server list.','Öffentliche APIs erhalten keine Suchtexte oder eigene Serverlisten.'),
'delete_confirm':('Remove “%s” from your saved list? Undo is available under More.','„%s“ aus der eigenen Liste entfernen? Rückgängig unter Mehr verfügbar.'),
'save_profile':('Save current filter profile','Aktuelle Filter als Profil speichern'),'profile_name':('Profile name','Profilname'),'load_profile':('Apply profile','Profil anwenden'),'delete_profile':('Delete profile','Profil löschen'),
'reset_filters':('Reset search and filters','Suche und Filter zurücksetzen'),'move_up':('Move saved server up','Gespeicherten Server nach oben'),'move_down':('Move saved server down','Gespeicherten Server nach unten'),
'undo':('Undo last server-list edit','Letzte Listenänderung rückgängig'),'copy_address':('Copy server address','Serveradresse kopieren'),'copied':('Server address copied.','Serveradresse kopiert.'),
'feedback':('Feedback / propose a server','Feedback / Server vorschlagen'),'website':('Project website','Projekt-Website'),'category':('Category','Kategorie'),'message':('Message','Nachricht'),
'mail_unconfigured':('Local draft only. Automatic email delivery is not configured.','Nur lokaler Entwurf. Automatischer Mailversand ist nicht eingerichtet.'),'draft_saved':('Draft saved locally. No email was sent.','Entwurf lokal gespeichert. Keine E-Mail versendet.'),
}
for key,en,de in [('saved','My servers','Meine Server'),('discover','Discover','Entdecken'),('favorites','Favorites','Favoriten'),('recent','Recent','Zuletzt'),('lan','LAN','LAN')]: PAIRS['tab.'+key]=(en,de)
for key,en,de in [('custom','Custom order','Eigene Reihenfolge'),('name','Name','Name'),('players','Players','Spieler'),('free_slots','Free slots','Freie Plätze'),('ping','Ping','Ping'),('version','Version','Version'),('last_played','Last played','Zuletzt gespielt'),('date_added','Date added','Hinzugefügt'),('favorites','Favorites first','Favoriten zuerst')]: PAIRS['sort.'+key]=(en,de)
for key,en,de in [('any','Any','Alle'),('unknown','Unknown','Unbekannt'),('online','Online','Online'),('offline','Offline','Offline'),('on','On','An'),('off','Off','Aus'),('ask','Ask','Nachfragen'),('feedback','Feedback','Feedback'),('server','Public server proposal','Öffentlicher Servervorschlag')]: PAIRS['value.'+key]=(en,de)
for key,en,de in [('unknown','Not checked','Nicht geprüft'),('pinging','Pinging…','Prüfe…'),('online','Online','Online'),('offline','Offline','Offline')]: PAIRS['health.'+key]=(en,de)
ROOT=Path(__file__).resolve().parents[1]/'common/src/main/resources/assets/server_browser/lang'
ROOT.mkdir(parents=True,exist_ok=True)
for i,lang in enumerate(('en_us','de_de')):
    (ROOT/f'{lang}.json').write_text(json.dumps({'server_browser.'+key:pair[i] for key,pair in PAIRS.items()},ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(f'{len(PAIRS)} strings in each of two languages')
