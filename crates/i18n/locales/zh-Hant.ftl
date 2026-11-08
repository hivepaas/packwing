# packwing in traditional Chinese, in Taiwan's words. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = 外觀：{ $name }。按一下切換為{ $name2 }。

# Stop and say so
tdc8b782b = 停止並告知

# Replace it
t1e17fc0c = 取代

# Skip it
tad9909cb = 略過

# Add
t9dc3aa14 = 新增

# The top of {name}
t369f9d4d = { $name } 的頂層

# Choose what to add
t48b969a8 = 選擇要加入的項目

# {count} item | {count} items
t534e422e =
    { $n ->
       *[other] { $count } 個項目
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
       *[other] { $count } 個檔案
    }

# Add to {name}
t0d53ac5e = 加入 { $name }

# Goes in
t36e95c22 = 放入位置

# If a file of that name is there
t34c69cec = 若已有同名檔案

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = 此封存檔已簽署。加入更多內容後，將不再符合其簽章，直到重新簽署為止。

# The new files are encrypted with the archive’s password.
t52b1d223 = 新檔案會以封存檔的密碼加密。

# Cancel
t35afca3b = 取消

# Name
t0fe07306 = 名稱

# Folder
t06357afd = 資料夾

# Its secrets could not be read: {why}
t2af0c8ab = 無法讀取其機密資料：{ $why }

# That folder is not on this computer.
ta4073fff = 該資料夾不在這台電腦上。

# Its secrets could not be kept: {why}
ta213827f = 無法保存其機密資料：{ $why }

# Local folder
te075bcbc = 本機資料夾

# Trying it…
t214cdad5 = 正在嘗試…

# It answers.
t3bd5bc9f = 有回應。

# It did not answer: {why}
t0ac8fa44 = 沒有回應：{ $why }

# Edit storage
t7bae68d0 = 編輯儲存位置

# Add a storage
t4fe8f806 = 新增儲存位置

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = 機密資料會存放在系統的鑰匙圈，不會與設定放在一起。

# Its files stay where they are.
t4e4acf9a = 其中的檔案會留在原處。

# Ask for this computer’s password to open it
t7a7d61e9 = 開啟時要求輸入這台電腦的密碼

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = 適用於未鎖定而離開的電腦：以解鎖這台電腦的方式即可開啟，閒置 5 分鐘後會再次詢問。

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing 免費版只能使用一個網路儲存位置。Pro 或 Enterprise 授權可解除此限制：設定 › 授權。這台電腦上的資料夾不受限制。

# Test connection
te0f3b6b7 = 測試連線

# Enter a license key
ta24f28a2 = 輸入授權金鑰

# Save
t4d2d5d68 = 儲存

# Paste the checksum given with the file
t8333758c = 貼上隨檔案提供的檢查碼

# They could not be saved: {why}
tb45ae457 = 無法儲存：{ $why }

# The file could not be read: {why}
t5daa8455 = 無法讀取檔案：{ $why }

# Reading…
t6aad8485 = 正在讀取…

# Copied
t8dc21305 = 已複製

# Copy
t658f3664 = 複製

# weak
tedbc2ec9 = 弱

# There is no checksum in this.
t231f838a = 這裡面沒有檢查碼。

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = 這不是 packwing 產生的檢查碼：它有 { $digits } 位數。

# It matches the file’s {name} checksum.
t4f0350e4 = 與檔案的 { $name } 檢查碼相符。

# It is compared once the file is read.
tcb5c1745 = 檔案讀取完成後會進行比較。

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = 與檔案的 { $name } 檢查碼不符：這不是產生該檢查碼的檔案，或是檔案在傳輸途中有所變動。

# It may be a {name} checksum, which is not made yet.
t3d713e04 = 這可能是尚未產生的 { $name } 檢查碼。

# Make it
tcbefe458 = 產生

# Compare with
tbddcfc14 = 比較對象

# Reading… {size} of {size2}
t80036d1c = 正在讀取… { $size } / { $size2 }

# Reading… {size}
t1569c388 = 正在讀取… { $size }

# Checksums of {name}
te88f3ed9 = { $name } 的檢查碼

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = 整個檔案都會經過這台電腦，而你的服務供應商可能會就讀取的資料量收費。

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = 弱檢查碼（{ $names }）能發現損壞的檔案，但無法發現被刻意修改成相符的檔案。

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = 已儲存在旁邊，檔名為 { $files }；sha256sum -c 等工具不需 packwing 即可驗證。

# Compare…
tec85be80 = 比較…

# Saving…
tbd79b37d = 正在儲存…

# Save beside it
t7f4d85e3 = 儲存在旁邊

# Choose the checksums shown
teec532fe = 選擇顯示的檢查碼

# Read it
t80fb8482 = 讀取

# Done
t8dd31791 = 完成

# Nothing chosen yet
tcf390b0a = 尚未選擇

# {join} and {more} more
t8399ef70 = { $join } 及其他 { $more } 項

# Hide what goes in
t5a420780 = 隱藏要加入的內容

# Show what goes in
t9a20eb53 = 顯示要加入的內容

# Take it out
t90a93ec7 = 移除

# Drop them to add them
t6a4aedbe = 放開即可加入

# What goes in
t3b7c797a = 要加入的內容

# + Files
tba8a1e71 = + 檔案

# + Folder
tcef8b0ce = + 資料夾

# Add the files or a folder to put in the archive.
t2e5c8c3e = 加入要放進封存檔的檔案或資料夾。

# Windows 11 and Linux open 7z on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tf7f64b21 = Windows 11 和 Linux 可直接開啟 7z；macOS、手機和較舊的 Windows 可能需要封存檔應用程式。若要與任何人分享，請選擇 .zip。

# Linux, macOS and Windows 11 open it on their own; phones and older Windows may need an archiver app. To share with anyone, pick .zip.
t051031b1 = Linux、macOS 和 Windows 11 可直接開啟；手機和較舊的 Windows 可能需要封存檔應用程式。若要與任何人分享，請選擇 .zip。

# Linux and Windows 11 open it on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tc2c29183 = Linux 和 Windows 11 可直接開啟；macOS、手機和較舊的 Windows 可能需要封存檔應用程式。若要與任何人分享，請選擇 .zip。

# Most computers need an archiver app or the lz4 tool to open it. To share with anyone, pick .zip.
te9afe1c2 = 多數電腦需要封存檔應用程式或 lz4 工具才能開啟。若要與任何人分享，請選擇 .zip。

# It holds this one file. Command-line tools open it; Windows and macOS may need an archiver app. To share with anyone, pick .zip.
t16463509 = 僅能存放這一個檔案。命令列工具可開啟；Windows 和 macOS 可能需要封存檔應用程式。若要與任何人分享，請選擇 .zip。

# Don’t split
t56ee6829 = 不分割

# 25 MB (email)
tc6ba1c9c = 25 MB（電子郵件）

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB（USB 隨身碟）

# Custom size…
t0fcf9e2d = 自訂大小…

# Size
ta6478e7c = 大小

# Password
t2cc30838 = 密碼

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…、age1pq1… 或 ssh-ed25519 …

# Save here
ta1f147a6 = 儲存於此

# Use these keys
t1a01e439 = 使用這些金鑰

# No compression
t2e9c9e78 = 不壓縮

# {name} compression
te2db10f6 = { $name } 壓縮

# Several frames
t84cc41f7 = 多幀

# One frame
tc6ef759e = 單幀

# Small blocks
t04032e8a = 小區塊

# Parts of 25 MB
ta999efe2 = 每份 25 MB

# Parts of 4 GB
t760df467 = 每份 4 GB

# Parts of {size}
tdd286479 = 每份 { $size }

# Parts of a size to type
tafd4b17d = 自行輸入每份大小

# Public key
t9981f45f = 公鑰

# Owners kept
t63e8a0df = 保留擁有者

# Attributes kept
te9e8289c = 保留屬性

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = 要在備份中保留擁有者與延伸屬性，請選擇 .tar 格式。

# Keep for a backup
t6a2bcc1a = 為備份保留

# Owners: user and group
t5350e3fe = 擁有者：使用者與群組

# Extended attributes and ACLs
t35aff4d1 = 延伸屬性與 ACL

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = 用於在這台電腦上還原的備份。開啟封存的人都能看到使用者名稱。

# Beside the files
t757989af = 與檔案放在一起

# What it holds, from
teae4ef1a = 內容來自

# Save in
tf2d34837 = 儲存到

# Folder…
tc8abcd43 = 資料夾…

# Storage…
td0d8c806 = 儲存位置…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = 分割檔會先在這台電腦的暫存資料夾中建立，再上傳，因此該處需要足夠空間。

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext } 會先在這台電腦的暫存資料夾中建立，再上傳，因此該處需要足夠空間。.zip 和 tar 格式則直接寫入此儲存位置。

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = 會先在這台電腦的暫存資料夾中建立，再上傳，因此該處需要足夠空間。

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } 最多可存放 { $most } 的檔案。若封存檔超過此大小，開始上傳時就會失敗。

# Choose a storage, and a folder in it
td7627e48 = 請選擇儲存位置及其中的資料夾

# New folder
t1926d127 = 新增資料夾

# Give it a name, without / or \ in it.
ta598b15b = 請輸入名稱，不可包含 / 或 \。

# A file of that name is there already: this one will be {name}.
t9451c3b1 = 已有同名檔案：此檔案將命名為 { $name }。

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = 有些名稱在 Windows 或 macOS 上無法使用，例如「{ $name }」。在那裡，這些檔案可能無法照原樣解壓縮。

# Hide advanced
tae2a96e9 = 隱藏進階選項

# Advanced options
t701d5ed9 = 進階選項

# Large blocks
tdc489452 = 大區塊

# Compression level
td386c9ef = 壓縮等級

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = 僅儲存會原樣放入檔案：建立最快，體積最大。

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = 等級越高，封存檔越小，建立時間越長，但開啟時間不變。

# Frames
tf03e266f = 幀

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = 多幀可在所有核心上開啟，封存檔會稍大一些：不到百分之零點五。

# One frame makes the smallest archive, which opens on one core.
ta097b317 = 單幀產生最小的封存檔，只在一個核心上開啟。

# Blocks
t036a71e3 = 區塊

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = 小區塊讓每個核心都在工作：最多快兩倍，壓縮檔大幾個百分點。

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = 大區塊產生最小的壓縮檔，要壓縮的內容較少時，用到的核心較少。

# Split into parts
tca4436bd = 分割成多個部分

# Each part
t642c444d = 每個部分

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = 分割檔可用 7-Zip 及類似的封存檔工具開啟，無法用 Windows 和 macOS 內建的工具開啟。

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = 每個部分的大小須為 1 MB 以上，例如 100 MB。

# Encrypt with age
t22784ad9 = 以 age 加密

# Protect with a password
tae8d8740 = 以密碼保護

# Suggest
ta6e44ce5 = 建議密碼

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = 請妥善保存：沒有它，這個封存檔將永遠無法開啟。

# Type a password, or turn this off.
t62b19318 = 請輸入密碼，或關閉此選項。

# Hide file names too
t2198d77b = 同時隱藏檔名

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = 設定密碼後，Windows 和 macOS 內建的工具無法開啟這個 ZIP，收件者需要使用封存檔應用程式。檔名也仍然可見；若要隱藏檔名，請選擇 .7z。

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = 已儲存為 { $name }。其中的名稱同樣已加密。Windows 和 macOS 可能需要 age 工具或封存檔應用程式才能開啟。

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
       *[other] 來自 { $name } 的 { $n } 組金鑰
    }

# Remove
t21a5901d = 移除

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
       *[other] 其中有 { $n } 組金鑰已略過，因為其類型不支援 age 加密。
    }

# From a file…
t40d7c944 = 從檔案…

# Type a public key, or choose a file of them.
tcb028b06 = 請輸入公鑰，或選擇包含公鑰的檔案。

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = 這是後量子金鑰：要開啟此檔案，age 工具必須為 1.3 以上版本。

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = 只有對應的私鑰才能開啟，若私鑰遺失，任何方式都無法開啟。

# Weak
t6de87ae9 = 弱

# Fair
t82fa0d53 = 普通

# Strong
t0dad409a = 強

# Choose what to put in it
t5461cfa6 = 選擇要放入的內容

# Convert
t92e29218 = 轉換

# New archive
tf352befb = 新增封存檔

# Create
t990de47d = 建立

# {n} item | {n} items
tfff7fd6c =
    { $n ->
       *[other] { $n } 個項目
    }

# Move {what} to the Trash?
tdf5a8d53 = 要將{ $what }移到垃圾桶嗎？

# Delete {what}?
ta297d6c5 = 要刪除{ $what }嗎？

# It can be put back from the system’s Trash.
te5e137a1 = 可以從系統垃圾桶還原。

# This cannot be undone from packwing.
t3947dab4 = 在 packwing 中無法復原此操作。

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
       *[other] 以及另外 { $n } 項
    }

# Move to Trash
ta036661b = 移到垃圾桶

# Delete
t5797ea6a = 刪除

# {many} items
t49912bd4 = { $many } 個項目

# Rename {what}
t83401013 = 重新命名{ $what }

# New name
taf4cad88 = 新名稱

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = 會從封存檔中永久移除，資料夾連同其中所有內容一併移除。此操作無法復原。

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = 此封存檔已簽署。變更後將不再符合其簽章，直到重新簽署為止。

# Rename
tc80631f3 = 重新命名

# The archive’s password
t8a168799 = 封存檔的密碼

# one folder
td571ce35 = 一個資料夾

# one file
t95c9ec7b = 一個檔案

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }，因此直接解壓縮到這裡

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }，因此解壓縮到新資料夾

# Extract here
t354232ec = 解壓縮到此處

# {picked} selected
t9addc37e = 已選取 { $picked }

# All {counted}
t649feaf8 = 全部 { $counted }

# What should it extract?
td12e23bf = 要解壓縮什麼？

# Smart
t54535972 = 智慧

# Recommended
t7865024c = 建議

# Where should the files go?
td35f540c = 檔案要放在哪裡？

# Right here
t900a8a39 = 就在這裡

# Another folder
t9f128ba8 = 其他資料夾

# Choose…
t58daae9c = 選擇…

# Goes to {destination}
t8195aed2 = 會放到 { $destination }

# Checked as it goes
tb6c9570b = 邊解壓縮邊檢查

# · Nothing leaves the folder or passes the size limit
t11424797 = · 沒有任何內容會超出資料夾或超過大小限制

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } 來自網際網路。它會保留標記，因此執行前會先詢問你。

# Extract archive
t834ba3c0 = 解壓縮封存檔

# Extract
t30518e1e = 解壓縮

# Open {name}?
tcfe84c2f = 要開啟 { $name } 嗎？

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing 會在暫存資料夾中放一份副本並開啟它。對副本的修改不會寫回封存檔。

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = 這是程式：開啟就會執行。請僅在信任封存檔來源時才這麼做。

# Don’t ask again
t00c5d168 = 不要再詢問

# Run it
t72430853 = 執行

# Open
t538b10e9 = 開啟

# Open from a storage
t45a8991d = 從儲存位置開啟

# Choose a storage, and an archive in it
t3c900950 = 請選擇儲存位置及其中的封存檔

# Choose
t73c66160 = 選擇

# This is not a key file packwing can use: {why}.
tcf6986a7 = 這不是 packwing 可用的金鑰檔案：{ $why }。

# Could not read {path}: {e}.
t1dc6961f = 無法讀取 { $path }：{ $e }。

# The passphrase
t1e921d00 = 密碼片語

# The key file’s passphrase
t4051f4d7 = 金鑰檔案的密碼片語

# Enter the passphrase
tbd0672da = 輸入密碼片語

# Passphrase
t191a2dc5 = 密碼片語

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = 這個檔案以 age 加密。packwing 只會在封存檔開啟期間保留密碼片語，絕不會寫入儲存。

# Enter the password
t513ea7ab = 輸入密碼

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = 此封存檔已加密。packwing 只會在封存檔開啟期間保留密碼，絕不會寫入儲存。

# Choose the key file
t3c521a4e = 選擇金鑰檔案

# Key file
t6ed7c612 = 金鑰檔案

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = 這個檔案以 age 針對某個金鑰加密：可能是 age 金鑰檔案或 SSH 金鑰。packwing 只會在封存檔開啟期間保留從中讀取的內容，絕不會寫入儲存。

# Enter the key file’s passphrase
t808b85c5 = 輸入金鑰檔案的密碼片語

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = 這個金鑰檔案本身有密碼保護。packwing 只會在封存檔開啟期間保留其密碼片語，絕不會寫入儲存。

# kept, {size}
t001f6992 = 已保留，{ $size }

# left out
t73aeedd4 = 已略過

# renamed {to}
t7aa9b778 = 重新命名為 { $to }

# It was gone by the time it was read.
t93fb1e03 = 讀取時該檔案已不存在。

# The job stopped before it was done.
t57a78885 = 工作在完成前停止了。

# Waiting for other jobs…
t16ec36d9 = 正在等待其他工作…

# A file is damaged
t8b1230ed = 有檔案已損毀

# Some names can’t be used here
t26835ec8 = 有些名稱在這裡無法使用

# Converted, but some files are damaged
t54ef3472 = 已轉換，但有些檔案已損毀

# Extracted, but some files are damaged
t8226d624 = 已解壓縮，但有些檔案已損毀

# Converted, with some files left out or renamed
t5cce0e50 = 已轉換，部分檔案被略過或重新命名

# Extracted, with some files left out or renamed
t5dc002d6 = 已解壓縮，部分檔案被略過或重新命名

# Extracting…
t94279f82 = 正在解壓縮…

# Uploading…
tc75fe478 = 正在上傳…

# Creating…
t01fcd896 = 正在建立…

# Adding…
t2060c750 = 正在加入…

# Changing the archive…
t186fbef5 = 正在變更封存檔…

# Opening…
tc40f2321 = 正在開啟…

# Testing…
t19d1cb65 = 正在測試…

# Copying…
t02dd17e0 = 正在複製…

# Moving…
t890f7ed1 = 正在移動…

# Signing…
t7b8f8780 = 正在簽署…

# Splitting…
t5452adf5 = 正在分割…

# Joining…
taa884c51 = 正在合併…

# Converting…
ta1b8f4cc = 正在轉換…

# Couldn’t split the file
t83d27d26 = 無法分割檔案

# Couldn’t join the parts
t1cf67e00 = 無法合併分割檔

# Couldn’t convert the archive
t39f973a9 = 無法轉換封存檔

# Couldn’t sign
t4489f97c = 無法簽署

# Couldn’t copy
t613e4e9c = 無法複製

# Couldn’t move
t9796adec = 無法移動

# Couldn’t extract
tacd22c56 = 無法解壓縮

# Couldn’t create the archive
t9d0e01bc = 無法建立封存檔

# Couldn’t add the files
tfbd989b0 = 無法加入檔案

# Couldn’t change the archive
tcdc4a49c = 無法變更封存檔

# Couldn’t open the file
t041198a4 = 無法開啟檔案

# The test failed
tfcd070a5 = 測試失敗

# Starts when one of the others ends
tbe2a0509 = 其他工作之一結束後開始

# Paused
tde281feb = 已暫停

# Starting…
t73727cef = 正在開始…

# Finishing…
t042f47fe = 正在完成…

# Almost done
ta550a6a9 = 即將完成

# About {secs} seconds left
t3045da16 = 剩餘約 { $secs } 秒

# About a minute left
tf727815c = 剩餘約 1 分鐘

# About {div_ceil} minutes left
tbd7a3305 = 剩餘約 { $div_ceil } 分鐘

# {name} · checking that every file reads
t0d56ec7c = { $name } · 正在檢查每個檔案是否可讀取

# and {count} more | and {count} more
te728628b =
    { $n ->
       *[other] 還有 { $count } 個
    }

# The test passed
tdf0ce240 = 測試通過

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
       *[other] 已讀取 { $count } 個檔案，共 { $size }，沒有錯誤。
    }

# Close
tcd86acc3 = 關閉

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = 「{ $path }」已損毀：{ $why }。要停止、略過它，還是保留已讀出的部分？

# Do the same for the other damaged files
t798693ad = 對其他損毀的檔案也這樣處理

# Stop
t4b7f7705 = 停止

# Leave it out
taaff8ec9 = 略過

# Keep what was read
tc4387d25 = 保留已讀出的部分

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
       *[other] { $count } 個檔案無法以其名稱在這裡解壓縮。尚未寫入任何內容：停止、略過還是重新命名？
    }

# Leave out
ta50cc084 = 略過

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
       *[other] { $count } 個連結指向資料夾之外、這台電腦上的其他位置。尚未寫入任何內容：略過，還是在你信任此壓縮檔時照原樣建立？
    }

# Create them
t9a4d6e7d = 建立

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
       *[other] { $count } 個檔案的資料已損毀。
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
       *[other] 已略過 { $count } 個檔案。
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
       *[other] { $count } 個檔案已重新命名，因為其名稱在這裡無法使用。
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
       *[other] 有 { $count } 個檔案在讀取時已不存在，已略過。
    }

# The rest is in the archive.
t977a5572 = 其餘內容已放入壓縮檔。

# The rest was extracted.
ta7833fcb = 其餘已解壓縮。

# Copy the list
t3c49b269 = 複製清單

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } / { $count2 } 個檔案 · { $size } / { $size2 }

# Open the folder when it’s done
tf0f1017b = 完成後開啟資料夾

# Run in background
tb800c0ef = 在背景執行

# Resume
t454d1d8a = 繼續

# Pause
t44f9bf2d = 暫停

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
       *[other] 還有 { $n } 個工作正在執行
    }

# Closing packwing stops them
t2e6491b3 = 關閉 packwing 會停止它們

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = 未完成的部分會保持未完成：正在建立的封存檔會被刪除，解壓縮會停在目前的進度。

# Keep running
t79c1b9c7 = 繼續執行

# Stop them and quit
tf2707857 = 停止並結束

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = 其中的檔案維持原樣。packwing 用來登入它的憑證會從系統鑰匙圈中移除。

# The folder and its files stay where they are.
t4850aaa8 = 該資料夾及其中的檔案維持原樣。

# Remove “{name}” from the list?
t5dd9cfcf = 要將「{ $name }」從清單中移除嗎？

# It can be added again at any time.
t80d51a99 = 可以隨時重新加入。

# A passphrase for the new key
tf5593f05 = 新金鑰的密碼片語

# The same passphrase again
tad5c08e6 = 再次輸入相同的密碼片語

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = 簽署需要 Pro 或 Enterprise 授權：設定 › 授權。驗證簽章在所有方案中皆免費。

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = 這是被切成多份的檔案中的一份。簽章只涵蓋單一檔案，卻會讓人以為涵蓋全部，因此 packwing 不會簽署分割檔。

# That passphrase does not open this key.
tc5357c97 = 這個密碼片語無法開啟此金鑰。

# This is not a key packwing can sign with: {why}.
t662c29f9 = 這不是 packwing 可用於簽署的金鑰：{ $why }。

# The two passphrases are not the same.
t335fe90b = 兩次輸入的密碼片語不一致。

# Keep the key here
t9306dafe = 將金鑰存放於此

# No key chosen
te58c13f5 = 尚未選擇金鑰

# Key
tcd1ac90c = 金鑰

# New key…
t9e1134da = 新金鑰…

# Again
t58fb56ef = 再次輸入

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = 新金鑰同時包含 Ed25519 和 ML-DSA-65，因此即使其中一種被攻破，簽章依然有效。只有 packwing 和 hivepack 能驗證這類簽章；若要用 ssh-keygen 驗證，請改用 SSH Ed25519 金鑰簽署。金鑰的密碼片語一旦遺失，就無法再開啟該金鑰。

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = 簽章會放在檔案旁，檔名為 { $signature }，並取代已存在的簽章。整個檔案都會經過這台電腦，而你的服務供應商可能會就讀取的資料量收費。

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = 簽章會放在檔案旁，檔名為 { $signature }，並取代已存在的簽章。

# Sign {name}
ted250f31 = 簽署 { $name }

# Anyone can check it, with the key’s public half
t04ecc8be = 任何人都能用金鑰的公開部分來驗證

# Make the key…
t6bba28d5 = 建立金鑰…

# Sign
t8f6340c4 = 簽署

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = 在旁邊產生 { $count } 個分割檔，從 { $first } 開始。「合併」可把它們拼回去；packwing 和 7-Zip 可直接開啟這樣分割的封存檔。

# Split
t16a15383 = 分割

# Move {what}
tc168ca56 = 移動{ $what }

# Copy {what}
teda01f26 = 複製{ $what }

# Move here
t1aaf57aa = 移到這裡

# Copy here
t85f0245a = 複製到這裡

# Move
t7f2c2a34 = 移動

# If a name is taken
tc99c037f = 若名稱已被使用

# They are in this folder already. Choose another.
t5318c14e = 它們已在這個資料夾中。請選擇其他位置。

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = 所有內容都會經過這台電腦，而你的服務供應商可能會就讀取的資料量收費。每個移動的原始檔案會在其副本讀回並確認相符後才移除。

# Each original goes once its copy has been read back and matches.
t8ecbab4e = 每個原始檔案會在其副本讀回並確認相符後才移除。

# The originals stay where they are.
t5937f050 = 原始檔案會留在原處。

# Too big to show here
t1b339d35 = 太大，無法在此顯示

# packwing can’t show this kind of file
tb909bf3c = packwing 無法顯示這種檔案

# It could not be read
tb7927177 = 無法讀取

# It could not be read: {why}
t26005fe3 = 無法讀取：{ $why }

# It could not be shown
tb60fb288 = 無法顯示

# Open with default app
t5ef80ec5 = 以預設應用程式開啟

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = 另一個工作正在那裡建立 { $name }：請為這個工作換個名稱，或等它結束。

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = 另一個工作正在解壓縮到 { $folder }：請選擇其他資料夾，或等它結束。

# no editor command
tbf2a5d3a = 未設定編輯器指令

# no text editor found
t6256b3da = 找不到文字編輯器

# Search this folder
t2c814039 = 搜尋此資料夾

# This folder could not be read: {why}
t2c6017b2 = 無法讀取此資料夾：{ $why }

# Could not load more: {why}
t466e87c4 = 無法載入更多：{ $why }

# Could not delete: {why}
te5951d38 = 無法刪除：{ $why }

# the text editor
t91cdab0c = 文字編輯器

# Cannot start {who}: {why}
t8deb160a = 無法啟動 { $who }：{ $why }

# Edit in text editor
td8d39fb8 = 在文字編輯器中編輯

# Edit in {editor}
te44cd57b = 在 { $editor } 中編輯

# Modified
tdd13a6b2 = 修改日期

# Loading…
tc4e2f181 = 正在載入…

# Only what is loaded was searched.
t39c5f536 = 只搜尋了已載入的項目。

# Search the whole folder
t41f9a787 = 搜尋整個資料夾

# Load {n} more
t30bed46a = 再載入 { $n } 項

# Refresh
t0815aad4 = 重新整理

# Hide hidden files
t3375c9c0 = 隱藏隱藏檔案

# Show hidden files
tf18c7057 = 顯示隱藏檔案

# View
t5e4baa18 = 檢視

# Extract…
t4a09490e = 解壓縮…

# Compress…
t690f9553 = 壓縮…

# Copy to…
t543cf15b = 複製到…

# Move to…
t7a32320b = 移動到…

# Sign…
td6d61b10 = 簽署…

# Checksums…
t2891102d = 檢查碼…

# Convert…
tae85a7c4 = 轉換…

# Split into parts…
t281db403 = 分割為多個檔案…

# Join the parts
t0ad56db8 = 合併分割檔

# Copy path
tf165cb2b = 複製路徑

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
       *[other] 壓縮 { $n } 個項目…
    }

# {n} loaded, more in this folder
t29a52578 = 已載入 { $n } 項，此資料夾還有更多

# Sorted among those loaded
t037d01ae = 僅在已載入的項目中排序

# {n} selected
tebda8468 = 已選取 { $n } 項

# Try again
t213e90fa = 再試一次

# Nothing here matches that.
t0f9a8a58 = 沒有符合的項目。

# Add a storage to browse it.
t0b82c83a = 新增儲存位置後即可瀏覽。

# No folders or archives here.
t6a090cc3 = 這裡沒有資料夾或封存檔。

# No folders here.
tc7731c8f = 這裡沒有資料夾。

# This folder is empty.
td9e47db6 = 這個資料夾是空的。

# Show in Finder
ta9442aa9 = 在 Finder 中顯示

# Show in File Explorer
t23c89eda = 在檔案總管中顯示

# Show in the file manager
tb347ece9 = 在檔案管理員中顯示

# Open in Finder
te2a467e0 = 在 Finder 中開啟

# Open in File Explorer
t3794d625 = 在檔案總管中開啟

# Open in the file manager
td1ceabc4 = 在檔案管理員中開啟

# Copy all
t6aef5c37 = 全部複製

# This system has no folder for settings.
tfd0ff0a1 = 這個系統沒有存放設定的資料夾。

# Archive
t73690b65 = 封存檔

# Search inside this archive
tb35c0c27 = 搜尋此封存檔內部

# Not text it can show
ta413eca5 = 不是可顯示的文字

# It needs the password to show it
t81f5dbce = 需要密碼才能顯示

# Open another…
tff74a552 = 開啟其他…

# From a storage…
tcfba8fd5 = 從儲存位置…

# Safe to extract
t3c93bb70 = 可安全解壓縮

# Reading… {percent}%
t83223c27 = 讀取中… { $percent }%

# Read {fetched} of {whole}
t613dd932 = 已讀取 { $fetched }，共 { $whole }

# Add files
tdd45c7bb = 加入檔案

# Test
t2ffcbe05 = 測試

# Reading what’s inside…
t1bbda9f5 = 正在讀取內容…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = 此封存檔中沒有符合「{ $query }」的項目。

# Nothing in this folder matches “{query}”.
teecdfa3d = 此資料夾中沒有符合「{ $query }」的項目。

# And {more} more. Type more to narrow it down.
t843253d0 = 另外還有 { $more } 項。請多輸入一些字以縮小範圍。

# Open an archive
t8382bd3e = 開啟封存檔

# ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…
t4a498ac8 = ZIP、7z 和 tar 封存檔、光碟映像檔和套件，以及以 age 壓縮或加密的檔案：.zip、.7z、.tar.gz、.tar.zst、.iso、.rpm、.pkg、.zst.age…

# Open an archive…
t2cf2a8ee = 開啟封存檔…

# Choose a file to see it here
t37854216 = 選擇檔案即可在此查看

# Symbolic link
tc7c20187 = 符號連結

# Leads to {target}
te7ddae48 = 指向 { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
       *[other] 此資料夾中的 { $count } 個可執行檔解壓縮後會保留網際網路標記。
    }

# This computer
tfa0d176a = 這台電腦

# Home
t52f50eae = 家目錄

# Desktop
t64215c53 = 桌面

# Downloads
t539307de = 下載

# Drop to open
tb255877b = 拖放即可開啟

# Browse
t0eac1f07 = 瀏覽

# Recent
t50e888fe = 最近使用

# Choose a file to see its information here
t50aac02b = 選擇檔案即可在此查看其資訊

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
       *[other] 已選取 { $n } 項
    }

# Files in them come to {size}
t0cca93f1 = 其中的檔案共 { $size }

# Delete…
t2fed07aa = 刪除…

# Link to a folder
t4b408baf = 資料夾的連結

# Link
t8e08e589 = 連結

# {tag} file
ta56bbbe9 = { $tag } 檔案

# {files}, {folders}
t72e46dc5 = { $files }、{ $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
       *[other] { $n } 個檔案
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
       *[other] { $n } 個資料夾
    }

# {size} so far
tc87daf59 = 目前 { $size }

# Kind
t3ff50c63 = 類型

# Holds
tffe19c31 = 內容

# Permissions
t8700fd83 = 權限

# Path
teb66e456 = 路徑

# Signature
tc049050d = 簽章

# Count size and files
t0f23db17 = 計算大小與檔案數

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
       *[other] 其中有 { $n } 個資料夾無法讀取。
    }

# Check the signature
t9d44730e = 驗證簽章

# Extracted
t3250471f = 已解壓縮

# Archive created
t22c69419 = 已建立封存檔

# Moved
t82869bf0 = 已移動

# Signed
t27c3ab35 = 已簽署

# Joined
tf8f2b670 = 已合併

# Converted
tc7b76e99 = 已轉換

# Waiting for a password or a key
t0e83ae32 = 正在等待密碼或金鑰

# Enter it…
tddb7844a = 輸入…

# Show
t29742e5c = 顯示

# Jobs
t4ae6a79d = 工作

# Clear finished
t30c178e8 = 清除已完成

# Nothing runs in the background.
t2573f39e = 背景中沒有正在執行的工作。

# Add to the right-click menu
t116d3868 = 加入右鍵選單

# Extract or compress from any folder.
t20864983 = 可在任何資料夾中解壓縮或壓縮。

# Welcome to packwing
tb38a86a6 = 歡迎使用 packwing

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = 開啟任何封存檔、製作更小的檔案以便分享，並避開有害的下載內容。

# Set things up
t63f9c1cd = 進行設定

# You can change these later in Settings.
t95f41ffb = 你之後可以在設定中變更這些項目。

# Get started
te83ca5ee = 開始使用

# Remove from Recent
t26cc8dba = 從最近使用中移除

# {location} · not there any more
t7d0c09c7 = { $location } · 已不在該處

# The archives you open will show here.
tc0cc82d5 = 你開啟過的封存檔會顯示在這裡。

# Copy version info
t3d37483c = 複製版本資訊

# Version {version} · {os} {arch}
t6bead5ae = 版本 { $version } · { $os } { $arch }

# A file archiver that opens archives safely.
t3c244df9 = 安全開啟封存檔的檔案封存工具。

# Updates
t483075a5 = 更新

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = 這是測試版：會通知你較新的測試版，以及隨後推出的正式版。

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = 這是正式版：會通知你較新的正式版，但不會通知測試版。

# Check now
t4f0e0677 = 立即檢查

# Look for updates once a day
tcc27ace1 = 每天檢查一次更新

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = 會從 GitHub 讀取版本清單，且僅在簽章驗證通過時才採信。未經你同意，不會下載或安裝任何內容。

# Skip this version
t2c33d756 = 略過此版本

# This build has no release keys, so it cannot tell of updates.
t378182cb = 此版本沒有發行金鑰，因此無法通知更新。

# packwing {version} is ready
tf02650f8 = packwing { $version } 已準備就緒

# It is installed when you quit, or now with Restart.
t6fecac86 = 會在結束時安裝，或按「重新啟動」立即安裝。

# Restart now
t54001a7c = 立即重新啟動

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = 正在檢查新版本…

# Release notes
tc52ab62f = 版本資訊

# packwing {version} is out
t36a30d8b = packwing { $version } 已發布

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = 請在終端機中執行這些指令。它們會根據已簽署的版本清單驗證下載內容，sudo 會詢問你的密碼。

# {by} updates it: {command}
tdf2cab9c = 透過 { $by } 更新：{ $command }

# {by} updates it.
t84b27295 = 透過 { $by } 更新。

# It is downloaded and checked, then installed when you quit.
tb46c0678 = 下載並驗證後，會在結束時安裝。

# Download it, and install it in place of this one.
t39312eae = 下載後安裝，取代目前的版本。

# The download failed: {why}
t5e96c075 = 下載失敗：{ $why }

# Download and install
t1a99173f = 下載並安裝

# Download
t6964bb19 = 下載

# The last look failed: {why}
td122a092 = 上次檢查失敗：{ $why }

# Up to date (looked {when}).
t4665ef60 = 已是最新版本（上次檢查：{ $when }）。

# Not looked yet.
t5a3ba058 = 尚未檢查。

# Free software
t731220a4 = 自由軟體

# Under the GNU General Public License, version 3 or later
t4e98bf3c = 依 GNU 通用公共授權條款第 3 版（或更新版本）授權

# View license
tfc21d38b = 檢視授權

# Made by HivePaaS
t10c7007a = 由 HivePaaS 開發

# On hivepack, our library for reading and writing archives
tfae09b61 = 建構於 hivepack，我們用來讀寫封存檔的函式庫

# Source code
tf6a49f4d = 原始碼

# Something wrong?
t610dfb6f = 發現問題？

# Tell us, with the version above
ta3e29034 = 請告訴我們，並附上上方的版本

# Report a problem
t7b1728e7 = 回報問題

# What changed
te8345543 = 更新內容

# Every release, and what is new in it
t8e6f2052 = 每個版本及其新增內容

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS。packwing 名稱與標誌為 HivePaaS 的商標。

# More from HivePaaS
tb25cf729 = HivePaaS 的更多產品

# Free and open source, like packwing.
tc1ece130 = 與 packwing 一樣，免費且開放原始碼。

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = 快速的原生 SSH 用戶端，支援 Linux、macOS 和 Windows。

# SSH and SFTP
t0f8af34d = SSH 與 SFTP

# Follows your ssh_config
tc1c4446d = 遵循你的 ssh_config

# A real terminal
tae9bcfa8 = 真正的終端機

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = 前往 shellwing.app

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = 以 Docker Swarm 打造、可自行架設的應用程式平台：Heroku、Render 和 Coolify 的開放原始碼替代方案。

# 300+ one-click apps
t6a770b0a = 300+ 一鍵安裝應用程式

# Deploy on push
tb0bfc3e9 = 推送即部署

# Pull request previews
t4e4180fc = Pull Request 預覽

# Automatic TLS
t9e9776ed = 自動 TLS

# Live logs and metrics
tebe3a172 = 即時記錄與指標

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = 前往 hivepaas.com

# Always
tad4bbd04 = 一律

# Extensions, apart by spaces
tc709375f = 副檔名，以空格分隔

# Format of new archives
t3b50d238 = 新封存檔的格式

# New archive starts with it; you can pick another there.
t7b361b55 = 新增封存檔時預設使用此格式；你也可以在該處選擇其他格式。

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = 僅儲存會原樣放入 ZIP 和 7z 封存檔中的檔案。tar 格式使用「最低」，因為只有 .tar 不壓縮。

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = 等級越高，封存檔越小，建立時間越長，但開啟時間不變。

# Write straight to a storage above
ta4975128 = 直接寫入上方的儲存位置

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = 較大的檔案不會佔用這台電腦的空間，但若連線中斷就得重新壓縮。S3 以此方式支援 tar 和 ZIP，SFTP 則支援所有格式。

# Don’t compress what already is
t940ec31d = 不再壓縮已壓縮的檔案

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = ZIP 和 7z：圖片、影片、音訊和封存檔原樣放入，更快且不會變大

# What goes in as it is
tc0104b29 = 原樣放入的檔案

# Files whose names end in these, whatever their case
ta3acef9b = 名稱以這些結尾的檔案，不分大小寫

# Reset to default
t0db9f7dc = 還原預設值

# Leave out Mac-only files
t27eec14f = 排除 Mac 專用檔案

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store、__MACOSX 和資源分支：在 Mac 以外都是多餘的檔案

# Hide file names behind the password
t93ff7715 = 以密碼隱藏檔名

# For 7z: nobody sees what is inside without it
t0f5354d7 = 適用於 7z：沒有密碼就看不到裡面的內容

# No limit
t6de7f47d = 不限制

# Where the files go
t77e60aba = 檔案放置位置

# Where Extract starts: you can pick another there.
t4b73082f = 「解壓縮」的預設位置；你也可以在該處選擇其他位置。

# One item right here, more in a new folder
ta5a4d31d = 單一項目直接放在此處，多個項目放進新資料夾

# A new folder next to it
t2095cbe9 = 旁邊的新資料夾

# Named after the archive
tb991d4c1 = 以封存檔命名

# Next to the archive
t4691c928 = 封存檔旁邊

# Straight into its folder
t99d1df43 = 直接放進它自己的資料夾

# When a file is already there
tca514dda = 若檔案已存在

# A file of the same name in the folder the files go to
t296ad686 = 放置檔案的資料夾中已有同名檔案時

# Largest an archive may unpack to
t5957c426 = 封存檔解開後的大小上限

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = 防範封存檔炸彈：很小的檔案解開後卻會塞滿磁碟

# Only names every system can use
tf47f4968 = 僅接受所有系統都能使用的名稱

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = Windows 無法使用的名稱（如 a:b.txt）以及僅大小寫不同的名稱，在這台電腦上也會詢問

# Light
t8ad2f8cf = 淺色

# Dark
t45570065 = 深色

# System
t8f3184bc = 依系統

# Same as system ({language})
t5bd525d3 = 與系統相同（{ $language }）

# The system’s
t1d68c60b = 系統預設

# Custom…
ta391dade = 自訂…

# The system’s ({size} px)
ta51f1877 = 跟隨系統（{ $size } px）

# All ({count})
tf65ec196 = 全部 ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = 例如 code -n，或 /path/to/editor --flag

# Language
t9a73db9b = 語言

# Appearance
t31018b9f = 外觀

# Text size
t2c041143 = 文字大小

# Processor cores to use
t2f9c3d17 = 使用的處理器核心數

# Text editor
t1ca82aab = 文字編輯器

# What Edit opens a file of a storage in
t62506ced = 用來開啟儲存位置中檔案以進行編輯的程式

# The file’s path is added at the end.
teba749d4 = 檔案路徑會加在最後面。

# Paste your license key
t3c4da5cf = 貼上你的授權金鑰

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = 除了簽署檔案外，所有功能皆可使用，並可使用 { $count } 個網路儲存位置。這台電腦上的資料夾不受限制，且可驗證任何簽章。

# Every feature, and any number of storages.
tc3e75ecf = 所有功能，儲存位置數量不限。

# Every feature, and any number of storages, for your organization.
t69b29f66 = 所有功能，儲存位置數量不限，供你的組織使用。

# Plan
t1eab5e32 = 方案

# Issued to
t44ef5cad = 授權對象

# Who this license is for
td514fb0d = 此授權的持有者

# Seats
t8f706321 = 席位

# What your organization’s agreement covers
t60b0c6c9 = 你組織的合約所涵蓋的範圍

# Trial, until {date}
t9ffc2b63 = 試用，至 { $date }

# Until {date}
t4d9bac91 = 至 { $date }

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = 已於 { $date } 結束。仍可再使用 { $days } 天。

# Ended {date}
tfccb9305 = 已於 { $date } 結束

# Valid
td2661011 = 有效

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = 超過此期限後，packwing 會變回免費版。除了無法新增儲存位置外，不會失去任何功能。

# Your organization manages this license.
te42a0ee6 = 此授權由你的組織管理。

# Activate
t3b8f0190 = 啟用

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = 金鑰只在這台電腦上檢查，不會傳送到任何地方，授權也不會改變你組織政策的效果。

# On
t60e47df0 = 開啟

# They show when you right-click files, folders and archives.
te216d656 = 在你對檔案、資料夾和封存檔按右鍵時顯示。

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = packwing 目前不在選單中：加入選單後，這些項目才會出現。

# Changing the menu…
tc7f1a2f0 = 正在變更選單…

# Good to know
t8d7b3a90 = 注意事項

# The menu could not be changed: {why}
t9fac5c06 = 無法變更選單：{ $why }

# Show packwing in the right-click menu
t5d0d23cc = 在右鍵選單中顯示 packwing

# Extract and compress from any folder
t94a46acb = 可在任何資料夾中解壓縮與壓縮

# Items in the menu
t67537dc4 = 選單中的項目

# Put them in a submenu
te3449a65 = 放進子選單

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 會把它們放在一個 Packwing 項目下，無論在其選單中還是「顯示更多選項」中

# One packwing item, with these under it
tb5fdc584 = 只顯示一個 packwing 項目，這些項目在其下方

# Keep the “downloaded from the Internet” mark
tfe574fa2 = 保留「從網際網路下載」標記

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = 系統會在執行下載的封存檔中的檔案前發出警告

# This system has no such mark, so this changes nothing here
t314d60c1 = 此系統沒有這種標記，因此在這裡不會有任何變化

# License
t38e70ee4 = 授權

# Your organization’s, in Settings › License
tff7c594e = 你組織的授權，請見設定 › 授權

# Treat files from storages as from the Internet
t0af8d466 = 將儲存位置中的檔案視為來自網際網路

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = 來自 S3、WebDAV 和 SFTP 的檔案也會加上標記

# Require encryption for new archives
t0c3e1337 = 要求新封存檔必須加密

# Applies to archives made and files added here, not to other apps
tc889f3a2 = 適用於在此建立的封存檔和加入的檔案，不影響其他應用程式

# Kinds of encryption allowed
t1b8b2116 = 允許的加密方式

# New archives can be encrypted only these ways
t71dcd29c = 新封存檔只能使用這些方式加密

# Minimum password length
tb7d64a44 = 密碼長度下限

# In characters, for every new archive with a password
tca6d6982 = 以字元數計，適用於每個設有密碼的新封存檔

# packwing has no policy of this name.
t7e03649c = packwing 沒有這個名稱的政策。

# Ask before opening files without extracting
t76d5d951 = 不解壓縮直接開啟檔案前先詢問

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = 開啟封存檔中的檔案時，會在暫存資料夾中放一份副本。程式一律會先詢問。

# Paste a public key
t0bf72f73 = 貼上公鑰

# This is not a public key packwing can check with: {why}.
td57933b9 = 這不是 packwing 可用於驗證的公鑰：{ $why }。

# This line cannot be read, so the key is not used
t90e7dd7b = 無法讀取這一行，因此不會使用該金鑰

# Keys you trust
ted99ced8 = 你信任的金鑰

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = 這些金鑰之一所簽的章，代表簽署者的身分。其他任何金鑰所簽的章，無論是否相符，都不能證明什麼。

# Choose a file…
t28a576c9 = 選擇檔案…

# {name} (weak)
tdb06c037 = { $name }（弱）

# Checksums
t47dd48d3 = 檢查碼

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = 檔案選單中「檢查碼…」顯示的內容，用來與檔案來源處提供的檢查碼比較。弱檢查碼能發現損壞的檔案，但無法發現被刻意修改的檔案。

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = 無法讀取其政策，因此它所保護的項目一律拒絕：{ $why }

# Some settings are managed by your organization
t7db7e7ec = 部分設定由你的組織管理

# Get help
t4a1ed54a = 取得協助

# Not understood
t068eea1b = 無法理解

# Off
t0dc6b4aa = 關閉

# General
t5561fd0b = 一般

# Extracting
tc82af3f2 = 解壓縮

# Compressing
t6d183c73 = 壓縮

# Security
tc2fe21db = 安全性

# Right-click menu
t1287ae97 = 右鍵選單

# About
t58e096de = 關於

# Settings
t4b058728 = 設定

# This computer has no way to ask that it is you.
t0c1b4d43 = 這台電腦無法要求確認是你本人。

# open a storage
t780455bf = 開啟儲存位置

# Open a storage in packwing
tf861e014 = 在 packwing 中開啟儲存位置

# That was not it. It stays locked.
tbb2a9838 = 不正確，仍保持鎖定。

# The system could not ask: {why}
td6905afe = 系統無法發出要求：{ $why }

# This copy of packwing cannot open it.
tfb56d8cf = 這個版本的 packwing 無法開啟它。

# Storages
tc776c52f = 儲存位置

# Add storage
te858d6b5 = 新增儲存位置

# Type the password you log in to this computer with to open it.
tb544cd9b = 輸入你登入這台電腦的密碼即可開啟。

# Show this computer it is you to open it.
tfb4c635b = 向這台電腦確認是你本人即可開啟。

# Checking…
t87f62cb7 = 正在檢查…

# It asks again after 5 minutes left unused.
t69c22a3b = 閒置 5 分鐘後會再次詢問。

# Edit…
t8e7efcd7 = 編輯…

# Remove from this list
t8c7c7c57 = 從此清單移除

# Extract here with packwing
tc5e54e4a = 使用 packwing 解壓縮到此處

# Extract to new folder
tda62691f = 解壓縮到新資料夾

# Extract with packwing…
tda9255a0 = 使用 packwing 解壓縮…

# Open with packwing
teb657427 = 使用 packwing 開啟

# Compress to ZIP
t344b1175 = 壓縮為 ZIP

# Compress with packwing…
t9ec9b77d = 使用 packwing 壓縮…

# packwing cannot find its program: {e}
t1934ce08 = packwing 找不到它的程式：{ $e }

# Signed by {signer}
t8eb35b10 = 由 { $signer } 簽署

# Signed by {signer}, not checked yet
t88b7186a = 由 { $signer } 簽署，尚未驗證

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = 由你不信任的金鑰簽署：{ $fingerprint }

# It does not match its signature
tb7d0e1ed = 與其簽章不符

# Its signature cannot be read
tadf2a401 = 無法讀取其簽章

# Its signature could not be checked
t69cbbb51 = 無法驗證其簽章

# Checking its signature…
tda8c2e03 = 正在驗證其簽章…

# the system's package manager
t11465c20 = 系統的套件管理員

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = 正在下載… { $got } / { $total }

# Downloading… {got}
t4873b921 = 正在下載… { $got }

# just now
ta715ef0f = 剛剛

# {n} minute ago | {n} minutes ago
t6c7302d6 =
    { $n ->
       *[other] { $n } 分鐘前
    }

# {n} hour ago | {n} hours ago
t137b052e =
    { $n ->
       *[other] { $n } 小時前
    }

# {n} day ago | {n} days ago
tb9fa7744 =
    { $n ->
       *[other] { $n } 天前
    }

# No folder to download to.
t2bfc1370 = 沒有可下載到的資料夾。

# Update available
t676409df = 有可用更新

# Restart to update
tbbbbda26 = 重新啟動以更新

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
       *[other] 有 { $n } 個檔案因名稱已被使用而略過。
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
       *[other] 有 { $n } 個連結維持在原處。
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = 此檔案以密碼片語加密。

# This archive needs its password.
td46b69a1 = 此封存檔需要密碼。

# This file is encrypted to a key.
tbd048b4e = 此檔案以金鑰加密。

# This key file needs its passphrase.
t81f4c775 = 此金鑰檔案需要密碼片語。

# {paths} archives
t27f3b860 = { $paths } 個封存檔

# {folder}, a folder for each
t75e7db7c = { $folder }，各自放進一個資料夾

# New
t8b242db1 = 新增

# There is no file to save it beside.
ta9906e3b = 沒有可在其旁邊儲存的檔案。

# Nothing in the archive matches the patterns given.
t90b19482 = 封存檔中沒有符合所給模式的內容。

# Nothing to compress.
t47339885 = 沒有可壓縮的內容。

# {path} is too big.
te9ab879e = { $path } 太大。

# This format holds one file, and no more.
t75c197d9 = 這種格式只能存放一個檔案。

# Store
t6da32dce = 僅儲存

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
tad0120eb = 無法讀取 { $path }：{ $e }

# {path} has no name.
tbd0e16dc = { $path } 沒有名稱。

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = { $path } 的名稱不是有效的 Unicode，因此無法放進封存檔。

# Could not write {path}: {e}
t8dd055c6 = 無法寫入 { $path }：{ $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } 及其後的名稱都已被使用。

# Could not encrypt: {e}
t485d71ab = 無法加密：{ $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = { $level } 約需 { $needed } 記憶體，目前可用 { $available }：將以約需 { $less } 的 { $lower } 建立。

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = 此層級約需 { $needed } 記憶體，目前可用 { $available }：電腦可能變慢，或 packwing 停止。

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = ZIP 或 7z 封存檔只有在由 age 整體加密時才能使用 age 金鑰。

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age 加密只能使用密碼片語或金鑰，不能兩者並用。

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = 後量子金鑰無法與其他金鑰並用：其他金鑰會讓檔案暴露於量子電腦的威脅之下。

# There are no keys in this file to encrypt to.
tc2809b2c = 這個檔案中沒有可用於加密的金鑰。

# {path} got shorter while it was being read.
t2e221d1f = { $path } 在讀取時變小了。

# A password has to be valid Unicode.
t5ef729bc = 密碼必須是有效的 Unicode。

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = 無法讀取 { $path } 的延伸屬性：{ $e }

# Could not start compressing: {e}
tc4faa70c = 無法開始壓縮：{ $e }

# Could not compress: {e}
tf6ff23f2 = 無法壓縮：{ $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } 是連結，只有 Unix 才會保留連結。

# Nothing changed.
tedfc294c = 沒有任何變更。

# The archive did not come out whole.
t2f015c79 = 封存檔未能完整產生。

# packwing can change only a ZIP archive that is one file.
t48628036 = packwing 只能變更單一檔案的 ZIP 封存檔。

# That is not in the archive.
t7530c419 = 封存檔中沒有這個項目。

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = 這已是封存檔中的所有內容。請改為刪除封存檔本身。

# “{to}” cannot be a name.
td53e55bf = 「{ $to }」不能作為名稱。

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = 「{ $to }」已存在於該資料夾中。未做任何變更。

# packwing can add files only to a ZIP archive that is one file.
t11a330d3 = packwing 只能將檔案加入單一檔案的 ZIP 封存檔。

# An archive can’t go inside itself.
tff43ce74 = 封存檔不能放進它自己裡面。

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = 「{ $bare }」已在封存檔中。未做任何變更。

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = 上傳前，這個封存檔會先在 { $folder } 中建立，可能需要多達 { $need }，但該處只剩 { $free } 可用。

# The upload stopped.
t528232e6 = 上傳已停止。

# “{pattern}” names no file or folder.
te17b2bfa = 「{ $pattern }」沒有指向任何檔案或資料夾。

# a colon, which Windows does not allow
t3ba6a1aa = 含有 Windows 不允許的冒號

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = 含有 Windows 不允許的字元 (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = 是 Windows 為裝置保留的名稱

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = 以空格或句點結尾，Windows 不允許

# longer than 255 bytes
t9218e5c0 = 長於 255 位元組

# a control character
t378bba3a = 含有控制字元

# not a name this system can read
t72b79e1d = 不是本系統能讀取的名稱

# the same file here as another, its name differing only in case
t1eead41f = 在這裡與另一個檔案相同，名稱僅大小寫不同

# a link that leads outside the folder
t00fdc25e = 指向資料夾外的連結

# {name} is already in the folder.
tbe7fab8a = { $name } 已在資料夾中。

# Could not make a folder in {parent}.
tec7ca3a9 = 無法在 { $parent } 中建立資料夾。

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } 是資料夾，而檔案會取代它。

# {name} is already in the folder
t5af584a0 = { $name } 已在資料夾中

# This file is not an archive packwing can open.
t660f149b = 這個檔案不是 packwing 可開啟的封存檔。

# The password is wrong, or the archive is damaged.
ta4d3227d = 密碼錯誤，或封存檔已損毀。

# The passphrase is wrong.
t7826c649 = 密碼片語錯誤。

# The key file’s passphrase is wrong.
t8ad75e3a = 金鑰檔案的密碼片語錯誤。

# This file is encrypted to a key packwing does not have.
te162af76 = 這個檔案是以 packwing 沒有的金鑰加密的。

# Cancelled.
t27c010d6 = 已取消。

# the archive could not be read
t2267c3b1 = 無法讀取封存檔

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = 無法解壓縮，因為有檔案的名稱在此無法使用：「{ $name }」（{ $why }）。

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
       *[other] { $n } 位元組
    }

# A volume of this archive is not there.
t3803103c = 缺少這個封存檔的某個分卷。

# {named}: its data is damaged.
t066227f9 = { $named }：其資料已損毀。

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = 封存檔已損毀：UnRAR 會以代碼 { $number } 結束。

# There is no public key here.
t9fd24078 = 這裡沒有公鑰。

# Could not make the key: {e}
t73610ecc = 無法建立金鑰：{ $e }

# There is no file to sign.
tc95e3a9c = 沒有可簽署的檔案。

# {name} is not there any more.
t07c1f2bc = { $name } 已不在原處。

# A part has to hold at least one byte.
te779b111 = 每個分割檔至少要有一個位元組。

# {path} is not a file.
tdfee38b2 = { $path } 不是檔案。

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = 這樣分出的分割檔會多到無法編號：請把分割檔設大一些。

# {part} is there already.
t2577cbb6 = { $part } 已經存在。

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path } 不是以 7-Zip 方式分割的檔案的分割檔（如 name.001）。

# {part} is missing: the parts start from it.
t5e65a10a = 缺少 { $part }：分割檔從它開始。

# {part} is missing, and the parts after it are there.
t24165c89 = 缺少 { $part }，而它之後的分割檔都在。

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = { $part } 有 { $got }，不是第一個分割檔的 { $size }：它可能不完整。

# The name is empty.
t1a9d8e4e = 名稱是空的。

# That name is kept for the system.
t74c8683a = 該名稱為系統保留。

# A name cannot hold / or other special characters.
t4c0596ec = 名稱不能包含 / 或其他特殊字元。

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = 在 Windows 上，名稱不能以句點或空格結尾。

# That name is taken in this folder.
t43a2bce1 = 此資料夾中已有此名稱。

# {name} is already there.
tb5e7b134 = { $name } 已存在。

# {name} cannot go inside itself.
t99f437a6 = { $name } 不能放進它自己裡面。

# Stopped.
t15558940 = 已停止。

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } 在一處是檔案，在另一處卻是資料夾。

# The copy of {name} does not match. The original was kept.
t779a64f3 = { $name } 的副本與原檔不符。已保留原始檔案。

# This is not a valid packwing license key.
td5ae978f = 這不是有效的 packwing 授權金鑰。

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = 無法讀取你組織對此的政策，因此 packwing 不會執行。請洽詢你的 IT 部門。

# Your organization requires new archives to be encrypted.
t83acf110 = 你的組織要求新封存檔必須加密。

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = 你的組織不允許這種加密方式：{ $method }。

# Your organization requires a password of at least {least} characters.
t2d21b125 = 你的組織要求密碼至少 { $least } 個字元。

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = 這個密碼太容易被猜到，而你的組織要求更強的密碼。

# Expected true or false, not {other}.
tcadcff7a = 應為 true 或 false，而不是 { $other }。

# Expected a number from 1 to {most}, not {other}.
tda498fad = 應為 1 到 { $most } 的數字，而不是 { $other }。

# Expected some text, not {other}.
t9386ce3b = 應為文字，而不是 { $other }。

# Expected an address starting with https://, not {other}.
t93bfd19e = 應為以 https:// 開頭的位址，而不是 { $other }。

# Expected a list, not {other}.
tc945cc93 = 應為清單，而不是 { $other }。

# No kind of encryption is called {name}.
tfddb7ba6 = 沒有名為 { $name } 的加密方式。

# The list names no kind of encryption.
t5f8076f5 = 清單中沒有列出任何加密方式。

# The file is too big to be a list of policies.
taf68e582 = 檔案太大，不可能是政策清單。

# The file has to hold one JSON object.
tf483aa65 = 檔案必須包含一個 JSON 物件。

# {path} has to belong to root.
tfc77ea2b = { $path } 必須屬於 root。

# {path} can be changed by others than root.
t0d4a4498 = { $path } 可被 root 以外的人變更。

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = 此儲存位置目前只接受 { $size } 以內的檔案，因為每個檔案在傳送時都會保存在記憶體中。

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = 此儲存位置無法直接重新命名資料夾，因為其下所有內容都得複製。請使用「移動到…」。

# S3 bucket
tc5e013b3 = S3 儲存貯體

# Bucket
te34e3557 = 儲存貯體

# Region
t492e789d = 區域

# Endpoint
t27e7d69c = 端點

# Empty for AWS, or https://…
td3a94af8 = AWS 請留空，或填入 https://…

# Start in
t5eb91ea6 = 起始位置

# A prefix, if not the whole bucket
t75255a6c = 前置詞，若不是整個儲存貯體

# Access key ID
tf9408d75 = 存取金鑰 ID

# Secret access key
te1c3d3a2 = 私密存取金鑰

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = 主機或 ssh_config 名稱

# Empty for the home folder
t67b5f42b = 留空則使用家目錄

# Empty to use the SSH agent and keys
tf10e6293 = 留空則使用 SSH 代理程式和金鑰

# WebDAV folder
tb2401428 = WebDAV 資料夾

# Address
t700560f3 = 位址

# User
te0a63e12 = 使用者

# Password or app token
t1c18cb9b = 密碼或應用程式權杖
