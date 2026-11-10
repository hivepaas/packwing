# packwing in Turkish. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = Görünüm: { $name }. { $name2 } için tıklayın.

# Stop and say so
tdc8b782b = Dur ve bildir

# Replace it
t1e17fc0c = Değiştir

# Skip it
tad9909cb = Atla

# Add
t9dc3aa14 = Ekle

# The top of {name}
t369f9d4d = En üst düzey: { $name }

# Choose what to add
t48b969a8 = Eklenecekleri seçin

# {count} item | {count} items
t534e422e =
    { $n ->
        [one] { $count } öğe
       *[other] { $count } öğe
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
        [one] { $count } dosya
       *[other] { $count } dosya
    }

# Add to {name}
t0d53ac5e = Ekle: { $name }

# Goes in
t36e95c22 = Gireceği yer

# If a file of that name is there
t34c69cec = Aynı adlı dosya varsa

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = Bu arşiv imzalı. İçine bir şey eklenirse imzasıyla artık eşleşmez; yeniden imzalanana kadar.

# The new files are encrypted with the archive’s password.
t52b1d223 = Yeni dosyalar arşivin parolasıyla şifrelenir.

# Cancel
t35afca3b = İptal

# Name
t0fe07306 = Ad

# Folder
t06357afd = Klasör

# Its secrets could not be read: {why}
t2af0c8ab = Gizli bilgileri okunamadı: { $why }

# That folder is not on this computer.
ta4073fff = Bu klasör bu bilgisayarda yok.

# Its secrets could not be kept: {why}
ta213827f = Gizli bilgileri saklanamadı: { $why }

# Local folder
te075bcbc = Yerel klasör

# Trying it…
t214cdad5 = Deneniyor…

# It answers.
t3bd5bc9f = Yanıt veriyor.

# It did not answer: {why}
t0ac8fa44 = Yanıt vermedi: { $why }

# Edit storage
t7bae68d0 = Depolama alanını düzenle

# Add a storage
t4fe8f806 = Depolama alanı ekle

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = Gizli bilgiler ayarlarla birlikte değil, sistemin anahtar zincirinde saklanır.

# Its files stay where they are.
t4e4acf9a = Dosyaları olduğu yerde kalır.

# Ask for this computer’s password to open it
t7a7d61e9 = Açmak için bu bilgisayarın parolası sorulsun

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = Kilidi açık bırakılan bir bilgisayar için: bu bilgisayarın kilidini açan şey bunu da açar ve 5 dakika kullanılmazsa yeniden sorar.

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing Ücretsiz, ağ üzerinde tek bir depolama alanı tutar. Pro veya Enterprise lisansı bu sınırı kaldırır: Ayarlar › Lisans. Bu bilgisayardaki klasörler sınırlı değildir.

# Test connection
te0f3b6b7 = Bağlantıyı dene

# Enter a license key
ta24f28a2 = Lisans anahtarı girin

# Save
t4d2d5d68 = Kaydet

# Paste the checksum given with the file
t8333758c = Dosyayla birlikte verilen sağlama toplamını yapıştırın

# They could not be saved: {why}
tb45ae457 = Kaydedilemedi: { $why }

# The file could not be read: {why}
t5daa8455 = Dosya okunamadı: { $why }

# Reading…
t6aad8485 = Okunuyor…

# Copied
t8dc21305 = Kopyalandı

# Copy
t658f3664 = Kopyala

# weak
tedbc2ec9 = zayıf

# There is no checksum in this.
t231f838a = Bunda sağlama toplamı yok.

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = Bu, packwing'in hesapladığı bir sağlama toplamı değil: { $digits } haneli.

# It matches the file’s {name} checksum.
t4f0350e4 = Dosyanın { $name } sağlama toplamıyla eşleşiyor.

# It is compared once the file is read.
tcb5c1745 = Dosya okununca karşılaştırılır.

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = Dosyanın { $name } sağlama toplamıyla eşleşmiyor: bu, toplamın hesaplandığı dosya değil ya da dosya yolda değişti.

# It may be a {name} checksum, which is not made yet.
t3d713e04 = Henüz hesaplanmamış bir { $name } sağlama toplamı olabilir.

# Make it
tcbefe458 = Hesapla

# Compare with
tbddcfc14 = Şununla karşılaştır

# Reading… {size} of {size2}
t80036d1c = Okunuyor… { $size } / { $size2 }

# Reading… {size}
t1569c388 = Okunuyor… { $size }

# Checksums of {name}
te88f3ed9 = { $name } sağlama toplamları

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = Dosyanın tamamı bu bilgisayardan geçer ve sağlayıcınız okunan veri için ücret alabilir.

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = Zayıf bir sağlama toplamı ({ $names }) bozuk bir dosyayı yakalar, ama eşleşsin diye bilerek değiştirilmiş bir dosyayı yakalamaz.

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = Yanına { $files } olarak kaydedildi; sha256sum -c ve benzerleri packwing olmadan denetler.

# Compare…
tec85be80 = Karşılaştır…

# Saving…
tbd79b37d = Kaydediliyor…

# Save beside it
t7f4d85e3 = Yanına kaydet

# Choose the checksums shown
teec532fe = Gösterilen sağlama toplamlarını seç

# Read it
t80fb8482 = Oku

# Done
t8dd31791 = Bitti

# Nothing chosen yet
tcf390b0a = Henüz bir şey seçilmedi

# {join} and {more} more
t8399ef70 = { $join } ve { $more } tane daha

# Hide what goes in
t5a420780 = Eklenecekleri gizle

# Show what goes in
t9a20eb53 = Eklenecekleri göster

# Take it out
t90a93ec7 = Çıkar

# Drop them to add them
t6a4aedbe = Eklemek için bırakın

# What goes in
t3b7c797a = Eklenecekler

# + Files
tba8a1e71 = + Dosya

# + Folder
tcef8b0ce = + Klasör

# Add the files or a folder to put in the archive.
t2e5c8c3e = Arşive konacak dosyaları veya bir klasörü ekleyin.

# Windows 11 and Linux open 7z on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tf7f64b21 = Windows 11 ve Linux 7z dosyalarını kendiliğinden açar; macOS, telefonlar ve eski Windows sürümleri için bir arşivleme uygulaması gerekebilir. Herkesle paylaşmak için .zip seçin.

# Linux, macOS and Windows 11 open it on their own; phones and older Windows may need an archiver app. To share with anyone, pick .zip.
t051031b1 = Linux, macOS ve Windows 11 bunu kendiliğinden açar; telefonlar ve eski Windows sürümleri için bir arşivleme uygulaması gerekebilir. Herkesle paylaşmak için .zip seçin.

# Linux and Windows 11 open it on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tc2c29183 = Linux ve Windows 11 bunu kendiliğinden açar; macOS, telefonlar ve eski Windows sürümleri için bir arşivleme uygulaması gerekebilir. Herkesle paylaşmak için .zip seçin.

# Most computers need an archiver app or the lz4 tool to open it. To share with anyone, pick .zip.
te9afe1c2 = Çoğu bilgisayarda açmak için bir arşivleme uygulaması veya lz4 aracı gerekir. Herkesle paylaşmak için .zip seçin.

# It holds this one file. Command-line tools open it; Windows and macOS may need an archiver app. To share with anyone, pick .zip.
t16463509 = Yalnızca bu tek dosyayı tutar. Komut satırı araçları açar; Windows ve macOS için bir arşivleme uygulaması gerekebilir. Herkesle paylaşmak için .zip seçin.

# Don’t split
t56ee6829 = Bölme

# 25 MB (email)
tc6ba1c9c = 25 MB (e-posta)

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB (USB bellek)

# Custom size…
t0fcf9e2d = Özel boyut…

# Size
ta6478e7c = Boyut

# Password
t2cc30838 = Parola

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…, age1pq1… veya ssh-ed25519 …

# Save here
ta1f147a6 = Buraya kaydet

# Use these keys
t1a01e439 = Bu anahtarları kullan

# No compression
t2e9c9e78 = Sıkıştırma yok

# {name} compression
te2db10f6 = { $name } sıkıştırma

# Several frames
t84cc41f7 = Birkaç çerçeve

# One frame
tc6ef759e = Tek çerçeve

# Small blocks
t04032e8a = Küçük bloklar

# Parts of 25 MB
ta999efe2 = 25 MB'lık parçalar

# Parts of 4 GB
t760df467 = 4 GB'lık parçalar

# Parts of {size}
tdd286479 = Parça boyutu: { $size }

# Parts of a size to type
tafd4b17d = Yazılacak boyutta parçalar

# Public key
t9981f45f = Genel anahtar

# Owners kept
t63e8a0df = Sahipler korunur

# Attributes kept
te9e8289c = Öznitelikler korunur

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = Bir yedekte sahipleri ve genişletilmiş öznitelikleri korumak için bir .tar biçimi seçin.

# Keep for a backup
t6a2bcc1a = Yedek için koru

# Owners: user and group
t5350e3fe = Sahipler: kullanıcı ve grup

# Extended attributes and ACLs
t35aff4d1 = Genişletilmiş öznitelikler ve ACL’ler

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = Bu bilgisayarda geri yüklenecek bir yedek için. Arşivi açan herkes kullanıcı adlarını görür.

# Beside the files
t757989af = Dosyaların yanına

# What it holds, from
teae4ef1a = İçeriğin kaynağı

# Save in
tf2d34837 = Kaydedilecek yer

# Folder…
tc8abcd43 = Klasör…

# Storage…
td0d8c806 = Depolama alanı…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = Parçalar önce bu bilgisayarın geçici klasöründe oluşturulur, sonra yüklenir; bu yüzden orada yer gerekir.

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext } önce bu bilgisayarın geçici klasöründe oluşturulur, sonra yüklenir; bu yüzden orada yer gerekir. .zip ve tar biçimleri doğrudan bu depolama alanına gider.

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = Önce bu bilgisayarın geçici klasöründe oluşturulur, sonra yüklenir; bu yüzden orada yer gerekir.

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } en çok { $most } boyutundaki dosyaları kabul eder. Daha büyük bir arşivin yüklenmesi başlarken başarısız olur.

# Choose a storage, and a folder in it
td7627e48 = Bir depolama alanı ve içinde bir klasör seçin

# New folder
t1926d127 = Yeni klasör

# Give it a name, without / or \ in it.
ta598b15b = Bir ad verin; içinde / veya \ olmasın.

# A file of that name is there already: this one will be {name}.
t9451c3b1 = Bu adda bir dosya zaten var: bu { $name } olacak.

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = Bazı adlar Windows'ta veya macOS'te kullanılamaz, örneğin “{ $name }”. Orada bu dosyalar olduğu gibi çıkarılamayabilir.

# Hide advanced
tae2a96e9 = Gelişmişi gizle

# Advanced options
t701d5ed9 = Gelişmiş seçenekler

# Large blocks
tdc489452 = Büyük bloklar

# Compression level
td386c9ef = Sıkıştırma düzeyi

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = Sakla dosyaları olduğu gibi koyar: en hızlı oluşturulan ve en büyük olan.

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = Daha yüksek düzey daha küçük bir arşiv üretir; arşivin oluşması uzun sürer, açılması değil.

# Frames
tf03e266f = Çerçeveler

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = Birkaç çerçeve her çekirdekte açılır ve arşivi biraz büyütür: yüzde yarımdan az.

# One frame makes the smallest archive, which opens on one core.
ta097b317 = Tek çerçeve en küçük arşivi verir; tek çekirdekte açılır.

# Blocks
t036a71e3 = Bloklar

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = Küçük bloklar her çekirdeği çalıştırır: iki kata kadar daha hızlı, arşiv ise yüzde birkaç daha büyük.

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = Büyük bloklar en küçük arşivi yapar; sıkıştırılacak az şey varken daha az çekirdekle.

# Split into parts
tca4436bd = Parçalara böl

# Each part
t642c444d = Her parça

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = Parçalar 7-Zip ve benzeri arşivleyicilerde açılır; Windows ve macOS'in yerleşik araçlarında açılmaz.

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = Bir parça en az 1 MB olmalı, örneğin 100 MB.

# Encrypt with age
t22784ad9 = age ile şifrele

# Protect with a password
tae8d8740 = Parola ile koru

# Suggest
ta6e44ce5 = Öner

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = Bunu güvenli bir yerde saklayın: olmadan bu arşiv bir daha açılamaz.

# Type a password, or turn this off.
t62b19318 = Bir parola yazın veya bunu kapatın.

# Hide file names too
t2198d77b = Dosya adlarını da gizle

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = Parola varsa Windows ve macOS'in yerleşik araçları bu ZIP'i açamaz. Alıcının bir arşivleme uygulaması olmalı. Dosya adları da görünür kalır; gizlemek için .7z seçin.

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = { $name } olarak kaydedildi. İçindeki adlar da şifrelidir. Windows ve macOS'te açmak için age aracı veya bir arşivleme uygulaması gerekebilir.

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
        [one] { $name } dosyasından { $n } anahtar
       *[other] { $name } dosyasından { $n } anahtar
    }

# Remove
t21a5901d = Kaldır

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
        [one] İçindeki { $n } anahtar, age'in şifrelemediği türde olduğu için dışarıda bırakıldı.
       *[other] İçindeki { $n } anahtar, age'in şifrelemediği türde olduğu için dışarıda bırakıldı.
    }

# From a file…
t40d7c944 = Dosyadan…

# Type a public key, or choose a file of them.
tcb028b06 = Bir genel anahtar yazın veya anahtarlardan oluşan bir dosya seçin.

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = Kuantum sonrası bir anahtar: bunu açmak için age aracı 1.3 veya daha yeni sürümde olmalı.

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = Bunu yalnızca eşi olan özel anahtar açar; o anahtar kaybolursa hiçbir şey açamaz.

# Weak
t6de87ae9 = Zayıf

# Fair
t82fa0d53 = Orta

# Strong
t0dad409a = Güçlü

# Choose what to put in it
t5461cfa6 = İçine konacakları seçin

# Convert
t92e29218 = Dönüştür

# New archive
tf352befb = Yeni arşiv

# Create
t990de47d = Oluştur

# {n} item | {n} items
tfff7fd6c =
    { $n ->
        [one] { $n } öğe
       *[other] { $n } öğe
    }

# Move {what} to the Trash?
tdf5a8d53 = { $what } Çöp Kutusu'na taşınsın mı?

# Delete {what}?
ta297d6c5 = { $what } silinsin mi?

# It can be put back from the system’s Trash.
te5e137a1 = Sistemin Çöp Kutusu'ndan geri konabilir.

# This cannot be undone from packwing.
t3947dab4 = Bu, packwing'den geri alınamaz.

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
        [one] ve { $n } tane daha
       *[other] ve { $n } tane daha
    }

# Move to Trash
ta036661b = Çöp Kutusu'na taşı

# Delete
t5797ea6a = Sil

# {many} items
t49912bd4 = { $many } öğe

# Rename {what}
t83401013 = Yeniden adlandır: { $what }

# New name
taf4cad88 = Yeni ad

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = Arşivden kalıcı olarak kaldırılır; klasörler içindekilerle birlikte. Bu geri alınamaz.

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = Bu arşiv imzalı. Bu değişiklikten sonra imzasıyla artık eşleşmez; yeniden imzalanana kadar.

# Rename
tc80631f3 = Yeniden adlandır

# The archive’s password
t8a168799 = Arşivin parolası

# one folder
td571ce35 = tek klasör

# one file
t95c9ec7b = tek dosya

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }, yani doğrudan burada

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }, yani yeni bir klasörde

# Extract here
t354232ec = Buraya çıkar

# {picked} selected
t9addc37e = { $picked } seçili

# All {counted}
t649feaf8 = Tümü: { $counted }

# What should it extract?
td12e23bf = Neler çıkarılsın?

# Smart
t54535972 = Akıllı

# Recommended
t7865024c = Önerilen

# Where should the files go?
td35f540c = Dosyalar nereye gitsin?

# Right here
t900a8a39 = Doğrudan buraya

# Another folder
t9f128ba8 = Başka bir klasör

# Choose…
t58daae9c = Seç…

# Goes to {destination}
t8195aed2 = Gideceği yer: { $destination }

# Checked as it goes
tb6c9570b = Çıkarılırken denetlenir

# · Nothing leaves the folder or passes the size limit
t11424797 = · Hiçbir şey klasörün dışına çıkmaz veya boyut sınırını aşmaz

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } İnternet'ten geldi. İşaretli kalacak; bu yüzden çalıştırılmadan önce size sorulur.

# Extract archive
t834ba3c0 = Arşivi çıkar

# Extract
t30518e1e = Çıkar

# Open {name}?
tcfe84c2f = { $name } açılsın mı?

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing bir kopyasını geçici bir klasöre koyar ve onu açar. Üzerinde yaptığınız değişiklikler arşive geri yazılmaz.

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = Bu bir program: açmak onu çalıştırır. Yalnızca arşivin geldiği yere güveniyorsanız yapın.

# Don’t ask again
t00c5d168 = Bir daha sorma

# Run it
t72430853 = Çalıştır

# Open
t538b10e9 = Aç

# Open from a storage
t45a8991d = Bir depolama alanından aç

# Choose a storage, and an archive in it
t3c900950 = Bir depolama alanı ve içinde bir arşiv seçin

# Choose
t73c66160 = Seç

# This is not a key file packwing can use: {why}.
tcf6986a7 = Bu, packwing'in kullanabileceği bir anahtar dosyası değil: { $why }.

# Could not read {path}: {e}.
t1dc6961f = { $path } okunamadı: { $e }.

# The passphrase
t1e921d00 = Parola ifadesi

# The key file’s passphrase
t4051f4d7 = Anahtar dosyasının parola ifadesi

# Enter the passphrase
tbd0672da = Parola ifadesini girin

# Passphrase
t191a2dc5 = Parola ifadesi

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = Bu dosya age ile şifrelenmiş. packwing parola ifadesini yalnızca arşiv açıkken tutar ve hiçbir yere yazmaz.

# Enter the password
t513ea7ab = Parolayı girin

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = Bu arşiv şifrelenmiş. packwing parolasını yalnızca arşiv açıkken tutar ve hiçbir yere yazmaz.

# Choose the key file
t3c521a4e = Anahtar dosyasını seçin

# Key file
t6ed7c612 = Anahtar dosyası

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = Bu dosya age ile bir anahtara şifrelenmiş: bir age anahtar dosyası veya bir SSH anahtarı. packwing ondan okuduğunu yalnızca arşiv açıkken tutar ve hiçbir yere yazmaz.

# Enter the key file’s passphrase
t808b85c5 = Anahtar dosyasının parola ifadesini girin

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = Anahtar dosyasının kendisi korunuyor. packwing parola ifadesini yalnızca arşiv açıkken tutar ve hiçbir yere yazmaz.

# kept, {size}
t001f6992 = tutuldu, { $size }

# left out
t73aeedd4 = atlandı

# renamed {to}
t7aa9b778 = { $to } olarak yeniden adlandırıldı

# It was gone by the time it was read.
t93fb1e03 = Okunacağı sırada artık yoktu.

# The job stopped before it was done.
t57a78885 = İş bitmeden durdu.

# Waiting for other jobs…
t16ec36d9 = Diğer işler bekleniyor…

# A file is damaged
t8b1230ed = Bir dosya hasarlı

# Some names can’t be used here
t26835ec8 = Bazı adlar burada kullanılamaz

# Converted, but some files are damaged
t54ef3472 = Dönüştürüldü, ancak bazı dosyalar hasarlı

# Extracted, but some files are damaged
t8226d624 = Çıkarıldı, ancak bazı dosyalar hasarlı

# Converted, with some files left out or renamed
t5cce0e50 = Dönüştürüldü, bazı dosyalar atlandı ya da yeniden adlandırıldı

# Extracted, with some files left out or renamed
t5dc002d6 = Çıkarıldı, bazı dosyalar atlandı ya da yeniden adlandırıldı

# Extracting…
t94279f82 = Çıkarılıyor…

# Uploading…
tc75fe478 = Yükleniyor…

# Creating…
t01fcd896 = Oluşturuluyor…

# Adding…
t2060c750 = Ekleniyor…

# Changing the archive…
t186fbef5 = Arşiv değiştiriliyor…

# Opening…
tc40f2321 = Açılıyor…

# Testing…
t19d1cb65 = Sınanıyor…

# Copying…
t02dd17e0 = Kopyalanıyor…

# Moving…
t890f7ed1 = Taşınıyor…

# Signing…
t7b8f8780 = İmzalanıyor…

# Splitting…
t5452adf5 = Bölünüyor…

# Joining…
taa884c51 = Birleştiriliyor…

# Converting…
ta1b8f4cc = Dönüştürülüyor…

# Couldn’t split the file
t83d27d26 = Dosya bölünemedi

# Couldn’t join the parts
t1cf67e00 = Parçalar birleştirilemedi

# Couldn’t convert the archive
t39f973a9 = Arşiv dönüştürülemedi

# Couldn’t sign
t4489f97c = İmzalanamadı

# Couldn’t copy
t613e4e9c = Kopyalanamadı

# Couldn’t move
t9796adec = Taşınamadı

# Couldn’t extract
tacd22c56 = Çıkarılamadı

# Couldn’t create the archive
t9d0e01bc = Arşiv oluşturulamadı

# Couldn’t add the files
tfbd989b0 = Dosyalar eklenemedi

# Couldn’t change the archive
tcdc4a49c = Arşiv değiştirilemedi

# Couldn’t open the file
t041198a4 = Dosya açılamadı

# The test failed
tfcd070a5 = Sınama başarısız oldu

# Starts when one of the others ends
tbe2a0509 = Diğerlerinden biri bitince başlar

# Paused
tde281feb = Duraklatıldı

# Starting…
t73727cef = Başlıyor…

# Finishing…
t042f47fe = Tamamlanıyor…

# Almost done
ta550a6a9 = Neredeyse bitti

# About {secs} seconds left
t3045da16 = Yaklaşık { $secs } saniye kaldı

# About a minute left
tf727815c = Yaklaşık bir dakika kaldı

# About {div_ceil} minutes left
tbd7a3305 = Yaklaşık { $div_ceil } dakika kaldı

# {name} · checking that every file reads
t0d56ec7c = { $name } · her dosyanın okunabildiği denetleniyor

# and {count} more | and {count} more
te728628b =
    { $n ->
        [one] ve { $count } tane daha
       *[other] ve { $count } tane daha
    }

# The test passed
tdf0ce240 = Sınama başarılı

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
        [one] { $count } dosya baştan sona okundu, { $size }, hatasız.
       *[other] { $count } dosya baştan sona okundu, { $size }, hatasız.
    }

# Close
tcd86acc3 = Kapat

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = “{ $path }” hasarlı: { $why }. Durdurulsun mu, atlansın mı, yoksa okunabilen kısmı tutulsun mu?

# Do the same for the other damaged files
t798693ad = Diğer hasarlı dosyalar için de aynısını yap

# Stop
t4b7f7705 = Durdur

# Leave it out
taaff8ec9 = Atla

# Keep what was read
tc4387d25 = Okunanı tut

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
        [one] { $count } dosya burada kendi adıyla çıkarılamıyor. Henüz hiçbir şey yazılmadı: durdurulsun mu, atlansın mı, yeniden mi adlandırılsın?
       *[other] { $count } dosya burada kendi adlarıyla çıkarılamıyor. Henüz hiçbir şey yazılmadı: durdurulsun mu, atlansın mı, yeniden mi adlandırılsın?
    }

# Leave out
ta50cc084 = Atla

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
        [one] { $count } bağlantı klasörün dışına, bu bilgisayarda başka bir yere gidiyor. Henüz hiçbir şey yazılmadı: atlansın mı, yoksa bu arşive güveniyorsanız olduğu gibi oluşturulsun mu?
       *[other] { $count } bağlantı klasörün dışına, bu bilgisayarda başka yerlere gidiyor. Henüz hiçbir şey yazılmadı: atlansınlar mı, yoksa bu arşive güveniyorsanız olduğu gibi oluşturulsunlar mı?
    }

# Create them
t9a4d6e7d = Oluştur

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
        [one] { $count } dosyanın verisi bozuktu.
       *[other] { $count } dosyanın verisi bozuktu.
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
        [one] { $count } dosya atlandı.
       *[other] { $count } dosya atlandı.
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
        [one] { $count } dosya, adı burada kullanılamadığı için yeniden adlandırıldı.
       *[other] { $count } dosya, adları burada kullanılamadığı için yeniden adlandırıldı.
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
        [one] { $count } dosya okunacağı sırada artık yoktu ve atlandı.
       *[other] { $count } dosya okunacağı sırada artık yoktu ve atlandı.
    }

# The rest is in the archive.
t977a5572 = Geri kalanı arşivde.

# The rest was extracted.
ta7833fcb = Geri kalanı çıkarıldı.

# Copy the list
t3c49b269 = Listeyi kopyala

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } / { $count2 } dosya · { $size } / { $size2 }

# Open the folder when it’s done
tf0f1017b = Bitince klasörü aç

# Run in background
tb800c0ef = Arka planda çalıştır

# Resume
t454d1d8a = Sürdür

# Pause
t44f9bf2d = Duraklat

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
        [one] { $n } iş hâlâ çalışıyor
       *[other] { $n } iş hâlâ çalışıyor
    }

# Closing packwing stops them
t2e6491b3 = packwing'i kapatmak onları durdurur

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = Bitmemiş olanlar yarım kalır: oluşturulan arşiv silinir, çıkarma geldiği yerde kalır.

# Keep running
t79c1b9c7 = Çalışmaya devam

# Stop them and quit
tf2707857 = Durdur ve çık

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = Dosyaları olduğu yerde kalır. packwing'in oturum açmak için kullandığı bilgiler sistemin anahtar zincirinden çıkarılır.

# The folder and its files stay where they are.
t4850aaa8 = Klasör ve dosyaları olduğu yerde kalır.

# Remove “{name}” from the list?
t5dd9cfcf = “{ $name }” listeden kaldırılsın mı?

# It can be added again at any time.
t80d51a99 = İstediğiniz zaman yeniden eklenebilir.

# A passphrase for the new key
tf5593f05 = Yeni anahtar için bir parola ifadesi

# The same passphrase again
tad5c08e6 = Aynı parola ifadesini tekrar girin

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = İmzalamak için Pro veya Enterprise lisansı gerekir: Ayarlar › Lisans. İmzaları denetlemek her planda ücretsizdir.

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = Bu, birkaç parçaya kesilmiş bir dosyanın tek parçası. Bir imza tek bir dosyayı kapsar ve hepsini kapsıyormuş gibi görünürdü; bu yüzden packwing parçaları imzalamaz.

# That passphrase does not open this key.
tc5357c97 = Bu parola ifadesi bu anahtarı açmıyor.

# This is not a key packwing can sign with: {why}.
t662c29f9 = Bu, packwing'in imzalayabileceği bir anahtar değil: { $why }.

# The two passphrases are not the same.
t335fe90b = İki parola ifadesi aynı değil.

# Keep the key here
t9306dafe = Anahtarı burada tut

# No key chosen
te58c13f5 = Anahtar seçilmedi

# Key
tcd1ac90c = Anahtar

# New key…
t9e1134da = Yeni anahtar…

# Again
t58fb56ef = Tekrar

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = Yeni bir anahtar, biri kırılsa da imza geçerli kalsın diye Ed25519 ve ML-DSA-65'i birlikte tutar. Bunları yalnızca packwing ve hivepack denetler: ssh-keygen için bir SSH Ed25519 anahtarıyla imzalayın. Parola ifadesi kaybolan bir anahtara geri dönüş yoktur.

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = İmza, yanına { $signature } olarak konur ve orada varsa onun yerine geçer. Dosyanın tamamı bu bilgisayardan geçer ve sağlayıcınız okunan veri için ücret alabilir.

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = İmza, yanına { $signature } olarak konur ve orada varsa onun yerine geçer.

# Sign {name}
ted250f31 = İmzala: { $name }

# Anyone can check it, with the key’s public half
t04ecc8be = Anahtarın genel yarısıyla herkes denetleyebilir

# Make the key…
t6bba28d5 = Anahtarı oluştur…

# Sign
t8f6340c4 = İmzala

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = Yanında { $first } ile başlayan { $count } parça. Birleştir onları yeniden bir araya getirir; packwing ve 7-Zip böyle bölünmüş bir arşivi olduğu gibi açar.

# Split
t16a15383 = Böl

# Move {what}
tc168ca56 = Taşı: { $what }

# Copy {what}
teda01f26 = Kopyala: { $what }

# Move here
t1aaf57aa = Buraya taşı

# Copy here
t85f0245a = Buraya kopyala

# Move
t7f2c2a34 = Taşı

# If a name is taken
tc99c037f = Ad kullanımdaysa

# They are in this folder already. Choose another.
t5318c14e = Bunlar bu klasörde zaten var. Başka birini seçin.

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = Hepsi bu bilgisayardan geçer ve sağlayıcınız okunan veri için ücret alabilir. Taşınan her asıl dosya, kopyası geri okunup eşleştiğinde silinir.

# Each original goes once its copy has been read back and matches.
t8ecbab4e = Her asıl dosya, kopyası geri okunup eşleştiğinde silinir.

# The originals stay where they are.
t5937f050 = Asıl dosyalar olduğu yerde kalır.

# Too big to show here
t1b339d35 = Burada gösterilemeyecek kadar büyük

# packwing can’t show this kind of file
tb909bf3c = packwing bu tür dosyayı gösteremez

# It could not be read
tb7927177 = Okunamadı

# It could not be read: {why}
t26005fe3 = Okunamadı: { $why }

# It could not be shown
tb60fb288 = Gösterilemedi

# Open with default app
t5ef80ec5 = Varsayılan uygulamayla aç

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = Başka bir iş şu an orada { $name } oluşturuyor: buna başka bir ad verin ya da onun bitmesini bekleyin.

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = Başka bir iş şu an { $folder } içine çıkarıyor: başka bir klasör seçin ya da onun bitmesini bekleyin.

# no editor command
tbf2a5d3a = düzenleyici komutu yok

# no text editor found
t6256b3da = metin düzenleyici bulunamadı

# Search this folder
t2c814039 = Bu klasörde ara

# This folder could not be read: {why}
t2c6017b2 = Bu klasör okunamadı: { $why }

# Could not load more: {why}
t466e87c4 = Daha fazlası yüklenemedi: { $why }

# Could not delete: {why}
te5951d38 = Silinemedi: { $why }

# the text editor
t91cdab0c = metin düzenleyici

# Cannot start {who}: {why}
t8deb160a = { $who } başlatılamıyor: { $why }

# Edit in text editor
td8d39fb8 = Metin düzenleyicide düzenle

# Edit in {editor}
te44cd57b = { $editor } ile düzenle

# Modified
tdd13a6b2 = Değiştirilme

# Loading…
tc4e2f181 = Yükleniyor…

# Only what is loaded was searched.
t39c5f536 = Yalnızca yüklenenlerde arandı.

# Search the whole folder
t41f9a787 = Tüm klasörde ara

# Load {n} more
t30bed46a = { $n } tane daha yükle

# Refresh
t0815aad4 = Yenile

# Hide hidden files
t3375c9c0 = Gizli dosyaları gizle

# Show hidden files
tf18c7057 = Gizli dosyaları göster

# View
t5e4baa18 = Görünüm

# Extract…
t4a09490e = Çıkar…

# Compress…
t690f9553 = Sıkıştır…

# Copy to…
t543cf15b = Şuraya kopyala…

# Move to…
t7a32320b = Şuraya taşı…

# Sign…
td6d61b10 = İmzala…

# Checksums…
t2891102d = Sağlama toplamları…

# Convert…
tae85a7c4 = Dönüştür…

# Split into parts…
t281db403 = Parçalara böl…

# Join the parts
t0ad56db8 = Parçaları birleştir

# Copy path
tf165cb2b = Yolu kopyala

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
        [one] { $n } öğeyi sıkıştır…
       *[other] { $n } öğeyi sıkıştır…
    }

# {n} loaded, more in this folder
t29a52578 = { $n } yüklendi, bu klasörde daha fazlası var

# Sorted among those loaded
t037d01ae = Yüklenenler arasında sıralandı

# {n} selected
tebda8468 = { $n } seçili

# Try again
t213e90fa = Tekrar dene

# Nothing here matches that.
t0f9a8a58 = Buna uyan bir şey yok.

# Add a storage to browse it.
t0b82c83a = Göz atmak için bir depolama alanı ekleyin.

# No folders or archives here.
t6a090cc3 = Burada klasör veya arşiv yok.

# No folders here.
tc7731c8f = Burada klasör yok.

# This folder is empty.
td9e47db6 = Bu klasör boş.

# Show in Finder
ta9442aa9 = Finder'da göster

# Show in File Explorer
t23c89eda = Dosya Gezgini'nde göster

# Show in the file manager
tb347ece9 = Dosya yöneticisinde göster

# Open in Finder
te2a467e0 = Finder'da aç

# Open in File Explorer
t3794d625 = Dosya Gezgini'nde aç

# Open in the file manager
td1ceabc4 = Dosya yöneticisinde aç

# Copy all
t6aef5c37 = Tümünü kopyala

# This system has no folder for settings.
tfd0ff0a1 = Bu sistemde ayarlar için bir klasör yok.

# Archive
t73690b65 = Arşiv

# Search inside this archive
tb35c0c27 = Bu arşivde ara

# Not text it can show
ta413eca5 = Gösterilebilecek bir metin değil

# It needs the password to show it
t81f5dbce = Göstermek için parola gerekir

# Open another…
tff74a552 = Başkasını aç…

# From a storage…
tcfba8fd5 = Bir depolama alanından…

# Safe to extract
t3c93bb70 = Çıkarmak güvenli

# Reading… {percent}%
t83223c27 = Okunuyor… %{ $percent }

# Read {fetched} of {whole}
t613dd932 = { $whole } içinden { $fetched } okundu

# Add files
tdd45c7bb = Dosya ekle

# Test
t2ffcbe05 = Sına

# Reading what’s inside…
t1bbda9f5 = İçindekiler okunuyor…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = Bu arşivde “{ $query }” ile eşleşen bir şey yok.

# Nothing in this folder matches “{query}”.
teecdfa3d = Bu klasörde “{ $query }” ile eşleşen bir şey yok.

# And {more} more. Type more to narrow it down.
t843253d0 = Ve { $more } tane daha. Daraltmak için daha fazla yazın.

# Open an archive
t8382bd3e = Bir arşiv açın

# ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…
t4a498ac8 = ZIP, 7z ve tar arşivleri, disk kalıpları ve paketler, sıkıştırılmış veya age ile şifrelenmiş: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…

# Open an archive…
t2cf2a8ee = Arşiv aç…

# Choose a file to see it here
t37854216 = Burada görmek için bir dosya seçin

# Symbolic link
tc7c20187 = Sembolik bağlantı

# Leads to {target}
te7ddae48 = Hedefi: { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
        [one] Bu klasördeki { $count } yürütülebilir dosya, çıkarılınca İnternet işaretini koruyacak.
       *[other] Bu klasördeki { $count } yürütülebilir dosya, çıkarılınca İnternet işaretini koruyacak.
    }

# Today
te7c0775e = Bugün

# Last 7 days
t35b9d98f = Son 7 gün

# Last 30 days
td0778b33 = Son 30 gün

# All days
tabb197ef = Tüm günler

# All results
t6235c0b2 = Tüm sonuçlar

# Not done
t0f88a146 = Tamamlanmadı

# Partly done
t3355a2a5 = Kısmen tamamlandı

# Failed
t5579c3d4 = Başarısız

# Cancelled
ta62b09bc = İptal edildi

# No end recorded
t38ab1371 = Bitişi kaydedilmedi

# Make an archive
t418bd6f0 = Arşiv oluştur

# Open a file from an archive
tc2dd3e41 = Arşivden dosya aç

# Preview a file
t1f706562 = Dosya önizleme

# Read a file from an archive
t4229020d = Arşivden dosya oku

# Add to an archive
t4e28d1fa = Arşive ekle

# Delete from an archive
t158044d3 = Arşivden sil

# Rename in an archive
tb3eef763 = Arşivde yeniden adlandır

# Check a signature
t6f441e5e = İmzayı denetle

# Make a signing key
tbdf6bcd0 = İmza anahtarı oluştur

# Join parts
t4eee8c5f = Parçaları birleştir

# Checksums
t47dd48d3 = Sağlama toplamları

# Audit log started
tfb8df56f = Denetim günlüğü başladı

# Audit log stopped
t54654bc3 = Denetim günlüğü durdu

# Audit log changed
td6b7bc4a = Denetim günlüğü değişti

# Old records taken away
t5f337ebf = Eski kayıtlar kaldırıldı

# All actions
t4ff15157 = Tüm işlemler

# Find a path, a name or an error
t30cb1a6c = Yol, ad ya da hata bul

# Read again
t2f0dcc5d = Yeniden oku

# Save as CSV…
tf103c3cc = CSV olarak kaydet…

# Checking that no record was changed…
t00deba79 = Hiçbir kaydın değiştirilmediği denetleniyor…

# The audit log cannot be read: {why}
t8783f4cf = Denetim günlüğü okunamıyor: { $why }

# A record was changed, taken out or put in by hand: in {file}, line {line}. | Records were changed, taken out or put in by hand, in {n} places: the first in {file}, line {line}.
t3cd011bd =
    { $n ->
        [one] Bir kayıt elle değiştirilmiş, çıkarılmış ya da eklenmiş: { $file }, satır { $line }.
       *[other] Kayıtlar { $n } yerde elle değiştirilmiş, çıkarılmış ya da eklenmiş: ilki { $file }, satır { $line }.
    }

# The oldest record follows one that is gone, and no record says it was taken away as past its time.
t4579f8a7 = En eski kayıttan önceki kayıt yok ve hiçbir kayıt onun süresi dolduğu için kaldırıldığını söylemiyor.

# {n} record, and none was changed or taken out. | {n} records, and none was changed or taken out.
tefc82060 =
    { $n ->
        [one] { $n } kayıt; hiçbiri değiştirilmemiş ya da çıkarılmamış.
       *[other] { $n } kayıt; hiçbiri değiştirilmemiş ya da çıkarılmamış.
    }

# Only the latest {most} records are shown: choose fewer days, or read all with pkw audit list.
t5fabfc51 = Yalnızca en yeni { $most } kayıt gösteriliyor: daha az gün seçin ya da tümünü pkw audit list ile okuyun.

# Saved to {path}.
te8346fa4 = { $path } konumuna kaydedildi.

# Could not save: {why}
t733a7cae = Kaydedilemedi: { $why }

# Record
t9f4565ec = Kayıt

# Started
t97b5823e = Başladı

# Ended
tec2b41db = Bitti

# Reading the audit log…
t403ccdff = Denetim günlüğü okunuyor…

# No records in these days.
t98687757 = Bu günlerde kayıt yok.

# No record is as chosen.
ta9ffcb9f = Seçime uyan kayıt yok.

# This computer
tfa0d176a = Bu bilgisayar

# Home
t52f50eae = Ev dizini

# Desktop
t64215c53 = Masaüstü

# Downloads
t539307de = İndirilenler

# Drop to open
tb255877b = Açmak için bırakın

# Browse
t0eac1f07 = Gözat

# Recent
t50e888fe = Son kullanılanlar

# Audit log
tcb83f9ca = Denetim günlüğü

# Choose a file to see its information here
t50aac02b = Bilgilerini burada görmek için bir dosya seçin

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
        [one] { $n } seçili
       *[other] { $n } seçili
    }

# Files in them come to {size}
t0cca93f1 = İçlerindeki dosyaların toplamı { $size }

# Delete…
t2fed07aa = Sil…

# Link to a folder
t4b408baf = Klasör bağlantısı

# Link
t8e08e589 = Bağlantı

# {tag} file
ta56bbbe9 = { $tag } dosyası

# {files}, {folders}
t72e46dc5 = { $files }, { $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
        [one] { $n } dosya
       *[other] { $n } dosya
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
        [one] { $n } klasör
       *[other] { $n } klasör
    }

# {size} so far
tc87daf59 = şimdiye dek { $size }

# Kind
t3ff50c63 = Tür

# Holds
tffe19c31 = İçeriği

# Permissions
t8700fd83 = İzinler

# Path
teb66e456 = Yol

# Signature
tc049050d = İmza

# Count size and files
t0f23db17 = Boyutu ve dosyaları say

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
        [one] İçindeki { $n } klasör okunamadı.
       *[other] İçindeki { $n } klasör okunamadı.
    }

# Check the signature
t9d44730e = İmzayı denetle

# Extracted
t3250471f = Çıkarıldı

# Archive created
t22c69419 = Arşiv oluşturuldu

# Moved
t82869bf0 = Taşındı

# Signed
t27c3ab35 = İmzalandı

# Joined
tf8f2b670 = Birleştirildi

# Converted
tc7b76e99 = Dönüştürüldü

# Waiting for a password or a key
t0e83ae32 = Bir parola ya da anahtar bekliyor

# Enter it…
tddb7844a = Gir…

# Show
t29742e5c = Göster

# Jobs
t4ae6a79d = İşler

# Clear finished
t30c178e8 = Bitenleri temizle

# Nothing runs in the background.
t2573f39e = Arka planda çalışan bir şey yok.

# Add to the right-click menu
t116d3868 = Sağ tık menüsüne ekle

# Extract or compress from any folder.
t20864983 = Herhangi bir klasörden çıkarın veya sıkıştırın.

# Welcome to packwing
tb38a86a6 = packwing'e hoş geldiniz

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = Her arşivi açın, paylaşmak için daha küçük dosyalar yapın ve zararlı indirmelerden korunun.

# Set things up
t63f9c1cd = Ayarları yapın

# You can change these later in Settings.
t95f41ffb = Bunları sonra Ayarlar'dan değiştirebilirsiniz.

# Get started
te83ca5ee = Başlayın

# Remove from Recent
t26cc8dba = Son kullanılanlardan kaldır

# {location} · not there any more
t7d0c09c7 = { $location } · artık yok

# The archives you open will show here.
tc0cc82d5 = Açtığınız arşivler burada görünür.

# Copy version info
t3d37483c = Sürüm bilgisini kopyala

# Version {version} · {os} {arch}
t6bead5ae = Sürüm { $version } · { $os } { $arch }

# A file archiver that puts every core to work and opens archives safely.
t5d2a9fa0 = Her çekirdeği çalıştıran ve arşivleri güvenle açan bir dosya arşivleyici.

# Updates
t483075a5 = Güncellemeler

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = Bu bir beta: daha yeni betalar ve onları izleyen kararlı sürüm size bildirilir.

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = Bu bir kararlı sürüm: daha yeni kararlı sürümler size bildirilir, betalar bildirilmez.

# Check now
t4f0e0677 = Şimdi denetle

# Look for updates once a day
tcc27ace1 = Günde bir kez güncelleme ara

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = Sürüm listesini GitHub'dan okur ve ona yalnızca imzaları doğrulandığında güvenir. Siz istemeden hiçbir şey indirilmez veya kurulmaz.

# Skip this version
t2c33d756 = Bu sürümü atla

# This build has no release keys, so it cannot tell of updates.
t378182cb = Bu derlemede sürüm anahtarları yok, bu yüzden güncellemeleri bildiremez.

# packwing {version} is ready
tf02650f8 = packwing { $version } hazır

# It is installed when you quit, or now with Restart.
t6fecac86 = Çıktığınızda ya da hemen Yeniden başlat ile kurulur.

# Restart now
t54001a7c = Şimdi yeniden başlat

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = Daha yeni bir sürüm aranıyor…

# Release notes
tc52ab62f = Sürüm notları

# packwing {version} is out
t36a30d8b = packwing { $version } yayımlandı

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = Bu komutları bir terminalde çalıştırın. İndirilen dosyayı imzalı sürüm listesiyle karşılaştırırlar ve sudo parolanızı sorar.

# {by} updates it: {command}
tdf2cab9c = { $by } tarafından güncellenir: { $command }

# {by} updates it.
t84b27295 = { $by } tarafından güncellenir.

# It is downloaded and checked, then installed when you quit.
tb46c0678 = İndirilir ve doğrulanır, sonra çıktığınızda kurulur.

# Download it, and install it in place of this one.
t39312eae = İndirin ve bunun yerine kurun.

# The download failed: {why}
t5e96c075 = İndirme başarısız oldu: { $why }

# Download and install
t1a99173f = İndir ve kur

# Download
t6964bb19 = İndir

# The last look failed: {why}
td122a092 = Son denetim başarısız oldu: { $why }

# Up to date (looked {when}).
t4665ef60 = Güncel (son denetim: { $when }).

# Not looked yet.
t5a3ba058 = Henüz denetlenmedi.

# Free software
t731220a4 = Özgür yazılım

# Under the GNU General Public License, version 3 or later
t4e98bf3c = GNU Genel Kamu Lisansı, sürüm 3 veya üzeri altında

# View license
tfc21d38b = Lisansı gör

# Made by HivePaaS
t10c7007a = HivePaaS yapımı

# On hivepack, our library for reading and writing archives
tfae09b61 = Arşivleri okuyup yazan kitaplığımız hivepack üzerine kurulu

# Source code
tf6a49f4d = Kaynak kodu

# Something wrong?
t610dfb6f = Bir sorun mu var?

# Tell us, with the version above
ta3e29034 = Yukarıdaki sürümle birlikte bize bildirin

# Report a problem
t7b1728e7 = Sorun bildir

# What changed
te8345543 = Neler değişti

# Every release, and what is new in it
t8e6f2052 = Her sürüm ve içindeki yenilikler

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS. packwing adı ve logosu HivePaaS'ın ticari markalarıdır.

# More from HivePaaS
tb25cf729 = HivePaaS'tan daha fazlası

# Free and open source, like packwing.
tc1ece130 = packwing gibi ücretsiz ve açık kaynak.

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = Linux, macOS ve Windows için hızlı, yerel bir SSH istemcisi.

# SSH and SFTP
t0f8af34d = SSH ve SFTP

# Follows your ssh_config
tc1c4446d = ssh_config dosyanızı izler

# A real terminal
tae9bcfa8 = Gerçek bir terminal

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = shellwing.app'i ziyaret edin

# A native HTTP(S) debugging proxy that keeps up with big sessions.
t86e5ad3e = Büyük oturumlara ayak uyduran yerel bir HTTP(S) hata ayıklama vekili.

# HTTP/2, WebSocket and SSE
t2c3c2377 = HTTP/2, WebSocket ve SSE

# A million requests, smooth
t60de9f6c = Bir milyon istek, akıcı

# Breakpoints and rewrites
t00c623a3 = Kesme noktaları ve yeniden yazma

# Phones and emulators
t8c26f435 = Telefonlar ve emülatörler

# Visit tracewing.app
t4668641f = tracewing.app'i ziyaret edin

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = Docker Swarm üzerine kurulu, uygulamalarınız için kendi sunucunuzda çalışan bir platform: Heroku, Render ve Coolify'a açık kaynaklı bir alternatif.

# 300+ one-click apps
t6a770b0a = 300+ tek tıkla kurulan uygulama

# Deploy on push
tb0bfc3e9 = Push ile dağıtım

# Pull request previews
t4e4180fc = Pull request önizlemeleri

# Automatic TLS
t9e9776ed = Otomatik TLS

# Live logs and metrics
tebe3a172 = Canlı günlükler ve metrikler

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = hivepaas.com sitesini ziyaret et

# Always
tad4bbd04 = Her zaman

# Extensions, apart by spaces
tc709375f = Uzantılar, boşlukla ayrılmış

# Format of new archives
t3b50d238 = Yeni arşivlerin biçimi

# New archive starts with it; you can pick another there.
t7b361b55 = Yeni arşiv bununla başlar; orada başkasını seçebilirsiniz.

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = Sakla, ZIP ve 7z arşivlerinin dosyalarını olduğu gibi koyar. tar biçimleri En düşük alır, çünkü sıkıştırmasız olan yalnızca .tar'dır.

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = Daha yüksek düzey daha küçük arşivler üretir; arşivlerin oluşması uzun sürer, açılması değil.

# Write straight to a storage above
ta4975128 = Doğrudan yukarıdaki bir depolama alanına yaz

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = Büyük dosyalar bu bilgisayarda yer kaplamaz, ama bağlantı kesilirse yeniden sıkıştırmak gerekir. S3 bu yolla tar ve ZIP kabul eder, SFTP ise her biçimi.

# Don’t compress what already is
t940ec31d = Zaten sıkıştırılmış olanı sıkıştırma

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = ZIP ve 7z için: resimler, video, ses ve arşivler olduğu gibi girer; daha hızlı ve daha büyük değil

# What goes in as it is
tc0104b29 = Olduğu gibi girenler

# Files whose names end in these, whatever their case
ta3acef9b = Adı bunlarla biten dosyalar, büyük küçük harf fark etmez

# Reset to default
t0db9f7dc = Varsayılana sıfırla

# Leave out Mac-only files
t27eec14f = Yalnızca Mac'e ait dosyaları dışarıda bırak

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store, __MACOSX ve kaynak çatalları: Mac dışında gereksizdir

# Hide file names behind the password
t93ff7715 = Dosya adlarını parolanın arkasında gizle

# For 7z: nobody sees what is inside without it
t0f5354d7 = 7z için: parola olmadan kimse içindekileri göremez

# Yours
t2635ea3b = Sizin

# By name
t983e37f7 = Ada göre

# By number
tcbbada8d = Numaraya göre

# No limit
t6de7f47d = Sınır yok

# Where the files go
t77e60aba = Dosyaların gideceği yer

# Where Extract starts: you can pick another there.
t4b73082f = Çıkarma işleminin başladığı yer: orada başkasını seçebilirsiniz.

# One item right here, more in a new folder
ta5a4d31d = Bir öğe doğrudan burada, daha fazlası yeni bir klasörde

# A new folder next to it
t2095cbe9 = Yanında yeni bir klasör

# Named after the archive
tb991d4c1 = Arşivin adını taşıyan

# Next to the archive
t4691c928 = Arşivin yanına

# Straight into its folder
t99d1df43 = Doğrudan kendi klasörüne

# When a file is already there
tca514dda = Dosya zaten varsa

# A file of the same name in the folder the files go to
t296ad686 = Dosyaların gideceği klasörde aynı adlı bir dosya

# Largest an archive may unpack to
t5957c426 = Bir arşivin açılabileceği en büyük boyut

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = Arşiv bombalarını durdurur: diski doldurmak üzere açılan küçük dosyalar

# Only names every system can use
tf47f4968 = Yalnızca her sistemin kullanabildiği adlar

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = Windows'un kullanamadığı a:b.txt gibi adlar ve yalnızca büyük/küçük harfle ayrılan adlar bu bilgisayarda da sorulur

# Owners from a backup
t38a7cf66 = Yedekten sahipler

# Where a tar archive kept them: yours, or the archive’s by name or by number. Only an administrator gives files to others
t46908bee = Bir tar arşivi onları koruduysa: sizin ya da arşivin, ada veya numaraya göre. Dosyaları başkalarına yalnızca bir yönetici verebilir

# Extended attributes from a backup
t764eed3a = Yedekten genişletilmiş öznitelikler

# Where a tar archive kept them, ACLs among them
ta28f3ae5 = Bir tar arşivi onları koruduysa, ACL’ler dahil

# Light
t8ad2f8cf = Açık

# Dark
t45570065 = Koyu

# System
t8f3184bc = Sistem

# Same as system ({language})
t5bd525d3 = Sistemle aynı ({ $language })

# The system’s
t1d68c60b = Sistemin

# Custom…
ta391dade = Özel…

# The system’s ({size} px)
ta51f1877 = Sistemin boyutu ({ $size } px)

# All ({count})
tf65ec196 = Tümü ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = örn. code -n veya /path/to/editor --flag

# Language
t9a73db9b = Dil

# Appearance
t31018b9f = Görünüm

# Text size
t2c041143 = Yazı boyutu

# Processor cores to use
t2f9c3d17 = Kullanılacak işlemci çekirdekleri

# Text editor
t1ca82aab = Metin düzenleyici

# What Edit opens a file of a storage in
t62506ced = Bir depolama alanındaki dosyayı Düzenle ile açan uygulama

# The file’s path is added at the end.
teba749d4 = Dosyanın yolu sona eklenir.

# Paste your license key
t3c4da5cf = Lisans anahtarınızı yapıştırın

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = Dosya imzalama dışındaki her özellik ve ağ üzerinde { $count } depolama alanı. Bu bilgisayardaki klasörler sınırlı değildir ve her imza denetlenebilir.

# Every feature, and any number of storages.
tc3e75ecf = Her özellik ve sınırsız sayıda depolama alanı.

# Every feature, and any number of storages, for your organization.
t69b29f66 = Her özellik ve kuruluşunuz için sınırsız sayıda depolama alanı.

# Plan
t1eab5e32 = Plan

# Issued to
t44ef5cad = Lisans sahibi

# Who this license is for
td514fb0d = Bu lisansın kime ait olduğu

# Seats
t8f706321 = Kullanıcı sayısı

# What your organization’s agreement covers
t60b0c6c9 = Kuruluşunuzun sözleşmesinin kapsadığı

# Trial, until {date}
t9ffc2b63 = Deneme, { $date } tarihine kadar

# Until {date}
t4d9bac91 = { $date } tarihine kadar

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = { $date } tarihinde bitti. { $days } gün daha çalışmaya devam eder.

# Ended {date}
tfccb9305 = { $date } tarihinde bitti

# Valid
td2661011 = Geçerli

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = Bundan sonra packwing Ücretsiz olur. Depolama alanı eklemek dışında hiçbir şey elinizden alınmaz.

# Your organization manages this license.
te42a0ee6 = Bu lisansı kuruluşunuz yönetiyor.

# Activate
t3b8f0190 = Etkinleştir

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = Anahtar bu bilgisayarda denetlenir. Hiçbir yere bir şey gönderilmez ve hiçbir lisans, kuruluşunuzun ilkelerinin yaptığı şeyi değiştirmez.

# On
t60e47df0 = Açık

# They show when you right-click files, folders and archives.
te216d656 = Dosyalara, klasörlere ve arşivlere sağ tıkladığınızda görünürler.

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = packwing şu an menüde değil: menüye girdiğinde bunlar da girer.

# Changing the menu…
tc7f1a2f0 = Menü değiştiriliyor…

# Good to know
t8d7b3a90 = Bilmekte fayda var

# The menu could not be changed: {why}
t9fac5c06 = Menü değiştirilemedi: { $why }

# Show packwing in the right-click menu
t5d0d23cc = packwing'i sağ tık menüsünde göster

# Extract and compress from any folder
t94a46acb = Herhangi bir klasörden çıkarın ve sıkıştırın

# Items in the menu
t67537dc4 = Menüdeki öğeler

# Put them in a submenu
te3449a65 = Alt menüye koy

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 bunları hem kendi menüsünde hem de “Daha fazla seçenek göster” altında tek bir Packwing öğesinde toplar

# One packwing item, with these under it
tb5fdc584 = Tek bir packwing öğesi, bunlar altında

# 30 days
t7a572af9 = 30 gün

# 90 days
t2d8a9817 = 90 gün

# 1 year
t4d0a98a5 = 1 yıl

# Keep all
te0b1775b = Tümünü sakla

# Keep the “downloaded from the Internet” mark
tfe574fa2 = “İnternet'ten indirildi” işaretini koru

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = Sistem, indirilmiş bir arşivden gelen dosyaları çalıştırmadan önce uyarır

# This system has no such mark, so this changes nothing here
t314d60c1 = Bu sistemde böyle bir işaret yok; bu yüzden burada bir şey değişmez

# License
t38e70ee4 = Lisans

# Your organization’s, in Settings › License
tff7c594e = Kuruluşunuzun lisansı, Ayarlar › Lisans'ta

# Treat files from storages as from the Internet
t0af8d466 = Depolama alanlarındaki dosyaları İnternet'ten gelmiş say

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = S3, WebDAV ve SFTP'den gelenler de işaret alır

# Require encryption for new archives
t0c3e1337 = Yeni arşivler için şifrelemeyi zorunlu kıl

# Applies to archives made and files added here, not to other apps
tc889f3a2 = Burada yapılan arşivler ve eklenen dosyalar için geçerlidir, başka uygulamalar için değil

# Kinds of encryption allowed
t1b8b2116 = İzin verilen şifreleme türleri

# New archives can be encrypted only these ways
t71dcd29c = Yeni arşivler yalnızca bu yollarla şifrelenebilir

# Minimum password length
tb7d64a44 = En düşük parola uzunluğu

# In characters, for every new archive with a password
tca6d6982 = Karakter olarak, parolalı her yeni arşiv için

# Keep an audit log
tf7fd8762 = Denetim günlüğü tut

# A record of what packwing does, whatever the license
t34fe4e25 = Lisanstan bağımsız olarak packwing'in yaptıklarının kaydı

# Where the audit log is kept
tbd999abe = Denetim günlüğünün yeri

# A folder your organization chose
td4e1f74a = Kuruluşunuzun seçtiği klasör

# How long the audit log is kept
td504acaf = Denetim günlüğünün saklanma süresi

# In days; 0 keeps every record
t127a3c54 = Gün olarak; 0 tüm kayıtları saklar

# Most the audit log may take
ta334bb40 = Denetim günlüğünün en büyük boyutu

# In MB; the oldest records go first
t03a543d2 = MB olarak; en eski kayıtlar önce silinir

# What the audit log says of archives
t8d6fca43 = Denetim günlüğünün arşivler hakkında yazdıkları

# “entries” names each file extracted, and those found damaged or left out
t1e33aba6 = “entries”, çıkarılan her dosyayı ve hasarlı bulunan ya da dışarıda bırakılanları adlandırır

# Every job recorded
t42369815 = Her işlem kaydedilir

# A job whose start the audit log cannot write does not start
tda48e1c9 = Başlangıcı denetim günlüğüne yazılamayan bir işlem başlamaz

# The audit log in the system’s log
t4a2fcd04 = Sistem günlüğünde denetim günlüğü

# Each record goes to {log} as well, where your organization’s tools read it
t3a4c104f = Her kayıt { $log } günlüğüne de yazılır; kuruluşunuzun araçları onu oradan okur

# packwing has no policy of this name.
t7e03649c = packwing'in bu adda bir ilkesi yok.

# Ask before opening files without extracting
t76d5d951 = Çıkarmadan dosya açmadan önce sor

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = Arşivdeki bir dosyayı açmak, geçici bir klasöre bir kopya koyar. Bir program için her zaman sorulur.

# Paste a public key
t0bf72f73 = Bir genel anahtar yapıştırın

# What packwing does, kept on this computer: archives extracted, made, converted and copied, when and by whom. Never a password or what is in a file.
t2162955d = packwing'in yaptıkları, bu bilgisayarda tutulur: çıkarılan, oluşturulan, dönüştürülen ve kopyalanan arşivler, ne zaman ve kim tarafından. Asla bir parola ya da dosya içeriği yazılmaz.

# Keep its records for
tf423722f = Kayıtları şu süre sakla

# Older ones are taken away, and that is recorded too
t7073f273 = Daha eskileri silinir ve bu da kaydedilir

# Takes a Pro or Enterprise license: Settings › License.
t2774cee6 = Pro veya Enterprise lisansı gerekir: Ayarlar › Lisans.

# Where the audit log is
t2702fcc6 = Denetim günlüğünün yeri

# View records
t27c5284a = Kayıtları gör

# Open folder
t713a842f = Klasörü aç

# The last record could not be written: {why}
t542148a1 = Son kayıt yazılamadı: { $why }

# pkw audit verify checks that no record was changed or taken out. Whoever runs this computer can still remove the log: it tells what packwing did, it does not stop anyone.
t482581ff = pkw audit verify hiçbir kaydın değiştirilmediğini ya da silinmediğini denetler. Bu bilgisayarı yöneten kişi günlüğü yine de silebilir: günlük packwing'in ne yaptığını söyler, kimseyi durdurmaz.

# This is not a public key packwing can check with: {why}.
td57933b9 = Bu, packwing'in denetleyebileceği bir genel anahtar değil: { $why }.

# This line cannot be read, so the key is not used
t90e7dd7b = Bu satır okunamıyor; bu yüzden anahtar kullanılmıyor

# Keys you trust
ted99ced8 = Güvendiğiniz anahtarlar

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = Bunlardan biriyle atılan imza, dosyayı kimin imzaladığını söyler. Başka bir anahtarla atılan imza, eşleşse bile hiçbir şeyi kanıtlamaz.

# Choose a file…
t28a576c9 = Dosya seç…

# {name} (weak)
tdb06c037 = { $name } (zayıf)

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = Bir dosyanın menüsündeki Sağlama toplamları… öğesinin gösterdikleri; dosyanın alındığı yerde verilen toplamla karşılaştırmak için. Zayıf olan bozuk bir dosyayı yakalar, ama bilerek değiştirilmiş olanı yakalamaz.

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = İlkeleri okunamıyor; bu yüzden koruduğu şey reddediyor: { $why }

# Some settings are managed by your organization
t7db7e7ec = Bazı ayarları kuruluşunuz yönetiyor

# Get help
t4a1ed54a = Yardım al

# Not understood
t068eea1b = Anlaşılamadı

# Off
t0dc6b4aa = Kapalı

# General
t5561fd0b = Genel

# Extracting
tc82af3f2 = Çıkarma

# Compressing
t6d183c73 = Sıkıştırma

# Security
tc2fe21db = Güvenlik

# Right-click menu
t1287ae97 = Sağ tık menüsü

# About
t58e096de = Hakkında

# Settings
t4b058728 = Ayarlar

# This computer has no way to ask that it is you.
t0c1b4d43 = Bu bilgisayarın, sizin olduğunuzu sorma yolu yok.

# open a storage
t780455bf = bir depolama alanını açmak

# Open a storage in packwing
tf861e014 = packwing'de bir depolama alanı aç

# That was not it. It stays locked.
tbb2a9838 = Bu o değildi. Kilitli kalıyor.

# The system could not ask: {why}
td6905afe = Sistem soramadı: { $why }

# This copy of packwing cannot open it.
tfb56d8cf = Bu packwing kopyası bunu açamaz.

# Storages
tc776c52f = Depolama alanları

# Add storage
te858d6b5 = Depolama alanı ekle

# Type the password you log in to this computer with to open it.
tb544cd9b = Açmak için bu bilgisayarda oturum açtığınız parolayı yazın.

# Show this computer it is you to open it.
tfb4c635b = Açmak için bu bilgisayara sizin olduğunuzu gösterin.

# Checking…
t87f62cb7 = Denetleniyor…

# It asks again after 5 minutes left unused.
t69c22a3b = 5 dakika kullanılmazsa yeniden sorar.

# Edit…
t8e7efcd7 = Düzenle…

# Remove from this list
t8c7c7c57 = Bu listeden kaldır

# Extract here with packwing
tc5e54e4a = packwing ile buraya çıkar

# Extract to new folder
tda62691f = Yeni klasöre çıkar

# Extract with packwing…
tda9255a0 = packwing ile çıkar…

# Open with packwing
teb657427 = packwing ile aç

# Compress to ZIP
t344b1175 = ZIP olarak sıkıştır

# Compress with packwing…
t9ec9b77d = packwing ile sıkıştır…

# packwing cannot find its program: {e}
t1934ce08 = packwing programını bulamıyor: { $e }

# Signed by {signer}
t8eb35b10 = İmzalayan: { $signer }

# Signed by {signer}, not checked yet
t88b7186a = İmzalayan: { $signer }, henüz denetlenmedi

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = Güvenmediğiniz bir anahtarla imzalanmış: { $fingerprint }

# It does not match its signature
tb7d0e1ed = İmzasıyla eşleşmiyor

# Its signature cannot be read
tadf2a401 = İmzası okunamıyor

# Its signature could not be checked
t69cbbb51 = İmzası denetlenemedi

# Checking its signature…
tda8c2e03 = İmzası denetleniyor…

# the system's package manager
t11465c20 = sistemin paket yöneticisi

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = İndiriliyor… { $got } / { $total }

# Downloading… {got}
t4873b921 = İndiriliyor… { $got }

# just now
ta715ef0f = az önce

# {n} minute ago | {n} minutes ago
t6c7302d6 =
    { $n ->
        [one] { $n } dakika önce
       *[other] { $n } dakika önce
    }

# {n} hour ago | {n} hours ago
t137b052e =
    { $n ->
        [one] { $n } saat önce
       *[other] { $n } saat önce
    }

# {n} day ago | {n} days ago
tb9fa7744 =
    { $n ->
        [one] { $n } gün önce
       *[other] { $n } gün önce
    }

# No folder to download to.
t2bfc1370 = İndirme için klasör yok.

# Update available
t676409df = Güncelleme var

# Restart to update
tbbbbda26 = Güncellemek için yeniden başlat

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
        [one] { $n } dosya, adı kullanımda olduğu için atlandı.
       *[other] { $n } dosya, adları kullanımda olduğu için atlandı.
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
        [one] { $n } bağlantı olduğu yerde kaldı.
       *[other] { $n } bağlantı olduğu yerde kaldı.
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = Bu dosya bir parola ifadesiyle şifrelenmiş.

# This archive needs its password.
td46b69a1 = Bu arşivin parolası gerekiyor.

# This file is encrypted to a key.
tbd048b4e = Bu dosya bir anahtara şifrelenmiş.

# This key file needs its passphrase.
t81f4c775 = Bu anahtar dosyasının parola ifadesi gerekiyor.

# {paths} archives
t27f3b860 = { $paths } arşiv

# {folder}, a folder for each
t75e7db7c = { $folder }, her biri için bir klasör

# New
t8b242db1 = Yeni

# Your organization requires each job to be recorded, and the audit log cannot be written: {why}
t3f254f09 = Kuruluşunuz her işlemin kaydedilmesini istiyor ve denetim günlüğü yazılamıyor: { $why }

# There is no file to save it beside.
ta9906e3b = Yanına kaydedilecek dosya yok.

# Nothing in the archive matches the patterns given.
t90b19482 = Arşivde verilen kalıplarla eşleşen bir şey yok.

# Nothing to compress.
t47339885 = Sıkıştırılacak bir şey yok.

# {path} is too big.
te9ab879e = { $path } çok büyük.

# This format holds one file, and no more.
t75c197d9 = Bu biçim tek bir dosya tutar, fazlasını değil.

# Store
t6da32dce = Sakla

# Lowest
t5d7dbfdb = En düşük

# Low
tb1dd9521 = Düşük

# Medium
t09be5f96 = Orta

# High
tbe0a061d = Yüksek

# Highest
t9487b22f = En yüksek

# Could not read {path}: {e}
tad0120eb = { $path } okunamadı: { $e }

# {path} has no name.
tbd0e16dc = { $path } için ad yok.

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = { $path } adı geçerli Unicode değil; bu yüzden arşive konamaz.

# Could not write {path}: {e}
t8dd055c6 = { $path } yazılamadı: { $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } ve ondan sonraki adlar kullanımda.

# Could not encrypt: {e}
t485d71ab = Şifrelenemedi: { $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = { $level } yaklaşık { $needed } bellek ister ve boşta { $available } var: yaklaşık { $less } isteyen { $lower } ile oluşturulur.

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = Bu düzey yaklaşık { $needed } bellek ister ve boşta { $available } var: bilgisayar yavaşlayabilir ya da packwing durabilir.

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = ZIP veya 7z arşivi age anahtarlarını yalnızca age onu bütünüyle şifrelediğinde alır.

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age bir parola ifadesine veya anahtarlara şifreler, ikisine birden değil.

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = Kuantum sonrası anahtarlar diğer anahtarlarla kullanılamaz: diğerleri dosyayı bir kuantum bilgisayara açık bırakırdı.

# There are no keys in this file to encrypt to.
tc2809b2c = Bu dosyada şifrelenecek anahtar yok.

# {path} got shorter while it was being read.
t2e221d1f = { $path } okunurken kısaldı.

# A password has to be valid Unicode.
t5ef729bc = Parola geçerli Unicode olmalı.

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = { $path } için genişletilmiş öznitelikler okunamadı: { $e }

# Could not start compressing: {e}
tc4faa70c = Sıkıştırma başlatılamadı: { $e }

# Could not compress: {e}
tf6ff23f2 = Sıkıştırılamadı: { $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } bir bağlantı; bağlantılar yalnızca Unix'te korunur.

# Nothing changed.
tedfc294c = Hiçbir şey değişmedi.

# The archive did not come out whole.
t2f015c79 = Arşiv eksiksiz çıkmadı.

# packwing can change only a ZIP or 7z archive that is one file.
t675882a8 = packwing yalnızca tek dosyalık bir ZIP veya 7z arşivini değiştirebilir.

# That is not in the archive.
t7530c419 = Bu arşivde yok.

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = Arşivin tuttuğu bunların hepsi. Onun yerine arşivin kendisini silin.

# packwing can change only a ZIP archive that is one file.
t48628036 = packwing yalnızca tek dosyalık bir ZIP arşivini değiştirebilir.

# “{to}” cannot be a name.
td53e55bf = “{ $to }” ad olamaz.

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = “{ $to }” bu klasörde zaten var. Hiçbir şey değiştirilmedi.

# packwing can add files only to a ZIP or 7z archive that is one file.
t8d1ef729 = packwing yalnızca tek dosyalık bir ZIP veya 7z arşivine dosya ekleyebilir.

# An archive can’t go inside itself.
tff43ce74 = Bir arşiv kendi içine konamaz.

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = “{ $bare }” arşivde zaten var. Hiçbir şey değiştirilmedi.

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = Yüklenmeden önce bu arşiv { $folder } içinde oluşturulur ve orada { $need } kadar yer tutabilir. Orada yalnızca { $free } boş.

# The upload stopped.
t528232e6 = Yükleme durdu.

# “{pattern}” names no file or folder.
te17b2bfa = “{ $pattern }” hiçbir dosya ya da klasörü belirtmiyor.

# a colon, which Windows does not allow
t3ba6a1aa = Windows'un izin vermediği iki nokta

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = Windows'un izin vermediği bir karakter (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = Windows'un bir aygıt için ayırdığı ad

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = Windows'un izin vermediği şekilde boşluk ya da noktayla bitiyor

# longer than 255 bytes
t9218e5c0 = 255 bayttan uzun

# a control character
t378bba3a = bir denetim karakteri

# not a name this system can read
t72b79e1d = bu sistemin okuyabildiği bir ad değil

# the same file here as another, its name differing only in case
t1eead41f = burada başka bir dosyayla aynı, adı yalnızca büyük/küçük harfle farklı

# a link that leads outside the folder
t00fdc25e = klasörün dışına giden bir bağlantı

# {name} is already in the folder.
tbe7fab8a = { $name } klasörde zaten var.

# Could not make a folder in {parent}.
tec7ca3a9 = { $parent } içinde klasör oluşturulamadı.

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } bir klasör ve bir dosya onun yerine geçerdi.

# {name} is already in the folder
t5af584a0 = { $name } klasörde zaten var

# This file is not an archive packwing can open.
t660f149b = Bu dosya packwing'in açabileceği bir arşiv değil.

# The password is wrong, or the archive is damaged.
ta4d3227d = Parola yanlış veya arşiv bozuk.

# The passphrase is wrong.
t7826c649 = Parola ifadesi yanlış.

# The key file’s passphrase is wrong.
t8ad75e3a = Anahtar dosyasının parola ifadesi yanlış.

# This file is encrypted to a key packwing does not have.
te162af76 = Bu dosya, packwing'de olmayan bir anahtara şifrelenmiş.

# Cancelled.
t27c010d6 = İptal edildi.

# the archive could not be read
t2267c3b1 = arşiv okunamadı

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = Bu çıkarılamaz, çünkü bir dosyanın adı burada kullanılamıyor: “{ $name }” ({ $why }).

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
        [one] { $n } bayt
       *[other] { $n } bayt
    }

# A volume of this archive is not there.
t3803103c = Bu arşivin bir cildi yok.

# {named}: its data is damaged.
t066227f9 = { $named }: verisi bozuk.

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = Arşiv bozuk: UnRAR { $number } koduyla çıkardı.

# There is no public key here.
t9fd24078 = Burada genel anahtar yok.

# Could not make the key: {e}
t73610ecc = Anahtar oluşturulamadı: { $e }

# There is no file to sign.
tc95e3a9c = İmzalanacak dosya yok.

# {name} is not there any more.
t07c1f2bc = { $name } artık yok.

# A part has to hold at least one byte.
te779b111 = Bir parça en az bir bayt içermelidir.

# {path} is not a file.
tdfee38b2 = { $path } bir dosya değil.

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = Bu, numaralandırılabilecekten çok parça çıkarır: parçaları büyütün.

# {part} is there already.
t2577cbb6 = { $part } zaten var.

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path }, 7-Zip’in böldüğü gibi bölünmüş bir dosyanın parçası değil (name.001 gibi).

# {part} is missing: the parts start from it.
t5e65a10a = { $part } eksik: parçalar ondan başlar.

# {part} is missing, and the parts after it are there.
t24165c89 = { $part } eksik, ama ondan sonraki parçalar duruyor.

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = { $part } { $got } içeriyor, ilk parça gibi { $size } değil: eksik olabilir.

# The name is empty.
t1a9d8e4e = Ad boş.

# That name is kept for the system.
t74c8683a = Bu ad sisteme ayrılmış.

# A name cannot hold / or other special characters.
t4c0596ec = Ad / veya başka özel karakterler içeremez.

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = Windows'ta bir ad nokta veya boşlukla bitemez.

# That name is taken in this folder.
t43a2bce1 = Bu ad bu klasörde kullanımda.

# {name} is already there.
tb5e7b134 = { $name } zaten orada.

# {name} cannot go inside itself.
t99f437a6 = { $name } kendi içine konamaz.

# Stopped.
t15558940 = Durduruldu.

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } bir yerde dosya, diğerinde klasör.

# The copy of {name} does not match. The original was kept.
t779a64f3 = { $name } kopyası eşleşmiyor. Asıl dosya korundu.

# This is not a valid packwing license key.
td5ae978f = Bu geçerli bir packwing lisans anahtarı değil.

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = Kuruluşunuzun bu konudaki ilkesi okunamıyor; bu yüzden packwing bunu yapmaz. BT biriminize danışın.

# Your organization requires new archives to be encrypted.
t83acf110 = Kuruluşunuz yeni arşivlerin şifrelenmesini zorunlu kılıyor.

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = Kuruluşunuz bu tür şifrelemeye izin vermiyor: { $method }.

# Your organization requires a password of at least {least} characters.
t2d21b125 = Kuruluşunuz en az { $least } karakterlik bir parola zorunlu kılıyor.

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = Bu parola tahmin edilmesi çok kolay ve kuruluşunuz daha iyisini zorunlu kılıyor.

# Expected true or false, not {other}.
tcadcff7a = Doğru veya yanlış bekleniyordu, { $other } değil.

# Expected a number from 1 to {most}, not {other}.
tda498fad = 1 ile { $most } arası bir sayı bekleniyordu, { $other } değil.

# Expected some text, not {other}.
t9386ce3b = Bir metin bekleniyordu, { $other } değil.

# Expected a whole path, such as /var/log/packwing or \\server\audit, not {other}.
t00d3565d = /var/log/packwing veya \\server\audit gibi tam bir yol bekleniyordu, { $other } değil.

# Expected a number of days from 0 (keep all) to {most}, not {other}.
tf8fba675 = 0 (tümünü sakla) ile { $most } arası bir gün sayısı bekleniyordu, { $other } değil.

# Expected a size in MB from 0 (no limit) to {most}, not {other}.
t9f7e002d = 0 (sınırsız) ile { $most } arası MB cinsinden bir boyut bekleniyordu, { $other } değil.

# Expected summary or entries, not {other}.
t27e9b96f = summary veya entries bekleniyordu, { $other } değil.

# Expected an address starting with https://, not {other}.
t93bfd19e = https:// ile başlayan bir adres bekleniyordu, { $other } değil.

# Expected a list, not {other}.
tc945cc93 = Bir liste bekleniyordu, { $other } değil.

# No kind of encryption is called {name}.
tfddb7ba6 = { $name } adında bir şifreleme türü yok.

# The list names no kind of encryption.
t5f8076f5 = Liste hiçbir şifreleme türünü belirtmiyor.

# The file is too big to be a list of policies.
taf68e582 = Dosya bir ilke listesi olamayacak kadar büyük.

# The file has to hold one JSON object.
tf483aa65 = Dosya tek bir JSON nesnesi içermeli.

# {path} has to belong to root.
tfc77ea2b = { $path } root'a ait olmalı.

# {path} can be changed by others than root.
t0d4a4498 = { $path } root dışındakiler tarafından değiştirilebiliyor.

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = Bu depolama alanı şimdilik en çok { $size } boyutundaki dosyaları kabul eder; çünkü her biri gönderilirken bellekte tutulur.

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = Bu depolama alanı bir klasörü tek seferde yeniden adlandıramaz, çünkü altındaki her şey kopyalanırdı. Şuraya taşı… kullanın.

# S3 bucket
tc5e013b3 = S3 kovası

# Bucket
te34e3557 = Bucket

# Region
t492e789d = Bölge

# Endpoint
t27e7d69c = Uç nokta

# Empty for AWS, or https://…
td3a94af8 = AWS için boş bırakın veya https://…

# Start in
t5eb91ea6 = Başlangıç yeri

# A prefix, if not the whole bucket
t75255a6c = Tüm kova değilse bir önek

# Access key ID
tf9408d75 = Erişim anahtarı kimliği

# Secret access key
te1c3d3a2 = Gizli erişim anahtarı

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = Ana makine veya ssh_config adı

# Empty for the home folder
t67b5f42b = Ana klasör için boş bırakın

# Empty to use the SSH agent and keys
tf10e6293 = SSH aracısını ve anahtarlarını kullanmak için boş bırakın

# WebDAV folder
tb2401428 = WebDAV klasörü

# Address
t700560f3 = Adres

# User
te0a63e12 = Kullanıcı

# Password or app token
t1c18cb9b = Parola veya uygulama belirteci
