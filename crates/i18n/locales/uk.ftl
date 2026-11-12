# packwing in Ukrainian. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = Вигляд: { $name }. Натисніть, щоб перейти на { $name2 }.

# Stop and say so
tdc8b782b = Зупинитися й повідомити

# Replace it
t1e17fc0c = Замінити

# Skip it
tad9909cb = Пропустити

# Add
t9dc3aa14 = Додати

# The top of {name}
t369f9d4d = Верхній рівень { $name }

# Choose what to add
t48b969a8 = Виберіть, що додати

# {count} item | {count} items
t534e422e =
    { $n ->
        [one] { $count } елемент
        [few] { $count } елементи
        [many] { $count } елементів
       *[other] { $count } елемента
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
        [one] { $count } файл
        [few] { $count } файли
        [many] { $count } файлів
       *[other] { $count } файлу
    }

# Add to {name}
t0d53ac5e = Додати до { $name }

# Goes in
t36e95c22 = Куди потрапить

# If a file of that name is there
t34c69cec = Якщо файл із такою назвою вже є

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = Цей архів підписано. Якщо додати до нього файли, він більше не збігатиметься зі своїм підписом, доки його не підписати знову.

# The new files are encrypted with the archive’s password.
t52b1d223 = Нові файли шифруються паролем архіву.

# Cancel
t35afca3b = Скасувати

# Name
t0fe07306 = Назва

# Folder
t06357afd = Папка

# Its secrets could not be read: {why}
t2af0c8ab = Не вдалося прочитати його секрети: { $why }

# That folder is not on this computer.
ta4073fff = Цієї папки немає на цьому комп’ютері.

# Its secrets could not be kept: {why}
ta213827f = Не вдалося зберегти його секрети: { $why }

# Local folder
te075bcbc = Локальна папка

# Trying it…
t214cdad5 = Перевірка…

# It answers.
t3bd5bc9f = Відповідає.

# It did not answer: {why}
t0ac8fa44 = Не відповідає: { $why }

# Edit storage
t7bae68d0 = Змінити сховище

# Add a storage
t4fe8f806 = Додати сховище

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = Секрети зберігаються в системному сховищі ключів, а не разом з налаштуваннями.

# Its files stay where they are.
t4e4acf9a = Його файли залишаються на своїх місцях.

# Ask for this computer’s password to open it
t7a7d61e9 = Запитувати пароль цього комп’ютера, щоб відкрити його

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = Для комп’ютера, який лишають розблокованим: відкрити можна тим, що розблоковує цей комп’ютер, а запит повторюється після 5 хвилин без користування.

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing Free дозволяє одне сховище в мережі. Ліцензія Pro або Enterprise знімає це обмеження: Налаштування › Ліцензія. Папки на цьому комп’ютері не обмежуються.

# Test connection
te0f3b6b7 = Перевірити з’єднання

# Enter a license key
ta24f28a2 = Ввести ліцензійний ключ

# Save
t4d2d5d68 = Зберегти

# Paste the checksum given with the file
t8333758c = Вставте контрольну суму, надану разом із файлом

# They could not be saved: {why}
tb45ae457 = Не вдалося зберегти: { $why }

# The file could not be read: {why}
t5daa8455 = Не вдалося прочитати файл: { $why }

# Reading…
t6aad8485 = Читання…

# Copied
t8dc21305 = Скопійовано

# Copy
t658f3664 = Копіювати

# weak
tedbc2ec9 = слабка

# There is no checksum in this.
t231f838a = Тут немає контрольної суми.

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = Це не контрольна сума, яку обчислює packwing: у ній { $digits } цифр.

# It matches the file’s {name} checksum.
t4f0350e4 = Збігається з контрольною сумою { $name } файлу.

# It is compared once the file is read.
tcb5c1745 = Порівняння — коли файл буде прочитано.

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = Не збігається з контрольною сумою { $name } файлу: це не той файл, з якого її обчислено, або файл змінився дорогою.

# It may be a {name} checksum, which is not made yet.
t3d713e04 = Можливо, це контрольна сума { $name }, яку ще не обчислено.

# Make it
tcbefe458 = Обчислити

# Compare with
tbddcfc14 = Порівняти з

# Reading… {size} of {size2}
t80036d1c = Читання… { $size } з { $size2 }

# Reading… {size}
t1569c388 = Читання… { $size }

# Checksums of {name}
te88f3ed9 = Контрольні суми { $name }

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = Увесь файл проходить через цей комп’ютер, і ваш провайдер може стягнути плату за прочитане з нього.

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = Слабка контрольна сума ({ $names }) виявить пошкоджений файл, але не файл, навмисно змінений так, щоб вона збіглася.

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = Збережено поруч як { $files }; sha256sum -c та подібні перевірять без packwing.

# Compare…
tec85be80 = Порівняти…

# Saving…
tbd79b37d = Збереження…

# Save beside it
t7f4d85e3 = Зберегти поруч

# Choose the checksums shown
teec532fe = Вибрати показувані контрольні суми

# Read it
t80fb8482 = Прочитати

# Done
t8dd31791 = Готово

# Nothing chosen yet
tcf390b0a = Ще нічого не вибрано

# {join} and {more} more
t8399ef70 = { $join } і ще { $more }

# Hide what goes in
t5a420780 = Сховати, що додається

# Show what goes in
t9a20eb53 = Показати, що додається

# Take it out
t90a93ec7 = Прибрати

# Drop them to add them
t6a4aedbe = Перетягніть, щоб додати

# What goes in
t3b7c797a = Що додається

# + Files
tba8a1e71 = + Файли

# + Folder
tcef8b0ce = + Папка

# Add the files or a folder to put in the archive.
t2e5c8c3e = Додайте файли або папку, які потрібно покласти в архів.

# Don’t split
t56ee6829 = Не ділити

# 25 MB (email)
tc6ba1c9c = 25 MB (ел. пошта)

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB (флешка)

# Custom size…
t0fcf9e2d = Свій розмір…

# Size
ta6478e7c = Розмір

# Password
t2cc30838 = Пароль

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…, age1pq1… або ssh-ed25519 …

# Save here
ta1f147a6 = Зберегти тут

# Use these keys
t1a01e439 = Використати ці ключі

# No compression
t2e9c9e78 = Без стиснення

# {name} compression
te2db10f6 = Стиснення { $name }

# Several frames
t84cc41f7 = Кілька фреймів

# One frame
tc6ef759e = Один фрейм

# Small blocks
t04032e8a = Дрібні блоки

# Parts of 25 MB
ta999efe2 = Частини по 25 MB

# Parts of 4 GB
t760df467 = Частини по 4 GB

# Parts of {size}
tdd286479 = Частини по { $size }

# Parts of a size to type
tafd4b17d = Частини вказаного розміру

# Public key
t9981f45f = Відкритий ключ

# Owners kept
t63e8a0df = Власників збережено

# Attributes kept
te9e8289c = Атрибути збережено

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = Щоб зберегти власників і розширені атрибути для резервної копії, виберіть формат .tar.

# Keep for a backup
t6a2bcc1a = Зберегти для резервної копії

# Owners: user and group
t5350e3fe = Власники: користувач і група

# Extended attributes and ACLs
t35aff4d1 = Розширені атрибути та ACL

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = Для резервної копії, яку відновлюють на цьому комп’ютері. Будь-хто, хто відкриє архів, побачить імена користувачів.

# Beside the files
t757989af = Поруч із файлами

# What it holds, from
teae4ef1a = Вміст із

# Save in
tf2d34837 = Зберегти в

# Folder…
tc8abcd43 = Папка…

# Storage…
td0d8c806 = Сховище…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = Спочатку частини створюються в тимчасовій папці цього комп’ютера, а потім вивантажуються, тож там потрібне місце.

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = Спочатку { $ext } створюється в тимчасовій папці цього комп’ютера, а потім вивантажується, тож там потрібне місце. Формати .zip і tar записуються просто в це сховище.

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = Спочатку він створюється в тимчасовій папці цього комп’ютера, а потім вивантажується, тож там потрібне місце.

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } приймає файли розміром до { $most }. Більший архів не вдасться вивантажити: помилка станеться на початку вивантаження.

# Choose a storage, and a folder in it
td7627e48 = Виберіть сховище й папку в ньому

# New folder
t1926d127 = Нова папка

# Give it a name, without / or \ in it.
ta598b15b = Дайте йому назву без / і \.

# A file of that name is there already: this one will be {name}.
t9451c3b1 = Файл із такою назвою вже є: цей називатиметься { $name }.

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = Деякі імена не можна використати у Windows чи macOS, наприклад «{ $name }». Там ці файли можуть видобутися не такими, як є.

# Hide advanced
tae2a96e9 = Сховати додаткові

# Advanced options
t701d5ed9 = Додаткові параметри

# Large blocks
tdc489452 = Великі блоки

# Compression level
td386c9ef = Рівень стиснення

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = «Без стиснення» кладе файли як є: найшвидше й найбільше за розміром.

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = Вищий рівень дає менший архів, який довше створюється, але не довше відкривається.

# Frames
tf03e266f = Фрейми

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = Кілька фреймів відкриваються на всіх ядрах і роблять архів трохи більшим: менш ніж на пів відсотка.

# One frame makes the smallest archive, which opens on one core.
ta097b317 = Один фрейм дає найменший архів, який відкривається на одному ядрі.

# Blocks
t036a71e3 = Блоки

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = Дрібні блоки завантажують усі ядра: до двох разів швидше, а архів на кілька відсотків більший.

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = Великі блоки дають найменший архів, на меншій кількості ядер, коли стискати небагато.

# Split into parts
tca4436bd = Ділити на частини

# Each part
t642c444d = Кожна частина

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = Частини відкриваються в 7-Zip та подібних архіваторах, але не у вбудованих засобах Windows і macOS.

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = Частина має бути розміром від 1 MB, наприклад 100 MB.

# Encrypt with age
t22784ad9 = Шифрувати за допомогою age

# Protect with a password
tae8d8740 = Захистити паролем

# Suggest
ta6e44ce5 = Запропонувати

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = Збережіть це в надійному місці: без цього архів ніколи не вдасться відкрити.

# Type a password, or turn this off.
t62b19318 = Введіть пароль або вимкніть це.

# Hide file names too
t2198d77b = Приховати також назви файлів

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = З паролем вбудовані засоби Windows і macOS не можуть відкрити цей ZIP. Одержувачу потрібна програма-архіватор. Назви файлів також залишаються видимими; щоб їх приховати, виберіть .7z.

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = Збережено як { $name }. Назви всередині також зашифровано. Щоб відкрити, у Windows і macOS може знадобитися інструмент age або програма-архіватор.

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
        [one] { $n } ключ із { $name }
        [few] { $n } ключі з { $name }
        [many] { $n } ключів із { $name }
       *[other] { $n } ключа з { $name }
    }

# Remove
t21a5901d = Видалити

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
        [one] { $n } ключ у ньому пропущено: age не шифрує для ключів такого виду.
        [few] { $n } ключі в ньому пропущено: age не шифрує для ключів такого виду.
        [many] { $n } ключів у ньому пропущено: age не шифрує для ключів такого виду.
       *[other] { $n } ключа в ньому пропущено: age не шифрує для ключів такого виду.
    }

# From a file…
t40d7c944 = З файлу…

# Type a public key, or choose a file of them.
tcb028b06 = Введіть відкритий ключ або виберіть файл із ними.

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = Постквантовий ключ: щоб відкрити це, інструмент age має бути версії 1.3 або новішої.

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = Відкрити це можна лише відповідним закритим ключем, а якщо його втрачено, не вдасться нічим.

# Weak
t6de87ae9 = Слабкий

# Fair
t82fa0d53 = Прийнятний

# Strong
t0dad409a = Надійний

# Choose what to put in it
t5461cfa6 = Виберіть, що помістити в нього

# Convert
t92e29218 = Перетворити

# New archive
tf352befb = Новий архів

# Create
t990de47d = Створити

# {n} item | {n} items
tfff7fd6c =
    { $n ->
        [one] { $n } елемент
        [few] { $n } елементи
        [many] { $n } елементів
       *[other] { $n } елемента
    }

# Move {what} to the Trash?
tdf5a8d53 = Перемістити { $what } до Кошика?

# Delete {what}?
ta297d6c5 = Видалити { $what }?

# It can be put back from the system’s Trash.
te5e137a1 = Його можна повернути із системного Кошика.

# This cannot be undone from packwing.
t3947dab4 = У packwing це не можна скасувати.

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
        [one] і ще { $n }
        [few] і ще { $n }
        [many] і ще { $n }
       *[other] і ще { $n }
    }

# Move to Trash
ta036661b = Перемістити до Кошика

# Delete
t5797ea6a = Видалити

# {many} items
t49912bd4 = Елементів: { $many }

# Rename {what}
t83401013 = Перейменувати { $what }

# New name
taf4cad88 = Нова назва

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = Його буде видалено з архіву остаточно, а папки разом з усім вмістом. Це не можна скасувати.

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = Цей архів підписано. Після цієї зміни він більше не збігатиметься зі своїм підписом, доки його не підписати знову.

# Rename
tc80631f3 = Перейменувати

# The archive’s password
t8a168799 = Пароль архіву

# one folder
td571ce35 = одна папка

# one file
t95c9ec7b = один файл

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }, тож просто тут

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }, тож у нову папку

# Extract here
t354232ec = Видобути сюди

# {picked} selected
t9addc37e = Вибрано: { $picked }

# All {counted}
t649feaf8 = Усе: { $counted }

# What should it extract?
td12e23bf = Що видобути?

# Smart
t54535972 = Розумно

# Recommended
t7865024c = Рекомендовано

# Where should the files go?
td35f540c = Куди помістити файли?

# Right here
t900a8a39 = Просто тут

# Another folder
t9f128ba8 = Інша папка

# Choose…
t58daae9c = Вибрати…

# Goes to {destination}
t8195aed2 = Піде до { $destination }

# Checked as it goes
tb6c9570b = Перевіряється під час видобування

# · Nothing leaves the folder or passes the size limit
t11424797 = · Ніщо не виходить за межі папки й не перевищує обмеження розміру

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } надійшов з Інтернету. Він залишиться з позначкою, тож перед запуском вас запитають.

# Extract archive
t834ba3c0 = Видобути архів

# Extract
t30518e1e = Видобути

# Open {name}?
tcfe84c2f = Відкрити { $name }?

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing кладе копію в тимчасову папку й відкриває її. Зміни в ній не повертаються в архів.

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = Це програма: відкриття запускає її. Робіть це, лише якщо довіряєте джерелу архіву.

# Don’t ask again
t00c5d168 = Більше не питати

# Run it
t72430853 = Запустити

# Open
t538b10e9 = Відкрити

# Open from a storage
t45a8991d = Відкрити зі сховища

# Choose a storage, and an archive in it
t3c900950 = Виберіть сховище й архів у ньому

# Choose
t73c66160 = Вибрати

# This is not a key file packwing can use: {why}.
tcf6986a7 = Це не файл ключа, який може використати packwing: { $why }.

# Could not read {path}: {e}.
t1dc6961f = Не вдалося прочитати { $path }: { $e }.

# The passphrase
t1e921d00 = Парольна фраза

# The key file’s passphrase
t4051f4d7 = Парольна фраза файлу ключа

# Enter the passphrase
tbd0672da = Введіть парольну фразу

# Passphrase
t191a2dc5 = Парольна фраза

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = Цей файл зашифровано за допомогою age. packwing зберігає парольну фразу лише доки архів відкритий і ніколи її не записує.

# Enter the password
t513ea7ab = Введіть пароль

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = Цей архів зашифровано. packwing зберігає його пароль лише доки архів відкритий і ніколи його не записує.

# Choose the key file
t3c521a4e = Виберіть файл ключа

# Key file
t6ed7c612 = Файл ключа

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = Цей файл зашифровано за допомогою age для ключа: файлу ключа age або SSH-ключа. packwing зберігає прочитане з нього лише доки архів відкритий і ніколи цього не записує.

# Enter the key file’s passphrase
t808b85c5 = Введіть парольну фразу файлу ключа

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = Сам файл ключа захищено. packwing зберігає його парольну фразу лише доки архів відкритий і ніколи її не записує.

# Version control
t0b7280ee = Контроль версій

# Packages and builds
t237b373e = Пакети та збірки

# System files
te8be6b83 = Системні файли

# Temporary files
td4fe1263 = Тимчасові файли

# Logs
t51a96da6 = Журнали

# Pictures
t0d70feae = Зображення

# Documents
t5a036285 = Документи

# Videos
t63609fed = Відео

# Music
t584a1d74 = Музика

# node_modules/, *.log, src/**/gen/
t56c5448d = node_modules/, *.log, src/**/gen/

# They leave nothing out.
t999b68fd = Нічого не виключено.

# They leave out {n} file, {size}. | They leave out {n} files, {size}.
tc709cfda =
    { $n ->
        [one] Виключають { $n } файл, { $size }.
        [few] Виключають { $n } файли, { $size }.
        [many] Виключають { $n } файлів, { $size }.
       *[other] Виключають { $n } файла, { $size }.
    }

# Clear
t04a57fc2 = Очистити

# Leave out
ta50cc084 = Пропустити

# Take only
t2fd02f84 = Лише це

# Add the usual
t17ad93a1 = Додати звичні

# Only what these name goes in, and the folders it is in.
ta0bd7335 = Потрапляє лише те, що вони називають, і теки, де воно лежить.

# A name is anywhere; a / at its end names a folder; ** is any folders.
t524d506a = Ім’я без / підходить будь-де; / у кінці означає теку; ** — будь-які теки.

# Filter
tf4a9c097 = Фільтр

# kept, {size}
t001f6992 = збережено, { $size }

# left out
t73aeedd4 = пропущено

# renamed {to}
t7aa9b778 = перейменовано на { $to }

# It was gone by the time it was read.
t93fb1e03 = На момент читання його вже не було.

# The job stopped before it was done.
t57a78885 = Завдання зупинилося, не завершившись.

# Waiting for other jobs…
t16ec36d9 = Чекає на інші завдання…

# A file is damaged
t8b1230ed = Файл пошкоджено

# Some names can’t be used here
t26835ec8 = Деякі імена тут не можна використати

# Converted, but some files are damaged
t54ef3472 = Перетворено, але деякі файли пошкоджено

# Extracted, but some files are damaged
t8226d624 = Видобуто, але деякі файли пошкоджено

# Converted, with some files left out or renamed
t5cce0e50 = Перетворено, деякі файли пропущено або перейменовано

# Extracted, with some files left out or renamed
t5dc002d6 = Видобуто, деякі файли пропущено або перейменовано

# Extracting…
t94279f82 = Видобування…

# Uploading…
tc75fe478 = Вивантаження…

# Creating…
t01fcd896 = Створення…

# Adding…
t2060c750 = Додавання…

# Changing the archive…
t186fbef5 = Зміна архіву…

# Opening…
tc40f2321 = Відкриття…

# Testing…
t19d1cb65 = Перевірка…

# Copying…
t02dd17e0 = Копіювання…

# Moving…
t890f7ed1 = Переміщення…

# Signing…
t7b8f8780 = Підписання…

# Splitting…
t5452adf5 = Розрізання…

# Joining…
taa884c51 = Склеювання…

# Converting…
ta1b8f4cc = Перетворення…

# Couldn’t split the file
t83d27d26 = Не вдалося розрізати файл

# Couldn’t join the parts
t1cf67e00 = Не вдалося склеїти частини

# Couldn’t convert the archive
t39f973a9 = Не вдалося перетворити архів

# Couldn’t sign
t4489f97c = Не вдалося підписати

# Couldn’t copy
t613e4e9c = Не вдалося скопіювати

# Couldn’t move
t9796adec = Не вдалося перемістити

# Couldn’t extract
tacd22c56 = Не вдалося видобути

# Couldn’t create the archive
t9d0e01bc = Не вдалося створити архів

# Couldn’t add the files
tfbd989b0 = Не вдалося додати файли

# Couldn’t change the archive
tcdc4a49c = Не вдалося змінити архів

# Couldn’t open the file
t041198a4 = Не вдалося відкрити файл

# The test failed
tfcd070a5 = Перевірка не пройшла

# Starts when one of the others ends
tbe2a0509 = Почнеться, коли завершиться одне з інших

# Paused
tde281feb = Призупинено

# Starting…
t73727cef = Початок…

# Finishing…
t042f47fe = Завершення…

# Almost done
ta550a6a9 = Майже готово

# About {secs} seconds left
t3045da16 = Залишилося близько { $secs } с

# About a minute left
tf727815c = Залишилося близько хвилини

# About {div_ceil} minutes left
tbd7a3305 = Залишилося близько { $div_ceil } хв

# {name} · checking that every file reads
t0d56ec7c = { $name } · перевірка, чи читається кожен файл

# and {count} more | and {count} more
te728628b =
    { $n ->
        [one] і ще { $count }
        [few] і ще { $count }
        [many] і ще { $count }
       *[other] і ще { $count }
    }

# The test passed
tdf0ce240 = Перевірку пройдено

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
        [one] Прочитано { $count } файл, { $size }, без помилок.
        [few] Прочитано { $count } файли, { $size }, без помилок.
        [many] Прочитано { $count } файлів, { $size }, без помилок.
       *[other] Прочитано { $count } файлу, { $size }, без помилок.
    }

# Close
tcd86acc3 = Закрити

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = «{ $path }» пошкоджено: { $why }. Зупинити, пропустити його чи зберегти те, що вдалося прочитати?

# Do the same for the other damaged files
t798693ad = Зробити так само з іншими пошкодженими файлами

# Stop
t4b7f7705 = Зупинити

# Leave it out
taaff8ec9 = Пропустити

# Keep what was read
tc4387d25 = Зберегти прочитане

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
        [one] { $count } файл не можна видобути тут під його іменем. Ще нічого не записано: зупинити, пропустити чи перейменувати?
        [few] { $count } файли не можна видобути тут під їхніми іменами. Ще нічого не записано: зупинити, пропустити чи перейменувати?
        [many] { $count } файлів не можна видобути тут під їхніми іменами. Ще нічого не записано: зупинити, пропустити чи перейменувати?
       *[other] { $count } файли не можна видобути тут під їхніми іменами. Ще нічого не записано: зупинити, пропустити чи перейменувати?
    }

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
        [one] { $count } посилання веде за межі теки, в інше місце на цьому комп’ютері. Ще нічого не записано: пропустити його чи створити як є, якщо ви довіряєте цьому архіву?
        [few] { $count } посилання ведуть за межі теки, в інші місця на цьому комп’ютері. Ще нічого не записано: пропустити їх чи створити як є, якщо ви довіряєте цьому архіву?
        [many] { $count } посилань ведуть за межі теки, в інші місця на цьому комп’ютері. Ще нічого не записано: пропустити їх чи створити як є, якщо ви довіряєте цьому архіву?
       *[other] { $count } посилання ведуть за межі теки, в інші місця на цьому комп’ютері. Ще нічого не записано: пропустити їх чи створити як є, якщо ви довіряєте цьому архіву?
    }

# Create them
t9a4d6e7d = Створити

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
        [one] Дані { $count } файлу пошкоджено.
        [few] Дані { $count } файлів пошкоджено.
        [many] Дані { $count } файлів пошкоджено.
       *[other] Дані { $count } файлу пошкоджено.
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
        [one] { $count } файл пропущено.
        [few] { $count } файли пропущено.
        [many] { $count } файлів пропущено.
       *[other] { $count } файлу пропущено.
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
        [one] { $count } файл перейменовано, бо його ім’я тут не можна використати.
        [few] { $count } файли перейменовано, бо їхні імена тут не можна використати.
        [many] { $count } файлів перейменовано, бо їхні імена тут не можна використати.
       *[other] { $count } файлу перейменовано, бо їхні імена тут не можна використати.
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
        [one] { $count } файл на момент читання вже зник і його пропущено.
        [few] { $count } файли на момент читання вже зникли і їх пропущено.
        [many] { $count } файлів на момент читання вже зникли і їх пропущено.
       *[other] { $count } файлу на момент читання вже зникли і їх пропущено.
    }

# The rest is in the archive.
t977a5572 = Решта — в архіві.

# The rest was extracted.
ta7833fcb = Решту видобуто.

# Copy the list
t3c49b269 = Скопіювати список

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } із { $count2 } файлів · { $size } із { $size2 }

# Open the folder when it’s done
tf0f1017b = Відкрити папку після завершення

# Run in background
tb800c0ef = Виконувати у фоні

# Resume
t454d1d8a = Продовжити

# Pause
t44f9bf2d = Призупинити

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
        [one] Ще виконується { $n } завдання
        [few] Ще виконуються { $n } завдання
        [many] Ще виконуються { $n } завдань
       *[other] Ще виконується { $n } завдання
    }

# Closing packwing stops them
t2e6491b3 = Закриття packwing зупинить їх

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = Незавершене залишиться незавершеним: архів, що створюється, буде видалено, а розпакування зупиниться там, де було.

# Keep running
t79c1b9c7 = Продовжити

# Stop them and quit
tf2707857 = Зупинити й вийти

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = Його файли залишаться на місці. Те, чим packwing до нього входить, буде вилучено зі зв’язки ключів системи.

# The folder and its files stay where they are.
t4850aaa8 = Тека та її файли залишаться на місці.

# Remove “{name}” from the list?
t5dd9cfcf = Прибрати «{ $name }» зі списку?

# It can be added again at any time.
t80d51a99 = Його можна знову додати будь-коли.

# A passphrase for the new key
tf5593f05 = Парольна фраза для нового ключа

# The same passphrase again
tad5c08e6 = Ще раз та сама парольна фраза

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = Підписання потребує ліцензії Pro або Enterprise: Налаштування › Ліцензія. Перевірка підписів безкоштовна на всіх тарифах.

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = Це одна частина файлу, розділеного на кілька. Підпис охоплює один файл і здавався б охопленим усі, тому packwing не підписує частини.

# That passphrase does not open this key.
tc5357c97 = Ця парольна фраза не відкриває цей ключ.

# This is not a key packwing can sign with: {why}.
t662c29f9 = Цим ключем packwing не може підписувати: { $why }.

# The two passphrases are not the same.
t335fe90b = Парольні фрази не збігаються.

# Keep the key here
t9306dafe = Зберегти ключ тут

# No key chosen
te58c13f5 = Ключ не вибрано

# Key
tcd1ac90c = Ключ

# New key…
t9e1134da = Новий ключ…

# Again
t58fb56ef = Ще раз

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = Новий ключ поєднує Ed25519 і ML-DSA-65, щоб підпис лишався дійсним, навіть якщо один із них зламають. Їх перевіряють лише packwing і hivepack: щоб працювати з ssh-keygen, підписуйте SSH-ключем Ed25519. Ключ, парольну фразу якого втрачено, відновити неможливо.

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = Підпис буде збережено поруч як { $signature } замість наявного. Увесь файл проходить через цей комп’ютер, і ваш провайдер може стягнути плату за прочитане з нього.

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = Підпис буде збережено поруч як { $signature } замість наявного.

# Sign {name}
ted250f31 = Підписати { $name }

# Anyone can check it, with the key’s public half
t04ecc8be = Перевірити може будь-хто, за відкритою половиною ключа

# Make the key…
t6bba28d5 = Створити ключ…

# Sign
t8f6340c4 = Підписати

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = Частин поруч: { $count }, починаючи з { $first }. «Склеїти» збере їх назад, а packwing і 7-Zip відкривають так розрізаний архів як є.

# Split
t16a15383 = Розрізати

# Move {what}
tc168ca56 = Перемістити { $what }

# Copy {what}
teda01f26 = Скопіювати { $what }

# Move here
t1aaf57aa = Перемістити сюди

# Copy here
t85f0245a = Скопіювати сюди

# Move
t7f2c2a34 = Перемістити

# If a name is taken
tc99c037f = Якщо назва вже зайнята

# They are in this folder already. Choose another.
t5318c14e = Вони вже в цій папці. Виберіть іншу.

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = Усе проходить через цей комп’ютер, і ваш провайдер може стягнути плату за прочитане з нього. Кожен переміщений оригінал видаляється, коли його копію прочитано назад і вона збігається.

# Each original goes once its copy has been read back and matches.
t8ecbab4e = Кожен оригінал видаляється, коли його копію прочитано назад і вона збігається.

# The originals stay where they are.
t5937f050 = Оригінали залишаються на місці.

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = Інше завдання зараз створює там { $name }: дайте цьому іншу назву або зачекайте, доки воно завершиться.

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = Інше завдання зараз розпаковує в { $folder }: виберіть іншу теку або зачекайте, доки воно завершиться.

# no editor command
tbf2a5d3a = немає команди редактора

# no text editor found
t6256b3da = текстовий редактор не знайдено

# Search this folder
t2c814039 = Шукати в цій папці

# This folder could not be read: {why}
t2c6017b2 = Не вдалося прочитати цю папку: { $why }

# Could not load more: {why}
t466e87c4 = Не вдалося завантажити більше: { $why }

# Could not delete: {why}
te5951d38 = Не вдалося видалити: { $why }

# the text editor
t91cdab0c = текстовий редактор

# Cannot start {who}: {why}
t8deb160a = Не вдалося запустити { $who }: { $why }

# Edit in text editor
td8d39fb8 = Редагувати в текстовому редакторі

# Edit in {editor}
te44cd57b = Редагувати в { $editor }

# Modified
tdd13a6b2 = Змінено

# Loading…
tc4e2f181 = Завантаження…

# Only what is loaded was searched.
t39c5f536 = Шукали лише серед завантаженого.

# Search the whole folder
t41f9a787 = Шукати в усій папці

# Load {n} more
t30bed46a = Завантажити ще { $n }

# Refresh
t0815aad4 = Оновити

# Hide hidden files
t3375c9c0 = Сховати приховані файли

# Show hidden files
tf18c7057 = Показати приховані файли

# View
t5e4baa18 = Вигляд

# Extract…
t4a09490e = Видобути…

# Open with default app
t5ef80ec5 = Відкрити в застосунку за замовчуванням

# Open as archive
t3373ce55 = Відкрити як архів

# Compress…
t690f9553 = Стиснути…

# Copy to…
t543cf15b = Скопіювати до…

# Move to…
t7a32320b = Перемістити до…

# Sign…
td6d61b10 = Підписати…

# Checksums…
t2891102d = Контрольні суми…

# Convert…
tae85a7c4 = Перетворити…

# Split into parts…
t281db403 = Розрізати на частини…

# Join the parts
t0ad56db8 = Склеїти частини

# Copy path
tf165cb2b = Копіювати шлях

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
        [one] Стиснути { $n } елемент…
        [few] Стиснути { $n } елементи…
        [many] Стиснути { $n } елементів…
       *[other] Стиснути { $n } елемента…
    }

# {n} loaded, more in this folder
t29a52578 = Завантажено { $n }, у цій папці є ще

# Sorted among those loaded
t037d01ae = Відсортовано серед завантаженого

# {n} selected
tebda8468 = Вибрано: { $n }

# Try again
t213e90fa = Спробувати ще раз

# Nothing here matches that.
t0f9a8a58 = Нічого відповідного.

# Add a storage to browse it.
t0b82c83a = Додайте сховище, щоб переглядати його.

# No folders or archives here.
t6a090cc3 = Тут немає папок чи архівів.

# No folders here.
tc7731c8f = Тут немає папок.

# This folder is empty.
td9e47db6 = Ця папка порожня.

# Show in Finder
ta9442aa9 = Показати у Finder

# Show in File Explorer
t23c89eda = Показати у Провіднику

# Show in the file manager
tb347ece9 = Показати у файловому менеджері

# Open in Finder
te2a467e0 = Відкрити у Finder

# Open in File Explorer
t3794d625 = Відкрити у Провіднику

# Open in the file manager
td1ceabc4 = Відкрити у файловому менеджері

# Copy all
t6aef5c37 = Копіювати все

# This system has no folder for settings.
tfd0ff0a1 = У цій системі немає папки для налаштувань.

# Archive
t73690b65 = Архів

# Search inside this archive
tb35c0c27 = Шукати в цьому архіві

# Open another…
tff74a552 = Відкрити інший…

# From a storage…
tcfba8fd5 = Зі сховища…

# Safe to extract
t3c93bb70 = Безпечно видобувати

# Reading… {percent}%
t83223c27 = Читання… { $percent }%

# Read {fetched} of {whole}
t613dd932 = Прочитано { $fetched } із { $whole }

# Add files
tdd45c7bb = Додати файли

# Test
t2ffcbe05 = Перевірити

# Reading what’s inside…
t1bbda9f5 = Читання вмісту…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = У цьому архіві немає нічого за запитом «{ $query }».

# Nothing in this folder matches “{query}”.
teecdfa3d = У цій папці немає нічого за запитом «{ $query }».

# And {more} more. Type more to narrow it down.
t843253d0 = І ще { $more }. Введіть більше, щоб звузити пошук.

# Open an archive
t8382bd3e = Відкрити архів

# ZIP, 7z and tar archives, disc images, packages, snaps and AppImages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .deb, .rpm, .pkg, .snap, .AppImage, .zst.age…
t2cb379b7 = Архіви ZIP, 7z і tar, образи дисків, пакунки, snap і AppImage, стиснуті або зашифровані за допомогою age: .zip, .7z, .tar.gz, .tar.zst, .iso, .deb, .rpm, .pkg, .snap, .AppImage, .zst.age…

# Back to {name}
t539b9c10 = Назад до { $name }

# Open an archive…
t2cf2a8ee = Відкрити архів…

# It could not be shown
tb60fb288 = Не вдалося показати

# Choose a file to see it here
t37854216 = Виберіть файл, щоб побачити його тут

# Symbolic link
tc7c20187 = Символічне посилання

# Leads to {target}
te7ddae48 = Веде до { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
        [one] { $count } виконуваний файл у цій папці збереже позначку з Інтернету після видобування.
        [few] { $count } виконувані файли в цій папці збережуть позначку з Інтернету після видобування.
        [many] { $count } виконуваних файлів у цій папці збережуть позначку з Інтернету після видобування.
       *[other] { $count } виконуваного файлу в цій папці збережуть позначку з Інтернету після видобування.
    }

# Today
te7c0775e = Сьогодні

# Last 7 days
t35b9d98f = Останні 7 днів

# Last 30 days
td0778b33 = Останні 30 днів

# All days
tabb197ef = За весь час

# All results
t6235c0b2 = Усі результати

# Not done
t0f88a146 = Не виконано

# Partly done
t3355a2a5 = Виконано частково

# Failed
t5579c3d4 = Помилка

# Cancelled
ta62b09bc = Скасовано

# No end recorded
t38ab1371 = Завершення не записано

# Make an archive
t418bd6f0 = Створення архіву

# Open a file from an archive
tc2dd3e41 = Відкриття файлу з архіву

# Preview a file
t1f706562 = Попередній перегляд файлу

# Read a file from an archive
t4229020d = Читання файлу з архіву

# Add to an archive
t4e28d1fa = Додавання до архіву

# Delete from an archive
t158044d3 = Видалення з архіву

# Rename in an archive
tb3eef763 = Перейменування в архіві

# Check a signature
t6f441e5e = Перевірка підпису

# Make a signing key
tbdf6bcd0 = Створення ключа підпису

# Join parts
t4eee8c5f = Об’єднання частин

# Checksums
t47dd48d3 = Контрольні суми

# Audit log started
tfb8df56f = Журнал аудиту запущено

# Audit log stopped
t54654bc3 = Журнал аудиту зупинено

# Audit log changed
td6b7bc4a = Журнал аудиту змінено

# Old records taken away
t5f337ebf = Старі записи видалено

# All actions
t4ff15157 = Усі дії

# Find a path, a name or an error
t30cb1a6c = Знайти шлях, назву або помилку

# Read again
t2f0dcc5d = Перечитати

# Save as CSV…
tf103c3cc = Зберегти як CSV…

# Checking that no record was changed…
t00deba79 = Перевірка, що жоден запис не змінено…

# The audit log cannot be read: {why}
t8783f4cf = Не вдається прочитати журнал аудиту: { $why }

# A record was changed, taken out or put in by hand: in {file}, line {line}. | Records were changed, taken out or put in by hand, in {n} places: the first in {file}, line {line}.
t3cd011bd =
    { $n ->
        [one] Запис змінено, видалено або вставлено вручну: { $file }, рядок { $line }.
        [few] Записи змінено, видалено або вставлено вручну в { $n } місцях: перше — { $file }, рядок { $line }.
        [many] Записи змінено, видалено або вставлено вручну в { $n } місцях: перше — { $file }, рядок { $line }.
       *[other] Записи змінено, видалено або вставлено вручну в { $n } місцях: перше — { $file }, рядок { $line }.
    }

# The oldest record follows one that is gone, and no record says it was taken away as past its time.
t4579f8a7 = Найстаріший запис іде за відсутнім, і жоден запис не каже, що той видалено після закінчення строку.

# {n} record, and none was changed or taken out. | {n} records, and none was changed or taken out.
tefc82060 =
    { $n ->
        [one] { $n } запис, жоден не змінено й не видалено.
        [few] { $n } записи, жоден не змінено й не видалено.
        [many] { $n } записів, жоден не змінено й не видалено.
       *[other] { $n } запису, жоден не змінено й не видалено.
    }

# Only the latest {most} records are shown: choose fewer days, or read all with pkw audit list.
t5fabfc51 = Показано лише останні { $most } записів: виберіть менше днів або прочитайте всі за допомогою pkw audit list.

# Saved to {path}.
te8346fa4 = Збережено в { $path }.

# Could not save: {why}
t733a7cae = Не вдалося зберегти: { $why }

# Record
t9f4565ec = Запис

# Started
t97b5823e = Почато

# Ended
tec2b41db = Завершено

# Reading the audit log…
t403ccdff = Читання журналу аудиту…

# No records in these days.
t98687757 = За ці дні записів немає.

# No record is as chosen.
ta9ffcb9f = Немає записів, що відповідають вибору.

# This computer
tfa0d176a = Цей комп'ютер

# Home
t52f50eae = Домашня

# Desktop
t64215c53 = Робочий стіл

# Downloads
t539307de = Завантаження

# Drop to open
tb255877b = Перетягніть, щоб відкрити

# Browse
t0eac1f07 = Огляд

# Recent
t50e888fe = Недавні

# Audit log
tcb83f9ca = Журнал аудиту

# Choose a file to see its information here
t50aac02b = Виберіть файл, щоб побачити тут відомості про нього

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
        [one] Вибрано { $n }
        [few] Вибрано { $n }
        [many] Вибрано { $n }
       *[other] Вибрано { $n }
    }

# Files in them come to {size}
t0cca93f1 = Файли в них разом займають { $size }

# Delete…
t2fed07aa = Видалити…

# Link to a folder
t4b408baf = Посилання на папку

# Link
t8e08e589 = Посилання

# {tag} file
ta56bbbe9 = Файл { $tag }

# {files}, {folders}
t72e46dc5 = { $files }, { $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
        [one] { $n } файл
        [few] { $n } файли
        [many] { $n } файлів
       *[other] { $n } файлу
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
        [one] { $n } папка
        [few] { $n } папки
        [many] { $n } папок
       *[other] { $n } папки
    }

# {size} so far
tc87daf59 = Поки що { $size }

# Kind
t3ff50c63 = Вид

# Holds
tffe19c31 = Містить

# Permissions
t8700fd83 = Права доступу

# Path
teb66e456 = Шлях

# Signature
tc049050d = Підпис

# Count size and files
t0f23db17 = Порахувати розмір і файли

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
        [one] { $n } папку в ньому не вдалося прочитати.
        [few] { $n } папки в ньому не вдалося прочитати.
        [many] { $n } папок у ньому не вдалося прочитати.
       *[other] { $n } папки в ньому не вдалося прочитати.
    }

# Check the signature
t9d44730e = Перевірити підпис

# Extracted
t3250471f = Розпаковано

# Archive created
t22c69419 = Архів створено

# Moved
t82869bf0 = Переміщено

# Signed
t27c3ab35 = Підписано

# Joined
tf8f2b670 = Склеєно

# Converted
tc7b76e99 = Перетворено

# Waiting for a password or a key
t0e83ae32 = Чекає на пароль або ключ

# Enter it…
tddb7844a = Ввести…

# Show
t29742e5c = Показати

# Jobs
t4ae6a79d = Завдання

# Clear finished
t30c178e8 = Прибрати завершені

# Nothing runs in the background.
t2573f39e = У фоні нічого не виконується.

# Add to the right-click menu
t116d3868 = Додати до контекстного меню

# Extract or compress from any folder.
t20864983 = Видобувайте чи стискайте з будь-якої папки.

# Welcome to packwing
tb38a86a6 = Вітаємо в packwing

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = Відкривайте будь-які архіви, робіть файли меншими, щоб ділитися, і будьте захищені від шкідливих завантажень.

# Set things up
t63f9c1cd = Налаштувати

# You can change these later in Settings.
t95f41ffb = Це можна змінити пізніше в Налаштуваннях.

# Get started
te83ca5ee = Почати

# Remove from Recent
t26cc8dba = Прибрати з недавніх

# {location} · not there any more
t7d0c09c7 = { $location } · його вже немає

# The archives you open will show here.
tc0cc82d5 = Тут з’являться архіви, які ви відкриєте.

# Copy version info
t3d37483c = Копіювати відомості про версію

# Version {version} · {os} {arch}
t6bead5ae = Версія { $version } · { $os } { $arch }

# A file archiver that puts every core to work and opens archives safely.
t5d2a9fa0 = Архіватор, який задіює всі ядра й безпечно відкриває архіви.

# Updates
t483075a5 = Оновлення

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = Це бета-версія: ви дізнаватиметеся про новіші бета-версії та про стабільну версію, що вийде після них.

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = Це стабільна версія: ви дізнаватиметеся про новіші стабільні версії, але не про бета-версії.

# Check now
t4f0e0677 = Перевірити зараз

# Look for updates once a day
tcc27ace1 = Перевіряти оновлення раз на день

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = Список випусків зчитується з GitHub і приймається, лише якщо його підписи правильні. Без вас нічого не завантажується й не встановлюється.

# Skip this version
t2c33d756 = Пропустити цю версію

# This build has no release keys, so it cannot tell of updates.
t378182cb = Ця збірка не має ключів випуску, тому не може повідомляти про оновлення.

# packwing {version} is ready
tf02650f8 = packwing { $version } готовий

# It is installed when you quit, or now with Restart.
t6fecac86 = Його буде встановлено, коли ви вийдете, або зараз — кнопкою «Перезапустити».

# Restart now
t54001a7c = Перезапустити зараз

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = Пошук нової версії…

# Release notes
tc52ab62f = Примітки до випуску

# packwing {version} is out
t36a30d8b = Вийшов packwing { $version }

# Your license covers versions released until {date}. This one came out after: with it, packwing is Free until you renew. The version you have stays Pro.
tfd489117 = Ваша ліцензія охоплює версії, випущені до { $date }. Ця вийшла пізніше: з нею packwing працює як Free, доки ви не подовжите ліцензію. Ваша теперішня версія лишається Pro.

# Renew
t9a23808e = Подовжити

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = Виконайте ці команди в терміналі. Вони звіряють завантажений файл із підписаним списком випусків, а sudo запитає ваш пароль.

# {by} updates it: {command}
tdf2cab9c = Оновлюється через { $by }: { $command }

# {by} updates it.
t84b27295 = Оновлюється через { $by }.

# It is downloaded and checked, then installed when you quit: Windows asks for an administrator's permission, as packwing is installed for everyone.
tc816fc67 = Його буде завантажено й перевірено, а встановлено — коли ви вийдете: Windows попросить дозволу адміністратора, бо packwing встановлено для всіх.

# It is downloaded and checked, then installed when you quit.
tb46c0678 = Його буде завантажено й перевірено, а встановлено — коли ви вийдете.

# Download it, and install it in place of this one.
t39312eae = Завантажте її та встановіть замість цієї.

# The download failed: {why}
t5e96c075 = Не вдалося завантажити: { $why }

# Download and install
t1a99173f = Завантажити та встановити

# Download
t6964bb19 = Завантажити

# The last look failed: {why}
td122a092 = Остання перевірка не вдалася: { $why }

# Up to date (looked {when}).
t4665ef60 = Оновлень немає (перевірено { $when }).

# Not looked yet.
t5a3ba058 = Ще не перевірялося.

# Free software
t731220a4 = Вільне програмне забезпечення

# Under the GNU General Public License, version 3 or later
t4e98bf3c = За ліцензією GNU General Public License версії 3 або новішої

# View license
tfc21d38b = Переглянути ліцензію

# Made by HivePaaS
t10c7007a = Створено HivePaaS

# On hivepack, our library for reading and writing archives
tfae09b61 = На hivepack, нашій бібліотеці для читання й запису архівів

# Source code
tf6a49f4d = Вихідний код

# Something wrong?
t610dfb6f = Щось не так?

# Tell us, with the version above
ta3e29034 = Повідомте нам, вказавши версію вище

# Report a problem
t7b1728e7 = Повідомити про проблему

# What changed
te8345543 = Що змінилося

# Every release, and what is new in it
t8e6f2052 = Кожен випуск і що в ньому нового

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS. Назва та логотип packwing є торговельними марками HivePaaS.

# More from HivePaaS
tb25cf729 = Більше від HivePaaS

# Free and open source, like packwing.
tc1ece130 = Безкоштовний і з відкритим кодом, як і packwing.

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = Швидкий нативний SSH-клієнт для Linux, macOS і Windows.

# SSH and SFTP
t0f8af34d = SSH і SFTP

# Follows your ssh_config
tc1c4446d = Дотримується вашого ssh_config

# A real terminal
tae9bcfa8 = Справжній термінал

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = Відвідати shellwing.app

# A native HTTP(S) debugging proxy that keeps up with big sessions.
t86e5ad3e = Нативний налагоджувальний HTTP(S)-проксі, що впорається з великими сесіями.

# HTTP/2, WebSocket and SSE
t2c3c2377 = HTTP/2, WebSocket і SSE

# A million requests, smooth
t60de9f6c = Мільйон запитів — плавно

# Breakpoints and rewrites
t00c623a3 = Точки зупину й переписування

# Phones and emulators
t8c26f435 = Телефони й емулятори

# Visit tracewing.app
t4668641f = Відвідати tracewing.app

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = Платформа для ваших застосунків на власному сервері, побудована на Docker Swarm: відкрита альтернатива Heroku, Render і Coolify.

# 300+ one-click apps
t6a770b0a = 300+ застосунків в один клік

# Deploy on push
tb0bfc3e9 = Розгортання за push

# Pull request previews
t4e4180fc = Попередній перегляд pull request

# Automatic TLS
t9e9776ed = Автоматичний TLS

# Live logs and metrics
tebe3a172 = Журнали й метрики в реальному часі

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = Перейти на hivepaas.com

# Always
tad4bbd04 = Завжди

# Extensions, apart by spaces
tc709375f = Розширення через пробіл

# Format of new archives
t3b50d238 = Формат нових архівів

# New archive starts with it; you can pick another there.
t7b361b55 = Новий архів починається з нього; там можна вибрати інший.

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = «Без стиснення» кладе файли архівів ZIP і 7z як є. Формати tar беруть Мінімум: без стиснення лише .tar.

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = Вищий рівень дає менші архіви, які довше створюються, але не довше відкриваються.

# Write straight to a storage above
ta4975128 = Писати прямо в сховище вище

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = Більші файли не займають місця на цьому комп’ютері, але якщо з’єднання втратиться, стискати доведеться заново. S3 приймає так tar і ZIP, SFTP також усі формати.

# Don’t compress what already is
t940ec31d = Не стискати вже стиснене

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = Для ZIP і 7z: зображення, відео, звук і архіви кладуться як є — швидше й не більші

# What goes in as it is
tc0104b29 = Що кладеться як є

# Files whose names end in these, whatever their case
ta3acef9b = Файли, чиї імена закінчуються на них, у будь-якому регістрі

# Reset to default
t0db9f7dc = Повернути типові

# Leave out Mac-only files
t27eec14f = Пропускати файли лише для Mac

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store, __MACOSX і гілки ресурсів: сміття всюди, крім Mac

# Hide file names behind the password
t93ff7715 = Приховувати назви файлів за паролем

# For 7z: nobody sees what is inside without it
t0f5354d7 = Для 7z: без пароля ніхто не побачить, що всередині

# Yours
t2635ea3b = Ваші

# By name
t983e37f7 = За іменем

# By number
tcbbada8d = За номером

# No limit
t6de7f47d = Без обмеження

# Where the files go
t77e60aba = Куди йдуть файли

# Where Extract starts: you can pick another there.
t4b73082f = З чого починається Видобування: там можна вибрати інше.

# One item right here, more in a new folder
ta5a4d31d = Один елемент просто тут, більше — у нову папку

# A new folder next to it
t2095cbe9 = Нова папка поруч

# Named after the archive
tb991d4c1 = З назвою архіву

# Next to the archive
t4691c928 = Поруч з архівом

# Straight into its folder
t99d1df43 = Прямо в його папку

# When a file is already there
tca514dda = Коли файл уже є

# A file of the same name in the folder the files go to
t296ad686 = Файл із такою самою назвою в папці, куди йдуть файли

# Largest an archive may unpack to
t5957c426 = Найбільший розмір, до якого може розпакуватися архів

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = Захищає від архівних бомб: малих файлів, що розпаковуються й заповнюють диск

# Only names every system can use
tf47f4968 = Лише імена, придатні для будь-якої системи

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = Імена, яких не дозволяє Windows, як-от a:b.txt, та імена, що різняться лише регістром, запитуються й на цьому комп’ютері

# Owners from a backup
t38a7cf66 = Власники з резервної копії

# Where a tar archive kept them: yours, or the archive’s by name or by number. Only an administrator gives files to others
t46908bee = Якщо tar-архів їх зберіг: ваші або з архіву — за іменем чи за номером. Віддати файли іншим може лише адміністратор

# Extended attributes from a backup
t764eed3a = Розширені атрибути з резервної копії

# Where a tar archive kept them, ACLs among them
ta28f3ae5 = Якщо tar-архів їх зберіг, разом з ACL

# Light
t8ad2f8cf = Світлий

# Dark
t45570065 = Темний

# System
t8f3184bc = Системний

# Same as system ({language})
t5bd525d3 = Як у системі ({ $language })

# The system’s
t1d68c60b = Системна

# Custom…
ta391dade = Інший…

# The system’s ({size} px)
ta51f1877 = Як у системі ({ $size } px)

# All ({count})
tf65ec196 = Усі ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = наприклад, code -n або /path/to/editor --flag

# Language
t9a73db9b = Мова

# Appearance
t31018b9f = Вигляд

# Text size
t2c041143 = Розмір тексту

# Processor cores to use
t2f9c3d17 = Скільки ядер процесора використовувати

# Text editor
t1ca82aab = Текстовий редактор

# What Edit opens a file of a storage in
t62506ced = У чому Редагувати відкриває файл зі сховища

# The file’s path is added at the end.
teba749d4 = Шлях до файлу додається в кінці.

# Paste your license key
t3c4da5cf = Вставте ключ ліцензії

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = Усі можливості, крім підписання файлів, і { $count } сховище в мережі. Папки на цьому комп’ютері не обмежуються, а будь-який підпис можна перевірити.

# Every feature, and any number of storages.
tc3e75ecf = Усі можливості й будь-яка кількість сховищ.

# Every feature, and any number of storages, for your organization.
t69b29f66 = Усі можливості й будь-яка кількість сховищ для вашої організації.

# Plan
t1eab5e32 = Тариф

# Issued to
t44ef5cad = Видано

# Who this license is for
td514fb0d = Для кого ця ліцензія

# Seats
t8f706321 = Місця

# What your organization’s agreement covers
t60b0c6c9 = Що охоплює угода вашої організації

# Ended {date}
tfccb9305 = Закінчилась { $date }

# This version came out after your updates ended, so it is Free. Renew, or use a version released by then: those stay Pro.
t545a1cd3 = Ця версія вийшла після завершення ваших оновлень, тому вона Free. Подовжте ліцензію або користуйтеся версією, випущеною до того: вони лишаються Pro.

# This version stays Pro for good, with its fixes. Versions released after need a renewal.
tbf1ce5b8 = Ця версія лишається Pro назавжди, разом з її виправленнями. Для пізніших версій потрібне подовження.

# Until {date}
t4d9bac91 = До { $date }

# Every version released by then is yours for good, with its fixes. Renew for a year more of new versions.
t5d8058ff = Кожна версія, випущена до цієї дати, лишається вашою назавжди, разом з виправленнями. Подовжте, щоб отримувати нові версії ще рік.

# Trial, until {date}
t9ffc2b63 = Пробна, до { $date }

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = Закінчилась { $date }. Вона діятиме ще { $days } дн.

# Valid
td2661011 = Дійсна

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = Після цього packwing стає Free. Забирається лише можливість додавати сховища.

# Your organization manages this license.
te42a0ee6 = Цією ліцензією керує ваша організація.

# Activate
t3b8f0190 = Активувати

# Buy
tcada780b = Купити

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = Ключ перевіряється на цьому комп’ютері. Нічого нікуди не надсилається, і жодна ліцензія не змінює того, що роблять політики вашої організації.

# On
t60e47df0 = Увімкнено

# They show when you right-click files, folders and archives.
te216d656 = Вони з’являються, коли ви клацаєте правою кнопкою файли, папки й архіви.

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = Зараз packwing немає в меню: вони з’являться, коли він там буде.

# Changing the menu…
tc7f1a2f0 = Зміна меню…

# Good to know
t8d7b3a90 = Варто знати

# The menu could not be changed: {why}
t9fac5c06 = Не вдалося змінити меню: { $why }

# Show packwing in the right-click menu
t5d0d23cc = Показувати packwing у контекстному меню

# Extract and compress from any folder
t94a46acb = Видобувайте й стискайте з будь-якої папки

# Items in the menu
t67537dc4 = Пункти меню

# Put them in a submenu
te3449a65 = Помістити їх у підменю

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 збирає їх в один пункт Packwing — і у своєму меню, і в «Показати додаткові параметри»

# One packwing item, with these under it
tb5fdc584 = Один пункт packwing, а ці — під ним

# 30 days
t7a572af9 = 30 днів

# 90 days
t2d8a9817 = 90 днів

# 1 year
t4d0a98a5 = 1 рік

# Keep all
te0b1775b = Зберігати все

# Keep the “downloaded from the Internet” mark
tfe574fa2 = Зберігати позначку «завантажено з Інтернету»

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = Система попереджає перед запуском файлів із завантаженого архіву

# This system has no such mark, so this changes nothing here
t314d60c1 = У цій системі немає такої позначки, тож тут це нічого не змінює

# License
t38e70ee4 = Ліцензія

# Your organization’s, in Settings › License
tff7c594e = Вашої організації, у Налаштування › Ліцензія

# Treat files from storages as from the Internet
t0af8d466 = Вважати файли зі сховищ такими, що з Інтернету

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = Усе з S3, WebDAV і SFTP також отримує позначку

# Require encryption for new archives
t0c3e1337 = Вимагати шифрування нових архівів

# Applies to archives made and files added here, not to other apps
tc889f3a2 = Стосується архівів, створених тут, і файлів, доданих тут, але не інших програм

# Kinds of encryption allowed
t1b8b2116 = Дозволені види шифрування

# New archives can be encrypted only these ways
t71dcd29c = Нові архіви можна шифрувати лише так

# Minimum password length
tb7d64a44 = Мінімальна довжина пароля

# In characters, for every new archive with a password
tca6d6982 = У символах, для кожного нового архіву з паролем

# Keep an audit log
tf7fd8762 = Вести журнал аудиту

# A record of what packwing does, whatever the license
t34fe4e25 = Записи про дії packwing, незалежно від ліцензії

# Where the audit log is kept
tbd999abe = Де зберігається журнал аудиту

# A folder your organization chose
td4e1f74a = Тека, яку обрала ваша організація

# How long the audit log is kept
td504acaf = Скільки зберігається журнал аудиту

# In days; 0 keeps every record
t127a3c54 = У днях; 0 — зберігати всі записи

# Most the audit log may take
ta334bb40 = Найбільший розмір журналу аудиту

# In MB; the oldest records go first
t03a543d2 = У МБ; першими видаляються найстаріші записи

# What the audit log says of archives
t8d6fca43 = Що журнал аудиту пише про архіви

# “entries” names each file extracted, and those found damaged or left out
t1e33aba6 = «entries» називає кожен видобутий файл, а також пошкоджені та пропущені

# Every job recorded
t42369815 = Кожна операція записується

# A job whose start the audit log cannot write does not start
tda48e1c9 = Операція, початок якої журнал аудиту не може записати, не запускається

# The audit log in the system’s log
t4a2fcd04 = Журнал аудиту в системному журналі

# Each record goes to {log} as well, where your organization’s tools read it
t3a4c104f = Кожен запис також потрапляє до { $log }, звідки його читають інструменти вашої організації

# packwing has no policy of this name.
t7e03649c = У packwing немає політики з такою назвою.

# Ask before opening files without extracting
t76d5d951 = Питати перед відкриттям файлів без видобування

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = Відкриття файлу з архіву кладе копію в тимчасову папку. Програму завжди запитують.

# Paste a public key
t0bf72f73 = Вставте відкритий ключ

# What packwing does, kept on this computer: archives extracted, made, converted and copied, when and by whom. Never a password or what is in a file.
t2162955d = Дії packwing, записані на цьому комп’ютері: які архіви видобуто, створено, перетворено й скопійовано, коли і ким. Ніколи — паролі чи вміст файлів.

# Keep its records for
tf423722f = Зберігати записи

# Older ones are taken away, and that is recorded too
t7073f273 = Старіші видаляються, і це теж записується

# Takes a Pro or Enterprise license: Settings › License.
t2774cee6 = Потрібна ліцензія Pro або Enterprise: Налаштування › Ліцензія.

# Where the audit log is
t2702fcc6 = Де журнал аудиту

# View records
t27c5284a = Переглянути записи

# Open folder
t713a842f = Відкрити теку

# The last record could not be written: {why}
t542148a1 = Не вдалося записати останній запис: { $why }

# pkw audit verify checks that no record was changed or taken out. Whoever runs this computer can still remove the log: it tells what packwing did, it does not stop anyone.
t482581ff = pkw audit verify перевіряє, що жоден запис не змінено й не видалено. Адміністратор цього комп’ютера все одно може видалити журнал: він показує, що зробив packwing, але нікого не зупиняє.

# This is not a public key packwing can check with: {why}.
td57933b9 = Цим відкритим ключем packwing не може перевіряти: { $why }.

# This line cannot be read, so the key is not used
t90e7dd7b = Цей рядок не вдається прочитати, тож ключ не використовується

# Keys you trust
ted99ced8 = Ключі, яким ви довіряєте

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = Підпис одного з цих ключів показує, хто підписав файл. Підпис будь-якого іншого ключа нічого не доводить, навіть якщо збігається.

# Choose a file…
t28a576c9 = Вибрати файл…

# {name} (weak)
tdb06c037 = { $name } (слабка)

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = Що показує пункт «Контрольні суми…» в меню файлу, для порівняння із сумою, наданою там, звідки взято файл. Слабка виявить пошкоджений файл, але не навмисно змінений.

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = Його політики не вдається прочитати, тож те, що вони охороняють, відхиляється: { $why }

# Some settings are managed by your organization
t7db7e7ec = Деякими налаштуваннями керує ваша організація

# Get help
t4a1ed54a = Отримати допомогу

# Not understood
t068eea1b = Не зрозуміло

# Off
t0dc6b4aa = Вимк.

# General
t5561fd0b = Загальні

# Extracting
tc82af3f2 = Видобування

# Compressing
t6d183c73 = Стиснення

# Security
tc2fe21db = Безпека

# Right-click menu
t1287ae97 = Контекстне меню

# About
t58e096de = Про застосунок

# Settings
t4b058728 = Налаштування

# This computer has no way to ask that it is you.
t0c1b4d43 = Цей комп’ютер не може запитати підтвердження, що це ви.

# open a storage
t780455bf = відкрити сховище

# Open a storage in packwing
tf861e014 = Відкрити сховище в packwing

# That was not it. It stays locked.
tbb2a9838 = Це було не те. Залишається заблоковано.

# The system could not ask: {why}
td6905afe = Системі не вдалося запитати: { $why }

# This copy of packwing cannot open it.
tfb56d8cf = Ця копія packwing не може його відкрити.

# Storages
tc776c52f = Сховища

# Add storage
te858d6b5 = Додати сховище

# Type the password you log in to this computer with to open it.
tb544cd9b = Введіть пароль, яким ви входите в цей комп’ютер, щоб відкрити його.

# Show this computer it is you to open it.
tfb4c635b = Підтвердьте цьому комп’ютеру, що це ви, щоб відкрити його.

# Checking…
t87f62cb7 = Перевірка…

# It asks again after 5 minutes left unused.
t69c22a3b = Запит повторюється після 5 хвилин без користування.

# Edit…
t8e7efcd7 = Редагувати…

# Remove from this list
t8c7c7c57 = Прибрати з цього списку

# Extract here with packwing
tc5e54e4a = Видобути сюди за допомогою packwing

# Extract to new folder
tda62691f = Видобути в нову папку

# Extract with packwing…
tda9255a0 = Видобути за допомогою packwing…

# Open with packwing
teb657427 = Відкрити за допомогою packwing

# Compress to ZIP
t344b1175 = Стиснути в ZIP

# Compress with packwing…
t9ec9b77d = Стиснути за допомогою packwing…

# packwing cannot find its program: {e}
t1934ce08 = packwing не знаходить своєї програми: { $e }

# Signed by {signer}
t8eb35b10 = Підписано: { $signer }

# Signed by {signer}, not checked yet
t88b7186a = Підписано: { $signer }, ще не перевірено

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = Підписано ключем, якому ви не довіряєте: { $fingerprint }

# It does not match its signature
tb7d0e1ed = Не збігається зі своїм підписом

# Its signature cannot be read
tadf2a401 = Його підпис не вдається прочитати

# Its signature could not be checked
t69cbbb51 = Його підпис не вдалося перевірити

# Checking its signature…
tda8c2e03 = Перевірка підпису…

# the system's package manager
t11465c20 = системний менеджер пакетів

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = Завантаження… { $got } з { $total }

# Downloading… {got}
t4873b921 = Завантаження… { $got }

# just now
ta715ef0f = щойно

# {n} minute ago | {n} minutes ago
t6c7302d6 =
    { $n ->
        [one] { $n } хвилину тому
        [few] { $n } хвилини тому
        [many] { $n } хвилин тому
       *[other] { $n } хвилини тому
    }

# {n} hour ago | {n} hours ago
t137b052e =
    { $n ->
        [one] { $n } годину тому
        [few] { $n } години тому
        [many] { $n } годин тому
       *[other] { $n } години тому
    }

# {n} day ago | {n} days ago
tb9fa7744 =
    { $n ->
        [one] { $n } день тому
        [few] { $n } дні тому
        [many] { $n } днів тому
       *[other] { $n } дня тому
    }

# No folder to download to.
t2bfc1370 = Немає папки для завантаження.

# packwing can’t show this kind of file
tb909bf3c = packwing не може показати файл такого виду

# Too big to show here
t1b339d35 = Завелике, щоб показати тут

# It needs the password to show it
t81f5dbce = Щоб показати, потрібен пароль

# It could not be read: {why}
t26005fe3 = Не вдалося прочитати: { $why }

# Find in text
t9cbe4be4 = Знайти в тексті

# Invalid pattern
t5b9dcaca = Неправильний шаблон

# No matches
td29421ef = Збігів немає

# Find
t3e1cd17a = Знайти

# Fit
t116f13cc = Вписати

# {count} line | {count} lines
t9fbb1078 =
    { $n ->
        [one] { $count } рядок
        [few] { $count } рядки
        [many] { $count } рядків
       *[other] { $count } рядка
    }

# Its first 1 MB is shown
t329dac80 = Показано перший 1 МБ

# Update available
t676409df = Є оновлення

# Restart to update
tbbbbda26 = Перезапустіть, щоб оновити

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
        [one] { $n } файл пропущено, бо його назва зайнята.
        [few] { $n } файли пропущено, бо їхні назви зайняті.
        [many] { $n } файлів пропущено, бо їхні назви зайняті.
       *[other] { $n } файлу пропущено, бо їхні назви зайняті.
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
        [one] { $n } посилання лишилося на місці.
        [few] { $n } посилання лишилися на місці.
        [many] { $n } посилань лишилося на місці.
       *[other] { $n } посилання лишилося на місці.
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = Цей файл зашифровано парольною фразою.

# This archive needs its password.
td46b69a1 = Цьому архіву потрібен пароль.

# This file is encrypted to a key.
tbd048b4e = Цей файл зашифровано для ключа.

# This key file needs its passphrase.
t81f4c775 = Цьому файлу ключа потрібна парольна фраза.

# {paths} archives
t27f3b860 = Архівів: { $paths }

# {folder}, a folder for each
t75e7db7c = { $folder }, окрема папка для кожного

# New
t8b242db1 = Новий

# Your organization requires each job to be recorded, and the audit log cannot be written: {why}
t3f254f09 = Ваша організація вимагає записувати кожну операцію, а журнал аудиту не вдається записати: { $why }

# There is no file to save it beside.
ta9906e3b = Немає файлу, поруч із яким зберегти.

# Nothing in the archive matches the patterns given.
t90b19482 = В архіві немає нічого, що відповідає заданим шаблонам.

# Nothing to compress.
t47339885 = Нічого стискати.

# {path} is too big.
te9ab879e = { $path } завеликий.

# This format holds one file, and no more.
t75c197d9 = Цей формат містить один файл, не більше.

# Store
t6da32dce = Без стиснення

# Lowest
t5d7dbfdb = Мінімум

# Low
tb1dd9521 = Низький

# Medium
t09be5f96 = Середній

# High
tbe0a061d = Високий

# Highest
t9487b22f = Максимум

# Could not read {path}: {e}
tad0120eb = Не вдалося прочитати { $path }: { $e }

# {path} has no name.
tbd0e16dc = У { $path } немає назви.

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = Назва { $path } не є дійсним Unicode, тому її не можна помістити в архів.

# Could not write {path}: {e}
t8dd055c6 = Не вдалося записати { $path }: { $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } та наступні за ним назви зайняті.

# Could not encrypt: {e}
t485d71ab = Не вдалося зашифрувати: { $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = Рівню «{ $level }» потрібно близько { $needed } пам’яті, а вільно { $available }: архів створюється на рівні «{ $lower }», якому потрібно близько { $less }.

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = Цьому рівню потрібно близько { $needed } пам’яті, а вільно { $available }: комп’ютер може сповільнитися або packwing зупинитися.

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = Архів ZIP або 7z приймає ключі age, лише коли age шифрує його повністю.

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age шифрує або парольною фразою, або ключами, але не обома.

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = Постквантові ключі не можна поєднувати з іншими: інші залишили б файл відкритим для квантового комп’ютера.

# There are no keys in this file to encrypt to.
tc2809b2c = У цьому файлі немає ключів, для яких можна шифрувати.

# {path} got shorter while it was being read.
t2e221d1f = { $path } став коротшим, поки його читали.

# A password has to be valid Unicode.
t5ef729bc = Пароль має бути дійсним Unicode.

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = Не вдалося прочитати розширені атрибути { $path }: { $e }

# Could not start compressing: {e}
tc4faa70c = Не вдалося почати стиснення: { $e }

# Could not compress: {e}
tf6ff23f2 = Не вдалося стиснути: { $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } — це посилання, які зберігаються лише в Unix.

# Changing this archive writes a new one beside it, which may take up to {need}. Only {free} is free in {folder}.
ta087fe64 = Щоб змінити цей архів, поруч із ним записується новий, якому може знадобитися до { $need }. У { $folder } вільно лише { $free }.

# Nothing changed.
tedfc294c = Нічого не змінилося.

# The archive did not come out whole.
t2f015c79 = Архів вийшов неповним.

# packwing can change only a ZIP or 7z archive that is one file.
t675882a8 = packwing може змінювати лише архів ZIP або 7z з одного файлу.

# That is not in the archive.
t7530c419 = Цього немає в архіві.

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = Це все, що містить архів. Натомість видаліть сам архів.

# “{to}” cannot be a name.
td53e55bf = «{ $to }» не може бути назвою.

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = «{ $to }» уже є в цій папці. Нічого не змінено.

# packwing can add files only to a ZIP or 7z archive that is one file.
t8d1ef729 = packwing може додавати файли лише до архіву ZIP або 7z з одного файлу.

# An archive can’t go inside itself.
tff43ce74 = Архів не можна помістити сам у себе.

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = «{ $bare }» уже є в архіві. Нічого не змінено.

# Nothing chosen matches the patterns given.
te132289a = Ніщо з вибраного не відповідає шаблонам.

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = Перш ніж вивантажити, цей архів створюється в { $folder }, де він може зайняти до { $need }. Там вільно лише { $free }.

# The upload stopped.
t528232e6 = Вивантаження зупинено.

# “{pattern}” names no file or folder.
te17b2bfa = «{ $pattern }» не вказує ні на файл, ні на теку.

# a colon, which Windows does not allow
t3ba6a1aa = двокрапка, якої Windows не дозволяє

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = символ, якого Windows не дозволяє (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = ім’я, яке Windows резервує для пристрою

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = закінчується пробілом або крапкою, чого Windows не дозволяє

# longer than 255 bytes
t9218e5c0 = довше за 255 байтів

# a control character
t378bba3a = керівний символ

# not a name this system can read
t72b79e1d = ім’я, яке ця система не може прочитати

# the same file here as another, its name differing only in case
t1eead41f = тут той самий файл, що й інший: імена відрізняються лише регістром

# a link that leads outside the folder
t00fdc25e = посилання, що веде за межі теки

# {name} is already in the folder.
tbe7fab8a = { $name } уже є в папці.

# Could not make a folder in {parent}.
tec7ca3a9 = Не вдалося створити папку в { $parent }.

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } — це папка, а файл замінив би її.

# {name} is already in the folder
t5af584a0 = { $name } уже є в папці

# This file is not an archive packwing can open.
t660f149b = Цей файл не є архівом, який може відкрити packwing.

# This is an ar archive, as static libraries are. packwing can’t open those yet.
t9c5564b8 = Це архів ar, як статичні бібліотеки. packwing поки що не вміє їх відкривати.

# This is a macOS disk image (.dmg). packwing can’t open those yet.
t080d0cb2 = Це образ диска macOS (.dmg). packwing поки що не вміє їх відкривати.

# The password is wrong, or the archive is damaged.
ta4d3227d = Пароль неправильний, або архів пошкоджено.

# The passphrase is wrong.
t7826c649 = Парольна фраза неправильна.

# The key file’s passphrase is wrong.
t8ad75e3a = Парольна фраза файлу ключа неправильна.

# This file is encrypted to a key packwing does not have.
te162af76 = Цей файл зашифровано для ключа, якого в packwing немає.

# Cancelled.
t27c010d6 = Скасовано.

# the archive could not be read
t2267c3b1 = не вдалося прочитати архів

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = Це не можна видобути, бо назву файлу тут використати не можна: «{ $name }» ({ $why }).

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
        [one] { $n } байт
        [few] { $n } байти
        [many] { $n } байтів
       *[other] { $n } байта
    }

# A volume of this archive is not there.
t3803103c = Одного тому цього архіву немає.

# {named}: its data is damaged.
t066227f9 = { $named }: його дані пошкоджено.

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = Архів пошкоджено: UnRAR завершився б з кодом { $number }.

# There is no public key here.
t9fd24078 = Тут немає відкритого ключа.

# Could not make the key: {e}
t73610ecc = Не вдалося створити ключ: { $e }

# There is no file to sign.
tc95e3a9c = Немає файлу для підписання.

# {name} is not there any more.
t07c1f2bc = { $name } уже немає.

# A part has to hold at least one byte.
te779b111 = Частина має містити хоча б один байт.

# {path} is not a file.
tdfee38b2 = { $path } — не файл.

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = Так вийде більше частин, ніж можна пронумерувати: зробіть їх більшими.

# {part} is there already.
t2577cbb6 = { $part } уже є.

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path } — не частина файлу, розрізаного так, як ріже 7-Zip, наприклад name.001.

# {part} is missing: the parts start from it.
t5e65a10a = Немає { $part }: з неї починаються частини.

# {part} is missing, and the parts after it are there.
t24165c89 = Немає { $part }, а частини після неї є.

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = У { $part } { $got }, а не { $size }, як у першій частині: можливо, вона неповна.

# The name is empty.
t1a9d8e4e = Назва порожня.

# That name is kept for the system.
t74c8683a = Ця назва зарезервована для системи.

# A name cannot hold / or other special characters.
t4c0596ec = Назва не може містити / чи інші спеціальні символи.

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = У Windows назва не може закінчуватися крапкою чи пробілом.

# That name is taken in this folder.
t43a2bce1 = Ця назва вже зайнята в цій папці.

# {name} is already there.
tb5e7b134 = { $name } уже є.

# {name} cannot go inside itself.
t99f437a6 = { $name } не можна помістити сам у себе.

# Stopped.
t15558940 = Зупинено.

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } в одному місці є файлом, а в іншому — папкою.

# The copy of {name} does not match. The original was kept.
t779a64f3 = Копія { $name } не збігається. Оригінал збережено.

# This is not a valid packwing license key.
td5ae978f = Це не дійсний ключ ліцензії packwing.

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = Політику вашої організації щодо цього не вдається прочитати, тому packwing цього не робитиме. Зверніться до ІТ-відділу.

# Your organization requires new archives to be encrypted.
t83acf110 = Ваша організація вимагає шифрувати нові архіви.

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = Ваша організація не дозволяє такого шифрування: { $method }.

# Your organization requires a password of at least {least} characters.
t2d21b125 = Ваша організація вимагає пароль завдовжки щонайменше { $least } символів.

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = Цей пароль надто легко вгадати, а ваша організація вимагає кращого.

# Expected true or false, not {other}.
tcadcff7a = Очікувалося true або false, а не { $other }.

# Expected a number from 1 to {most}, not {other}.
tda498fad = Очікувалося число від 1 до { $most }, а не { $other }.

# Expected some text, not {other}.
t9386ce3b = Очікувався текст, а не { $other }.

# Expected a whole path, such as /var/log/packwing or \\server\audit, not {other}.
t00d3565d = Очікувався повний шлях, наприклад /var/log/packwing або \\server\audit, а не { $other }.

# Expected a number of days from 0 (keep all) to {most}, not {other}.
tf8fba675 = Очікувалася кількість днів від 0 (зберігати все) до { $most }, а не { $other }.

# Expected a size in MB from 0 (no limit) to {most}, not {other}.
t9f7e002d = Очікувався розмір у МБ від 0 (без обмежень) до { $most }, а не { $other }.

# Expected summary or entries, not {other}.
t27e9b96f = Очікувалося summary або entries, а не { $other }.

# Expected an address starting with https://, not {other}.
t93bfd19e = Очікувалася адреса, що починається з https://, а не { $other }.

# Expected a list, not {other}.
tc945cc93 = Очікувався список, а не { $other }.

# No kind of encryption is called {name}.
tfddb7ba6 = Немає виду шифрування з назвою { $name }.

# The list names no kind of encryption.
t5f8076f5 = У списку немає жодного виду шифрування.

# The file is too big to be a list of policies.
taf68e582 = Файл завеликий, щоб бути списком політик.

# The file has to hold one JSON object.
tf483aa65 = Файл має містити один об’єкт JSON.

# {path} has to belong to root.
tfc77ea2b = { $path } має належати root.

# {path} can be changed by others than root.
t0d4a4498 = { $path } можуть змінювати не лише root.

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = Це сховище поки що приймає файли не більші за { $size }, бо кожен тримається в пам’яті, поки надсилається.

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = Це сховище не може одразу перейменувати папку, бо все під нею довелося б копіювати. Скористайтеся «Перемістити до…».

# S3 bucket
tc5e013b3 = Бакет S3

# Bucket
te34e3557 = Бакет

# Region
t492e789d = Регіон

# Endpoint
t27e7d69c = Endpoint

# Empty for AWS, or https://…
td3a94af8 = Порожньо для AWS, або https://…

# Start in
t5eb91ea6 = Почати з

# A prefix, if not the whole bucket
t75255a6c = Префікс, якщо не весь бакет

# Access key ID
tf9408d75 = ID ключа доступу

# Secret access key
te1c3d3a2 = Секретний ключ доступу

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = Хост або назва з ssh_config

# Empty for the home folder
t67b5f42b = Порожньо для домашньої папки

# Empty to use the SSH agent and keys
tf10e6293 = Порожньо, щоб використати агент SSH і ключі

# WebDAV folder
tb2401428 = Папка WebDAV

# Address
t700560f3 = Адреса

# User
te0a63e12 = Користувач

# Password or app token
t1c18cb9b = Пароль або токен застосунку
