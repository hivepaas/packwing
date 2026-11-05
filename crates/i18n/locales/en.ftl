# packwing's English, from the code (scripts/i18n.py update writes it:
# do not edit). The source for translations.

t17634ee0 = Appearance: { $name }. Click for { $name2 }.
tdc8b782b = Stop and say so
t1e17fc0c = Replace it
tad9909cb = Skip it
t9dc3aa14 = Add
t369f9d4d = The top of { $name }
t48b969a8 = Choose what to add
t534e422e =
    { $n ->
        [one] { $count } item
       *[other] { $count } items
    }
tde674bd8 =
    { $n ->
        [one] { $count } file
       *[other] { $count } files
    }
t0d53ac5e = Add to { $name }
t36e95c22 = Goes in
t34c69cec = If a file of that name is there
t55ed1cfd = This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t52b1d223 = The new files are encrypted with the archive’s password.
t35afca3b = Cancel
t0fe07306 = Name
t06357afd = Folder
t2af0c8ab = Its secrets could not be read: { $why }
ta4073fff = That folder is not on this computer.
ta213827f = Its secrets could not be kept: { $why }
te075bcbc = Local folder
t214cdad5 = Trying it…
t3bd5bc9f = It answers.
t0ac8fa44 = It did not answer: { $why }
t7bae68d0 = Edit storage
t4fe8f806 = Add a storage
t3f9544ac = Secrets go in the system’s keychain, not with the settings.
t4e4acf9a = Its files stay where they are.
t7a7d61e9 = Ask for this computer’s password to open it
t1b5ad6b1 = For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t89f1bf95 = packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
te0f3b6b7 = Test connection
ta24f28a2 = Enter a license key
t4d2d5d68 = Save
t8333758c = Paste the checksum given with the file
tb45ae457 = They could not be saved: { $why }
t5daa8455 = The file could not be read: { $why }
t6aad8485 = Reading…
t8dc21305 = Copied
t658f3664 = Copy
tedbc2ec9 = weak
t231f838a = There is no checksum in this.
t65dd697c = This is not a checksum packwing makes: it has { $digits } digits.
t4f0350e4 = It matches the file’s { $name } checksum.
tcb5c1745 = It is compared once the file is read.
t55d6aeb7 = It does not match the file’s { $name } checksum: this is not the file it was made of, or the file changed on the way.
t3d713e04 = It may be a { $name } checksum, which is not made yet.
tcbefe458 = Make it
tbddcfc14 = Compare with
t80036d1c = Reading… { $size } of { $size2 }
t1569c388 = Reading… { $size }
te88f3ed9 = Checksums of { $name }
t8cca3c05 = All of the file passes through this computer, and your provider may charge for what is read from it.
t164fdec6 = A weak checksum ({ $names }) still tells a damaged file, but not one changed on purpose, which can be made to match.
t1d91952b = Saved beside it as { $files }, where sha256sum -c and the like check it without packwing.
tec85be80 = Compare…
tbd79b37d = Saving…
t7f4d85e3 = Save beside it
t80fb8482 = Read it
t8dd31791 = Done
tcf390b0a = Nothing chosen yet
t8399ef70 = { $join } and { $more } more
t5a420780 = Hide what goes in
t9a20eb53 = Show what goes in
t90a93ec7 = Take it out
t6a4aedbe = Drop them to add them
t3b7c797a = What goes in
tba8a1e71 = + Files
tcef8b0ce = + Folder
t2e5c8c3e = Add the files or a folder to put in the archive.
tf7f64b21 = Windows 11 and Linux open 7z on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
t051031b1 = Linux, macOS and Windows 11 open it on their own; phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tc2c29183 = Linux and Windows 11 open it on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
te9afe1c2 = Most computers need an archiver app or the lz4 tool to open it. To share with anyone, pick .zip.
t16463509 = It holds this one file. Command-line tools open it; Windows and macOS may need an archiver app. To share with anyone, pick .zip.
t6da32dce = Store
t5d7dbfdb = Lowest
tb1dd9521 = Low
t09be5f96 = Medium
tbe0a061d = High
t9487b22f = Highest
t56ee6829 = Don’t split
tc6ba1c9c = 25 MB (email)
tfc6b3ea5 = 4 GB (USB stick)
t0fcf9e2d = Custom size…
ta6478e7c = Size
t2cc30838 = Password
t76cb89a8 = age1…, age1pq1… or ssh-ed25519 …
ta1f147a6 = Save here
t1a01e439 = Use these keys
t2e9c9e78 = No compression
te2db10f6 = { $name } compression
ta999efe2 = Parts of 25 MB
t760df467 = Parts of 4 GB
tdd286479 = Parts of { $size }
tafd4b17d = Parts of a size to type
t9981f45f = Public key
t757989af = Beside the files
teae4ef1a = What it holds, from
tf2d34837 = Save in
tc8abcd43 = Folder…
td0d8c806 = Storage…
tdc045fbd = Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tb6b5d19b = { $ext } is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
ted6f948e = It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
t91729834 = { $place } takes files of up to { $most }. An archive larger than that fails when it starts to upload.
td7627e48 = Choose a storage, and a folder in it
t1926d127 = New folder
ta598b15b = Give it a name, without / or \ in it.
t9451c3b1 = A file of that name is there already: this one will be { $name }.
t0da27ae4 = Some names can’t be used on Windows or macOS, such as “{ $name }”. There, those files may not extract as they are.
tae2a96e9 = Hide advanced
t701d5ed9 = Advanced options
td386c9ef = Compression level
t076ef350 = Store puts the files in as they are: the quickest to make, and the largest.
t273a832f = Higher makes a smaller archive, which takes longer to make, not to open.
tca4436bd = Split into parts
t642c444d = Each part
t2fd629a2 = Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t804e2099 = A part takes a size of 1 MB or more, such as 100 MB.
t22784ad9 = Encrypt with age
tae8d8740 = Protect with a password
ta6e44ce5 = Suggest
t8eb937cf = Keep this somewhere safe: without it, this archive can never be opened.
t62b19318 = Type a password, or turn this off.
t2198d77b = Hide file names too
t659d3827 = With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
tdc4a7daf = Saved as { $name }. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
t575a1a6c =
    { $n ->
        [one] { $n } key from { $name }
       *[other] { $n } keys from { $name }
    }
t21a5901d = Remove
tbb501800 =
    { $n ->
        [one] { $n } key in it was left out, of a kind age does not encrypt to.
       *[other] { $n } keys in it were left out, of a kind age does not encrypt to.
    }
t40d7c944 = From a file…
tcb028b06 = Type a public key, or choose a file of them.
ta856ab34 = A post-quantum key: to open this, the age tool must be version 1.3 or newer.
t39644118 = Only the private key that goes with it opens this, and nothing can if that key is lost.
t6de87ae9 = Weak
t82fa0d53 = Fair
t0dad409a = Strong
t5461cfa6 = Choose what to put in it
t92e29218 = Convert
tf352befb = New archive
t990de47d = Create
tfff7fd6c =
    { $n ->
        [one] { $n } item
       *[other] { $n } items
    }
tdf5a8d53 = Move { $what } to the Trash?
ta297d6c5 = Delete { $what }?
te5e137a1 = It can be put back from the system’s Trash.
t3947dab4 = This cannot be undone from packwing.
t6f358d35 =
    { $n ->
        [one] and { $n } more
       *[other] and { $n } more
    }
ta036661b = Move to Trash
t5797ea6a = Delete
t49912bd4 = { $many } items
t83401013 = Rename { $what }
taf4cad88 = New name
t33cb3e89 = It goes from the archive for good, folders with all they hold. This can’t be undone.
t8f575534 = This archive is signed. After this change it will no longer match its signature, until it is signed again.
tc80631f3 = Rename
t8a168799 = The archive’s password
td571ce35 = one folder
t95c9ec7b = one file
t63f6caf1 = { $location } › { $one } · { $kind }, so right here
t2380eff7 = { $new_folder } · { $counted }, so in a new folder
t354232ec = Extract here
t9addc37e = { $picked } selected
t649feaf8 = All { $counted }
td12e23bf = What should it extract?
t54535972 = Smart
t7865024c = Recommended
td35f540c = Where should the files go?
t900a8a39 = Right here
t9f128ba8 = Another folder
t58daae9c = Choose…
t8195aed2 = Goes to { $destination }
tb6c9570b = Checked as it goes
t11424797 = · Nothing leaves the folder or passes the size limit
t60cb2adf = { $name } came from the Internet. It will stay marked, so you’ll be asked before it runs.
t834ba3c0 = Extract archive
t30518e1e = Extract
tcfe84c2f = Open { $name }?
t5b9fc10b = packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
tdc4d48bd = This is a program: opening it runs it. Only do so if you trust where the archive came from.
t00c5d168 = Don’t ask again
t72430853 = Run it
t538b10e9 = Open
t45a8991d = Open from a storage
t3c900950 = Choose a storage, and an archive in it
t73c66160 = Choose
tcf6986a7 = This is not a key file packwing can use: { $why }.
t1dc6961f = Could not read { $path }: { $e }.
t1e921d00 = The passphrase
t4051f4d7 = The key file’s passphrase
tbd0672da = Enter the passphrase
t191a2dc5 = Passphrase
t663ac047 = This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t513ea7ab = Enter the password
t0f356f7c = This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t3c521a4e = Choose the key file
t6ed7c612 = Key file
tcda9a50a = This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
t808b85c5 = Enter the key file’s passphrase
t2a39d1e8 = The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t001f6992 = kept, { $size }
t73aeedd4 = left out
t7aa9b778 = renamed { $to }
t57a78885 = The job stopped before it was done.
t16ec36d9 = Waiting for other jobs…
t8b1230ed = A file is damaged
t26835ec8 = Some names can’t be used here
t54ef3472 = Converted, but some files are damaged
t8226d624 = Extracted, but some files are damaged
t5cce0e50 = Converted, with some files left out or renamed
t5dc002d6 = Extracted, with some files left out or renamed
t94279f82 = Extracting…
tc75fe478 = Uploading…
t01fcd896 = Creating…
t2060c750 = Adding…
t186fbef5 = Changing the archive…
tc40f2321 = Opening…
t19d1cb65 = Testing…
t02dd17e0 = Copying…
t890f7ed1 = Moving…
t7b8f8780 = Signing…
t5452adf5 = Splitting…
taa884c51 = Joining…
ta1b8f4cc = Converting…
t83d27d26 = Couldn’t split the file
t1cf67e00 = Couldn’t join the parts
t39f973a9 = Couldn’t convert the archive
t4489f97c = Couldn’t sign
t613e4e9c = Couldn’t copy
t9796adec = Couldn’t move
tacd22c56 = Couldn’t extract
t9d0e01bc = Couldn’t create the archive
tfbd989b0 = Couldn’t add the files
tcdc4a49c = Couldn’t change the archive
t041198a4 = Couldn’t open the file
tfcd070a5 = The test failed
tbe2a0509 = Starts when one of the others ends
tde281feb = Paused
t73727cef = Starting…
ta550a6a9 = Almost done
t3045da16 = About { $secs } seconds left
tf727815c = About a minute left
tbd7a3305 = About { $div_ceil } minutes left
t0d56ec7c = { $name } · checking that every file reads
te728628b =
    { $n ->
        [one] and { $count } more
       *[other] and { $count } more
    }
tdf0ce240 = The test passed
tc1421590 =
    { $n ->
        [one] { $count } file read through, { $size }, with no errors.
       *[other] { $count } files read through, { $size }, with no errors.
    }
tcd86acc3 = Close
t145d7a09 = “{ $path }” is damaged: { $why }. Stop, leave it out, or keep what could be read of it?
t798693ad = Do the same for the other damaged files
t4b7f7705 = Stop
taaff8ec9 = Leave it out
tc4387d25 = Keep what was read
tb75acfb1 =
    { $n ->
        [one] { $count } file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it?
       *[other] { $count } files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
    }
ta50cc084 = Leave out
t85469b0f =
    { $n ->
        [one] { $count } file’s data was damaged.
       *[other] { $count } files’ data was damaged.
    }
t2f0042b8 =
    { $n ->
        [one] { $count } file was left out.
       *[other] { $count } files were left out.
    }
tec751fa9 =
    { $n ->
        [one] { $count } file was renamed, as its name can’t be used here.
       *[other] { $count } files were renamed, as their names can’t be used here.
    }
ta7833fcb = The rest was extracted.
t3c49b269 = Copy the list
t27f1ab7d = { $count } of { $count2 } files · { $size } of { $size2 }
tf0f1017b = Open the folder when it’s done
tb800c0ef = Run in background
t454d1d8a = Resume
t44f9bf2d = Pause
tf2047916 =
    { $n ->
        [one] { $n } job is still running
       *[other] { $n } jobs are still running
    }
t2e6491b3 = Closing packwing stops them
tf2a64f8b = What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
t79c1b9c7 = Keep running
tf2707857 = Stop them and quit
td68bff45 = Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
t4850aaa8 = The folder and its files stay where they are.
t5dd9cfcf = Remove “{ $name }” from the list?
t80d51a99 = It can be added again at any time.
tf5593f05 = A passphrase for the new key
tad5c08e6 = The same passphrase again
t29db17cb = Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t020be41a = This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
tc5357c97 = That passphrase does not open this key.
t662c29f9 = This is not a key packwing can sign with: { $why }.
t335fe90b = The two passphrases are not the same.
t9306dafe = Keep the key here
te58c13f5 = No key chosen
tcd1ac90c = Key
t9e1134da = New key…
t58fb56ef = Again
tbdc850a3 = A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
t5e187bdd = The signature goes beside it as { $signature }, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t6035bc3f = The signature goes beside it as { $signature }, in place of one that is there.
ted250f31 = Sign { $name }
t04ecc8be = Anyone can check it, with the key’s public half
t6bba28d5 = Make the key…
t8f6340c4 = Sign
tab12cc19 = { $count } parts beside it, { $first } and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
t16a15383 = Split
tc168ca56 = Move { $what }
teda01f26 = Copy { $what }
t1aaf57aa = Move here
t85f0245a = Copy here
t7f2c2a34 = Move
tc99c037f = If a name is taken
t5318c14e = They are in this folder already. Choose another.
t51019905 = All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t8ecbab4e = Each original goes once its copy has been read back and matches.
t5937f050 = The originals stay where they are.
t1b339d35 = Too big to show here
tb909bf3c = packwing can’t show this kind of file
tb7927177 = It could not be read
t26005fe3 = It could not be read: { $why }
tb60fb288 = It could not be shown
t5ef80ec5 = Open with default app
tc8c2bae0 = Another job is making { $name } there now: give this one another name, or wait for it to end.
t042cbb2a = Another job is extracting into { $folder } now: choose another folder, or wait for it to end.
tbf2a5d3a = no editor command
t6256b3da = no text editor found
t2c814039 = Search this folder
t2c6017b2 = This folder could not be read: { $why }
t466e87c4 = Could not load more: { $why }
te5951d38 = Could not delete: { $why }
t91cdab0c = the text editor
t8deb160a = Cannot start { $who }: { $why }
td8d39fb8 = Edit in text editor
te44cd57b = Edit in { $editor }
tdd13a6b2 = Modified
tc4e2f181 = Loading…
t39c5f536 = Only what is loaded was searched.
t41f9a787 = Search the whole folder
t30bed46a = Load { $n } more
t0815aad4 = Refresh
t3375c9c0 = Hide hidden files
tf18c7057 = Show hidden files
t5e4baa18 = View
t4a09490e = Extract…
t690f9553 = Compress…
t543cf15b = Copy to…
t7a32320b = Move to…
td6d61b10 = Sign…
t2891102d = Checksums…
tae85a7c4 = Convert…
t281db403 = Split into parts…
t0ad56db8 = Join the parts
tf165cb2b = Copy path
tad1b72f6 =
    { $n ->
        [one] Compress { $n } item…
       *[other] Compress { $n } items…
    }
t29a52578 = { $n } loaded, more in this folder
t037d01ae = Sorted among those loaded
tebda8468 = { $n } selected
t213e90fa = Try again
t0f9a8a58 = Nothing here matches that.
t0b82c83a = Add a storage to browse it.
t6a090cc3 = No folders or archives here.
tc7731c8f = No folders here.
td9e47db6 = This folder is empty.
ta9442aa9 = Show in Finder
t23c89eda = Show in File Explorer
tb347ece9 = Show in the file manager
te2a467e0 = Open in Finder
t3794d625 = Open in File Explorer
td1ceabc4 = Open in the file manager
t6aef5c37 = Copy all
tfd0ff0a1 = This system has no folder for settings.
t73690b65 = Archive
tb35c0c27 = Search inside this archive
ta413eca5 = Not text it can show
t81f5dbce = It needs the password to show it
tff74a552 = Open another…
tcfba8fd5 = From a storage…
t3c93bb70 = Safe to extract
t83223c27 = Reading… { $percent }%
t613dd932 = Read { $fetched } of { $whole }
tdd45c7bb = Add files
t2ffcbe05 = Test
t1bbda9f5 = Reading what’s inside…
t36c3e2f5 = Nothing in this archive matches “{ $query }”.
teecdfa3d = Nothing in this folder matches “{ $query }”.
t843253d0 = And { $more } more. Type more to narrow it down.
t8382bd3e = Open an archive
t4a498ac8 = ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…
t2cf2a8ee = Open an archive…
t37854216 = Choose a file to see it here
td7688f56 =
    { $n ->
        [one] { $count } executable in this folder will keep its Internet mark when extracted.
       *[other] { $count } executables in this folder will keep their Internet mark when extracted.
    }
tfa0d176a = This computer
t52f50eae = Home
t64215c53 = Desktop
t539307de = Downloads
tb255877b = Drop to open
t0eac1f07 = Browse
t50e888fe = Recent
t50aac02b = Choose a file to see its information here
tbb5a2d5f =
    { $n ->
        [one] { $n } selected
       *[other] { $n } selected
    }
t0cca93f1 = Files in them come to { $size }
t2fed07aa = Delete…
t4b408baf = Link to a folder
t8e08e589 = Link
ta56bbbe9 = { $tag } file
t72e46dc5 = { $files }, { $folders }
teee8d81a =
    { $n ->
        [one] { $n } file
       *[other] { $n } files
    }
te89ea942 =
    { $n ->
        [one] { $n } folder
       *[other] { $n } folders
    }
tc87daf59 = { $size } so far
t3ff50c63 = Kind
tffe19c31 = Holds
t8700fd83 = Permissions
teb66e456 = Path
tc049050d = Signature
t0f23db17 = Count size and files
t6d165412 =
    { $n ->
        [one] { $n } folder in it could not be read.
       *[other] { $n } folders in it could not be read.
    }
t9d44730e = Check the signature
t3250471f = Extracted
t22c69419 = Archive created
t82869bf0 = Moved
t27c3ab35 = Signed
tf8f2b670 = Joined
tc7b76e99 = Converted
t0e83ae32 = Waiting for a password or a key
tddb7844a = Enter it…
t29742e5c = Show
t4ae6a79d = Jobs
t30c178e8 = Clear finished
t2573f39e = Nothing runs in the background.
t116d3868 = Add to the right-click menu
t20864983 = Extract or compress from any folder.
tb38a86a6 = Welcome to packwing
t0265387b = Open any archive, make smaller files to share, and stay safe from harmful downloads.
t63f9c1cd = Set things up
t95f41ffb = You can change these later in Settings.
te83ca5ee = Get started
t26cc8dba = Remove from Recent
t7d0c09c7 = { $location } · not there any more
tc0cc82d5 = The archives you open will show here.
t3d37483c = Copy version info
t6bead5ae = Version { $version } · { $os } { $arch }
t3c244df9 = A file archiver that opens archives safely.
t483075a5 = Updates
t1d6a96bc = This is a beta: you are told of newer betas, and of the release that follows them.
t9d2d8250 = This is a release: you are told of newer releases, not of betas.
t4f0e0677 = Check now
tcc27ace1 = Look for updates once a day
t258cdfd9 = It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t2c33d756 = Skip this version
t378182cb = This build has no release keys, so it cannot tell of updates.
tf02650f8 = packwing { $version } is ready
t6fecac86 = It is installed when you quit, or now with Restart.
t54001a7c = Restart now
t20bb8a0d = packwing { $version }
tf1905fe9 = Looking for a newer version…
tc52ab62f = Release notes
t36a30d8b = packwing { $version } is out
t98b9753c = Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
tdf2cab9c = { $by } updates it: { $command }
t84b27295 = { $by } updates it.
tb46c0678 = It is downloaded and checked, then installed when you quit.
t39312eae = Download it, and install it in place of this one.
t5e96c075 = The download failed: { $why }
t1a99173f = Download and install
t6964bb19 = Download
td122a092 = The last look failed: { $why }
t4665ef60 = Up to date (looked { $when }).
t5a3ba058 = Not looked yet.
t731220a4 = Free software
t4e98bf3c = Under the GNU General Public License, version 3 or later
tfc21d38b = View license
t10c7007a = Made by HivePaaS
tfae09b61 = On hivepack, our library for reading and writing archives
tf6a49f4d = Source code
t610dfb6f = Something wrong?
ta3e29034 = Tell us, with the version above
t7b1728e7 = Report a problem
te8345543 = What changed
t8e6f2052 = Every release, and what is new in it
t11d91ba2 = © { $year } HivePaaS. The packwing name and logo are trademarks of HivePaaS.
tb25cf729 = More from HivePaaS
tc1ece130 = Free and open source, like packwing.
t0f4c758a = A fast, native SSH client for Linux, macOS and Windows.
t0f8af34d = SSH and SFTP
tc1c4446d = Follows your ssh_config
tae9bcfa8 = A real terminal
t8622a5c1 = GPL-3.0-or-later
t1d2ed540 = Visit shellwing.app
tc9d13419 = A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
t6a770b0a = 300+ one-click apps
tb0bfc3e9 = Deploy on push
t4e4180fc = Pull request previews
t9e9776ed = Automatic TLS
tebe3a172 = Live logs and metrics
t0274f036 = Apache-2.0
t0632975a = Visit hivepaas.com
tad4bbd04 = Always
tc709375f = Extensions, apart by spaces
t3b50d238 = Format of new archives
t7b361b55 = New archive starts with it; you can pick another there.
t2595cbcb = Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t9184e778 = Higher makes smaller archives, which take longer to make, not to open.
ta4975128 = Write straight to a storage above
t2b8f2459 = Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t940ec31d = Don’t compress what already is
t4de1e81e = For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
tc0104b29 = What goes in as it is
ta3acef9b = Files whose names end in these, whatever their case
t0db9f7dc = Reset to default
t27eec14f = Leave out Mac-only files
t3bdd25aa = .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t93ff7715 = Hide file names behind the password
t0f5354d7 = For 7z: nobody sees what is inside without it
t6de7f47d = No limit
t77e60aba = Where the files go
t4b73082f = Where Extract starts: you can pick another there.
ta5a4d31d = One item right here, more in a new folder
t2095cbe9 = A new folder next to it
tb991d4c1 = Named after the archive
t4691c928 = Next to the archive
t99d1df43 = Straight into its folder
tca514dda = When a file is already there
t296ad686 = A file of the same name in the folder the files go to
t5957c426 = Largest an archive may unpack to
t87a41242 = Stops archive bombs: small files that unpack to fill the disk
tf47f4968 = Only names every system can use
t44f108af = Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t8ad2f8cf = Light
t45570065 = Dark
t8f3184bc = System
t5bd525d3 = Same as system ({ $language })
t1d68c60b = The system’s
ta391dade = Custom…
ta51f1877 = The system’s ({ $size } px)
tf65ec196 = All ({ $count })
t455dc834 = e.g. code -n, or /path/to/editor --flag
t5d0d23cc = Show packwing in the right-click menu
t94a46acb = Extract and compress from any folder
t9a73db9b = Language
t31018b9f = Appearance
t2c041143 = Text size
t2f9c3d17 = Processor cores to use
t1ca82aab = Text editor
t62506ced = What Edit opens a file of a storage in
teba749d4 = The file’s path is added at the end.
t3c4da5cf = Paste your license key
tf9725115 = Every feature but signing files, and { $count } storage across a network. Folders on this computer are not limited, and any signature can be checked.
tc3e75ecf = Every feature, and any number of storages.
t69b29f66 = Every feature, and any number of storages, for your organization.
t1eab5e32 = Plan
t44ef5cad = Issued to
td514fb0d = Who this license is for
t8f706321 = Seats
t60b0c6c9 = What your organization’s agreement covers
t9ffc2b63 = Trial, until { $date }
t4d9bac91 = Until { $date }
t7b9eb499 = Ended { $date }. It keeps working for { $days } days more.
tfccb9305 = Ended { $date }
td2661011 = Valid
td7098137 = Past this, packwing is Free. Nothing is taken away but adding storages.
te42a0ee6 = Your organization manages this license.
t3b8f0190 = Activate
t9e6ecd3d = A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t60e47df0 = On
te216d656 = They show when you right-click files, folders and archives.
t3ddc5e66 = packwing is not in the menu now: these go in when it is.
tc7f1a2f0 = Changing the menu…
t8d7b3a90 = Good to know
t9fac5c06 = The menu could not be changed: { $why }
t67537dc4 = Items in the menu
te3449a65 = Put them in a submenu
t7730ec1a = Windows 11 puts them under one Packwing item, in its menu and under Show more options
tb5fdc584 = One packwing item, with these under it
tfe574fa2 = Keep the “downloaded from the Internet” mark
t9fbe84c0 = The system warns before running files that came from a downloaded archive
t314d60c1 = This system has no such mark, so this changes nothing here
t38e70ee4 = License
tff7c594e = Your organization’s, in Settings › License
t0af8d466 = Treat files from storages as from the Internet
tf02df9d4 = What comes from S3, WebDAV and SFTP gets the mark too
t0c3e1337 = Require encryption for new archives
tc889f3a2 = Applies to archives made and files added here, not to other apps
t1b8b2116 = Kinds of encryption allowed
t71dcd29c = New archives can be encrypted only these ways
tb7d64a44 = Minimum password length
tca6d6982 = In characters, for every new archive with a password
t7e03649c = packwing has no policy of this name.
t76d5d951 = Ask before opening files without extracting
t9173caaa = Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t0bf72f73 = Paste a public key
td57933b9 = This is not a public key packwing can check with: { $why }.
t90e7dd7b = This line cannot be read, so the key is not used
ted99ced8 = Keys you trust
td25843b8 = A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
t28a576c9 = Choose a file…
tdb06c037 = { $name } (weak)
t47dd48d3 = Checksums
t6edeb650 = What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t7b4eb9c1 = Its policies cannot be read, so what they guard refuses: { $why }
t7db7e7ec = Some settings are managed by your organization
t4a1ed54a = Get help
t068eea1b = Not understood
t0dc6b4aa = Off
t5561fd0b = General
tc82af3f2 = Extracting
t6d183c73 = Compressing
tc2fe21db = Security
t1287ae97 = Right-click menu
t58e096de = About
t4b058728 = Settings
t0c1b4d43 = This computer has no way to ask that it is you.
t780455bf = open a storage
tf861e014 = Open a storage in packwing
tbb2a9838 = That was not it. It stays locked.
td6905afe = The system could not ask: { $why }
tfb56d8cf = This copy of packwing cannot open it.
tc776c52f = Storages
te858d6b5 = Add storage
tb544cd9b = Type the password you log in to this computer with to open it.
tfb4c635b = Show this computer it is you to open it.
t87f62cb7 = Checking…
t69c22a3b = It asks again after 5 minutes left unused.
t8e7efcd7 = Edit…
t8c7c7c57 = Remove from this list
tc5e54e4a = Extract here with packwing
tda62691f = Extract to new folder
tda9255a0 = Extract with packwing…
teb657427 = Open with packwing
t344b1175 = Compress to ZIP
t9ec9b77d = Compress with packwing…
t1934ce08 = packwing cannot find its program: { $e }
t8eb35b10 = Signed by { $signer }
t88b7186a = Signed by { $signer }, not checked yet
tec0df1c3 = Signed by a key you do not trust: { $fingerprint }
tb7d0e1ed = It does not match its signature
tadf2a401 = Its signature cannot be read
t69cbbb51 = Its signature could not be checked
tda8c2e03 = Checking its signature…
t11465c20 = the system's package manager
ta8cbd72e = the App Store
t1df799a1 = the Microsoft Store
taee668c2 = Downloading… { $got } of { $total }
t4873b921 = Downloading… { $got }
ta715ef0f = just now
t6c7302d6 =
    { $n ->
        [one] { $n } minute ago
       *[other] { $n } minutes ago
    }
t137b052e =
    { $n ->
        [one] { $n } hour ago
       *[other] { $n } hours ago
    }
tb9fa7744 =
    { $n ->
        [one] { $n } day ago
       *[other] { $n } days ago
    }
t2bfc1370 = No folder to download to.
t676409df = Update available
tbbbbda26 = Restart to update
t843cf8f1 =
    { $n ->
        [one] { $n } file was skipped, as its name was taken.
       *[other] { $n } files were skipped, as their names were taken.
    }
tbdeb1b33 =
    { $n ->
        [one] { $n } link stayed where it was.
       *[other] { $n } links stayed where they were.
    }
t23ab4fa0 = This file is encrypted with a passphrase.
td46b69a1 = This archive needs its password.
tbd048b4e = This file is encrypted to a key.
t81f4c775 = This key file needs its passphrase.
t27f3b860 = { $paths } archives
t75e7db7c = { $folder }, a folder for each
t8b242db1 = New
ta9906e3b = There is no file to save it beside.
t90b19482 = Nothing in the archive matches the patterns given.
t47339885 = Nothing to compress.
te9ab879e = { $path } is too big.
t75c197d9 = This format holds one file, and no more.
tad0120eb = Could not read { $path }: { $e }
tbd0e16dc = { $path } has no name.
t2588efb5 = The name of { $path } is not valid Unicode, so it cannot go in an archive.
t8dd055c6 = Could not write { $path }: { $e }
t40554bf6 = { $wanted } and the names after it are taken.
t485d71ab = Could not encrypt: { $e }
t582284c9 = A ZIP or 7z archive takes age keys only when age encrypts it whole.
t83582c17 = age encrypts to a passphrase or to keys, not to both.
t500313af = Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
tc2809b2c = There are no keys in this file to encrypt to.
t2e221d1f = { $path } got shorter while it was being read.
t5ef729bc = A password has to be valid Unicode.
tc4faa70c = Could not start compressing: { $e }
tf6ff23f2 = Could not compress: { $e }
t030316d9 = { $path } is a link, which is kept only on Unix.
tedfc294c = Nothing changed.
t2f015c79 = The archive did not come out whole.
t48628036 = packwing can change only a ZIP archive that is one file.
t7530c419 = That is not in the archive.
t495f94b1 = That is all the archive holds. Delete the archive itself instead.
td53e55bf = “{ $to }” cannot be a name.
t5516dd83 = “{ $to }” is already in that folder. Nothing was changed.
t11a330d3 = packwing can add files only to a ZIP archive that is one file.
tff43ce74 = An archive can’t go inside itself.
t2c3e91c4 = “{ $bare }” is already in the archive. Nothing was changed.
tdb9326d3 = Before it is uploaded, this archive is made in { $folder }, where it may take up to { $need }. Only { $free } is free there.
t528232e6 = The upload stopped.
te17b2bfa = “{ $pattern }” names no file or folder.
t3ba6a1aa = a colon, which Windows does not allow
t0b11ab21 = a character Windows does not allow (\ < > " | ? *)
t8cac89ee = a name Windows keeps for a device
t7ae9a209 = ends in a space or a dot, which Windows does not allow
t9218e5c0 = longer than 255 bytes
t378bba3a = a control character
t72b79e1d = not a name this system can read
t1eead41f = the same file here as another, its name differing only in case
t00fdc25e = a link that leads outside the folder
tbe7fab8a = { $name } is already in the folder.
tec7ca3a9 = Could not make a folder in { $parent }.
tf606b3b3 = { $to } is a folder, and a file would replace it.
t5af584a0 = { $name } is already in the folder
t660f149b = This file is not an archive packwing can open.
ta4d3227d = The password is wrong, or the archive is damaged.
t7826c649 = The passphrase is wrong.
t8ad75e3a = The key file’s passphrase is wrong.
te162af76 = This file is encrypted to a key packwing does not have.
t27c010d6 = Cancelled.
t2267c3b1 = the archive could not be read
tcf18f6e9 = This can’t be extracted, as a file’s name can’t be used here: “{ $name }” ({ $why }).
te8f231ba =
    { $n ->
        [one] { $n } byte
       *[other] { $n } bytes
    }
t3803103c = A volume of this archive is not there.
t066227f9 = { $named }: its data is damaged.
t7923a417 = The archive is damaged: UnRAR would exit with { $number }.
t9fd24078 = There is no public key here.
t73610ecc = Could not make the key: { $e }
tc95e3a9c = There is no file to sign.
t07c1f2bc = { $name } is not there any more.
te779b111 = A part has to hold at least one byte.
tdfee38b2 = { $path } is not a file.
t7ce88e46 = That would make more parts than can be numbered: make them bigger.
t2577cbb6 = { $part } is there already.
t64c1c1d0 = { $path } is not a part of a file cut as 7-Zip cuts it, as name.001.
t5e65a10a = { $part } is missing: the parts start from it.
t24165c89 = { $part } is missing, and the parts after it are there.
tb472eff0 = { $part } holds { $got }, not { $size } as the first part does: it may not be whole.
t1a9d8e4e = The name is empty.
t74c8683a = That name is kept for the system.
t4c0596ec = A name cannot hold / or other special characters.
tf16ba7f1 = On Windows a name cannot end with a dot or a space.
t43a2bce1 = That name is taken in this folder.
tb5e7b134 = { $name } is already there.
t99f437a6 = { $name } cannot go inside itself.
t15558940 = Stopped.
t02a45af5 = { $name } is a file in one place and a folder in the other.
t779a64f3 = The copy of { $name } does not match. The original was kept.
td5ae978f = This is not a valid packwing license key.
t3c605280 = Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t83acf110 = Your organization requires new archives to be encrypted.
te8eb49bf = Your organization does not allow this kind of encryption: { $method }.
t2d21b125 = Your organization requires a password of at least { $least } characters.
t7020f9d1 = That password is too easy to guess, and your organization requires a better one.
tcadcff7a = Expected true or false, not { $other }.
tda498fad = Expected a number from 1 to { $most }, not { $other }.
t9386ce3b = Expected some text, not { $other }.
t93bfd19e = Expected an address starting with https://, not { $other }.
tc945cc93 = Expected a list, not { $other }.
tfddb7ba6 = No kind of encryption is called { $name }.
t5f8076f5 = The list names no kind of encryption.
taf68e582 = The file is too big to be a list of policies.
tf483aa65 = The file has to hold one JSON object.
tfc77ea2b = { $path } has to belong to root.
t0d4a4498 = { $path } can be changed by others than root.
t6485842e = This storage takes files of at most { $size } for now, as each is held in memory while it is sent.
ta87a5d75 = This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
tc5e013b3 = S3 bucket
te34e3557 = Bucket
t492e789d = Region
t27e7d69c = Endpoint
td3a94af8 = Empty for AWS, or https://…
t5eb91ea6 = Start in
t75255a6c = A prefix, if not the whole bucket
tf9408d75 = Access key ID
te1c3d3a2 = Secret access key
t62703ae8 = SFTP
t7b943498 = Host or ssh_config name
t67b5f42b = Empty for the home folder
tf10e6293 = Empty to use the SSH agent and keys
tb2401428 = WebDAV folder
t700560f3 = Address
te0a63e12 = User
t1c18cb9b = Password or app token
