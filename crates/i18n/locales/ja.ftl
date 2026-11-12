# packwing in Japanese. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = 外観：{ $name }。クリックすると { $name2 } になります。

# Stop and say so
tdc8b782b = 停止して知らせる

# Replace it
t1e17fc0c = 置き換える

# Skip it
tad9909cb = スキップする

# Add
t9dc3aa14 = 追加

# The top of {name}
t369f9d4d = { $name } の先頭

# Choose what to add
t48b969a8 = 追加するものを選択

# {count} item | {count} items
t534e422e =
    { $n ->
       *[other] { $count } 個の項目
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
       *[other] { $count } 個のファイル
    }

# Add to {name}
t0d53ac5e = { $name } に追加

# Goes in
t36e95c22 = 追加先

# If a file of that name is there
t34c69cec = 同じ名前のファイルがある場合

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = このアーカイブには署名があります。内容を追加すると、再度署名するまで署名と一致しなくなります。

# The new files are encrypted with the archive’s password.
t52b1d223 = 新しいファイルはアーカイブのパスワードで暗号化されます。

# Cancel
t35afca3b = キャンセル

# Name
t0fe07306 = 名前

# Folder
t06357afd = フォルダ

# Its secrets could not be read: {why}
t2af0c8ab = シークレットを読み取れませんでした：{ $why }

# That folder is not on this computer.
ta4073fff = そのフォルダはこのコンピューターにありません。

# Its secrets could not be kept: {why}
ta213827f = シークレットを保存できませんでした：{ $why }

# Local folder
te075bcbc = ローカルフォルダ

# Trying it…
t214cdad5 = 試しています…

# It answers.
t3bd5bc9f = 応答がありました。

# It did not answer: {why}
t0ac8fa44 = 応答がありませんでした：{ $why }

# Edit storage
t7bae68d0 = ストレージを編集

# Add a storage
t4fe8f806 = ストレージを追加

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = シークレットは設定とは別に、システムのキーチェーンに保存されます。

# Its files stay where they are.
t4e4acf9a = ファイルは元の場所に残ります。

# Ask for this computer’s password to open it
t7a7d61e9 = 開くときにこのコンピューターのパスワードを求める

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = ロックしたままにしたコンピューター向けです。このコンピューターのロック解除と同じ方法で開き、5 分間使わないと再度求められます。

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing Free では、ネットワーク越しのストレージは 1 つまで使えます。Pro または Enterprise ライセンスで制限が解除されます：設定 › ライセンス。このコンピューター上のフォルダには制限がありません。

# Test connection
te0f3b6b7 = 接続をテスト

# Enter a license key
ta24f28a2 = ライセンスキーを入力

# Save
t4d2d5d68 = 保存

# Paste the checksum given with the file
t8333758c = ファイルと一緒に提供されたチェックサムを貼り付け

# They could not be saved: {why}
tb45ae457 = 保存できませんでした: { $why }

# The file could not be read: {why}
t5daa8455 = ファイルを読み取れませんでした: { $why }

# Reading…
t6aad8485 = 読み取り中…

# Copied
t8dc21305 = コピーしました

# Copy
t658f3664 = コピー

# weak
tedbc2ec9 = 弱い

# There is no checksum in this.
t231f838a = チェックサムが含まれていません。

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = packwing が作成するチェックサムではありません（{ $digits } 桁）。

# It matches the file’s {name} checksum.
t4f0350e4 = ファイルの { $name } チェックサムと一致します。

# It is compared once the file is read.
tcb5c1745 = ファイルの読み取りが終わると比較します。

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = ファイルの { $name } チェックサムと一致しません。このチェックサムの元のファイルではないか、途中でファイルが変化しました。

# It may be a {name} checksum, which is not made yet.
t3d713e04 = まだ作成していない { $name } チェックサムかもしれません。

# Make it
tcbefe458 = 作成

# Compare with
tbddcfc14 = 比較対象

# Reading… {size} of {size2}
t80036d1c = 読み取り中… { $size } / { $size2 }

# Reading… {size}
t1569c388 = 読み取り中… { $size }

# Checksums of {name}
te88f3ed9 = { $name } のチェックサム

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = ファイル全体がこのコンピューターを通過し、読み取った分についてプロバイダーから料金がかかる場合があります。

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = 弱いチェックサム（{ $names }）は破損したファイルは見分けられますが、一致するように意図的に変更されたファイルは見分けられません。

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = { $files } として隣に保存しました。sha256sum -c などで packwing なしで確認できます。

# Compare…
tec85be80 = 比較…

# Saving…
tbd79b37d = 保存中…

# Save beside it
t7f4d85e3 = 隣に保存

# Choose the checksums shown
teec532fe = 表示するチェックサムを選ぶ

# Read it
t80fb8482 = 読み取る

# Done
t8dd31791 = 完了

# Nothing chosen yet
tcf390b0a = まだ何も選択されていません

# {join} and {more} more
t8399ef70 = { $join } ほか { $more } 件

# Hide what goes in
t5a420780 = 追加するものを隠す

# Show what goes in
t9a20eb53 = 追加するものを表示

# Take it out
t90a93ec7 = 外す

# Drop them to add them
t6a4aedbe = ここにドロップして追加

# What goes in
t3b7c797a = 追加するもの

# + Files
tba8a1e71 = + ファイル

# + Folder
tcef8b0ce = + フォルダ

# Add the files or a folder to put in the archive.
t2e5c8c3e = アーカイブに入れるファイルまたはフォルダを追加してください。

# Don’t split
t56ee6829 = 分割しない

# 25 MB (email)
tc6ba1c9c = 25 MB（メール用）

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB（USB メモリ用）

# Custom size…
t0fcf9e2d = サイズを指定…

# Size
ta6478e7c = サイズ

# Password
t2cc30838 = パスワード

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…、age1pq1…、または ssh-ed25519 …

# Save here
ta1f147a6 = ここに保存

# Use these keys
t1a01e439 = これらのキーを使う

# No compression
t2e9c9e78 = 圧縮なし

# {name} compression
te2db10f6 = { $name } 圧縮

# Several frames
t84cc41f7 = 複数フレーム

# One frame
tc6ef759e = 1フレーム

# Small blocks
t04032e8a = 小さいブロック

# Parts of 25 MB
ta999efe2 = 25 MB ごとのパート

# Parts of 4 GB
t760df467 = 4 GB ごとのパート

# Parts of {size}
tdd286479 = { $size } ごとのパート

# Parts of a size to type
tafd4b17d = 入力したサイズごとのパート

# Public key
t9981f45f = 公開鍵

# Owners kept
t63e8a0df = 所有者を保持

# Attributes kept
te9e8289c = 属性を保持

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = バックアップで所有者と拡張属性を保持するには、.tar 形式を選んでください。

# Keep for a backup
t6a2bcc1a = バックアップ用に保持

# Owners: user and group
t5350e3fe = 所有者: ユーザーとグループ

# Extended attributes and ACLs
t35aff4d1 = 拡張属性と ACL

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = このコンピュータで復元するバックアップ用です。アーカイブを開いた人にはユーザー名が見えます。

# Beside the files
t757989af = ファイルと同じ場所

# What it holds, from
teae4ef1a = 中身の取得元

# Save in
tf2d34837 = 保存先

# Folder…
tc8abcd43 = フォルダ…

# Storage…
td0d8c806 = ストレージ…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = パートはまずこのコンピューターの一時フォルダに作成されてからアップロードされるため、そこに空き容量が必要です。

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext } はまずこのコンピューターの一時フォルダに作成されてからアップロードされるため、そこに空き容量が必要です。.zip と tar 形式はこのストレージに直接書き込まれます。

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = まずこのコンピューターの一時フォルダに作成されてからアップロードされるため、そこに空き容量が必要です。

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } が受け付けるファイルは最大 { $most } です。これより大きいアーカイブは、アップロードの開始時に失敗します。

# Choose a storage, and a folder in it
td7627e48 = ストレージと、その中のフォルダを選択してください

# New folder
t1926d127 = 新しいフォルダ

# Give it a name, without / or \ in it.
ta598b15b = / や \ を含まない名前を付けてください。

# A file of that name is there already: this one will be {name}.
t9451c3b1 = 同じ名前のファイルがすでにあります。このファイルは { $name } になります。

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = Windows や macOS で使えない名前があります（例: “{ $name }”）。そこではこれらのファイルがそのまま展開されないことがあります。

# Hide advanced
tae2a96e9 = 詳細設定を隠す

# Advanced options
t701d5ed9 = 詳細設定

# Large blocks
tdc489452 = 大きいブロック

# Compression level
td386c9ef = 圧縮レベル

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = 無圧縮はファイルをそのまま格納します。作成は最も速く、サイズは最も大きくなります。

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = 高くするとアーカイブは小さくなりますが、作成に時間がかかります。開く速さは変わりません。

# Frames
tf03e266f = フレーム

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = 複数フレームはすべてのコアで開けますが、アーカイブは少し大きくなります（0.5%未満）。

# One frame makes the smallest archive, which opens on one core.
ta097b317 = 1フレームは最も小さいアーカイブになり、1つのコアで開きます。

# Blocks
t036a71e3 = ブロック

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = 小さいブロックはすべてのコアを働かせます。最大2倍速くなり、アーカイブは数パーセント大きくなります。

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = 大きいブロックは最も小さいアーカイブになります。圧縮するものが少ないと、使うコアは少なくなります。

# Split into parts
tca4436bd = パートに分割

# Each part
t642c444d = 1 パートあたり

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = パートは 7-Zip やそれに類するアーカイバで開けます。Windows や macOS に組み込まれたツールでは開けません。

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = パートのサイズは 1 MB 以上で指定してください（例：100 MB）。

# Encrypt with age
t22784ad9 = age で暗号化

# Protect with a password
tae8d8740 = パスワードで保護

# Suggest
ta6e44ce5 = 提案

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = 安全な場所に保管してください。これがないと、このアーカイブは二度と開けません。

# Type a password, or turn this off.
t62b19318 = パスワードを入力するか、オフにしてください。

# Hide file names too
t2198d77b = ファイル名も隠す

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = パスワードを付けた ZIP は、Windows や macOS の組み込みツールでは開けません。受け取る人にはアーカイバアプリが必要です。ファイル名も見えたままになります。隠すには .7z を選んでください。

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = { $name } として保存しました。中の名前も暗号化されています。Windows と macOS では、開くのに age ツールまたはアーカイバアプリが必要な場合があります。

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
       *[other] { $name } から { $n } 個のキー
    }

# Remove
t21a5901d = 削除

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
       *[other] age が暗号化の宛先にできない種類のため、{ $n } 個のキーが除外されました。
    }

# From a file…
t40d7c944 = ファイルから…

# Type a public key, or choose a file of them.
tcb028b06 = 公開鍵を入力するか、公開鍵のファイルを選択してください。

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = ポスト量子鍵です。これを開くには、age ツールのバージョンが 1.3 以降である必要があります。

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = 対応する秘密鍵だけがこれを開けます。その鍵を失うと、誰にも開けなくなります。

# Weak
t6de87ae9 = 弱い

# Fair
t82fa0d53 = 普通

# Strong
t0dad409a = 強い

# Choose what to put in it
t5461cfa6 = 入れるものを選択

# Convert
t92e29218 = 変換

# New archive
tf352befb = 新規アーカイブ

# Create
t990de47d = 作成

# {n} item | {n} items
tfff7fd6c =
    { $n ->
       *[other] { $n } 個の項目
    }

# Move {what} to the Trash?
tdf5a8d53 = { $what } をゴミ箱に移動しますか？

# Delete {what}?
ta297d6c5 = { $what } を削除しますか？

# It can be put back from the system’s Trash.
te5e137a1 = システムのゴミ箱から元に戻せます。

# This cannot be undone from packwing.
t3947dab4 = packwing からは元に戻せません。

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
       *[other] ほか { $n } 件
    }

# Move to Trash
ta036661b = ゴミ箱に移動

# Delete
t5797ea6a = 削除

# {many} items
t49912bd4 = { $many } 個の項目

# Rename {what}
t83401013 = { $what } の名前を変更

# New name
taf4cad88 = 新しい名前

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = アーカイブから完全に削除されます。フォルダは中身ごと削除されます。元に戻せません。

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = このアーカイブには署名があります。この変更の後は、再度署名するまで署名と一致しなくなります。

# Rename
tc80631f3 = 名前を変更

# The archive’s password
t8a168799 = アーカイブのパスワード

# one folder
td571ce35 = 1 個のフォルダ

# one file
t95c9ec7b = 1 個のファイル

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }、つまりこの場所

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }、つまり新しいフォルダ内

# Extract here
t354232ec = ここに展開

# {picked} selected
t9addc37e = { $picked } を選択中

# All {counted}
t649feaf8 = すべての { $counted }

# What should it extract?
td12e23bf = 何を展開しますか？

# Smart
t54535972 = スマート

# Recommended
t7865024c = おすすめ

# Where should the files go?
td35f540c = ファイルをどこに置きますか？

# Right here
t900a8a39 = この場所

# Another folder
t9f128ba8 = 別のフォルダ

# Choose…
t58daae9c = 選択…

# Goes to {destination}
t8195aed2 = { $destination } に展開されます

# Checked as it goes
tb6c9570b = 展開しながら検査

# · Nothing leaves the folder or passes the size limit
t11424797 = · フォルダの外に出るものや、サイズ制限を超えるものはありません

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } はインターネットから取得したものです。マークは残るため、実行前に確認が表示されます。

# Extract archive
t834ba3c0 = アーカイブを展開

# Extract
t30518e1e = 展開

# Open {name}?
tcfe84c2f = { $name } を開きますか？

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing は一時フォルダにコピーを作って、それを開きます。そのコピーへの変更はアーカイブには戻りません。

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = これはプログラムです。開くと実行されます。アーカイブの入手元を信頼できる場合のみ実行してください。

# Don’t ask again
t00c5d168 = 今後は確認しない

# Run it
t72430853 = 実行する

# Open
t538b10e9 = 開く

# Open from a storage
t45a8991d = ストレージから開く

# Choose a storage, and an archive in it
t3c900950 = ストレージと、その中のアーカイブを選択してください

# Choose
t73c66160 = 選択

# This is not a key file packwing can use: {why}.
tcf6986a7 = packwing で使えるキーファイルではありません：{ $why }。

# Could not read {path}: {e}.
t1dc6961f = { $path } を読み取れませんでした：{ $e }。

# The passphrase
t1e921d00 = パスフレーズ

# The key file’s passphrase
t4051f4d7 = キーファイルのパスフレーズ

# Enter the passphrase
tbd0672da = パスフレーズを入力

# Passphrase
t191a2dc5 = パスフレーズ

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = このファイルは age で暗号化されています。packwing はパスフレーズをアーカイブを開いている間だけ保持し、書き留めることはありません。

# Enter the password
t513ea7ab = パスワードを入力

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = このアーカイブは暗号化されています。packwing はパスワードをアーカイブを開いている間だけ保持し、書き留めることはありません。

# Choose the key file
t3c521a4e = キーファイルを選択

# Key file
t6ed7c612 = 鍵ファイル

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = このファイルは、age のキーファイルまたは SSH キーを宛先として age で暗号化されています。packwing はそこから読み取った内容をアーカイブを開いている間だけ保持し、書き留めることはありません。

# Enter the key file’s passphrase
t808b85c5 = キーファイルのパスフレーズを入力

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = このキーファイル自体が保護されています。packwing はそのパスフレーズをアーカイブを開いている間だけ保持し、書き留めることはありません。

# Version control
t0b7280ee = バージョン管理

# Packages and builds
t237b373e = パッケージとビルド

# System files
te8be6b83 = システムファイル

# Temporary files
td4fe1263 = 一時ファイル

# Logs
t51a96da6 = ログ

# Pictures
t0d70feae = 画像

# Documents
t5a036285 = 文書

# Videos
t63609fed = 動画

# Music
t584a1d74 = 音楽

# node_modules/, *.log, src/**/gen/
t56c5448d = node_modules/, *.log, src/**/gen/

# They leave nothing out.
t999b68fd = 除外されるものはありません。

# They leave out {n} file, {size}. | They leave out {n} files, {size}.
tc709cfda =
    { $n ->
       *[other] { $n } 個のファイル ({ $size }) を除外します。
    }

# Clear
t04a57fc2 = クリア

# Leave out
ta50cc084 = 除外

# Take only
t2fd02f84 = これだけ含める

# Add the usual
t17ad93a1 = よく使うものを追加

# Only what these name goes in, and the folders it is in.
ta0bd7335 = 名前が一致するものと、それを含むフォルダーだけが入ります。

# A name is anywhere; a / at its end names a folder; ** is any folders.
t524d506a = 名前だけならどこでも一致。末尾の / はフォルダー、** は任意のフォルダーです。

# Filter
tf4a9c097 = フィルター

# kept, {size}
t001f6992 = 残しました、{ $size }

# left out
t73aeedd4 = 除外しました

# renamed {to}
t7aa9b778 = { $to } に名前を変更

# It was gone by the time it was read.
t93fb1e03 = 読み込もうとした時点で、すでに存在しませんでした。

# The job stopped before it was done.
t57a78885 = ジョブが完了する前に止まりました。

# Waiting for other jobs…
t16ec36d9 = 他のジョブを待っています…

# A file is damaged
t8b1230ed = ファイルが破損しています

# Some names can’t be used here
t26835ec8 = ここでは使えない名前があります

# Converted, but some files are damaged
t54ef3472 = 変換しましたが、一部のファイルが破損しています

# Extracted, but some files are damaged
t8226d624 = 展開しましたが、一部のファイルが破損しています

# Converted, with some files left out or renamed
t5cce0e50 = 変換しました（一部のファイルは除外または名前を変更）

# Extracted, with some files left out or renamed
t5dc002d6 = 展開しました（一部のファイルは除外または名前を変更）

# Extracting…
t94279f82 = 展開中…

# Uploading…
tc75fe478 = アップロード中…

# Creating…
t01fcd896 = 作成中…

# Adding…
t2060c750 = 追加中…

# Changing the archive…
t186fbef5 = アーカイブを変更中…

# Opening…
tc40f2321 = 開いています…

# Testing…
t19d1cb65 = テスト中…

# Copying…
t02dd17e0 = コピー中…

# Moving…
t890f7ed1 = 移動中…

# Signing…
t7b8f8780 = 署名中…

# Splitting…
t5452adf5 = 分割しています…

# Joining…
taa884c51 = 結合しています…

# Converting…
ta1b8f4cc = 変換しています…

# Couldn’t split the file
t83d27d26 = ファイルを分割できませんでした

# Couldn’t join the parts
t1cf67e00 = パートを結合できませんでした

# Couldn’t convert the archive
t39f973a9 = アーカイブを変換できませんでした

# Couldn’t sign
t4489f97c = 署名できませんでした

# Couldn’t copy
t613e4e9c = コピーできませんでした

# Couldn’t move
t9796adec = 移動できませんでした

# Couldn’t extract
tacd22c56 = 展開できませんでした

# Couldn’t create the archive
t9d0e01bc = アーカイブを作成できませんでした

# Couldn’t add the files
tfbd989b0 = ファイルを追加できませんでした

# Couldn’t change the archive
tcdc4a49c = アーカイブを変更できませんでした

# Couldn’t open the file
t041198a4 = ファイルを開けませんでした

# The test failed
tfcd070a5 = テストに失敗しました

# Starts when one of the others ends
tbe2a0509 = 他のジョブが終わると始まります

# Paused
tde281feb = 一時停止中

# Starting…
t73727cef = 開始しています…

# Finishing…
t042f47fe = 仕上げ中…

# Almost done
ta550a6a9 = まもなく完了

# About {secs} seconds left
t3045da16 = 残り約 { $secs } 秒

# About a minute left
tf727815c = 残り約 1 分

# About {div_ceil} minutes left
tbd7a3305 = 残り約 { $div_ceil } 分

# {name} · checking that every file reads
t0d56ec7c = { $name } · すべてのファイルが読み取れるか確認中

# and {count} more | and {count} more
te728628b =
    { $n ->
       *[other] ほか { $count } 個
    }

# The test passed
tdf0ce240 = テストに合格しました

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
       *[other] { $count } 個のファイル（{ $size }）をエラーなしで最後まで読み取りました。
    }

# Close
tcd86acc3 = 閉じる

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = 「{ $path }」は破損しています: { $why }。中止しますか、このファイルを除外しますか、読み取れた部分を残しますか?

# Do the same for the other damaged files
t798693ad = ほかの破損ファイルにも同じようにする

# Stop
t4b7f7705 = 停止

# Leave it out
taaff8ec9 = 除外する

# Keep what was read
tc4387d25 = 読み取れた部分を残す

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
       *[other] { $count } 個のファイルは、ここではその名前で展開できません。まだ何も書き込んでいません。中止、除外、名前の変更のどれにしますか?
    }

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
       *[other] { $count } 個のリンクがフォルダーの外、このコンピューター上の別の場所を指しています。まだ何も書き込んでいません。除外しますか？ それとも、このアーカイブを信頼するなら、そのまま作成しますか？
    }

# Create them
t9a4d6e7d = 作成

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
       *[other] { $count } 個のファイルのデータが壊れていました。
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
       *[other] { $count } 個のファイルを除外しました。
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
       *[other] { $count } 個のファイルは、名前がここで使えないため名前を変更しました。
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
       *[other] { $count } 個のファイルは読み込む時点で存在しなかったため、除外しました。
    }

# The rest is in the archive.
t977a5572 = 残りはアーカイブに入っています。

# The rest was extracted.
ta7833fcb = 残りは展開しました。

# Copy the list
t3c49b269 = リストをコピー

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } / { $count2 } 個のファイル · { $size } / { $size2 }

# Open the folder when it’s done
tf0f1017b = 完了したらフォルダを開く

# Run in background
tb800c0ef = バックグラウンドで実行

# Resume
t454d1d8a = 再開

# Pause
t44f9bf2d = 一時停止

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
       *[other] { $n } 件のジョブがまだ実行中です
    }

# Closing packwing stops them
t2e6491b3 = packwing を閉じると停止します

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = 終わっていない作業は途中のままになります。作成中のアーカイブは削除され、展開は途中まで残ります。

# Keep running
t79c1b9c7 = 続ける

# Stop them and quit
tf2707857 = 停止して終了

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = ファイルはそのまま残ります。packwing がサインインに使う情報はシステムのキーチェーンから削除されます。

# The folder and its files stay where they are.
t4850aaa8 = フォルダとそのファイルはそのまま残ります。

# Remove “{name}” from the list?
t5dd9cfcf = 「{ $name }」を一覧から削除しますか？

# It can be added again at any time.
t80d51a99 = いつでも追加し直せます。

# A passphrase for the new key
tf5593f05 = 新しいキーのパスフレーズ

# The same passphrase again
tad5c08e6 = もう一度同じパスフレーズ

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = 署名には Pro または Enterprise ライセンスが必要です：設定 › ライセンス。署名の確認は、どのプランでも無料です。

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = これは複数に分割されたファイルの 1 パートです。署名は 1 つのファイルを対象とし、全体に署名したように見えてしまうため、packwing はパートには署名しません。

# That passphrase does not open this key.
tc5357c97 = そのパスフレーズではこのキーを開けません。

# This is not a key packwing can sign with: {why}.
t662c29f9 = packwing で署名に使えるキーではありません：{ $why }。

# The two passphrases are not the same.
t335fe90b = 2 つのパスフレーズが一致しません。

# Keep the key here
t9306dafe = キーをここに保存

# No key chosen
te58c13f5 = キーが選択されていません

# Key
tcd1ac90c = 鍵

# New key…
t9e1134da = 新しいキー…

# Again
t58fb56ef = 確認

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = 新しいキーは Ed25519 と ML-DSA-65 を併せ持ち、どちらか一方が破られても署名が有効なままになります。これを確認できるのは packwing と hivepack だけです。ssh-keygen で使うには、SSH Ed25519 キーで署名してください。パスフレーズを忘れたキーは開く方法がありません。

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = 署名は { $signature } として隣に保存され、すでにあるものは置き換えられます。ファイル全体がこのコンピューターを通過し、読み取った分についてプロバイダーから料金がかかる場合があります。

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = 署名は { $signature } として隣に保存され、すでにあるものは置き換えられます。

# Sign {name}
ted250f31 = { $name } に署名

# Anyone can check it, with the key’s public half
t04ecc8be = キーの公開鍵があれば誰でも確認できます

# Make the key…
t6bba28d5 = キーを作成…

# Sign
t8f6340c4 = 署名

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = 隣に { $first } から始まる { $count } 個のパートを作ります。「結合」で元に戻せます。packwing と 7-Zip は、このように分割したアーカイブをそのまま開けます。

# Split
t16a15383 = 分割

# Move {what}
tc168ca56 = { $what } を移動

# Copy {what}
teda01f26 = { $what } をコピー

# Move here
t1aaf57aa = ここに移動

# Copy here
t85f0245a = ここにコピー

# Move
t7f2c2a34 = 移動

# If a name is taken
tc99c037f = 同じ名前がある場合

# They are in this folder already. Choose another.
t5318c14e = すでにこのフォルダにあります。別のフォルダを選んでください。

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = すべてがこのコンピューターを通過し、読み取った分についてプロバイダーから料金がかかる場合があります。移動した元のファイルは、コピーを読み戻して一致を確認した後に削除されます。

# Each original goes once its copy has been read back and matches.
t8ecbab4e = 元のファイルは、コピーを読み戻して一致を確認した後に削除されます。

# The originals stay where they are.
t5937f050 = 元のファイルはそのまま残ります。

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = 別のジョブがそこで { $name } を作成中です。別の名前にするか、終わるまでお待ちください。

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = 別のジョブが { $folder } に展開中です。別のフォルダーを選ぶか、終わるまでお待ちください。

# no editor command
tbf2a5d3a = エディタのコマンドがありません

# no text editor found
t6256b3da = テキストエディタが見つかりません

# Search this folder
t2c814039 = このフォルダを検索

# This folder could not be read: {why}
t2c6017b2 = このフォルダを読み取れませんでした：{ $why }

# Could not load more: {why}
t466e87c4 = これ以上読み込めませんでした：{ $why }

# Could not delete: {why}
te5951d38 = 削除できませんでした：{ $why }

# the text editor
t91cdab0c = テキストエディタ

# Cannot start {who}: {why}
t8deb160a = { $who } を起動できません：{ $why }

# Edit in text editor
td8d39fb8 = テキストエディタで編集

# Edit in {editor}
te44cd57b = { $editor } で編集

# Modified
tdd13a6b2 = 更新日時

# Loading…
tc4e2f181 = 読み込み中…

# Only what is loaded was searched.
t39c5f536 = 読み込み済みのものだけを検索しました。

# Search the whole folder
t41f9a787 = フォルダ全体を検索

# Load {n} more
t30bed46a = さらに { $n } 件を読み込む

# Refresh
t0815aad4 = 更新

# Hide hidden files
t3375c9c0 = 隠しファイルを隠す

# Show hidden files
tf18c7057 = 隠しファイルを表示

# View
t5e4baa18 = 表示

# Extract…
t4a09490e = 展開…

# Open with default app
t5ef80ec5 = 既定のアプリで開く

# Open as archive
t3373ce55 = アーカイブとして開く

# Compress…
t690f9553 = 圧縮…

# Copy to…
t543cf15b = コピー先…

# Move to…
t7a32320b = 移動先…

# Sign…
td6d61b10 = 署名…

# Checksums…
t2891102d = チェックサム…

# Convert…
tae85a7c4 = 変換…

# Split into parts…
t281db403 = パートに分割…

# Join the parts
t0ad56db8 = パートを結合

# Copy path
tf165cb2b = パスをコピー

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
       *[other] { $n } 個の項目を圧縮…
    }

# {n} loaded, more in this folder
t29a52578 = { $n } 件を読み込み済み、このフォルダにはさらにあります

# Sorted among those loaded
t037d01ae = 読み込み済みのものの中で並べ替えています

# {n} selected
tebda8468 = { $n } 件を選択中

# Try again
t213e90fa = もう一度試す

# Nothing here matches that.
t0f9a8a58 = 一致するものがありません。

# Add a storage to browse it.
t0b82c83a = ストレージを追加すると、ここで閲覧できます。

# No folders or archives here.
t6a090cc3 = フォルダもアーカイブもありません。

# No folders here.
tc7731c8f = フォルダがありません。

# This folder is empty.
td9e47db6 = このフォルダは空です。

# Show in Finder
ta9442aa9 = Finder で表示

# Show in File Explorer
t23c89eda = エクスプローラーで表示

# Show in the file manager
tb347ece9 = ファイルマネージャーで表示

# Open in Finder
te2a467e0 = Finder で開く

# Open in File Explorer
t3794d625 = エクスプローラーで開く

# Open in the file manager
td1ceabc4 = ファイルマネージャーで開く

# Copy all
t6aef5c37 = すべてコピー

# This system has no folder for settings.
tfd0ff0a1 = このシステムには設定用のフォルダがありません。

# Archive
t73690b65 = アーカイブ

# Search inside this archive
tb35c0c27 = このアーカイブ内を検索

# Open another…
tff74a552 = ほかを開く…

# From a storage…
tcfba8fd5 = ストレージから…

# Safe to extract
t3c93bb70 = 安全に展開できます

# Reading… {percent}%
t83223c27 = 読み込み中… { $percent }%

# Read {fetched} of {whole}
t613dd932 = { $whole } のうち { $fetched } を読み取り済み

# Add files
tdd45c7bb = ファイルを追加

# Test
t2ffcbe05 = テスト

# Reading what’s inside…
t1bbda9f5 = 中身を読み取っています…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = このアーカイブに「{ $query }」に一致するものはありません。

# Nothing in this folder matches “{query}”.
teecdfa3d = このフォルダに「{ $query }」に一致するものはありません。

# And {more} more. Type more to narrow it down.
t843253d0 = ほか { $more } 件あります。入力を増やして絞り込んでください。

# Open an archive
t8382bd3e = アーカイブを開く

# ZIP, 7z and tar archives, disc images, packages, snaps and AppImages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .deb, .rpm, .pkg, .snap, .AppImage, .zst.age…
t2cb379b7 = ZIP、7z、tar アーカイブ、ディスクイメージ、パッケージ、snap、AppImage、age で圧縮または暗号化したもの：.zip、.7z、.tar.gz、.tar.zst、.iso、.deb、.rpm、.pkg、.snap、.AppImage、.zst.age…

# Back to {name}
t539b9c10 = { $name } に戻る

# Open an archive…
t2cf2a8ee = アーカイブを開く…

# It could not be shown
tb60fb288 = 表示できませんでした

# Choose a file to see it here
t37854216 = ファイルを選択するとここに表示されます

# Symbolic link
tc7c20187 = シンボリックリンク

# Leads to {target}
te7ddae48 = リンク先: { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
       *[other] このフォルダ内の実行ファイル { $count } 個は、展開してもインターネットのマークが付いたままになります。
    }

# Today
te7c0775e = 今日

# Last 7 days
t35b9d98f = 過去 7 日間

# Last 30 days
td0778b33 = 過去 30 日間

# All days
tabb197ef = すべての日

# All results
t6235c0b2 = すべての結果

# Not done
t0f88a146 = 完了していない

# Partly done
t3355a2a5 = 一部完了

# Failed
t5579c3d4 = 失敗

# Cancelled
ta62b09bc = キャンセル

# No end recorded
t38ab1371 = 終了の記録なし

# Make an archive
t418bd6f0 = アーカイブを作成

# Open a file from an archive
tc2dd3e41 = アーカイブ内のファイルを開く

# Preview a file
t1f706562 = ファイルをプレビュー

# Read a file from an archive
t4229020d = アーカイブ内のファイルを読み取る

# Add to an archive
t4e28d1fa = アーカイブに追加

# Delete from an archive
t158044d3 = アーカイブから削除

# Rename in an archive
tb3eef763 = アーカイブ内で名前を変更

# Check a signature
t6f441e5e = 署名を確認

# Make a signing key
tbdf6bcd0 = 署名鍵を作成

# Join parts
t4eee8c5f = パーツを結合

# Checksums
t47dd48d3 = チェックサム

# Audit log started
tfb8df56f = 監査ログを開始

# Audit log stopped
t54654bc3 = 監査ログを停止

# Audit log changed
td6b7bc4a = 監査ログの設定を変更

# Old records taken away
t5f337ebf = 古い記録を削除

# All actions
t4ff15157 = すべての操作

# Find a path, a name or an error
t30cb1a6c = パス、名前、エラーを検索

# Read again
t2f0dcc5d = 再読み込み

# Save as CSV…
tf103c3cc = CSV として保存…

# Checking that no record was changed…
t00deba79 = 記録が変更されていないか確認しています…

# The audit log cannot be read: {why}
t8783f4cf = 監査ログを読み取れません: { $why }

# A record was changed, taken out or put in by hand: in {file}, line {line}. | Records were changed, taken out or put in by hand, in {n} places: the first in {file}, line {line}.
t3cd011bd =
    { $n ->
       *[other] { $n } か所で記録が手作業で変更、削除、挿入されています。最初は { $file } の { $line } 行目です。
    }

# The oldest record follows one that is gone, and no record says it was taken away as past its time.
t4579f8a7 = 最も古い記録の前にあった記録が失われていますが、保存期間切れで削除されたことを示す記録はありません。

# {n} record, and none was changed or taken out. | {n} records, and none was changed or taken out.
tefc82060 =
    { $n ->
       *[other] { $n } 件の記録があり、変更や削除はありません。
    }

# Only the latest {most} records are shown: choose fewer days, or read all with pkw audit list.
t5fabfc51 = 最新の { $most } 件だけを表示しています。日数を減らすか、pkw audit list ですべて読んでください。

# Saved to {path}.
te8346fa4 = { $path } に保存しました。

# Could not save: {why}
t733a7cae = 保存できませんでした: { $why }

# Record
t9f4565ec = 記録

# Started
t97b5823e = 開始

# Ended
tec2b41db = 終了

# Reading the audit log…
t403ccdff = 監査ログを読み取っています…

# No records in these days.
t98687757 = この期間の記録はありません。

# No record is as chosen.
ta9ffcb9f = 選択に合う記録はありません。

# This computer
tfa0d176a = このコンピュータ

# Home
t52f50eae = ホーム

# Desktop
t64215c53 = デスクトップ

# Downloads
t539307de = ダウンロード

# Drop to open
tb255877b = ドロップして開く

# Browse
t0eac1f07 = 参照

# Recent
t50e888fe = 最近使用

# Audit log
tcb83f9ca = 監査ログ

# Choose a file to see its information here
t50aac02b = ファイルを選択すると、ここに情報が表示されます

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
       *[other] { $n } 件を選択中
    }

# Files in them come to {size}
t0cca93f1 = 中のファイルの合計は { $size } です

# Delete…
t2fed07aa = 削除…

# Link to a folder
t4b408baf = フォルダへのリンク

# Link
t8e08e589 = リンク

# {tag} file
ta56bbbe9 = { $tag } ファイル

# {files}, {folders}
t72e46dc5 = { $files }、{ $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
       *[other] { $n } 個のファイル
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
       *[other] { $n } 個のフォルダ
    }

# {size} so far
tc87daf59 = ここまで { $size }

# Kind
t3ff50c63 = 種類

# Holds
tffe19c31 = 内容

# Permissions
t8700fd83 = パーミッション

# Path
teb66e456 = パス

# Signature
tc049050d = 署名

# Count size and files
t0f23db17 = サイズとファイル数を数える

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
       *[other] 中の { $n } 個のフォルダを読み取れませんでした。
    }

# Check the signature
t9d44730e = 署名を確認

# Extracted
t3250471f = 展開しました

# Archive created
t22c69419 = アーカイブを作成しました

# Moved
t82869bf0 = 移動しました

# Signed
t27c3ab35 = 署名しました

# Joined
tf8f2b670 = 結合しました

# Converted
tc7b76e99 = 変換しました

# Waiting for a password or a key
t0e83ae32 = パスワードまたは鍵を待っています

# Enter it…
tddb7844a = 入力…

# Show
t29742e5c = 表示

# Jobs
t4ae6a79d = ジョブ

# Clear finished
t30c178e8 = 完了したものを消去

# Nothing runs in the background.
t2573f39e = バックグラウンドで実行中のものはありません。

# Add to the right-click menu
t116d3868 = 右クリックメニューに追加

# Extract or compress from any folder.
t20864983 = どのフォルダからでも展開や圧縮ができます。

# Welcome to packwing
tb38a86a6 = packwing へようこそ

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = あらゆるアーカイブを開き、共有用に小さなファイルを作り、危険なダウンロードから身を守ります。

# Set things up
t63f9c1cd = 設定する

# You can change these later in Settings.
t95f41ffb = これらは後で設定から変更できます。

# Get started
te83ca5ee = 始める

# Remove from Recent
t26cc8dba = 最近使用から削除

# {location} · not there any more
t7d0c09c7 = { $location } · すでに存在しません

# The archives you open will show here.
tc0cc82d5 = 開いたアーカイブがここに表示されます。

# Copy version info
t3d37483c = バージョン情報をコピー

# Version {version} · {os} {arch}
t6bead5ae = バージョン { $version } · { $os } { $arch }

# A file archiver that puts every core to work and opens archives safely.
t5d2a9fa0 = すべてのコアを働かせ、アーカイブを安全に開くファイルアーカイバ。

# Updates
t483075a5 = アップデート

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = これはベータ版です：新しいベータ版と、それに続く安定版をお知らせします。

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = これは安定版です：新しい安定版をお知らせします。ベータ版はお知らせしません。

# Check now
t4f0e0677 = 今すぐ確認

# Look for updates once a day
tcc27ace1 = 1 日 1 回アップデートを確認

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = リリース一覧を GitHub から読み込み、署名が検証できた場合のみ信頼します。あなたの操作なしに何もダウンロード・インストールされません。

# Skip this version
t2c33d756 = このバージョンをスキップ

# This build has no release keys, so it cannot tell of updates.
t378182cb = このビルドにはリリース鍵がないため、アップデートを知らせることができません。

# packwing {version} is ready
tf02650f8 = packwing { $version } の準備ができました

# It is installed when you quit, or now with Restart.
t6fecac86 = 終了時にインストールされます。今すぐなら「再起動」を押してください。

# Restart now
t54001a7c = 今すぐ再起動

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = 新しいバージョンを確認しています…

# Release notes
tc52ab62f = リリースノート

# packwing {version} is out
t36a30d8b = packwing { $version } がリリースされました

# Your license covers versions released until {date}. This one came out after: with it, packwing is Free until you renew. The version you have stays Pro.
tfd489117 = ご利用のライセンスは { $date } までにリリースされたバージョンが対象です。このバージョンはその後に出たため、更新するまで packwing は Free になります。今お使いのバージョンは Pro のままです。

# Renew
t9a23808e = 更新する

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = これらのコマンドをターミナルで実行してください。署名付きリリース一覧と照合してダウンロードを確認し、sudo がパスワードを尋ねます。

# {by} updates it: {command}
tdf2cab9c = { $by } でアップデートします：{ $command }

# {by} updates it.
t84b27295 = { $by } でアップデートします。

# It is downloaded and checked, then installed when you quit: Windows asks for an administrator's permission, as packwing is installed for everyone.
tc816fc67 = ダウンロードして検証し、終了時にインストールします。packwing はすべてのユーザー向けにインストールされているため、Windows が管理者の許可を求めます。

# It is downloaded and checked, then installed when you quit.
tb46c0678 = ダウンロードして検証し、終了時にインストールします。

# Download it, and install it in place of this one.
t39312eae = ダウンロードして、このバージョンの代わりにインストールしてください。

# The download failed: {why}
t5e96c075 = ダウンロードに失敗しました: { $why }

# Download and install
t1a99173f = ダウンロードしてインストール

# Download
t6964bb19 = ダウンロード

# The last look failed: {why}
td122a092 = 前回の確認に失敗しました：{ $why }

# Up to date (looked {when}).
t4665ef60 = 最新です（確認：{ $when }）。

# Not looked yet.
t5a3ba058 = まだ確認していません。

# Free software
t731220a4 = フリーソフトウェア

# Under the GNU General Public License, version 3 or later
t4e98bf3c = GNU General Public License バージョン 3 以降に基づきます

# View license
tfc21d38b = ライセンスを表示

# Made by HivePaaS
t10c7007a = HivePaaS 製

# On hivepack, our library for reading and writing archives
tfae09b61 = アーカイブの読み書きを行う当社のライブラリ hivepack を採用

# Source code
tf6a49f4d = ソースコード

# Something wrong?
t610dfb6f = 問題がありますか？

# Tell us, with the version above
ta3e29034 = 上記のバージョンを添えてお知らせください

# Report a problem
t7b1728e7 = 問題を報告

# What changed
te8345543 = 変更点

# Every release, and what is new in it
t8e6f2052 = すべてのリリースと、その新機能

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS. packwing の名称とロゴは HivePaaS の商標です。

# More from HivePaaS
tb25cf729 = HivePaaS のその他の製品

# Free and open source, like packwing.
tc1ece130 = packwing と同じく、無料のオープンソースです。

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = Linux、macOS、Windows で動作する、高速なネイティブ SSH クライアント。

# SSH and SFTP
t0f8af34d = SSH と SFTP

# Follows your ssh_config
tc1c4446d = ssh_config に対応

# A real terminal
tae9bcfa8 = 本格的なターミナル

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = shellwing.app を開く

# A native HTTP(S) debugging proxy that keeps up with big sessions.
t86e5ad3e = 大きなセッションにもついていくネイティブ HTTP(S) デバッグプロキシ。

# HTTP/2, WebSocket and SSE
t2c3c2377 = HTTP/2、WebSocket、SSE

# A million requests, smooth
t60de9f6c = 100 万件のリクエストもなめらか

# Breakpoints and rewrites
t00c623a3 = ブレークポイントと書き換え

# Phones and emulators
t8c26f435 = スマートフォンとエミュレータ

# Visit tracewing.app
t4668641f = tracewing.app を開く

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = Docker Swarm 上に構築された、アプリ向けのセルフホスト型プラットフォーム。Heroku、Render、Coolify に代わるオープンソースです。

# 300+ one-click apps
t6a770b0a = 300 以上のワンクリックアプリ

# Deploy on push
tb0bfc3e9 = push で自動デプロイ

# Pull request previews
t4e4180fc = プルリクエストのプレビュー

# Automatic TLS
t9e9776ed = 自動 TLS

# Live logs and metrics
tebe3a172 = ライブログとメトリクス

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = hivepaas.com を見る

# Always
tad4bbd04 = 常に

# Extensions, apart by spaces
tc709375f = 拡張子（スペース区切り）

# Format of new archives
t3b50d238 = 新規アーカイブの形式

# New archive starts with it; you can pick another there.
t7b361b55 = 新規アーカイブはこの形式で始まります。そこで別の形式も選べます。

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = 無圧縮は ZIP と 7z のファイルをそのまま格納します。tar 形式では「最低」になります。圧縮しないのは .tar だけです。

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = 高くするとアーカイブは小さくなりますが、作成に時間がかかります。開く速さは変わりません。

# Write straight to a storage above
ta4975128 = 上のストレージに直接書き込む

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = 大きなファイルがこのコンピューターの容量を使いませんが、接続が切れると圧縮からやり直しになります。S3 では tar と ZIP、SFTP ではすべての形式がこの方法で使えます。

# Don’t compress what already is
t940ec31d = 圧縮済みのファイルは圧縮しない

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = ZIP と 7z の場合: 画像・動画・音声・アーカイブをそのまま格納するので、速く、サイズも大きくなりません

# What goes in as it is
tc0104b29 = そのまま格納するもの

# Files whose names end in these, whatever their case
ta3acef9b = 名前がこれらで終わるファイル（大文字・小文字は問いません）

# Reset to default
t0db9f7dc = 既定に戻す

# Leave out Mac-only files
t27eec14f = Mac 専用のファイルを除外

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store、__MACOSX、リソースフォーク: Mac 以外では不要なファイルです

# Hide file names behind the password
t93ff7715 = パスワードでファイル名を隠す

# For 7z: nobody sees what is inside without it
t0f5354d7 = 7z 用：パスワードがないと中身が一切見えません

# Yours
t2635ea3b = 自分

# By name
t983e37f7 = 名前で

# By number
tcbbada8d = 番号で

# No limit
t6de7f47d = 制限なし

# Where the files go
t77e60aba = ファイルの置き場所

# Where Extract starts: you can pick another there.
t4b73082f = 展開の初期設定です。そこで別の場所も選べます。

# One item right here, more in a new folder
ta5a4d31d = 1 件ならこの場所、複数なら新しいフォルダ

# A new folder next to it
t2095cbe9 = 隣に新しいフォルダを作る

# Named after the archive
tb991d4c1 = アーカイブと同じ名前

# Next to the archive
t4691c928 = アーカイブの隣

# Straight into its folder
t99d1df43 = そのフォルダに直接

# When a file is already there
tca514dda = ファイルがすでにある場合

# A file of the same name in the folder the files go to
t296ad686 = 展開先のフォルダに同じ名前のファイルがある場合

# Largest an archive may unpack to
t5957c426 = アーカイブを展開できる最大サイズ

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = アーカイブ爆弾（展開するとディスクを埋め尽くす小さなファイル）を防ぎます

# Only names every system can use
tf47f4968 = すべてのシステムで使える名前だけ

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = a:b.txt など Windows で使えない名前や、大文字・小文字だけが違う名前も、このコンピューターで確認します

# Owners from a backup
t38a7cf66 = バックアップの所有者

# Where a tar archive kept them: yours, or the archive’s by name or by number. Only an administrator gives files to others
t46908bee = tar アーカイブが保持している場合：自分のもの、またはアーカイブのものを名前か番号で。ほかのユーザーにファイルを渡せるのは管理者だけです

# Extended attributes from a backup
t764eed3a = バックアップの拡張属性

# Where a tar archive kept them, ACLs among them
ta28f3ae5 = tar アーカイブが保持している場合（ACL を含む）

# Light
t8ad2f8cf = ライト

# Dark
t45570065 = ダーク

# System
t8f3184bc = システム

# Same as system ({language})
t5bd525d3 = システムと同じ（{ $language }）

# The system’s
t1d68c60b = システムの設定

# Custom…
ta391dade = カスタム…

# The system’s ({size} px)
ta51f1877 = システムの設定（{ $size } px）

# All ({count})
tf65ec196 = すべて ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = 例：code -n、または /path/to/editor --flag

# Language
t9a73db9b = 言語

# Appearance
t31018b9f = 外観

# Text size
t2c041143 = 文字サイズ

# Processor cores to use
t2f9c3d17 = 使用するプロセッサコア数

# Text editor
t1ca82aab = テキストエディタ

# What Edit opens a file of a storage in
t62506ced = ストレージ上のファイルを「編集」で開くアプリ

# The file’s path is added at the end.
teba749d4 = ファイルのパスが末尾に追加されます。

# Paste your license key
t3c4da5cf = ライセンスキーを貼り付け

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = ファイルへの署名を除くすべての機能と、ネットワーク越しのストレージ { $count } 件。このコンピューター上のフォルダには制限がなく、署名の確認はいつでもできます。

# Every feature, and any number of storages.
tc3e75ecf = すべての機能と、無制限の数のストレージ。

# Every feature, and any number of storages, for your organization.
t69b29f66 = すべての機能と、組織向けの無制限の数のストレージ。

# Plan
t1eab5e32 = プラン

# Issued to
t44ef5cad = 発行先

# Who this license is for
td514fb0d = このライセンスの対象者

# Seats
t8f706321 = シート数

# What your organization’s agreement covers
t60b0c6c9 = 組織の契約が対象とする範囲

# Ended {date}
tfccb9305 = { $date } に終了しました

# This version came out after your updates ended, so it is Free. Renew, or use a version released by then: those stay Pro.
t545a1cd3 = このバージョンはアップデート期間の終了後に出たため Free です。更新するか、それまでにリリースされたバージョンをお使いください。それらは Pro のままです。

# This version stays Pro for good, with its fixes. Versions released after need a renewal.
tbf1ce5b8 = このバージョンは修正版も含めてずっと Pro のままです。その後のバージョンには更新が必要です。

# Until {date}
t4d9bac91 = { $date } まで

# Every version released by then is yours for good, with its fixes. Renew for a year more of new versions.
t5d8058ff = それまでにリリースされたバージョンは、修正版も含めてずっとお使いいただけます。更新すると新しいバージョンをさらに 1 年間受け取れます。

# Trial, until {date}
t9ffc2b63 = トライアル、{ $date } まで

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = { $date } に終了しました。あと { $days } 日間は引き続き使えます。

# Valid
td2661011 = 有効

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = この期間を過ぎると、packwing は Free になります。ストレージの追加ができなくなるだけで、ほかは変わりません。

# Your organization manages this license. Once a day, packwing asks the portal for a newer key of it, sending only the key.
t93c21721 = このライセンスは組織が管理しています。packwing は 1 日に 1 回、キーだけを送ってポータルに新しいキーがないか確認します。

# Your organization manages this license.
te42a0ee6 = このライセンスは組織が管理しています。

# Activate
t3b8f0190 = 有効化

# Buy
tcada780b = 購入

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = キーはこのコンピューター上で確認されます。何も外部には送信されず、ライセンスによって組織のポリシーの効果が変わることもありません。

# On
t60e47df0 = オン

# They show when you right-click files, folders and archives.
te216d656 = ファイル、フォルダ、アーカイブを右クリックしたときに表示されます。

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = packwing は現在メニューにありません。メニューに追加されると、これらも追加されます。

# Changing the menu…
tc7f1a2f0 = メニューを変更中…

# Good to know
t8d7b3a90 = ご注意

# The menu could not be changed: {why}
t9fac5c06 = メニューを変更できませんでした：{ $why }

# Show packwing in the right-click menu
t5d0d23cc = 右クリックメニューに packwing を表示

# Extract and compress from any folder
t94a46acb = どのフォルダからでも展開・圧縮

# Items in the menu
t67537dc4 = メニューの項目

# Put them in a submenu
te3449a65 = サブメニューにまとめる

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 では、メニューと「その他のオプションを確認」の両方で、1 つの Packwing 項目の下にまとめられます

# One packwing item, with these under it
tb5fdc584 = packwing の項目を 1 つにして、この下にまとめます

# 30 days
t7a572af9 = 30 日

# 90 days
t2d8a9817 = 90 日

# 1 year
t4d0a98a5 = 1 年

# Keep all
te0b1775b = すべて保持

# Keep the “downloaded from the Internet” mark
tfe574fa2 = 「インターネットからダウンロード」のマークを保持する

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = ダウンロードしたアーカイブ由来のファイルを実行する前に、システムが警告します

# This system has no such mark, so this changes nothing here
t314d60c1 = このシステムにはそのようなマークがないため、ここでは何も変わりません

# License
t38e70ee4 = ライセンス

# Your organization’s, in Settings › License
tff7c594e = 組織のものです。設定 › ライセンス をご覧ください

# Newer license keys from the portal
t6a8c1060 = ポータルからの新しいライセンスキー

# Once a day, packwing asks for a newer key of your organization’s, sending only the key
t8c9b329a = packwing は 1 日に 1 回、キーだけを送って組織の新しいキーを確認します

# Treat files from storages as from the Internet
t0af8d466 = ストレージのファイルをインターネット由来として扱う

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = S3、WebDAV、SFTP から取得したものにもマークを付けます

# Require encryption for new archives
t0c3e1337 = 新規アーカイブに暗号化を必須にする

# Applies to archives made and files added here, not to other apps
tc889f3a2 = ここで作成したアーカイブと追加したファイルが対象です。ほかのアプリは対象外です

# Kinds of encryption allowed
t1b8b2116 = 許可する暗号化の種類

# New archives can be encrypted only these ways
t71dcd29c = 新規アーカイブは、これらの方法でのみ暗号化できます

# Minimum password length
tb7d64a44 = パスワードの最小文字数

# In characters, for every new archive with a password
tca6d6982 = 文字数。パスワード付きの新規アーカイブすべてに適用されます

# Keep an audit log
tf7fd8762 = 監査ログを記録

# A record of what packwing does, whatever the license
t34fe4e25 = packwing の操作を、ライセンスにかかわらず記録します

# Where the audit log is kept
tbd999abe = 監査ログの保存場所

# A folder your organization chose
td4e1f74a = 組織が選んだフォルダー

# How long the audit log is kept
td504acaf = 監査ログの保存期間

# In days; 0 keeps every record
t127a3c54 = 日数。0 ですべての記録を保持

# Most the audit log may take
ta334bb40 = 監査ログの最大サイズ

# In MB; the oldest records go first
t03a543d2 = MB 単位。古い記録から削除されます

# What the audit log says of archives
t8d6fca43 = アーカイブについて監査ログに記録する内容

# “entries” names each file extracted, and those found damaged or left out
t1e33aba6 = 「entries」は展開した各ファイルと、破損または除外されたファイルの名前を記録します

# Every job recorded
t42369815 = すべての処理を記録

# A job whose start the audit log cannot write does not start
tda48e1c9 = 監査ログに開始を書き込めない処理は開始しません

# The audit log in the system’s log
t4a2fcd04 = システムのログへの監査ログ

# Each record goes to {log} as well, where your organization’s tools read it
t3a4c104f = 各記録は { $log } にも書き込まれ、組織のツールがそこから読み取ります

# packwing has no policy of this name.
t7e03649c = packwing にこの名前のポリシーはありません。

# Ask before opening files without extracting
t76d5d951 = 展開せずに開く前に確認する

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = アーカイブ内のファイルを開くと、一時フォルダにコピーが作られます。プログラムは常に確認されます。

# Paste a public key
t0bf72f73 = 公開鍵を貼り付け

# What packwing does, kept on this computer: archives extracted, made, converted and copied, when and by whom. Never a password or what is in a file.
t2162955d = packwing の操作をこのコンピューターに記録します：展開・作成・変換・コピーしたアーカイブ、その日時と実行者。パスワードやファイルの内容は記録しません。

# Keep its records for
tf423722f = 記録の保存期間

# Older ones are taken away, and that is recorded too
t7073f273 = 古い記録は削除され、その削除も記録されます

# Takes a Pro or Enterprise license: Settings › License.
t2774cee6 = Pro または Enterprise ライセンスが必要です：設定 › ライセンス。

# Where the audit log is
t2702fcc6 = 監査ログの場所

# View records
t27c5284a = 記録を表示

# Open folder
t713a842f = フォルダーを開く

# The last record could not be written: {why}
t542148a1 = 直前の記録を書き込めませんでした：{ $why }

# pkw audit verify checks that no record was changed or taken out. Whoever runs this computer can still remove the log: it tells what packwing did, it does not stop anyone.
t482581ff = pkw audit verify は、記録が変更・削除されていないことを確認します。このコンピューターの管理者はログを削除できます。ログは packwing の操作を示すもので、誰かを止めるものではありません。

# This is not a public key packwing can check with: {why}.
td57933b9 = packwing で確認に使える公開鍵ではありません：{ $why }。

# This line cannot be read, so the key is not used
t90e7dd7b = この行は読み取れないため、キーは使われません

# Keys you trust
ted99ced8 = 信頼するキー

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = これらのキーのいずれかによる署名は、誰がファイルに署名したかを示します。それ以外のキーによる署名は、一致していても何の証明にもなりません。

# Choose a file…
t28a576c9 = ファイルを選択…

# {name} (weak)
tdb06c037 = { $name }（弱い）

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = ファイルのメニューの「チェックサム…」に表示するもの。ファイルの入手元に記載されたチェックサムと比較できます。弱いものは破損したファイルは見分けられますが、意図的に変更されたファイルは見分けられません。

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = ポリシーを読み取れないため、保護対象は拒否されます：{ $why }

# Some settings are managed by your organization
t7db7e7ec = 一部の設定は組織によって管理されています

# Get help
t4a1ed54a = ヘルプを見る

# Not understood
t068eea1b = 解釈できません

# Off
t0dc6b4aa = オフ

# General
t5561fd0b = 一般

# Extracting
tc82af3f2 = 展開

# Compressing
t6d183c73 = 圧縮

# Security
tc2fe21db = セキュリティ

# Right-click menu
t1287ae97 = 右クリックメニュー

# About
t58e096de = 情報

# Settings
t4b058728 = 設定

# This computer has no way to ask that it is you.
t0c1b4d43 = このコンピューターには、本人確認を求める手段がありません。

# open a storage
t780455bf = ストレージを開く

# Open a storage in packwing
tf861e014 = packwing でストレージを開く

# That was not it. It stays locked.
tbb2a9838 = 確認できませんでした。ロックされたままです。

# The system could not ask: {why}
td6905afe = システムが確認を求められませんでした：{ $why }

# This copy of packwing cannot open it.
tfb56d8cf = このバージョンの packwing では開けません。

# Storages
tc776c52f = ストレージ

# Add storage
te858d6b5 = ストレージを追加

# Type the password you log in to this computer with to open it.
tb544cd9b = 開くには、このコンピューターへのログインに使うパスワードを入力してください。

# Show this computer it is you to open it.
tfb4c635b = 開くには、このコンピューターに本人であることを示してください。

# Checking…
t87f62cb7 = 確認中…

# It asks again after 5 minutes left unused.
t69c22a3b = 5 分間使わないと再度求められます。

# Edit…
t8e7efcd7 = 編集…

# Remove from this list
t8c7c7c57 = この一覧から削除

# Extract here with packwing
tc5e54e4a = packwing でここに展開

# Extract to new folder
tda62691f = 新しいフォルダに展開

# Extract with packwing…
tda9255a0 = packwing で展開…

# Open with packwing
teb657427 = packwing で開く

# Compress to ZIP
t344b1175 = ZIP に圧縮

# Compress with packwing…
t9ec9b77d = packwing で圧縮…

# packwing cannot find its program: {e}
t1934ce08 = packwing がプログラムを見つけられません：{ $e }

# Signed by {signer}
t8eb35b10 = { $signer } による署名

# Signed by {signer}, not checked yet
t88b7186a = { $signer } による署名、未確認

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = 信頼していないキーによる署名：{ $fingerprint }

# It does not match its signature
tb7d0e1ed = 署名と一致しません

# Its signature cannot be read
tadf2a401 = 署名を読み取れません

# Its signature could not be checked
t69cbbb51 = 署名を確認できませんでした

# Checking its signature…
tda8c2e03 = 署名を確認中…

# the system's package manager
t11465c20 = システムのパッケージマネージャー

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = ダウンロード中… { $got } / { $total }

# Downloading… {got}
t4873b921 = ダウンロード中… { $got }

# just now
ta715ef0f = たった今

# {n} minute ago | {n} minutes ago
t6c7302d6 = { $n } 分前

# {n} hour ago | {n} hours ago
t137b052e = { $n } 時間前

# {n} day ago | {n} days ago
tb9fa7744 = { $n } 日前

# No folder to download to.
t2bfc1370 = ダウンロード先のフォルダーがありません。

# packwing can’t show this kind of file
tb909bf3c = packwing ではこの種類のファイルを表示できません

# Too big to show here
t1b339d35 = 大きすぎてここでは表示できません

# It needs the password to show it
t81f5dbce = 表示するにはパスワードが必要です

# It could not be read: {why}
t26005fe3 = 読み取れませんでした：{ $why }

# Find in text
t9cbe4be4 = テキスト内を検索

# Invalid pattern
t5b9dcaca = 無効なパターン

# No matches
td29421ef = 一致なし

# Find
t3e1cd17a = 検索

# Fit the window (0)
tbe39720a = ウィンドウに合わせる (0)

# Fit
t116f13cc = 全体表示

# Show at its own size (1)
tb1eec64a = 実際のサイズで表示 (1)

# {count} line | {count} lines
t9fbb1078 =
    { $n ->
       *[other] { $count } 行
    }

# Its first 1 MB is shown
t329dac80 = 先頭の 1 MB を表示しています

# Update available
t676409df = アップデートがあります

# Restart to update
tbbbbda26 = 再起動して更新

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
       *[other] 名前がすでに使われていたため、{ $n } 個のファイルをスキップしました。
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
       *[other] { $n } 個のリンクは元の場所のままです。
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = このファイルはパスフレーズで暗号化されています。

# This archive needs its password.
td46b69a1 = このアーカイブにはパスワードが必要です。

# This file is encrypted to a key.
tbd048b4e = このファイルはキーを宛先として暗号化されています。

# This key file needs its passphrase.
t81f4c775 = このキーファイルにはパスフレーズが必要です。

# {paths} archives
t27f3b860 = { $paths } 個のアーカイブ

# {folder}, a folder for each
t75e7db7c = { $folder }、それぞれ専用のフォルダ

# New
t8b242db1 = 新規

# Your organization requires each job to be recorded, and the audit log cannot be written: {why}
t3f254f09 = 組織はすべての処理の記録を求めていますが、監査ログに書き込めません: { $why }

# There is no file to save it beside.
ta9906e3b = 隣に保存するファイルがありません。

# Nothing in the archive matches the patterns given.
t90b19482 = アーカイブの中に、指定したパターンに一致するものはありません。

# Nothing to compress.
t47339885 = 圧縮するものがありません。

# {path} is too big.
te9ab879e = { $path } は大きすぎます。

# This format holds one file, and no more.
t75c197d9 = この形式に入れられるのは 1 つのファイルだけです。

# Store
t6da32dce = 無圧縮

# Lowest
t5d7dbfdb = 最低

# Low
tb1dd9521 = 低

# Medium
t09be5f96 = 中

# High
tbe0a061d = 高

# Highest
t9487b22f = 最高

# Could not read {path}: {e}
tad0120eb = { $path } を読み取れませんでした：{ $e }

# {path} has no name.
tbd0e16dc = { $path } には名前がありません。

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = { $path } の名前は有効な Unicode ではないため、アーカイブに入れられません。

# Could not write {path}: {e}
t8dd055c6 = { $path } を書き込めませんでした：{ $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } とそれ以降の名前はすでに使われています。

# Could not encrypt: {e}
t485d71ab = 暗号化できませんでした：{ $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = { $level } には約 { $needed } のメモリが必要ですが、空きは { $available } です。約 { $less } で済む { $lower } で作成します。

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = このレベルには約 { $needed } のメモリが必要ですが、空きは { $available } です。コンピュータが遅くなるか、packwing が止まるおそれがあります。

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = ZIP や 7z のアーカイブで age の鍵を使えるのは、age で全体を暗号化する場合だけです。

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age の暗号化はパスフレーズかキーのどちらか一方で、両方は使えません。

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = ポスト量子鍵はほかのキーと併用できません。併用すると、残りのキーのせいで量子コンピューターにファイルを開かれてしまいます。

# There are no keys in this file to encrypt to.
tc2809b2c = このファイルには、暗号化の宛先にできるキーがありません。

# {path} got shorter while it was being read.
t2e221d1f = { $path } は読み取り中に小さくなりました。

# A password has to be valid Unicode.
t5ef729bc = パスワードは有効な Unicode である必要があります。

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = { $path } の拡張属性を読み取れませんでした: { $e }

# Could not start compressing: {e}
tc4faa70c = 圧縮を開始できませんでした：{ $e }

# Could not compress: {e}
tf6ff23f2 = 圧縮できませんでした：{ $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } はリンクです。リンクは Unix でのみ保持されます。

# Changing this archive writes a new one beside it, which may take up to {need}. Only {free} is free in {folder}.
ta087fe64 = このアーカイブを変更すると、隣に新しいアーカイブが書き込まれ、最大 { $need } 必要になる場合があります。{ $folder } の空きは { $free } しかありません。

# Nothing changed.
tedfc294c = 変更はありません。

# The archive did not come out whole.
t2f015c79 = アーカイブが完全な形で作成されませんでした。

# packwing can change only a ZIP or 7z archive that is one file.
t675882a8 = packwing が変更できるのは、1 つのファイルからなる ZIP または 7z アーカイブだけです。

# That is not in the archive.
t7530c419 = それはアーカイブにありません。

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = アーカイブにはこれしか入っていません。代わりにアーカイブ自体を削除してください。

# “{to}” cannot be a name.
td53e55bf = 「{ $to }」は名前にできません。

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = 「{ $to }」はすでにそのフォルダにあります。何も変更されていません。

# packwing can add files only to a ZIP or 7z archive that is one file.
t8d1ef729 = packwing がファイルを追加できるのは、1 つのファイルからなる ZIP または 7z アーカイブだけです。

# An archive can’t go inside itself.
tff43ce74 = アーカイブを自分自身の中に入れることはできません。

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = 「{ $bare }」はすでにアーカイブにあります。何も変更されていません。

# Nothing chosen matches the patterns given.
te132289a = 選んだものに、パターンに一致するものがありません。

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = アップロードの前に、このアーカイブは { $folder } に作成されます。最大 { $need } 必要になる場合がありますが、空きは { $free } しかありません。

# The upload stopped.
t528232e6 = アップロードが停止しました。

# “{pattern}” names no file or folder.
te17b2bfa = 「{ $pattern }」はファイルもフォルダも指していません。

# a colon, which Windows does not allow
t3ba6a1aa = Windows で使えないコロン

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = Windows で使えない文字 (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = Windows がデバイス用に予約している名前

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = Windows で使えない末尾の空白またはピリオド

# longer than 255 bytes
t9218e5c0 = 255 バイトを超える長さ

# a control character
t378bba3a = 制御文字

# not a name this system can read
t72b79e1d = このシステムで読めない名前

# the same file here as another, its name differing only in case
t1eead41f = ここでは別のファイルと同じ（名前の大文字・小文字だけが違う）

# a link that leads outside the folder
t00fdc25e = フォルダーの外を指すリンク

# {name} is already in the folder.
tbe7fab8a = { $name } はすでにフォルダにあります。

# Could not make a folder in {parent}.
tec7ca3a9 = { $parent } にフォルダを作成できませんでした。

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } はフォルダで、ファイルに置き換えられてしまいます。

# {name} is already in the folder
t5af584a0 = { $name } はすでにフォルダにあります

# This file is not an archive packwing can open.
t660f149b = このファイルは packwing で開けるアーカイブではありません。

# This is an ar archive, as static libraries are. packwing can’t open those yet.
t9c5564b8 = これは静的ライブラリなどの ar アーカイブです。packwing はまだ開けません。

# This is a macOS disk image (.dmg). packwing can’t open those yet.
t080d0cb2 = これは macOS のディスクイメージ (.dmg) です。packwing はまだ開けません。

# The password is wrong, or the archive is damaged.
ta4d3227d = パスワードが違うか、アーカイブが壊れています。

# The passphrase is wrong.
t7826c649 = パスフレーズが違います。

# The key file’s passphrase is wrong.
t8ad75e3a = キーファイルのパスフレーズが違います。

# This file is encrypted to a key packwing does not have.
te162af76 = このファイルは、packwing が持っていないキーを宛先として暗号化されています。

# Cancelled.
t27c010d6 = キャンセルしました。

# the archive could not be read
t2267c3b1 = アーカイブを読み取れませんでした

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = ここでは使えないファイル名があるため展開できません：「{ $name }」（{ $why }）。

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
       *[other] { $n } バイト
    }

# A volume of this archive is not there.
t3803103c = このアーカイブのボリュームが見つかりません。

# {named}: its data is damaged.
t066227f9 = { $named }：データが壊れています。

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = アーカイブが壊れています：UnRAR は { $number } で終了します。

# There is no public key here.
t9fd24078 = ここに公開鍵がありません。

# Could not make the key: {e}
t73610ecc = キーを作成できませんでした：{ $e }

# There is no file to sign.
tc95e3a9c = 署名するファイルがありません。

# {name} is not there any more.
t07c1f2bc = { $name } はもう存在しません。

# A part has to hold at least one byte.
te779b111 = 各パートは 1 バイト以上必要です。

# {path} is not a file.
tdfee38b2 = { $path } はファイルではありません。

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = 番号を付けられる数より多くのパートになります。パートを大きくしてください。

# {part} is there already.
t2577cbb6 = { $part } はすでにあります。

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path } は、7-Zip の方式で分割されたファイルのパート (name.001 など) ではありません。

# {part} is missing: the parts start from it.
t5e65a10a = { $part } がありません。パートはここから始まります。

# {part} is missing, and the parts after it are there.
t24165c89 = { $part } がありませんが、その後のパートはあります。

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = { $part } は { $got } で、最初のパートの { $size } と違います。完全ではないかもしれません。

# The name is empty.
t1a9d8e4e = 名前が空です。

# That name is kept for the system.
t74c8683a = その名前はシステムが使用するため使えません。

# A name cannot hold / or other special characters.
t4c0596ec = 名前に / やその他の特殊文字は使えません。

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = Windows では、名前の末尾にドットやスペースは使えません。

# That name is taken in this folder.
t43a2bce1 = その名前はこのフォルダですでに使われています。

# {name} is already there.
tb5e7b134 = { $name } はすでにあります。

# {name} cannot go inside itself.
t99f437a6 = { $name } を自分自身の中に入れることはできません。

# Stopped.
t15558940 = 停止しました。

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } は片方ではファイル、もう片方ではフォルダです。

# The copy of {name} does not match. The original was kept.
t779a64f3 = { $name } のコピーが一致しません。元のファイルは残しました。

# This is not a valid packwing license key.
td5ae978f = 有効な packwing のライセンスキーではありません。

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = これに関する組織のポリシーを読み取れないため、packwing はこの操作を行いません。IT 部門にご相談ください。

# Your organization requires new archives to be encrypted.
t83acf110 = 組織の方針により、新規アーカイブは暗号化が必須です。

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = 組織の方針により、この種類の暗号化は許可されていません：{ $method }。

# Your organization requires a password of at least {least} characters.
t2d21b125 = 組織の方針により、{ $least } 文字以上のパスワードが必要です。

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = そのパスワードは推測されやすすぎます。組織の方針により、より強いパスワードが必要です。

# Expected true or false, not {other}.
tcadcff7a = true か false が必要ですが、{ $other } が指定されました。

# Expected a number from 1 to {most}, not {other}.
tda498fad = 1 から { $most } までの数値が必要ですが、{ $other } が指定されました。

# Expected some text, not {other}.
t9386ce3b = テキストが必要ですが、{ $other } が指定されました。

# Expected a whole path, such as /var/log/packwing or \\server\audit, not {other}.
t00d3565d = /var/log/packwing や \\server\audit のような完全なパスが必要ですが、{ $other } が指定されました。

# Expected a number of days from 0 (keep all) to {most}, not {other}.
tf8fba675 = 0（すべて保持）から { $most } までの日数が必要ですが、{ $other } が指定されました。

# Expected a size in MB from 0 (no limit) to {most}, not {other}.
t9f7e002d = 0（無制限）から { $most } までの MB 単位のサイズが必要ですが、{ $other } が指定されました。

# Expected summary or entries, not {other}.
t27e9b96f = summary または entries が必要ですが、{ $other } が指定されました。

# Expected an address starting with https://, not {other}.
t93bfd19e = https:// で始まるアドレスが必要ですが、{ $other } が指定されました。

# Expected a list, not {other}.
tc945cc93 = リストが必要ですが、{ $other } が指定されました。

# No kind of encryption is called {name}.
tfddb7ba6 = { $name } という名前の暗号化の種類はありません。

# The list names no kind of encryption.
t5f8076f5 = リストに暗号化の種類が 1 つも指定されていません。

# The file is too big to be a list of policies.
taf68e582 = ファイルが大きすぎて、ポリシーのリストとして扱えません。

# The file has to hold one JSON object.
tf483aa65 = ファイルには JSON オブジェクトが 1 つだけ必要です。

# {path} has to belong to root.
tfc77ea2b = { $path } の所有者は root である必要があります。

# {path} can be changed by others than root.
t0d4a4498 = { $path } は root 以外でも変更できる状態です。

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = このストレージが今のところ受け付けるファイルは最大 { $size } です。送信中は各ファイルがメモリに保持されるためです。

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = このストレージではフォルダ名を一括で変更できません。配下のすべてがコピーされてしまうためです。「移動先…」を使ってください。

# S3 bucket
tc5e013b3 = S3 バケット

# Bucket
te34e3557 = バケット

# Region
t492e789d = リージョン

# Endpoint
t27e7d69c = エンドポイント

# Empty for AWS, or https://…
td3a94af8 = AWS の場合は空欄、または https://…

# Start in
t5eb91ea6 = 開始位置

# A prefix, if not the whole bucket
t75255a6c = バケット全体でない場合はプレフィックス

# Access key ID
tf9408d75 = アクセスキー ID

# Secret access key
te1c3d3a2 = シークレットアクセスキー

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = ホスト名または ssh_config の名前

# Empty for the home folder
t67b5f42b = 空欄の場合はホームフォルダ

# Empty to use the SSH agent and keys
tf10e6293 = 空欄の場合は SSH エージェントとキーを使用

# WebDAV folder
tb2401428 = WebDAV フォルダ

# Address
t700560f3 = アドレス

# User
te0a63e12 = ユーザー

# Password or app token
t1c18cb9b = パスワードまたはアプリトークン
