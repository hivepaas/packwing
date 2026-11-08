# packwing in German. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = Darstellung: { $name }. Klicken für { $name2 }.

# Stop and say so
tdc8b782b = Anhalten und melden

# Replace it
t1e17fc0c = Ersetzen

# Skip it
tad9909cb = Überspringen

# Add
t9dc3aa14 = Hinzufügen

# The top of {name}
t369f9d4d = Oberste Ebene von { $name }

# Choose what to add
t48b969a8 = Auswählen, was hinzugefügt wird

# {count} item | {count} items
t534e422e =
    { $n ->
        [one] { $count } Element
       *[other] { $count } Elemente
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
        [one] { $count } Datei
       *[other] { $count } Dateien
    }

# Add to {name}
t0d53ac5e = Zu { $name } hinzufügen

# Goes in
t36e95c22 = Ziel

# If a file of that name is there
t34c69cec = Wenn es dort eine Datei mit dem Namen gibt

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = Dieses Archiv ist signiert. Mit weiterem Inhalt passt es nicht mehr zu seiner Signatur, bis es neu signiert wird.

# The new files are encrypted with the archive’s password.
t52b1d223 = Die neuen Dateien werden mit dem Passwort des Archivs verschlüsselt.

# Cancel
t35afca3b = Abbrechen

# Name
t0fe07306 = Name

# Folder
t06357afd = Ordner

# Its secrets could not be read: {why}
t2af0c8ab = Die Zugangsdaten konnten nicht gelesen werden: { $why }

# That folder is not on this computer.
ta4073fff = Dieser Ordner ist nicht auf diesem Computer.

# Its secrets could not be kept: {why}
ta213827f = Die Zugangsdaten konnten nicht gespeichert werden: { $why }

# Local folder
te075bcbc = Lokaler Ordner

# Trying it…
t214cdad5 = Wird ausprobiert…

# It answers.
t3bd5bc9f = Er antwortet.

# It did not answer: {why}
t0ac8fa44 = Er hat nicht geantwortet: { $why }

# Edit storage
t7bae68d0 = Speicherort bearbeiten

# Add a storage
t4fe8f806 = Speicherort hinzufügen

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = Zugangsdaten liegen im Schlüsselbund des Systems, nicht bei den Einstellungen.

# Its files stay where they are.
t4e4acf9a = Die Dateien bleiben, wo sie sind.

# Ask for this computer’s password to open it
t7a7d61e9 = Zum Öffnen nach dem Passwort dieses Computers fragen

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = Für einen entsperrt zurückgelassenen Computer: Was diesen Computer entsperrt, öffnet auch den Speicherort, und nach 5 Minuten ohne Nutzung wird erneut gefragt.

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing Free erlaubt einen Speicherort im Netzwerk. Mit einer Pro- oder Enterprise-Lizenz entfällt das: Einstellungen › Lizenz. Ordner auf diesem Computer sind nicht begrenzt.

# Test connection
te0f3b6b7 = Verbindung testen

# Enter a license key
ta24f28a2 = Lizenzschlüssel eingeben

# Save
t4d2d5d68 = Speichern

# Paste the checksum given with the file
t8333758c = Füge die Prüfsumme ein, die mit der Datei kam

# They could not be saved: {why}
tb45ae457 = Sie konnten nicht gespeichert werden: { $why }

# The file could not be read: {why}
t5daa8455 = Die Datei konnte nicht gelesen werden: { $why }

# Reading…
t6aad8485 = Wird gelesen…

# Copied
t8dc21305 = Kopiert

# Copy
t658f3664 = Kopieren

# weak
tedbc2ec9 = schwach

# There is no checksum in this.
t231f838a = Darin ist keine Prüfsumme.

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = Das ist keine Prüfsumme, die packwing bildet: Sie hat { $digits } Stellen.

# It matches the file’s {name} checksum.
t4f0350e4 = Sie passt zur { $name }-Prüfsumme der Datei.

# It is compared once the file is read.
tcb5c1745 = Verglichen wird, sobald die Datei gelesen ist.

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = Sie passt nicht zur { $name }-Prüfsumme der Datei: Das ist nicht die Datei, aus der sie gebildet wurde, oder die Datei hat sich unterwegs verändert.

# It may be a {name} checksum, which is not made yet.
t3d713e04 = Es könnte eine { $name }-Prüfsumme sein, die noch nicht gebildet ist.

# Make it
tcbefe458 = Bilden

# Compare with
tbddcfc14 = Vergleichen mit

# Reading… {size} of {size2}
t80036d1c = Wird gelesen… { $size } von { $size2 }

# Reading… {size}
t1569c388 = Wird gelesen… { $size }

# Checksums of {name}
te88f3ed9 = Prüfsummen von { $name }

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = Die gesamte Datei läuft über diesen Computer, und dein Anbieter berechnet womöglich das Gelesene.

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = Eine schwache Prüfsumme ({ $names }) erkennt eine beschädigte Datei, aber keine absichtlich veränderte, die passend gemacht werden kann.

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = Daneben gespeichert als { $files }, wo sha256sum -c und Ähnliche sie ohne packwing prüfen.

# Compare…
tec85be80 = Vergleichen…

# Saving…
tbd79b37d = Wird gespeichert…

# Save beside it
t7f4d85e3 = Daneben speichern

# Choose the checksums shown
teec532fe = Angezeigte Prüfsummen wählen

# Read it
t80fb8482 = Lesen

# Done
t8dd31791 = Fertig

# Nothing chosen yet
tcf390b0a = Noch nichts ausgewählt

# {join} and {more} more
t8399ef70 = { $join } und { $more } weitere

# Hide what goes in
t5a420780 = Hinzuzufügendes ausblenden

# Show what goes in
t9a20eb53 = Hinzuzufügendes anzeigen

# Take it out
t90a93ec7 = Entfernen

# Drop them to add them
t6a4aedbe = Zum Hinzufügen hier ablegen

# What goes in
t3b7c797a = Wird hinzugefügt

# + Files
tba8a1e71 = + Dateien

# + Folder
tcef8b0ce = + Ordner

# Add the files or a folder to put in the archive.
t2e5c8c3e = Füge die Dateien oder einen Ordner für das Archiv hinzu.

# Windows 11 and Linux open 7z on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tf7f64b21 = Windows 11 und Linux öffnen 7z von selbst; macOS, Smartphones und ältere Windows-Versionen brauchen womöglich eine Archiv-App. Zum Teilen mit jedem wähle .zip.

# Linux, macOS and Windows 11 open it on their own; phones and older Windows may need an archiver app. To share with anyone, pick .zip.
t051031b1 = Linux, macOS und Windows 11 öffnen es von selbst; Smartphones und ältere Windows-Versionen brauchen womöglich eine Archiv-App. Zum Teilen mit jedem wähle .zip.

# Linux and Windows 11 open it on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tc2c29183 = Linux und Windows 11 öffnen es von selbst; macOS, Smartphones und ältere Windows-Versionen brauchen womöglich eine Archiv-App. Zum Teilen mit jedem wähle .zip.

# Most computers need an archiver app or the lz4 tool to open it. To share with anyone, pick .zip.
te9afe1c2 = Die meisten Computer brauchen eine Archiv-App oder das lz4-Tool, um es zu öffnen. Zum Teilen mit jedem wähle .zip.

# It holds this one file. Command-line tools open it; Windows and macOS may need an archiver app. To share with anyone, pick .zip.
t16463509 = Es enthält nur diese eine Datei. Kommandozeilen-Tools öffnen es; Windows und macOS brauchen womöglich eine Archiv-App. Zum Teilen mit jedem wähle .zip.

# Don’t split
t56ee6829 = Nicht teilen

# 25 MB (email)
tc6ba1c9c = 25 MB (E-Mail)

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB (USB-Stick)

# Custom size…
t0fcf9e2d = Eigene Größe…

# Size
ta6478e7c = Größe

# Password
t2cc30838 = Passwort

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…, age1pq1… oder ssh-ed25519 …

# Save here
ta1f147a6 = Hier speichern

# Use these keys
t1a01e439 = Diese Schlüssel verwenden

# No compression
t2e9c9e78 = Keine Komprimierung

# {name} compression
te2db10f6 = { $name }-Komprimierung

# Several frames
t84cc41f7 = Mehrere Frames

# One frame
tc6ef759e = Ein Frame

# Small blocks
t04032e8a = Kleine Blöcke

# Parts of 25 MB
ta999efe2 = Teile à 25 MB

# Parts of 4 GB
t760df467 = Teile à 4 GB

# Parts of {size}
tdd286479 = Teile à { $size }

# Parts of a size to type
tafd4b17d = Teile in selbst eingegebener Größe

# Public key
t9981f45f = Öffentlicher Schlüssel

# Owners kept
t63e8a0df = Besitzer behalten

# Attributes kept
te9e8289c = Attribute behalten

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = Um Besitzer und erweiterte Attribute für eine Sicherung zu behalten, wähle ein .tar-Format.

# Keep for a backup
t6a2bcc1a = Für eine Sicherung behalten

# Owners: user and group
t5350e3fe = Besitzer: Benutzer und Gruppe

# Extended attributes and ACLs
t35aff4d1 = Erweiterte Attribute und ACLs

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = Für eine Sicherung, die auf diesem Computer wiederhergestellt wird. Wer das Archiv öffnet, sieht die Benutzernamen.

# Beside the files
t757989af = Neben den Dateien

# What it holds, from
teae4ef1a = Inhalt aus

# Save in
tf2d34837 = Speichern in

# Folder…
tc8abcd43 = Ordner…

# Storage…
td0d8c806 = Speicherort…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = Die Teile werden zuerst im temporären Ordner dieses Computers erstellt und dann hochgeladen, daher brauchen sie dort Platz.

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext } wird zuerst im temporären Ordner dieses Computers erstellt und dann hochgeladen, daher braucht es dort Platz. .zip und tar-Formate gehen direkt an diesen Speicherort.

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = Es wird zuerst im temporären Ordner dieses Computers erstellt und dann hochgeladen, daher braucht es dort Platz.

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } nimmt Dateien bis { $most } an. Ein größeres Archiv schlägt fehl, sobald der Upload beginnt.

# Choose a storage, and a folder in it
td7627e48 = Wähle einen Speicherort und darin einen Ordner

# New folder
t1926d127 = Neuer Ordner

# Give it a name, without / or \ in it.
ta598b15b = Gib einen Namen ohne / oder \ an.

# A file of that name is there already: this one will be {name}.
t9451c3b1 = Eine Datei dieses Namens ist schon da: diese wird { $name }.

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = Einige Namen sind unter Windows oder macOS nicht möglich, etwa „{ $name }“. Dort werden diese Dateien womöglich nicht so entpackt, wie sie sind.

# Hide advanced
tae2a96e9 = Erweitert ausblenden

# Advanced options
t701d5ed9 = Erweiterte Optionen

# Large blocks
tdc489452 = Große Blöcke

# Compression level
td386c9ef = Komprimierungsstufe

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = Speichern legt die Dateien unverändert ab: am schnellsten erstellt und am größten.

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = Höher ergibt ein kleineres Archiv, dessen Erstellung länger dauert, nicht das Öffnen.

# Frames
tf03e266f = Frames

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = Mehrere Frames öffnen sich auf jedem Kern und machen das Archiv etwas größer: um weniger als ein halbes Prozent.

# One frame makes the smallest archive, which opens on one core.
ta097b317 = Ein Frame ergibt das kleinste Archiv, das sich auf einem Kern öffnet.

# Blocks
t036a71e3 = Blöcke

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = Kleine Blöcke halten jeden Kern beschäftigt: bis zu doppelt so schnell, und das Archiv wird ein paar Prozent größer.

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = Große Blöcke ergeben das kleinste Archiv, auf weniger Kernen, wenn es wenig zu komprimieren gibt.

# Split into parts
tca4436bd = In Teile aufteilen

# Each part
t642c444d = Jeder Teil

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = Teile öffnen sich in 7-Zip und ähnlichen Archiv-Apps, nicht mit den in Windows und macOS eingebauten Tools.

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = Ein Teil braucht mindestens 1 MB, etwa 100 MB.

# Encrypt with age
t22784ad9 = Mit age verschlüsseln

# Protect with a password
tae8d8740 = Mit Passwort schützen

# Suggest
ta6e44ce5 = Vorschlagen

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = Bewahre es sicher auf: Ohne es lässt sich dieses Archiv nie wieder öffnen.

# Type a password, or turn this off.
t62b19318 = Gib ein Passwort ein oder schalte dies aus.

# Hide file names too
t2198d77b = Auch Dateinamen verbergen

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = Mit Passwort können die eingebauten Tools von Windows und macOS dieses ZIP nicht öffnen. Der Empfänger braucht eine Archiv-App. Dateinamen bleiben außerdem sichtbar; wähle .7z, um sie zu verbergen.

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = Als { $name } gespeichert. Auch die Namen darin sind verschlüsselt. Windows und macOS brauchen womöglich das age-Tool oder eine Archiv-App zum Öffnen.

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
        [one] { $n } Schlüssel aus { $name }
       *[other] { $n } Schlüssel aus { $name }
    }

# Remove
t21a5901d = Entfernen

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
        [one] { $n } Schlüssel darin wurde ausgelassen, da age für diese Art nicht verschlüsselt.
       *[other] { $n } Schlüssel darin wurden ausgelassen, da age für diese Art nicht verschlüsselt.
    }

# From a file…
t40d7c944 = Aus einer Datei…

# Type a public key, or choose a file of them.
tcb028b06 = Gib einen öffentlichen Schlüssel ein oder wähle eine Datei mit Schlüsseln.

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = Ein Post-Quanten-Schlüssel: Zum Öffnen muss das age-Tool mindestens Version 1.3 haben.

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = Nur der zugehörige private Schlüssel öffnet dies, und geht er verloren, kann es nichts mehr öffnen.

# Weak
t6de87ae9 = Schwach

# Fair
t82fa0d53 = Mittelmäßig

# Strong
t0dad409a = Stark

# Choose what to put in it
t5461cfa6 = Auswählen, was hineinkommt

# Convert
t92e29218 = Umwandeln

# New archive
tf352befb = Neues Archiv

# Create
t990de47d = Erstellen

# {n} item | {n} items
tfff7fd6c =
    { $n ->
        [one] { $n } Element
       *[other] { $n } Elemente
    }

# Move {what} to the Trash?
tdf5a8d53 = { $what } in den Papierkorb verschieben?

# Delete {what}?
ta297d6c5 = { $what } löschen?

# It can be put back from the system’s Trash.
te5e137a1 = Es lässt sich aus dem Papierkorb des Systems wiederherstellen.

# This cannot be undone from packwing.
t3947dab4 = Das lässt sich in packwing nicht rückgängig machen.

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
        [one] und { $n } weiteres
       *[other] und { $n } weitere
    }

# Move to Trash
ta036661b = In den Papierkorb

# Delete
t5797ea6a = Löschen

# {many} items
t49912bd4 = { $many } Elemente

# Rename {what}
t83401013 = { $what } umbenennen

# New name
taf4cad88 = Neuer Name

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = Es wird endgültig aus dem Archiv entfernt, Ordner samt Inhalt. Das lässt sich nicht rückgängig machen.

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = Dieses Archiv ist signiert. Nach dieser Änderung passt es nicht mehr zu seiner Signatur, bis es neu signiert wird.

# Rename
tc80631f3 = Umbenennen

# The archive’s password
t8a168799 = Das Passwort des Archivs

# one folder
td571ce35 = ein Ordner

# one file
t95c9ec7b = eine Datei

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }, also direkt hier

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }, also in einem neuen Ordner

# Extract here
t354232ec = Hier entpacken

# {picked} selected
t9addc37e = { $picked } ausgewählt

# All {counted}
t649feaf8 = Alle { $counted }

# What should it extract?
td12e23bf = Was soll entpackt werden?

# Smart
t54535972 = Intelligent

# Recommended
t7865024c = Empfohlen

# Where should the files go?
td35f540c = Wohin sollen die Dateien?

# Right here
t900a8a39 = Direkt hier

# Another folder
t9f128ba8 = Anderer Ordner

# Choose…
t58daae9c = Auswählen…

# Goes to {destination}
t8195aed2 = Geht nach { $destination }

# Checked as it goes
tb6c9570b = Wird währenddessen geprüft

# · Nothing leaves the folder or passes the size limit
t11424797 = · Nichts verlässt den Ordner oder überschreitet die Größengrenze

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } stammt aus dem Internet. Die Markierung bleibt erhalten, daher wirst du vor dem Ausführen gefragt.

# Extract archive
t834ba3c0 = Archiv entpacken

# Extract
t30518e1e = Entpacken

# Open {name}?
tcfe84c2f = { $name } öffnen?

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing legt eine Kopie in einem temporären Ordner ab und öffnet diese. Änderungen daran gelangen nicht ins Archiv zurück.

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = Das ist ein Programm: Beim Öffnen wird es ausgeführt. Tu das nur, wenn du der Herkunft des Archivs vertraust.

# Don’t ask again
t00c5d168 = Nicht mehr fragen

# Run it
t72430853 = Ausführen

# Open
t538b10e9 = Öffnen

# Open from a storage
t45a8991d = Aus einem Speicherort öffnen

# Choose a storage, and an archive in it
t3c900950 = Wähle einen Speicherort und darin ein Archiv

# Choose
t73c66160 = Auswählen

# This is not a key file packwing can use: {why}.
tcf6986a7 = Das ist keine Schlüsseldatei, die packwing verwenden kann: { $why }.

# Could not read {path}: {e}.
t1dc6961f = { $path } konnte nicht gelesen werden: { $e }.

# The passphrase
t1e921d00 = Die Passphrase

# The key file’s passphrase
t4051f4d7 = Die Passphrase der Schlüsseldatei

# Enter the passphrase
tbd0672da = Passphrase eingeben

# Passphrase
t191a2dc5 = Passphrase

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = Diese Datei ist mit age verschlüsselt. packwing behält die Passphrase nur, solange das Archiv geöffnet ist, und schreibt sie nie auf.

# Enter the password
t513ea7ab = Passwort eingeben

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = Dieses Archiv ist verschlüsselt. packwing behält sein Passwort nur, solange das Archiv geöffnet ist, und schreibt es nie auf.

# Choose the key file
t3c521a4e = Schlüsseldatei wählen

# Key file
t6ed7c612 = Schlüsseldatei

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = Diese Datei ist mit age für einen Schlüssel verschlüsselt: eine age-Schlüsseldatei oder ein SSH-Schlüssel. packwing behält, was es daraus liest, nur, solange das Archiv geöffnet ist, und schreibt es nie auf.

# Enter the key file’s passphrase
t808b85c5 = Passphrase der Schlüsseldatei eingeben

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = Die Schlüsseldatei ist selbst geschützt. packwing behält ihre Passphrase nur, solange das Archiv geöffnet ist, und schreibt sie nie auf.

# kept, {size}
t001f6992 = behalten, { $size }

# left out
t73aeedd4 = weggelassen

# renamed {to}
t7aa9b778 = umbenannt in { $to }

# It was gone by the time it was read.
t93fb1e03 = Sie war nicht mehr da, als sie gelesen werden sollte.

# The job stopped before it was done.
t57a78885 = Der Auftrag hat aufgehört, bevor er fertig war.

# Waiting for other jobs…
t16ec36d9 = Wartet auf andere Aufgaben…

# A file is damaged
t8b1230ed = Eine Datei ist beschädigt

# Some names can’t be used here
t26835ec8 = Einige Namen sind hier nicht möglich

# Converted, but some files are damaged
t54ef3472 = Umgewandelt, aber einige Dateien sind beschädigt

# Extracted, but some files are damaged
t8226d624 = Entpackt, aber einige Dateien sind beschädigt

# Converted, with some files left out or renamed
t5cce0e50 = Konvertiert, einige Dateien ausgelassen oder umbenannt

# Extracted, with some files left out or renamed
t5dc002d6 = Entpackt, einige Dateien ausgelassen oder umbenannt

# Extracting…
t94279f82 = Wird entpackt…

# Uploading…
tc75fe478 = Wird hochgeladen…

# Creating…
t01fcd896 = Wird erstellt…

# Adding…
t2060c750 = Wird hinzugefügt…

# Changing the archive…
t186fbef5 = Archiv wird geändert…

# Opening…
tc40f2321 = Wird geöffnet…

# Testing…
t19d1cb65 = Wird getestet…

# Copying…
t02dd17e0 = Wird kopiert…

# Moving…
t890f7ed1 = Wird verschoben…

# Signing…
t7b8f8780 = Wird signiert…

# Splitting…
t5452adf5 = Wird geteilt…

# Joining…
taa884c51 = Wird zusammengefügt…

# Converting…
ta1b8f4cc = Wird umgewandelt…

# Couldn’t split the file
t83d27d26 = Die Datei konnte nicht geteilt werden

# Couldn’t join the parts
t1cf67e00 = Die Teile konnten nicht zusammengefügt werden

# Couldn’t convert the archive
t39f973a9 = Das Archiv konnte nicht umgewandelt werden

# Couldn’t sign
t4489f97c = Signieren fehlgeschlagen

# Couldn’t copy
t613e4e9c = Kopieren fehlgeschlagen

# Couldn’t move
t9796adec = Verschieben fehlgeschlagen

# Couldn’t extract
tacd22c56 = Entpacken fehlgeschlagen

# Couldn’t create the archive
t9d0e01bc = Archiv konnte nicht erstellt werden

# Couldn’t add the files
tfbd989b0 = Dateien konnten nicht hinzugefügt werden

# Couldn’t change the archive
tcdc4a49c = Archiv konnte nicht geändert werden

# Couldn’t open the file
t041198a4 = Datei konnte nicht geöffnet werden

# The test failed
tfcd070a5 = Der Test ist fehlgeschlagen

# Starts when one of the others ends
tbe2a0509 = Startet, wenn eine der anderen fertig ist

# Paused
tde281feb = Pausiert

# Starting…
t73727cef = Wird gestartet…

# Finishing…
t042f47fe = Wird abgeschlossen…

# Almost done
ta550a6a9 = Fast fertig

# About {secs} seconds left
t3045da16 = Noch etwa { $secs } Sekunden

# About a minute left
tf727815c = Noch etwa eine Minute

# About {div_ceil} minutes left
tbd7a3305 = Noch etwa { $div_ceil } Minuten

# {name} · checking that every file reads
t0d56ec7c = { $name } · es wird geprüft, ob sich jede Datei lesen lässt

# and {count} more | and {count} more
te728628b =
    { $n ->
        [one] und { $count } weitere
       *[other] und { $count } weitere
    }

# The test passed
tdf0ce240 = Der Test war erfolgreich

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
        [one] { $count } Datei gelesen, { $size }, ohne Fehler.
       *[other] { $count } Dateien gelesen, { $size }, ohne Fehler.
    }

# Close
tcd86acc3 = Schließen

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = „{ $path }“ ist beschädigt: { $why }. Anhalten, weglassen oder das Lesbare behalten?

# Do the same for the other damaged files
t798693ad = Bei den anderen beschädigten Dateien genauso verfahren

# Stop
t4b7f7705 = Stoppen

# Leave it out
taaff8ec9 = Weglassen

# Keep what was read
tc4387d25 = Lesbares behalten

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
        [one] { $count } Datei kann hier nicht unter ihrem Namen entpackt werden. Noch ist nichts geschrieben: anhalten, auslassen oder umbenennen?
       *[other] { $count } Dateien können hier nicht unter ihren Namen entpackt werden. Noch ist nichts geschrieben: anhalten, auslassen oder umbenennen?
    }

# Leave out
ta50cc084 = Auslassen

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
        [one] { $count } Verknüpfung führt aus dem Ordner hinaus, an einen anderen Ort auf diesem Computer. Noch ist nichts geschrieben: auslassen, oder so anlegen, wie sie ist, wenn du diesem Archiv vertraust?
       *[other] { $count } Verknüpfungen führen aus dem Ordner hinaus, an andere Orte auf diesem Computer. Noch ist nichts geschrieben: auslassen, oder so anlegen, wie sie sind, wenn du diesem Archiv vertraust?
    }

# Create them
t9a4d6e7d = Anlegen

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
        [one] Die Daten von { $count } Datei waren beschädigt.
       *[other] Die Daten von { $count } Dateien waren beschädigt.
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
        [one] { $count } Datei wurde ausgelassen.
       *[other] { $count } Dateien wurden ausgelassen.
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
        [one] { $count } Datei wurde umbenannt, da ihr Name hier nicht möglich ist.
       *[other] { $count } Dateien wurden umbenannt, da ihre Namen hier nicht möglich sind.
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
        [one] { $count } Datei war nicht mehr da, als sie gelesen werden sollte, und wurde ausgelassen.
       *[other] { $count } Dateien waren nicht mehr da, als sie gelesen werden sollten, und wurden ausgelassen.
    }

# The rest is in the archive.
t977a5572 = Der Rest ist im Archiv.

# The rest was extracted.
ta7833fcb = Der Rest wurde entpackt.

# Copy the list
t3c49b269 = Liste kopieren

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } von { $count2 } Dateien · { $size } von { $size2 }

# Open the folder when it’s done
tf0f1017b = Ordner nach Abschluss öffnen

# Run in background
tb800c0ef = Im Hintergrund ausführen

# Resume
t454d1d8a = Fortsetzen

# Pause
t44f9bf2d = Pausieren

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
        [one] { $n } Aufgabe läuft noch
       *[other] { $n } Aufgaben laufen noch
    }

# Closing packwing stops them
t2e6491b3 = Wenn packwing schließt, stoppen sie

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = Was sie nicht fertig haben, bleibt unfertig: ein Archiv im Entstehen wird entfernt, und ein Entpacken bleibt, so weit es kam.

# Keep running
t79c1b9c7 = Weiterlaufen lassen

# Stop them and quit
tf2707857 = Stoppen und beenden

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = Seine Dateien bleiben, wo sie sind. Womit packwing sich dort anmeldet, wird aus dem Schlüsselbund des Systems entfernt.

# The folder and its files stay where they are.
t4850aaa8 = Der Ordner und seine Dateien bleiben, wo sie sind.

# Remove “{name}” from the list?
t5dd9cfcf = „{ $name }“ aus der Liste entfernen?

# It can be added again at any time.
t80d51a99 = Er kann jederzeit wieder hinzugefügt werden.

# A passphrase for the new key
tf5593f05 = Eine Passphrase für den neuen Schlüssel

# The same passphrase again
tad5c08e6 = Dieselbe Passphrase noch einmal

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = Zum Signieren braucht es eine Pro- oder Enterprise-Lizenz: Einstellungen › Lizenz. Signaturen prüfen ist in jedem Tarif kostenlos.

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = Das ist ein Teil einer in mehrere Teile zerlegten Datei. Eine Signatur gilt für eine Datei und würde scheinbar für alle gelten, daher signiert packwing keine Teile.

# That passphrase does not open this key.
tc5357c97 = Diese Passphrase öffnet diesen Schlüssel nicht.

# This is not a key packwing can sign with: {why}.
t662c29f9 = Das ist kein Schlüssel, mit dem packwing signieren kann: { $why }.

# The two passphrases are not the same.
t335fe90b = Die beiden Passphrasen stimmen nicht überein.

# Keep the key here
t9306dafe = Schlüssel hier behalten

# No key chosen
te58c13f5 = Kein Schlüssel ausgewählt

# Key
tcd1ac90c = Schlüssel

# New key…
t9e1134da = Neuer Schlüssel…

# Again
t58fb56ef = Wiederholen

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = Ein neuer Schlüssel enthält Ed25519 und ML-DSA-65 zusammen, sodass eine Signatur gültig bleibt, auch wenn eines davon gebrochen wird. Nur packwing und hivepack prüfen diese: Für ssh-keygen signiere mit einem SSH-Ed25519-Schlüssel. Ein Schlüssel, dessen Passphrase verloren ist, lässt sich nicht wiederherstellen.

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = Die Signatur liegt daneben als { $signature } und ersetzt eine vorhandene. Die gesamte Datei läuft über diesen Computer, und dein Anbieter berechnet womöglich das Gelesene.

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = Die Signatur liegt daneben als { $signature } und ersetzt eine vorhandene.

# Sign {name}
ted250f31 = { $name } signieren

# Anyone can check it, with the key’s public half
t04ecc8be = Jeder kann sie mit der öffentlichen Hälfte des Schlüssels prüfen

# Make the key…
t6bba28d5 = Schlüssel erstellen…

# Sign
t8f6340c4 = Signieren

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = { $count } Teile daneben, ab { $first }. Zusammenfügen setzt sie wieder zusammen, und packwing und 7-Zip öffnen ein so geteiltes Archiv direkt.

# Split
t16a15383 = Teilen

# Move {what}
tc168ca56 = { $what } verschieben

# Copy {what}
teda01f26 = { $what } kopieren

# Move here
t1aaf57aa = Hierher verschieben

# Copy here
t85f0245a = Hierher kopieren

# Move
t7f2c2a34 = Verschieben

# If a name is taken
tc99c037f = Wenn ein Name schon vergeben ist

# They are in this folder already. Choose another.
t5318c14e = Sie sind schon in diesem Ordner. Wähle einen anderen.

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = Alles läuft über diesen Computer, und dein Anbieter berechnet womöglich das Gelesene. Jedes verschobene Original wird entfernt, sobald seine Kopie zurückgelesen wurde und übereinstimmt.

# Each original goes once its copy has been read back and matches.
t8ecbab4e = Jedes Original wird entfernt, sobald seine Kopie zurückgelesen wurde und übereinstimmt.

# The originals stay where they are.
t5937f050 = Die Originale bleiben, wo sie sind.

# Too big to show here
t1b339d35 = Zu groß für die Anzeige

# packwing can’t show this kind of file
tb909bf3c = packwing kann diese Art von Datei nicht anzeigen

# It could not be read
tb7927177 = Sie ließ sich nicht lesen

# It could not be read: {why}
t26005fe3 = Sie ließ sich nicht lesen: { $why }

# It could not be shown
tb60fb288 = Sie ließ sich nicht anzeigen

# Open with default app
t5ef80ec5 = Mit Standard-App öffnen

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = Eine andere Aufgabe erstellt dort gerade { $name }: gib diesem einen anderen Namen oder warte, bis sie fertig ist.

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = Eine andere Aufgabe entpackt gerade nach { $folder }: wähle einen anderen Ordner oder warte, bis sie fertig ist.

# no editor command
tbf2a5d3a = kein Editor-Befehl

# no text editor found
t6256b3da = kein Texteditor gefunden

# Search this folder
t2c814039 = In diesem Ordner suchen

# This folder could not be read: {why}
t2c6017b2 = Dieser Ordner konnte nicht gelesen werden: { $why }

# Could not load more: {why}
t466e87c4 = Mehr konnte nicht geladen werden: { $why }

# Could not delete: {why}
te5951d38 = Löschen nicht möglich: { $why }

# the text editor
t91cdab0c = den Texteditor

# Cannot start {who}: {why}
t8deb160a = { $who } lässt sich nicht starten: { $why }

# Edit in text editor
td8d39fb8 = Im Texteditor bearbeiten

# Edit in {editor}
te44cd57b = In { $editor } bearbeiten

# Modified
tdd13a6b2 = Geändert

# Loading…
tc4e2f181 = Lädt…

# Only what is loaded was searched.
t39c5f536 = Es wurde nur das Geladene durchsucht.

# Search the whole folder
t41f9a787 = Den ganzen Ordner durchsuchen

# Load {n} more
t30bed46a = { $n } weitere laden

# Refresh
t0815aad4 = Aktualisieren

# Hide hidden files
t3375c9c0 = Versteckte Dateien ausblenden

# Show hidden files
tf18c7057 = Versteckte Dateien anzeigen

# View
t5e4baa18 = Ansicht

# Extract…
t4a09490e = Entpacken…

# Compress…
t690f9553 = Komprimieren…

# Copy to…
t543cf15b = Kopieren nach…

# Move to…
t7a32320b = Verschieben nach…

# Sign…
td6d61b10 = Signieren…

# Checksums…
t2891102d = Prüfsummen…

# Convert…
tae85a7c4 = Umwandeln…

# Split into parts…
t281db403 = In Teile teilen…

# Join the parts
t0ad56db8 = Teile zusammenfügen

# Copy path
tf165cb2b = Pfad kopieren

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
        [one] { $n } Element komprimieren…
       *[other] { $n } Elemente komprimieren…
    }

# {n} loaded, more in this folder
t29a52578 = { $n } geladen, mehr in diesem Ordner

# Sorted among those loaded
t037d01ae = Unter den geladenen sortiert

# {n} selected
tebda8468 = { $n } ausgewählt

# Try again
t213e90fa = Erneut versuchen

# Nothing here matches that.
t0f9a8a58 = Dazu passt hier nichts.

# Add a storage to browse it.
t0b82c83a = Füge einen Speicherort hinzu, um ihn zu durchsuchen.

# No folders or archives here.
t6a090cc3 = Hier gibt es keine Ordner oder Archive.

# No folders here.
tc7731c8f = Hier gibt es keine Ordner.

# This folder is empty.
td9e47db6 = Dieser Ordner ist leer.

# Show in Finder
ta9442aa9 = Im Finder anzeigen

# Show in File Explorer
t23c89eda = Im Datei-Explorer anzeigen

# Show in the file manager
tb347ece9 = Im Dateimanager anzeigen

# Open in Finder
te2a467e0 = Im Finder öffnen

# Open in File Explorer
t3794d625 = Im Datei-Explorer öffnen

# Open in the file manager
td1ceabc4 = Im Dateimanager öffnen

# Copy all
t6aef5c37 = Alles kopieren

# This system has no folder for settings.
tfd0ff0a1 = Dieses System hat keinen Ordner für Einstellungen.

# Archive
t73690b65 = Archiv

# Search inside this archive
tb35c0c27 = In diesem Archiv suchen

# Not text it can show
ta413eca5 = Kein Text, den es anzeigen kann

# It needs the password to show it
t81f5dbce = Zum Anzeigen braucht es das Passwort

# Open another…
tff74a552 = Anderes öffnen…

# From a storage…
tcfba8fd5 = Aus einem Speicherort…

# Safe to extract
t3c93bb70 = Sicher zu entpacken

# Reading… {percent}%
t83223c27 = Wird gelesen… { $percent } %

# Read {fetched} of {whole}
t613dd932 = { $fetched } von { $whole } gelesen

# Add files
tdd45c7bb = Dateien hinzufügen

# Test
t2ffcbe05 = Testen

# Reading what’s inside…
t1bbda9f5 = Inhalt wird gelesen…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = Nichts in diesem Archiv passt zu „{ $query }“.

# Nothing in this folder matches “{query}”.
teecdfa3d = Nichts in diesem Ordner passt zu „{ $query }“.

# And {more} more. Type more to narrow it down.
t843253d0 = Und { $more } weitere. Gib mehr ein, um einzugrenzen.

# Open an archive
t8382bd3e = Archiv öffnen

# ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…
t4a498ac8 = ZIP-, 7z- und tar-Archive, Datenträgerabbilder und Pakete, komprimiert oder mit age verschlüsselt: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…

# Open an archive…
t2cf2a8ee = Archiv öffnen…

# Choose a file to see it here
t37854216 = Wähle eine Datei, um sie hier zu sehen

# Symbolic link
tc7c20187 = Symbolischer Link

# Leads to {target}
te7ddae48 = Führt zu { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
        [one] { $count } ausführbare Datei in diesem Ordner behält beim Entpacken ihre Internet-Markierung.
       *[other] { $count } ausführbare Dateien in diesem Ordner behalten beim Entpacken ihre Internet-Markierung.
    }

# This computer
tfa0d176a = Dieser Computer

# Home
t52f50eae = Home

# Desktop
t64215c53 = Desktop

# Downloads
t539307de = Downloads

# Drop to open
tb255877b = Zum Öffnen ablegen

# Browse
t0eac1f07 = Durchsuchen

# Recent
t50e888fe = Zuletzt

# Choose a file to see its information here
t50aac02b = Wähle eine Datei, um hier ihre Informationen zu sehen

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
        [one] { $n } ausgewählt
       *[other] { $n } ausgewählt
    }

# Files in them come to {size}
t0cca93f1 = Die Dateien darin ergeben zusammen { $size }

# Delete…
t2fed07aa = Löschen…

# Link to a folder
t4b408baf = Verknüpfung zu einem Ordner

# Link
t8e08e589 = Link

# {tag} file
ta56bbbe9 = { $tag }-Datei

# {files}, {folders}
t72e46dc5 = { $files }, { $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
        [one] { $n } Datei
       *[other] { $n } Dateien
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
        [one] { $n } Ordner
       *[other] { $n } Ordner
    }

# {size} so far
tc87daf59 = bisher { $size }

# Kind
t3ff50c63 = Art

# Holds
tffe19c31 = Enthält

# Permissions
t8700fd83 = Berechtigungen

# Path
teb66e456 = Pfad

# Signature
tc049050d = Signatur

# Count size and files
t0f23db17 = Größe und Dateien zählen

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
        [one] { $n } Ordner darin konnte nicht gelesen werden.
       *[other] { $n } Ordner darin konnten nicht gelesen werden.
    }

# Check the signature
t9d44730e = Signatur prüfen

# Extracted
t3250471f = Entpackt

# Archive created
t22c69419 = Archiv erstellt

# Moved
t82869bf0 = Verschoben

# Signed
t27c3ab35 = Signiert

# Joined
tf8f2b670 = Zusammengefügt

# Converted
tc7b76e99 = Umgewandelt

# Waiting for a password or a key
t0e83ae32 = Wartet auf ein Passwort oder einen Schlüssel

# Enter it…
tddb7844a = Eingeben…

# Show
t29742e5c = Anzeigen

# Jobs
t4ae6a79d = Aufgaben

# Clear finished
t30c178e8 = Erledigte entfernen

# Nothing runs in the background.
t2573f39e = Im Hintergrund läuft nichts.

# Add to the right-click menu
t116d3868 = Zum Kontextmenü hinzufügen

# Extract or compress from any folder.
t20864983 = Aus jedem Ordner entpacken oder komprimieren.

# Welcome to packwing
tb38a86a6 = Willkommen bei packwing

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = Öffne jedes Archiv, erstelle kleinere Dateien zum Teilen und bleibe vor schädlichen Downloads geschützt.

# Set things up
t63f9c1cd = Einrichten

# You can change these later in Settings.
t95f41ffb = Du kannst das später in den Einstellungen ändern.

# Get started
te83ca5ee = Los geht’s

# Remove from Recent
t26cc8dba = Aus „Zuletzt“ entfernen

# {location} · not there any more
t7d0c09c7 = { $location } · nicht mehr vorhanden

# The archives you open will show here.
tc0cc82d5 = Die Archive, die du öffnest, erscheinen hier.

# Copy version info
t3d37483c = Versionsinfo kopieren

# Version {version} · {os} {arch}
t6bead5ae = Version { $version } · { $os } { $arch }

# A file archiver that opens archives safely.
t3c244df9 = Ein Datei-Archivierer, der Archive sicher öffnet.

# Updates
t483075a5 = Updates

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = Dies ist eine Beta: Du erfährst von neueren Betas und von der stabilen Version, die auf sie folgt.

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = Dies ist eine stabile Version: Du erfährst von neueren stabilen Versionen, nicht von Betas.

# Check now
t4f0e0677 = Jetzt prüfen

# Look for updates once a day
tcc27ace1 = Einmal täglich nach Updates suchen

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = Die Liste der Releases wird von GitHub gelesen und nur geglaubt, wenn ihre Signaturen gültig sind. Ohne Sie wird nichts heruntergeladen oder installiert.

# Skip this version
t2c33d756 = Diese Version überspringen

# This build has no release keys, so it cannot tell of updates.
t378182cb = Diesem Build fehlen die Release-Schlüssel, daher kann er nicht über Updates informieren.

# packwing {version} is ready
tf02650f8 = packwing { $version } ist bereit

# It is installed when you quit, or now with Restart.
t6fecac86 = Es wird beim Beenden installiert, oder jetzt mit Neu starten.

# Restart now
t54001a7c = Jetzt neu starten

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = Suche nach einer neueren Version…

# Release notes
tc52ab62f = Versionshinweise

# packwing {version} is out
t36a30d8b = packwing { $version } ist erschienen

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = Führen Sie diese Befehle in einem Terminal aus. Sie prüfen den Download anhand der signierten Release-Liste, und sudo fragt nach Ihrem Passwort.

# {by} updates it: {command}
tdf2cab9c = Das Update übernimmt { $by }: { $command }

# {by} updates it.
t84b27295 = Das Update übernimmt { $by }.

# It is downloaded and checked, then installed when you quit.
tb46c0678 = Es wird heruntergeladen und geprüft, dann beim Beenden installiert.

# Download it, and install it in place of this one.
t39312eae = Lade sie herunter und installiere sie anstelle dieser Version.

# The download failed: {why}
t5e96c075 = Der Download ist fehlgeschlagen: { $why }

# Download and install
t1a99173f = Herunterladen und installieren

# Download
t6964bb19 = Herunterladen

# The last look failed: {why}
td122a092 = Die letzte Prüfung ist fehlgeschlagen: { $why }

# Up to date (looked {when}).
t4665ef60 = Auf dem neuesten Stand (geprüft { $when }).

# Not looked yet.
t5a3ba058 = Noch nicht geprüft.

# Free software
t731220a4 = Freie Software

# Under the GNU General Public License, version 3 or later
t4e98bf3c = Unter der GNU General Public License, Version 3 oder neuer

# View license
tfc21d38b = Lizenz ansehen

# Made by HivePaaS
t10c7007a = Entwickelt von HivePaaS

# On hivepack, our library for reading and writing archives
tfae09b61 = Auf hivepack, unserer Bibliothek zum Lesen und Schreiben von Archiven

# Source code
tf6a49f4d = Quellcode

# Something wrong?
t610dfb6f = Stimmt etwas nicht?

# Tell us, with the version above
ta3e29034 = Sag es uns, mit der Version oben

# Report a problem
t7b1728e7 = Problem melden

# What changed
te8345543 = Was sich geändert hat

# Every release, and what is new in it
t8e6f2052 = Jede Version und was darin neu ist

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS. Der Name und das Logo von packwing sind Marken von HivePaaS.

# More from HivePaaS
tb25cf729 = Mehr von HivePaaS

# Free and open source, like packwing.
tc1ece130 = Frei und quelloffen, wie packwing.

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = Ein schneller, nativer SSH-Client für Linux, macOS und Windows.

# SSH and SFTP
t0f8af34d = SSH und SFTP

# Follows your ssh_config
tc1c4446d = Folgt deiner ssh_config

# A real terminal
tae9bcfa8 = Ein echtes Terminal

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = shellwing.app besuchen

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = Eine selbst gehostete Plattform für deine Apps auf Basis von Docker Swarm: eine Open-Source-Alternative zu Heroku, Render und Coolify.

# 300+ one-click apps
t6a770b0a = 300+ Apps mit einem Klick

# Deploy on push
tb0bfc3e9 = Deploy bei jedem Push

# Pull request previews
t4e4180fc = Vorschau für Pull Requests

# Automatic TLS
t9e9776ed = Automatisches TLS

# Live logs and metrics
tebe3a172 = Live-Logs und Metriken

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = hivepaas.com besuchen

# Always
tad4bbd04 = Immer

# Extensions, apart by spaces
tc709375f = Endungen, durch Leerzeichen getrennt

# Format of new archives
t3b50d238 = Format neuer Archive

# New archive starts with it; you can pick another there.
t7b361b55 = Ein neues Archiv beginnt damit; dort kannst du ein anderes wählen.

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = Speichern legt die Dateien von ZIP- und 7z-Archiven unverändert ab. tar-Formate nehmen Niedrigste, da nur .tar ohne Komprimierung ist.

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = Höher ergibt kleinere Archive, deren Erstellung länger dauert, nicht das Öffnen.

# Write straight to a storage above
ta4975128 = Direkt in einen der obigen Speicherorte schreiben

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = Größere Dateien brauchen keinen Platz auf diesem Computer, aber bei Verbindungsabbruch muss neu komprimiert werden. S3 nimmt tar und ZIP so an, SFTP jedes Format.

# Don’t compress what already is
t940ec31d = Bereits Komprimiertes nicht komprimieren

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = Für ZIP und 7z: Bilder, Videos, Audio und Archive kommen unverändert hinein, schneller und nicht größer

# What goes in as it is
tc0104b29 = Was unverändert hineinkommt

# Files whose names end in these, whatever their case
ta3acef9b = Dateien, deren Namen auf diese enden, unabhängig von Groß- und Kleinschreibung

# Reset to default
t0db9f7dc = Auf Standard zurücksetzen

# Leave out Mac-only files
t27eec14f = Nur auf dem Mac vorhandene Dateien auslassen

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store, __MACOSX und Resource Forks: nur auf dem Mac von Nutzen, anderswo Ballast

# Hide file names behind the password
t93ff7715 = Dateinamen hinter dem Passwort verbergen

# For 7z: nobody sees what is inside without it
t0f5354d7 = Für 7z: Ohne es sieht niemand, was darin ist

# Yours
t2635ea3b = Deine

# By name
t983e37f7 = Nach Name

# By number
tcbbada8d = Nach Nummer

# No limit
t6de7f47d = Keine Begrenzung

# Where the files go
t77e60aba = Wohin die Dateien kommen

# Where Extract starts: you can pick another there.
t4b73082f = Wo Entpacken beginnt: Dort kannst du einen anderen wählen.

# One item right here, more in a new folder
ta5a4d31d = Ein Element direkt hier, mehrere in einem neuen Ordner

# A new folder next to it
t2095cbe9 = Ein neuer Ordner daneben

# Named after the archive
tb991d4c1 = Nach dem Archiv benannt

# Next to the archive
t4691c928 = Neben dem Archiv

# Straight into its folder
t99d1df43 = Direkt in dessen Ordner

# When a file is already there
tca514dda = Wenn schon eine Datei da ist

# A file of the same name in the folder the files go to
t296ad686 = Eine Datei mit demselben Namen im Zielordner

# Largest an archive may unpack to
t5957c426 = Größe, auf die ein Archiv höchstens entpackt werden darf

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = Schützt vor Archivbomben: kleine Dateien, die entpackt den Datenträger füllen

# Only names every system can use
tf47f4968 = Nur Namen, die jedes System verwenden kann

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = Namen, die Windows nicht erlaubt, etwa a:b.txt, und Namen, die sich nur in Groß-/Kleinschreibung unterscheiden, werden auch auf diesem Computer abgefragt

# Owners from a backup
t38a7cf66 = Besitzer aus einer Sicherung

# Where a tar archive kept them: yours, or the archive’s by name or by number. Only an administrator gives files to others
t46908bee = Wo ein tar-Archiv sie behalten hat: deine, oder die des Archivs nach Name oder nach Nummer. Nur ein Administrator kann Dateien anderen geben

# Extended attributes from a backup
t764eed3a = Erweiterte Attribute aus einer Sicherung

# Where a tar archive kept them, ACLs among them
ta28f3ae5 = Wo ein tar-Archiv sie behalten hat, ACLs eingeschlossen

# Light
t8ad2f8cf = Hell

# Dark
t45570065 = Dunkel

# System
t8f3184bc = System

# Same as system ({language})
t5bd525d3 = Wie das System ({ $language })

# The system’s
t1d68c60b = Die des Systems

# Custom…
ta391dade = Benutzerdefiniert…

# The system’s ({size} px)
ta51f1877 = Wie im System ({ $size } px)

# All ({count})
tf65ec196 = Alle ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = z. B. code -n oder /path/to/editor --flag

# Language
t9a73db9b = Sprache

# Appearance
t31018b9f = Darstellung

# Text size
t2c041143 = Textgröße

# Processor cores to use
t2f9c3d17 = Zu nutzende Prozessorkerne

# Text editor
t1ca82aab = Texteditor

# What Edit opens a file of a storage in
t62506ced = Womit „Bearbeiten“ eine Datei eines Speicherorts öffnet

# The file’s path is added at the end.
teba749d4 = Der Pfad der Datei wird am Ende angehängt.

# Paste your license key
t3c4da5cf = Füge deinen Lizenzschlüssel ein

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = Alle Funktionen außer dem Signieren von Dateien, und { $count } Speicherort im Netzwerk. Ordner auf diesem Computer sind nicht begrenzt, und jede Signatur lässt sich prüfen.

# Every feature, and any number of storages.
tc3e75ecf = Alle Funktionen und beliebig viele Speicherorte.

# Every feature, and any number of storages, for your organization.
t69b29f66 = Alle Funktionen und beliebig viele Speicherorte, für deine Organisation.

# Plan
t1eab5e32 = Tarif

# Issued to
t44ef5cad = Ausgestellt für

# Who this license is for
td514fb0d = Für wen diese Lizenz gilt

# Seats
t8f706321 = Lizenzplätze

# What your organization’s agreement covers
t60b0c6c9 = Was der Vertrag deiner Organisation abdeckt

# Trial, until {date}
t9ffc2b63 = Testversion, bis { $date }

# Until {date}
t4d9bac91 = Bis { $date }

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = Abgelaufen am { $date }. Sie funktioniert noch { $days } Tage weiter.

# Ended {date}
tfccb9305 = Abgelaufen am { $date }

# Valid
td2661011 = Gültig

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = Danach ist packwing Free. Nur das Hinzufügen von Speicherorten entfällt.

# Your organization manages this license.
te42a0ee6 = Deine Organisation verwaltet diese Lizenz.

# Activate
t3b8f0190 = Aktivieren

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = Ein Schlüssel wird auf diesem Computer geprüft. Nichts wird irgendwohin gesendet, und keine Lizenz ändert, was die Richtlinien deiner Organisation bewirken.

# On
t60e47df0 = An

# They show when you right-click files, folders and archives.
te216d656 = Sie erscheinen, wenn du Dateien, Ordner und Archive mit der rechten Maustaste anklickst.

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = packwing ist gerade nicht im Menü: Diese kommen hinein, wenn es drin ist.

# Changing the menu…
tc7f1a2f0 = Menü wird geändert…

# Good to know
t8d7b3a90 = Gut zu wissen

# The menu could not be changed: {why}
t9fac5c06 = Das Menü ließ sich nicht ändern: { $why }

# Show packwing in the right-click menu
t5d0d23cc = packwing im Kontextmenü anzeigen

# Extract and compress from any folder
t94a46acb = Aus jedem Ordner entpacken und komprimieren

# Items in the menu
t67537dc4 = Einträge im Menü

# Put them in a submenu
te3449a65 = In ein Untermenü legen

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 fasst sie unter einem Eintrag „Packwing“ zusammen, im eigenen Menü und unter „Weitere Optionen anzeigen“

# One packwing item, with these under it
tb5fdc584 = Ein packwing-Eintrag mit diesen darunter

# 30 days
t7a572af9 = 30 Tage

# 90 days
t2d8a9817 = 90 Tage

# 1 year
t4d0a98a5 = 1 Jahr

# Keep all
te0b1775b = Alles behalten

# Keep the “downloaded from the Internet” mark
tfe574fa2 = Die Markierung „aus dem Internet heruntergeladen“ beibehalten

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = Das System warnt vor dem Ausführen von Dateien aus einem heruntergeladenen Archiv

# This system has no such mark, so this changes nothing here
t314d60c1 = Dieses System kennt keine solche Markierung, daher ändert das hier nichts

# License
t38e70ee4 = Lizenz

# Your organization’s, in Settings › License
tff7c594e = Die deiner Organisation, unter Einstellungen › Lizenz

# Treat files from storages as from the Internet
t0af8d466 = Dateien aus Speicherorten wie aus dem Internet behandeln

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = Was von S3, WebDAV und SFTP kommt, bekommt die Markierung ebenfalls

# Require encryption for new archives
t0c3e1337 = Verschlüsselung für neue Archive verlangen

# Applies to archives made and files added here, not to other apps
tc889f3a2 = Gilt für hier erstellte Archive und hinzugefügte Dateien, nicht für andere Apps

# Kinds of encryption allowed
t1b8b2116 = Erlaubte Verschlüsselungsarten

# New archives can be encrypted only these ways
t71dcd29c = Neue Archive dürfen nur auf diese Arten verschlüsselt werden

# Minimum password length
tb7d64a44 = Mindestlänge des Passworts

# In characters, for every new archive with a password
tca6d6982 = In Zeichen, für jedes neue Archiv mit Passwort

# Keep an audit log
tf7fd8762 = Ein Audit-Protokoll führen

# A record of what packwing does, whatever the license
t34fe4e25 = Was packwing tut, festgehalten, unabhängig von der Lizenz

# Where the audit log is kept
tbd999abe = Wo das Audit-Protokoll liegt

# A folder your organization chose
td4e1f74a = Ein Ordner, den deine Organisation gewählt hat

# How long the audit log is kept
td504acaf = Wie lange das Audit-Protokoll aufbewahrt wird

# In days; 0 keeps every record
t127a3c54 = In Tagen; 0 behält jeden Eintrag

# Most the audit log may take
ta334bb40 = Höchstgröße des Audit-Protokolls

# In MB; the oldest records go first
t03a543d2 = In MB; die ältesten Einträge gehen zuerst

# What the audit log says of archives
t8d6fca43 = Was das Audit-Protokoll über Archive festhält

# “entries” names the files found damaged or left out, as well as counting them
tf1125538 = „entries“ nennt die Dateien, die beschädigt oder ausgelassen wurden, und zählt sie

# packwing has no policy of this name.
t7e03649c = packwing hat keine Richtlinie mit diesem Namen.

# Ask before opening files without extracting
t76d5d951 = Vor dem Öffnen von Dateien ohne Entpacken fragen

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = Beim Öffnen einer Datei in einem Archiv wird eine Kopie in einem temporären Ordner abgelegt. Bei einem Programm wird immer gefragt.

# Paste a public key
t0bf72f73 = Einen öffentlichen Schlüssel einfügen

# What packwing does, kept on this computer: archives extracted, made, converted and copied, when and by whom. Never a password or what is in a file.
t2162955d = Was packwing tut, auf diesem Computer festgehalten: welche Archive entpackt, erstellt, umgewandelt und kopiert wurden, wann und von wem. Nie ein Passwort oder der Inhalt einer Datei.

# Keep its records for
tf423722f = Einträge behalten für

# Older ones are taken away, and that is recorded too
t7073f273 = Ältere werden entfernt, und auch das wird festgehalten

# Takes a Pro or Enterprise license: Settings › License.
t2774cee6 = Erfordert eine Pro- oder Enterprise-Lizenz: Einstellungen › Lizenz.

# Where the audit log is
t2702fcc6 = Wo das Audit-Protokoll liegt

# Open folder
t713a842f = Ordner öffnen

# The last record could not be written: {why}
t542148a1 = Der letzte Eintrag konnte nicht geschrieben werden: { $why }

# Audit log
tcb83f9ca = Audit-Protokoll

# pkw audit verify checks that no record was changed or taken out. Whoever runs this computer can still remove the log: it tells what packwing did, it does not stop anyone.
t482581ff = pkw audit verify prüft, dass kein Eintrag geändert oder entfernt wurde. Wer diesen Computer verwaltet, kann das Protokoll trotzdem löschen: Es zeigt, was packwing getan hat, es hält niemanden auf.

# This is not a public key packwing can check with: {why}.
td57933b9 = Das ist kein öffentlicher Schlüssel, mit dem packwing prüfen kann: { $why }.

# This line cannot be read, so the key is not used
t90e7dd7b = Diese Zeile lässt sich nicht lesen, daher wird der Schlüssel nicht verwendet

# Keys you trust
ted99ced8 = Schlüssel, denen du vertraust

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = Eine Signatur mit einem dieser Schlüssel sagt, wer eine Datei signiert hat. Eine mit einem anderen Schlüssel beweist nichts, egal was sie trifft.

# Choose a file…
t28a576c9 = Datei auswählen…

# {name} (weak)
tdb06c037 = { $name } (schwach)

# Checksums
t47dd48d3 = Prüfsummen

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = Was „Prüfsummen…“ im Menü einer Datei zeigt, zum Vergleich mit der Prüfsumme, die dort angegeben ist, wo die Datei herkommt. Eine schwache erkennt eine beschädigte Datei, aber keine absichtlich veränderte.

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = Ihre Richtlinien lassen sich nicht lesen, daher verweigert, was sie schützen: { $why }

# Some settings are managed by your organization
t7db7e7ec = Manche Einstellungen werden von deiner Organisation verwaltet

# Get help
t4a1ed54a = Hilfe

# Not understood
t068eea1b = Nicht verstanden

# Off
t0dc6b4aa = Aus

# General
t5561fd0b = Allgemein

# Extracting
tc82af3f2 = Entpacken

# Compressing
t6d183c73 = Komprimieren

# Security
tc2fe21db = Sicherheit

# Right-click menu
t1287ae97 = Kontextmenü

# About
t58e096de = Über

# Settings
t4b058728 = Einstellungen

# This computer has no way to ask that it is you.
t0c1b4d43 = Dieser Computer kann nicht bestätigen lassen, dass du es bist.

# open a storage
t780455bf = einen Speicherort zu öffnen

# Open a storage in packwing
tf861e014 = Einen Speicherort in packwing öffnen

# That was not it. It stays locked.
tbb2a9838 = Das war es nicht. Er bleibt gesperrt.

# The system could not ask: {why}
td6905afe = Das System konnte nicht nachfragen: { $why }

# This copy of packwing cannot open it.
tfb56d8cf = Diese Kopie von packwing kann ihn nicht öffnen.

# Storages
tc776c52f = Speicherorte

# Add storage
te858d6b5 = Speicherort hinzufügen

# Type the password you log in to this computer with to open it.
tb544cd9b = Gib zum Öffnen das Passwort ein, mit dem du dich an diesem Computer anmeldest.

# Show this computer it is you to open it.
tfb4c635b = Bestätige diesem Computer, dass du es bist, um ihn zu öffnen.

# Checking…
t87f62cb7 = Wird geprüft…

# It asks again after 5 minutes left unused.
t69c22a3b = Nach 5 Minuten ohne Nutzung wird erneut gefragt.

# Edit…
t8e7efcd7 = Bearbeiten…

# Remove from this list
t8c7c7c57 = Aus dieser Liste entfernen

# Extract here with packwing
tc5e54e4a = Mit packwing hier entpacken

# Extract to new folder
tda62691f = In neuen Ordner entpacken

# Extract with packwing…
tda9255a0 = Mit packwing entpacken…

# Open with packwing
teb657427 = Mit packwing öffnen

# Compress to ZIP
t344b1175 = In ZIP komprimieren

# Compress with packwing…
t9ec9b77d = Mit packwing komprimieren…

# packwing cannot find its program: {e}
t1934ce08 = packwing findet sein Programm nicht: { $e }

# Signed by {signer}
t8eb35b10 = Signiert von { $signer }

# Signed by {signer}, not checked yet
t88b7186a = Signiert von { $signer }, noch nicht geprüft

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = Signiert mit einem Schlüssel, dem du nicht vertraust: { $fingerprint }

# It does not match its signature
tb7d0e1ed = Es passt nicht zu seiner Signatur

# Its signature cannot be read
tadf2a401 = Seine Signatur lässt sich nicht lesen

# Its signature could not be checked
t69cbbb51 = Seine Signatur konnte nicht geprüft werden

# Checking its signature…
tda8c2e03 = Signatur wird geprüft…

# the system's package manager
t11465c20 = der Paketmanager des Systems

# the App Store
ta8cbd72e = der App Store

# the Microsoft Store
t1df799a1 = der Microsoft Store

# Downloading… {got} of {total}
taee668c2 = Wird heruntergeladen… { $got } von { $total }

# Downloading… {got}
t4873b921 = Wird heruntergeladen… { $got }

# just now
ta715ef0f = gerade eben

# {n} minute ago | {n} minutes ago
t6c7302d6 =
    { $n ->
        [one] vor { $n } Minute
       *[other] vor { $n } Minuten
    }

# {n} hour ago | {n} hours ago
t137b052e =
    { $n ->
        [one] vor { $n } Stunde
       *[other] vor { $n } Stunden
    }

# {n} day ago | {n} days ago
tb9fa7744 =
    { $n ->
        [one] vor { $n } Tag
       *[other] vor { $n } Tagen
    }

# No folder to download to.
t2bfc1370 = Kein Ordner zum Herunterladen.

# Update available
t676409df = Update verfügbar

# Restart to update
tbbbbda26 = Zum Aktualisieren neu starten

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
        [one] { $n } Datei wurde übersprungen, da ihr Name vergeben war.
       *[other] { $n } Dateien wurden übersprungen, da ihre Namen vergeben waren.
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
        [one] { $n } Link blieb, wo er war.
       *[other] { $n } Links blieben, wo sie waren.
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = Diese Datei ist mit einer Passphrase verschlüsselt.

# This archive needs its password.
td46b69a1 = Dieses Archiv braucht sein Passwort.

# This file is encrypted to a key.
tbd048b4e = Diese Datei ist für einen Schlüssel verschlüsselt.

# This key file needs its passphrase.
t81f4c775 = Diese Schlüsseldatei braucht ihre Passphrase.

# {paths} archives
t27f3b860 = { $paths } Archive

# {folder}, a folder for each
t75e7db7c = { $folder }, je ein Ordner

# New
t8b242db1 = Neu

# There is no file to save it beside.
ta9906e3b = Es gibt keine Datei, neben der gespeichert werden kann.

# Nothing in the archive matches the patterns given.
t90b19482 = Nichts im Archiv passt zu den angegebenen Mustern.

# Nothing to compress.
t47339885 = Nichts zu komprimieren.

# {path} is too big.
te9ab879e = { $path } ist zu groß.

# This format holds one file, and no more.
t75c197d9 = Dieses Format enthält eine Datei und nicht mehr.

# Store
t6da32dce = Speichern

# Lowest
t5d7dbfdb = Niedrigste

# Low
tb1dd9521 = Niedrig

# Medium
t09be5f96 = Mittel

# High
tbe0a061d = Hoch

# Highest
t9487b22f = Höchste

# Could not read {path}: {e}
tad0120eb = { $path } konnte nicht gelesen werden: { $e }

# {path} has no name.
tbd0e16dc = { $path } hat keinen Namen.

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = Der Name von { $path } ist kein gültiges Unicode und kann nicht in ein Archiv.

# Could not write {path}: {e}
t8dd055c6 = { $path } konnte nicht geschrieben werden: { $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } und die Namen danach sind vergeben.

# Could not encrypt: {e}
t485d71ab = Verschlüsseln nicht möglich: { $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = { $level } braucht etwa { $needed } Arbeitsspeicher, frei sind { $available }: Es wird mit { $lower } erstellt, das etwa { $less } braucht.

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = Diese Stufe braucht etwa { $needed } Arbeitsspeicher, frei sind { $available }: Der Computer kann langsamer werden oder packwing anhalten.

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = Ein ZIP- oder 7z-Archiv nimmt age-Schlüssel nur, wenn age es als Ganzes verschlüsselt.

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age verschlüsselt mit einer Passphrase oder mit Schlüsseln, nicht mit beidem.

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = Post-Quanten-Schlüssel lassen sich nicht mit anderen Schlüsseln verwenden: Die anderen ließen die Datei für einen Quantencomputer offen.

# There are no keys in this file to encrypt to.
tc2809b2c = In dieser Datei gibt es keine Schlüssel, für die verschlüsselt werden kann.

# {path} got shorter while it was being read.
t2e221d1f = { $path } wurde beim Lesen kürzer.

# A password has to be valid Unicode.
t5ef729bc = Ein Passwort muss gültiges Unicode sein.

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = Die erweiterten Attribute von { $path } konnten nicht gelesen werden: { $e }

# Could not start compressing: {e}
tc4faa70c = Komprimieren ließ sich nicht starten: { $e }

# Could not compress: {e}
tf6ff23f2 = Komprimieren nicht möglich: { $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } ist ein Link, und Links bleiben nur unter Unix erhalten.

# Nothing changed.
tedfc294c = Nichts wurde geändert.

# The archive did not come out whole.
t2f015c79 = Das Archiv ist nicht vollständig entstanden.

# packwing can change only a ZIP archive that is one file.
t48628036 = packwing kann nur ein ZIP-Archiv ändern, das aus einer Datei besteht.

# That is not in the archive.
t7530c419 = Das ist nicht im Archiv.

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = Das ist alles, was das Archiv enthält. Lösche stattdessen das Archiv selbst.

# “{to}” cannot be a name.
td53e55bf = „{ $to }“ kann kein Name sein.

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = „{ $to }“ ist schon in diesem Ordner. Nichts wurde geändert.

# packwing can add files only to a ZIP archive that is one file.
t11a330d3 = packwing kann nur zu einem ZIP-Archiv, das aus einer Datei besteht, Dateien hinzufügen.

# An archive can’t go inside itself.
tff43ce74 = Ein Archiv kann nicht in sich selbst liegen.

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = „{ $bare }“ ist schon im Archiv. Nichts wurde geändert.

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = Vor dem Hochladen wird dieses Archiv in { $folder } erstellt und braucht dort bis zu { $need }. Dort sind nur { $free } frei.

# The upload stopped.
t528232e6 = Der Upload wurde gestoppt.

# “{pattern}” names no file or folder.
te17b2bfa = „{ $pattern }“ bezeichnet keine Datei und keinen Ordner.

# a colon, which Windows does not allow
t3ba6a1aa = ein Doppelpunkt, den Windows nicht erlaubt

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = ein Zeichen, das Windows nicht erlaubt (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = ein Name, den Windows für ein Gerät reserviert

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = endet auf ein Leerzeichen oder einen Punkt, was Windows nicht erlaubt

# longer than 255 bytes
t9218e5c0 = länger als 255 Bytes

# a control character
t378bba3a = ein Steuerzeichen

# not a name this system can read
t72b79e1d = kein Name, den dieses System lesen kann

# the same file here as another, its name differing only in case
t1eead41f = hier dieselbe Datei wie eine andere, der Name unterscheidet sich nur in Groß-/Kleinschreibung

# a link that leads outside the folder
t00fdc25e = ein Link, der aus dem Ordner hinausführt

# {name} is already in the folder.
tbe7fab8a = { $name } ist schon im Ordner.

# Could not make a folder in {parent}.
tec7ca3a9 = In { $parent } ließ sich kein Ordner erstellen.

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } ist ein Ordner, und eine Datei würde ihn ersetzen.

# {name} is already in the folder
t5af584a0 = { $name } ist schon im Ordner

# This file is not an archive packwing can open.
t660f149b = Diese Datei ist kein Archiv, das packwing öffnen kann.

# The password is wrong, or the archive is damaged.
ta4d3227d = Das Passwort ist falsch, oder das Archiv ist beschädigt.

# The passphrase is wrong.
t7826c649 = Die Passphrase ist falsch.

# The key file’s passphrase is wrong.
t8ad75e3a = Die Passphrase der Schlüsseldatei ist falsch.

# This file is encrypted to a key packwing does not have.
te162af76 = Diese Datei ist für einen Schlüssel verschlüsselt, den packwing nicht hat.

# Cancelled.
t27c010d6 = Abgebrochen.

# the archive could not be read
t2267c3b1 = das Archiv ließ sich nicht lesen

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = Das lässt sich nicht entpacken, da der Name einer Datei hier nicht verwendet werden kann: „{ $name }“ ({ $why }).

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
        [one] { $n } Byte
       *[other] { $n } Bytes
    }

# A volume of this archive is not there.
t3803103c = Ein Teil dieses Archivs fehlt.

# {named}: its data is damaged.
t066227f9 = { $named }: Die Daten sind beschädigt.

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = Das Archiv ist beschädigt: UnRAR würde mit { $number } enden.

# There is no public key here.
t9fd24078 = Hier gibt es keinen öffentlichen Schlüssel.

# Could not make the key: {e}
t73610ecc = Der Schlüssel ließ sich nicht erstellen: { $e }

# There is no file to sign.
tc95e3a9c = Es gibt keine Datei zum Signieren.

# {name} is not there any more.
t07c1f2bc = { $name } ist nicht mehr vorhanden.

# A part has to hold at least one byte.
te779b111 = Ein Teil muss mindestens ein Byte enthalten.

# {path} is not a file.
tdfee38b2 = { $path } ist keine Datei.

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = Das ergäbe mehr Teile, als sich nummerieren lassen: Wähle größere.

# {part} is there already.
t2577cbb6 = { $part } gibt es schon.

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path } ist kein Teil einer Datei, die wie bei 7-Zip geteilt wurde, etwa name.001.

# {part} is missing: the parts start from it.
t5e65a10a = { $part } fehlt: Mit ihm beginnen die Teile.

# {part} is missing, and the parts after it are there.
t24165c89 = { $part } fehlt, die Teile danach sind aber da.

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = { $part } enthält { $got }, nicht { $size } wie der erste Teil: Er ist vielleicht nicht vollständig.

# The name is empty.
t1a9d8e4e = Der Name ist leer.

# That name is kept for the system.
t74c8683a = Dieser Name ist für das System reserviert.

# A name cannot hold / or other special characters.
t4c0596ec = Ein Name darf kein / oder andere Sonderzeichen enthalten.

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = Unter Windows darf ein Name nicht mit einem Punkt oder Leerzeichen enden.

# That name is taken in this folder.
t43a2bce1 = Dieser Name ist in diesem Ordner vergeben.

# {name} is already there.
tb5e7b134 = { $name } ist schon da.

# {name} cannot go inside itself.
t99f437a6 = { $name } kann nicht in sich selbst liegen.

# Stopped.
t15558940 = Gestoppt.

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } ist an einer Stelle eine Datei und an der anderen ein Ordner.

# The copy of {name} does not match. The original was kept.
t779a64f3 = Die Kopie von { $name } stimmt nicht überein. Das Original blieb erhalten.

# This is not a valid packwing license key.
td5ae978f = Das ist kein gültiger packwing-Lizenzschlüssel.

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = Die Richtlinie deiner Organisation dazu lässt sich nicht lesen, daher tut packwing es nicht. Frag deine IT-Abteilung.

# Your organization requires new archives to be encrypted.
t83acf110 = Deine Organisation verlangt, dass neue Archive verschlüsselt werden.

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = Deine Organisation erlaubt diese Art der Verschlüsselung nicht: { $method }.

# Your organization requires a password of at least {least} characters.
t2d21b125 = Deine Organisation verlangt ein Passwort mit mindestens { $least } Zeichen.

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = Dieses Passwort ist zu leicht zu erraten, und deine Organisation verlangt ein besseres.

# Expected true or false, not {other}.
tcadcff7a = Erwartet wurde true oder false, nicht { $other }.

# Expected a number from 1 to {most}, not {other}.
tda498fad = Erwartet wurde eine Zahl von 1 bis { $most }, nicht { $other }.

# Expected some text, not {other}.
t9386ce3b = Erwartet wurde Text, nicht { $other }.

# Expected a whole path, such as /var/log/packwing or \\server\audit, not {other}.
t00d3565d = Erwartet wurde ein vollständiger Pfad wie /var/log/packwing oder \\server\audit, nicht { $other }.

# Expected a number of days from 0 (keep all) to {most}, not {other}.
tf8fba675 = Erwartet wurde eine Anzahl Tage von 0 (alles behalten) bis { $most }, nicht { $other }.

# Expected a size in MB from 0 (no limit) to {most}, not {other}.
t9f7e002d = Erwartet wurde eine Größe in MB von 0 (unbegrenzt) bis { $most }, nicht { $other }.

# Expected summary or entries, not {other}.
t27e9b96f = Erwartet wurde summary oder entries, nicht { $other }.

# Expected an address starting with https://, not {other}.
t93bfd19e = Erwartet wurde eine Adresse, die mit https:// beginnt, nicht { $other }.

# Expected a list, not {other}.
tc945cc93 = Erwartet wurde eine Liste, nicht { $other }.

# No kind of encryption is called {name}.
tfddb7ba6 = Es gibt keine Verschlüsselungsart namens { $name }.

# The list names no kind of encryption.
t5f8076f5 = Die Liste nennt keine Verschlüsselungsart.

# The file is too big to be a list of policies.
taf68e582 = Die Datei ist zu groß für eine Liste von Richtlinien.

# The file has to hold one JSON object.
tf483aa65 = Die Datei muss ein JSON-Objekt enthalten.

# {path} has to belong to root.
tfc77ea2b = { $path } muss root gehören.

# {path} can be changed by others than root.
t0d4a4498 = { $path } kann von anderen als root geändert werden.

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = Dieser Speicherort nimmt vorerst nur Dateien bis { $size } an, da jede beim Senden im Speicher gehalten wird.

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = Dieser Speicherort kann einen Ordner nicht in einem Zug umbenennen, da alles darunter kopiert würde. Nutze „Verschieben nach…“.

# S3 bucket
tc5e013b3 = S3-Bucket

# Bucket
te34e3557 = Bucket

# Region
t492e789d = Region

# Endpoint
t27e7d69c = Endpoint

# Empty for AWS, or https://…
td3a94af8 = Leer für AWS, oder https://…

# Start in
t5eb91ea6 = Starten in

# A prefix, if not the whole bucket
t75255a6c = Ein Präfix, falls nicht der ganze Bucket

# Access key ID
tf9408d75 = Zugriffsschlüssel-ID

# Secret access key
te1c3d3a2 = Geheimer Zugriffsschlüssel

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = Host oder ssh_config-Name

# Empty for the home folder
t67b5f42b = Leer für den Home-Ordner

# Empty to use the SSH agent and keys
tf10e6293 = Leer, um den SSH-Agent und Schlüssel zu verwenden

# WebDAV folder
tb2401428 = WebDAV-Ordner

# Address
t700560f3 = Adresse

# User
te0a63e12 = Benutzer

# Password or app token
t1c18cb9b = Passwort oder App-Token
