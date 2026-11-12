# packwing in simplified Chinese. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = 外观：{ $name }。点击切换为{ $name2 }。

# Stop and say so
tdc8b782b = 停止并提示

# Replace it
t1e17fc0c = 替换

# Skip it
tad9909cb = 跳过

# Add
t9dc3aa14 = 添加

# The top of {name}
t369f9d4d = { $name } 的顶层

# Choose what to add
t48b969a8 = 选择要添加的内容

# {count} item | {count} items
t534e422e =
    { $n ->
       *[other] { $count } 项
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
       *[other] { $count } 个文件
    }

# Add to {name}
t0d53ac5e = 添加到 { $name }

# Goes in
t36e95c22 = 放入位置

# If a file of that name is there
t34c69cec = 如果已有同名文件

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = 此压缩包已签名。加入更多内容后，它将与签名不再匹配，需重新签名。

# The new files are encrypted with the archive’s password.
t52b1d223 = 新文件将使用压缩包的密码加密。

# Cancel
t35afca3b = 取消

# Name
t0fe07306 = 名称

# Folder
t06357afd = 文件夹

# Its secrets could not be read: {why}
t2af0c8ab = 无法读取其机密信息：{ $why }

# That folder is not on this computer.
ta4073fff = 此电脑上没有该文件夹。

# Its secrets could not be kept: {why}
ta213827f = 无法保存其机密信息：{ $why }

# Local folder
te075bcbc = 本地文件夹

# Trying it…
t214cdad5 = 正在测试…

# It answers.
t3bd5bc9f = 有响应。

# It did not answer: {why}
t0ac8fa44 = 无响应：{ $why }

# Edit storage
t7bae68d0 = 编辑存储

# Add a storage
t4fe8f806 = 添加存储

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = 机密信息保存在系统钥匙串中，而不是与设置放在一起。

# Its files stay where they are.
t4e4acf9a = 其中的文件保留在原处。

# Ask for this computer’s password to open it
t7a7d61e9 = 打开前要求输入此电脑的密码

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = 适用于离开时未锁屏的电脑：需用解锁此电脑的方式才能打开，闲置 5 分钟后会再次询问。

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing 免费版只能使用一个网络存储。Pro 或 Enterprise 许可证可取消此限制：设置 › 许可证。此电脑上的文件夹不受限制。

# Test connection
te0f3b6b7 = 测试连接

# Enter a license key
ta24f28a2 = 输入许可证密钥

# Save
t4d2d5d68 = 保存

# Paste the checksum given with the file
t8333758c = 粘贴随文件提供的校验和

# They could not be saved: {why}
tb45ae457 = 无法保存：{ $why }

# The file could not be read: {why}
t5daa8455 = 无法读取文件：{ $why }

# Reading…
t6aad8485 = 正在读取…

# Copied
t8dc21305 = 已复制

# Copy
t658f3664 = 复制

# weak
tedbc2ec9 = 弱

# There is no checksum in this.
t231f838a = 这里面没有校验和。

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = 这不是 packwing 生成的校验和：它有 { $digits } 位。

# It matches the file’s {name} checksum.
t4f0350e4 = 与文件的 { $name } 校验和一致。

# It is compared once the file is read.
tcb5c1745 = 文件读取完成后进行比较。

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = 与文件的 { $name } 校验和不一致：这不是生成该校验和的文件，或者文件在传输途中发生了变化。

# It may be a {name} checksum, which is not made yet.
t3d713e04 = 这可能是尚未生成的 { $name } 校验和。

# Make it
tcbefe458 = 生成

# Compare with
tbddcfc14 = 比较对象

# Reading… {size} of {size2}
t80036d1c = 正在读取… { $size } / { $size2 }

# Reading… {size}
t1569c388 = 正在读取… { $size }

# Checksums of {name}
te88f3ed9 = { $name } 的校验和

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = 整个文件都会经过此电脑，读取数据可能会产生服务商收费。

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = 弱校验和（{ $names }）能发现损坏的文件，但发现不了被故意修改以使其一致的文件。

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = 已保存在旁边，文件名为 { $files }；sha256sum -c 等工具无需 packwing 即可校验。

# Compare…
tec85be80 = 比较…

# Saving…
tbd79b37d = 正在保存…

# Save beside it
t7f4d85e3 = 保存在旁边

# Choose the checksums shown
teec532fe = 选择显示的校验和

# Read it
t80fb8482 = 读取

# Done
t8dd31791 = 完成

# Nothing chosen yet
tcf390b0a = 尚未选择

# {join} and {more} more
t8399ef70 = { $join } 等另外 { $more } 项

# Hide what goes in
t5a420780 = 隐藏放入的内容

# Show what goes in
t9a20eb53 = 显示放入的内容

# Take it out
t90a93ec7 = 移除

# Drop them to add them
t6a4aedbe = 拖放到此处即可添加

# What goes in
t3b7c797a = 放入的内容

# + Files
tba8a1e71 = + 文件

# + Folder
tcef8b0ce = + 文件夹

# Add the files or a folder to put in the archive.
t2e5c8c3e = 添加要放入压缩包的文件或文件夹。

# Don’t split
t56ee6829 = 不分卷

# 25 MB (email)
tc6ba1c9c = 25 MB（电子邮件）

# 4 GB (USB stick)
tfc6b3ea5 = 4 GB（U 盘）

# Custom size…
t0fcf9e2d = 自定义大小…

# Size
ta6478e7c = 大小

# Password
t2cc30838 = 密码

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…、age1pq1… 或 ssh-ed25519 …

# Save here
ta1f147a6 = 保存到这里

# Use these keys
t1a01e439 = 使用这些密钥

# No compression
t2e9c9e78 = 不压缩

# {name} compression
te2db10f6 = { $name } 压缩

# Several frames
t84cc41f7 = 多帧

# One frame
tc6ef759e = 单帧

# Small blocks
t04032e8a = 小块

# Parts of 25 MB
ta999efe2 = 每卷 25 MB

# Parts of 4 GB
t760df467 = 每卷 4 GB

# Parts of {size}
tdd286479 = 每卷 { $size }

# Parts of a size to type
tafd4b17d = 每卷大小自行输入

# Public key
t9981f45f = 公钥

# Owners kept
t63e8a0df = 保留所有者

# Attributes kept
te9e8289c = 保留属性

# To keep owners and extended attributes for a backup, pick a .tar format.
tcf2585da = 要在备份中保留所有者和扩展属性，请选择 .tar 格式。

# Keep for a backup
t6a2bcc1a = 为备份保留

# Owners: user and group
t5350e3fe = 所有者：用户和组

# Extended attributes and ACLs
t35aff4d1 = 扩展属性和 ACL

# For a backup to restore on this computer. Whoever opens the archive sees the names of the users.
ta47cc163 = 用于在这台电脑上恢复的备份。打开归档的人都能看到用户名。

# Beside the files
t757989af = 与文件放在一起

# What it holds, from
teae4ef1a = 内容来自

# Save in
tf2d34837 = 保存到

# Folder…
tc8abcd43 = 文件夹…

# Storage…
td0d8c806 = 存储…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = 分卷会先在此电脑的临时文件夹中生成，然后再上传，因此需要有足够的空间。

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext } 会先在此电脑的临时文件夹中生成，然后再上传，因此需要有足够的空间。.zip 和 tar 格式会直接写入此存储。

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = 它会先在此电脑的临时文件夹中生成，然后再上传，因此需要有足够的空间。

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place } 接受最大 { $most } 的文件。比这更大的压缩包会在开始上传时失败。

# Choose a storage, and a folder in it
td7627e48 = 请选择存储及其中的文件夹

# New folder
t1926d127 = 新建文件夹

# Give it a name, without / or \ in it.
ta598b15b = 请起个名字，不要包含 / 或 \。

# A file of that name is there already: this one will be {name}.
t9451c3b1 = 已有同名文件：此文件将命名为 { $name }。

# Some names can’t be used on Windows or macOS, such as “{name}”. There, those files may not extract as they are.
t0da27ae4 = 有些名称在 Windows 或 macOS 上无法使用，例如“{ $name }”。在那里，这些文件可能无法按原样解压。

# Hide advanced
tae2a96e9 = 隐藏高级选项

# Advanced options
t701d5ed9 = 高级选项

# Large blocks
tdc489452 = 大块

# Compression level
td386c9ef = 压缩级别

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = 仅存储会原样放入文件：创建最快，体积最大。

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = 级别越高，压缩包越小，但生成时间更长，打开时间不变。

# Frames
tf03e266f = 帧

# Several frames open on every core, and make the archive a little larger: under half a percent.
tf461cde7 = 多帧可在所有核心上打开，压缩包会稍大一些：不到百分之零点五。

# One frame makes the smallest archive, which opens on one core.
ta097b317 = 单帧生成最小的压缩包，只在一个核心上打开。

# Blocks
t036a71e3 = 块

# Small blocks keep every core at work: up to twice as fast, and the archive a few percent larger.
t32239aa8 = 小块让每个核心都在工作：最多快两倍，压缩包大几个百分点。

# Large blocks make the smallest archive, on fewer cores where there is less to compress.
t66a490d9 = 大块生成最小的压缩包，要压缩的内容较少时，用到的核心较少。

# Split into parts
tca4436bd = 分卷

# Each part
t642c444d = 每卷

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = 分卷可用 7-Zip 及类似的压缩软件打开，Windows 和 macOS 自带的工具无法打开。

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = 每卷大小至少为 1 MB，例如 100 MB。

# Encrypt with age
t22784ad9 = 使用 age 加密

# Protect with a password
tae8d8740 = 使用密码保护

# Suggest
ta6e44ce5 = 建议

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = 请妥善保管：没有它，此压缩包将永远无法打开。

# Type a password, or turn this off.
t62b19318 = 请输入密码，或关闭此项。

# Hide file names too
t2198d77b = 同时隐藏文件名

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = 设置密码后，Windows 和 macOS 内置工具无法打开此 ZIP，收件人需要压缩软件。文件名也仍然可见；选择 .7z 可隐藏文件名。

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = 已保存为 { $name }。其中的文件名也已加密。Windows 和 macOS 可能需要 age 工具或压缩软件才能打开。

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
       *[other] 来自 { $name } 的 { $n } 个密钥
    }

# Remove
t21a5901d = 移除

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
       *[other] 其中有 { $n } 个密钥已被忽略，因为 age 不支持加密到这类密钥。
    }

# From a file…
t40d7c944 = 从文件…

# Type a public key, or choose a file of them.
tcb028b06 = 请输入公钥，或选择包含公钥的文件。

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = 这是后量子密钥：要打开它，age 工具必须是 1.3 或更高版本。

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = 只有与之配对的私钥才能打开，私钥一旦丢失，任何方式都无法打开。

# Weak
t6de87ae9 = 弱

# Fair
t82fa0d53 = 一般

# Strong
t0dad409a = 强

# Choose what to put in it
t5461cfa6 = 选择要放入的内容

# Convert
t92e29218 = 转换

# New archive
tf352befb = 新建压缩包

# Create
t990de47d = 创建

# {n} item | {n} items
tfff7fd6c =
    { $n ->
       *[other] { $n } 项
    }

# Move {what} to the Trash?
tdf5a8d53 = 将 { $what } 移到废纸篓？

# Delete {what}?
ta297d6c5 = 删除 { $what }？

# It can be put back from the system’s Trash.
te5e137a1 = 可从系统废纸篓中恢复。

# This cannot be undone from packwing.
t3947dab4 = 此操作无法在 packwing 中撤销。

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
       *[other] 另有 { $n } 项
    }

# Move to Trash
ta036661b = 移到废纸篓

# Delete
t5797ea6a = 删除

# {many} items
t49912bd4 = { $many } 项

# Rename {what}
t83401013 = 重命名 { $what }

# New name
taf4cad88 = 新名称

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = 它将从压缩包中永久删除，文件夹及其中所有内容也一并删除。此操作无法撤销。

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = 此压缩包已签名。此更改后它将与签名不再匹配，需重新签名。

# Rename
tc80631f3 = 重命名

# The archive’s password
t8a168799 = 压缩包的密码

# one folder
td571ce35 = 一个文件夹

# one file
t95c9ec7b = 一个文件

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }，因此就在这里

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }，因此放入新文件夹

# Extract here
t354232ec = 解压到此处

# {picked} selected
t9addc37e = 已选择 { $picked }

# All {counted}
t649feaf8 = 全部 { $counted }

# What should it extract?
td12e23bf = 要解压哪些内容？

# Smart
t54535972 = 智能

# Recommended
t7865024c = 推荐

# Where should the files go?
td35f540c = 文件要放到哪里？

# Right here
t900a8a39 = 就在这里

# Another folder
t9f128ba8 = 其他文件夹

# Choose…
t58daae9c = 选择…

# Goes to {destination}
t8195aed2 = 放到 { $destination }

# Checked as it goes
tb6c9570b = 边解压边检查

# · Nothing leaves the folder or passes the size limit
t11424797 = · 没有文件会超出该文件夹或超过大小限制

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name } 来自互联网。它将保持标记，因此运行前会先询问你。

# Extract archive
t834ba3c0 = 解压压缩包

# Extract
t30518e1e = 解压

# Open {name}?
tcfe84c2f = 打开 { $name }？

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing 会在临时文件夹中放一份副本并打开它。对它的修改不会写回压缩包。

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = 这是一个程序：打开即会运行。仅当你信任压缩包来源时才这样做。

# Don’t ask again
t00c5d168 = 不再询问

# Run it
t72430853 = 运行

# Open
t538b10e9 = 打开

# Open from a storage
t45a8991d = 从存储打开

# Choose a storage, and an archive in it
t3c900950 = 请选择存储及其中的压缩包

# Choose
t73c66160 = 选择

# This is not a key file packwing can use: {why}.
tcf6986a7 = 这不是 packwing 可用的密钥文件：{ $why }。

# Could not read {path}: {e}.
t1dc6961f = 无法读取 { $path }：{ $e }。

# The passphrase
t1e921d00 = 密码短语

# The key file’s passphrase
t4051f4d7 = 密钥文件的密码短语

# Enter the passphrase
tbd0672da = 输入密码短语

# Passphrase
t191a2dc5 = 密码短语

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = 此文件已用 age 加密。packwing 仅在压缩包打开期间保留密码短语，且从不将其写下来。

# Enter the password
t513ea7ab = 输入密码

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = 此压缩包已加密。packwing 仅在压缩包打开期间保留其密码，且从不将其写下来。

# Choose the key file
t3c521a4e = 选择密钥文件

# Key file
t6ed7c612 = 密钥文件

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = 此文件已用 age 加密到某个密钥：age 密钥文件或 SSH 密钥。packwing 仅在压缩包打开期间保留从中读取的内容，且从不将其写下来。

# Enter the key file’s passphrase
t808b85c5 = 输入密钥文件的密码短语

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = 密钥文件本身受到保护。packwing 仅在压缩包打开期间保留其密码短语，且从不将其写下来。

# Version control
t0b7280ee = 版本控制

# Packages and builds
t237b373e = 依赖包与构建产物

# System files
te8be6b83 = 系统文件

# Temporary files
td4fe1263 = 临时文件

# Logs
t51a96da6 = 日志

# Pictures
t0d70feae = 图片

# Documents
t5a036285 = 文档

# Videos
t63609fed = 视频

# Music
t584a1d74 = 音乐

# node_modules/, *.log, src/**/gen/
t56c5448d = node_modules/, *.log, src/**/gen/

# They leave nothing out.
t999b68fd = 不排除任何内容。

# They leave out {n} file, {size}. | They leave out {n} files, {size}.
tc709cfda =
    { $n ->
       *[other] 将排除 { $n } 个文件，共 { $size }。
    }

# Clear
t04a57fc2 = 清除

# Leave out
ta50cc084 = 跳过

# Take only
t2fd02f84 = 仅包含

# Add the usual
t17ad93a1 = 添加常用项

# Only what these name goes in, and the folders it is in.
ta0bd7335 = 只有匹配的内容及其所在文件夹会放入。

# A name is anywhere; a / at its end names a folder; ** is any folders.
t524d506a = 单独的名称在任何位置都匹配；末尾的 / 表示文件夹；** 表示任意层文件夹。

# Filter
tf4a9c097 = 筛选

# kept, {size}
t001f6992 = 已保留，{ $size }

# left out
t73aeedd4 = 已跳过

# renamed {to}
t7aa9b778 = 重命名为 { $to }

# It was gone by the time it was read.
t93fb1e03 = 读取时该文件已不存在。

# The job stopped before it was done.
t57a78885 = 任务在完成前停止了。

# Waiting for other jobs…
t16ec36d9 = 正在等待其他任务…

# A file is damaged
t8b1230ed = 有文件已损坏

# Some names can’t be used here
t26835ec8 = 有些名称在这里无法使用

# Converted, but some files are damaged
t54ef3472 = 已转换，但有些文件已损坏

# Extracted, but some files are damaged
t8226d624 = 已解压，但有些文件已损坏

# Converted, with some files left out or renamed
t5cce0e50 = 已转换，部分文件被跳过或重命名

# Extracted, with some files left out or renamed
t5dc002d6 = 已解压，部分文件被跳过或重命名

# Extracting…
t94279f82 = 正在解压…

# Uploading…
tc75fe478 = 正在上传…

# Creating…
t01fcd896 = 正在创建…

# Adding…
t2060c750 = 正在添加…

# Changing the archive…
t186fbef5 = 正在修改压缩包…

# Opening…
tc40f2321 = 正在打开…

# Testing…
t19d1cb65 = 正在测试…

# Copying…
t02dd17e0 = 正在复制…

# Moving…
t890f7ed1 = 正在移动…

# Signing…
t7b8f8780 = 正在签名…

# Splitting…
t5452adf5 = 正在切分…

# Joining…
taa884c51 = 正在合并…

# Converting…
ta1b8f4cc = 正在转换…

# Couldn’t split the file
t83d27d26 = 无法切分文件

# Couldn’t join the parts
t1cf67e00 = 无法合并分卷

# Couldn’t convert the archive
t39f973a9 = 无法转换归档

# Couldn’t sign
t4489f97c = 无法签名

# Couldn’t copy
t613e4e9c = 无法复制

# Couldn’t move
t9796adec = 无法移动

# Couldn’t extract
tacd22c56 = 无法解压

# Couldn’t create the archive
t9d0e01bc = 无法创建压缩包

# Couldn’t add the files
tfbd989b0 = 无法添加文件

# Couldn’t change the archive
tcdc4a49c = 无法修改压缩包

# Couldn’t open the file
t041198a4 = 无法打开文件

# The test failed
tfcd070a5 = 测试未通过

# Starts when one of the others ends
tbe2a0509 = 其他任务之一结束后开始

# Paused
tde281feb = 已暂停

# Starting…
t73727cef = 正在开始…

# Finishing…
t042f47fe = 正在完成…

# Almost done
ta550a6a9 = 即将完成

# About {secs} seconds left
t3045da16 = 约剩 { $secs } 秒

# About a minute left
tf727815c = 约剩 1 分钟

# About {div_ceil} minutes left
tbd7a3305 = 约剩 { $div_ceil } 分钟

# {name} · checking that every file reads
t0d56ec7c = { $name } · 正在检查每个文件是否可读

# and {count} more | and {count} more
te728628b =
    { $n ->
       *[other] 还有 { $count } 个
    }

# The test passed
tdf0ce240 = 测试通过

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
       *[other] 已读取 { $count } 个文件，共 { $size }，没有错误。
    }

# Close
tcd86acc3 = 关闭

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = “{ $path }”已损坏：{ $why }。要停止、跳过它，还是保留已读出的部分？

# Do the same for the other damaged files
t798693ad = 对其他损坏的文件也这样处理

# Stop
t4b7f7705 = 停止

# Leave it out
taaff8ec9 = 跳过

# Keep what was read
tc4387d25 = 保留已读出的部分

# {count} file can’t be extracted under its name here. Nothing is written yet: stop, leave it out, or rename it? | {count} files can’t be extracted under their names here. Nothing is written yet: stop, leave them out, or rename them?
tb75acfb1 =
    { $n ->
       *[other] { $count } 个文件无法以其名称在这里解压。尚未写入任何内容：停止、跳过还是重命名？
    }

# {count} link leads outside the folder, elsewhere on this computer. Nothing is written yet: leave it out, or create it as it is if you trust this archive? | {count} links lead outside the folder, elsewhere on this computer. Nothing is written yet: leave them out, or create them as they are if you trust this archive?
t0eeaf0b8 =
    { $n ->
       *[other] { $count } 个链接指向文件夹之外、这台电脑上的其他位置。尚未写入任何内容：跳过，还是在你信任此压缩包时按原样创建？
    }

# Create them
t9a4d6e7d = 创建

# {count} file’s data was damaged. | {count} files’ data was damaged.
t85469b0f =
    { $n ->
       *[other] { $count } 个文件的数据已损坏。
    }

# {count} file was left out. | {count} files were left out.
t2f0042b8 =
    { $n ->
       *[other] 已跳过 { $count } 个文件。
    }

# {count} file was renamed, as its name can’t be used here. | {count} files were renamed, as their names can’t be used here.
tec751fa9 =
    { $n ->
       *[other] { $count } 个文件已重命名，因为其名称在这里无法使用。
    }

# {count} file was gone by the time it was read, and left out. | {count} files were gone by the time they were read, and left out.
ta9bd0a7d =
    { $n ->
       *[other] 有 { $count } 个文件在读取时已不存在，已略过。
    }

# The rest is in the archive.
t977a5572 = 其余内容已放入压缩包。

# The rest was extracted.
ta7833fcb = 其余已解压。

# Copy the list
t3c49b269 = 复制列表

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = { $count } / { $count2 } 个文件 · { $size } / { $size2 }

# Open the folder when it’s done
tf0f1017b = 完成后打开文件夹

# Run in background
tb800c0ef = 在后台运行

# Resume
t454d1d8a = 继续

# Pause
t44f9bf2d = 暂停

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
       *[other] 还有 { $n } 个任务正在运行
    }

# Closing packwing stops them
t2e6491b3 = 关闭 packwing 会停止它们

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = 未完成的部分将保持未完成：正在创建的归档会被删除，解压会停在当前进度。

# Keep running
t79c1b9c7 = 继续运行

# Stop them and quit
tf2707857 = 停止并退出

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = 其中的文件保持原样。packwing 用于登录它的凭据将从系统钥匙串中移除。

# The folder and its files stay where they are.
t4850aaa8 = 该文件夹及其中的文件保持原样。

# Remove “{name}” from the list?
t5dd9cfcf = 要将“{ $name }”从列表中移除吗？

# It can be added again at any time.
t80d51a99 = 可以随时重新添加。

# A passphrase for the new key
tf5593f05 = 新密钥的密码短语

# The same passphrase again
tad5c08e6 = 再次输入密码短语

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = 签名需要 Pro 或 Enterprise 许可证：设置 › 许可证。所有套餐均可免费验证签名。

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = 这是被切分为多个部分的文件中的一部分。一个签名只对应一个文件，却看似涵盖全部，因此 packwing 不对分卷签名。

# That passphrase does not open this key.
tc5357c97 = 该密码短语无法打开此密钥。

# This is not a key packwing can sign with: {why}.
t662c29f9 = 这不是 packwing 可用于签名的密钥：{ $why }。

# The two passphrases are not the same.
t335fe90b = 两次输入的密码短语不一致。

# Keep the key here
t9306dafe = 将密钥保存在此处

# No key chosen
te58c13f5 = 未选择密钥

# Key
tcd1ac90c = 密钥

# New key…
t9e1134da = 新建密钥…

# Again
t58fb56ef = 再次输入

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = 新密钥同时包含 Ed25519 和 ML-DSA-65，因此即使其中一种被攻破，签名依然有效。只有 packwing 和 hivepack 能验证这类签名：若要使用 ssh-keygen，请用 SSH Ed25519 密钥签名。密钥的密码短语一旦丢失，就无法找回。

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = 签名将以 { $signature } 放在它旁边，并替换已有的签名。整个文件都会经过此电脑，读取数据可能会产生服务商收费。

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = 签名将以 { $signature } 放在它旁边，并替换已有的签名。

# Sign {name}
ted250f31 = 为 { $name } 签名

# Anyone can check it, with the key’s public half
t04ecc8be = 任何人都可以用密钥的公钥部分进行验证

# Make the key…
t6bba28d5 = 生成密钥…

# Sign
t8f6340c4 = 签名

# {count} parts beside it, {first} and on. Join puts them back together, and packwing and 7-Zip open an archive cut so as it is.
tab12cc19 = 在旁边生成 { $count } 个分卷，从 { $first } 开始。“合并”可把它们拼回去；packwing 和 7-Zip 可直接打开这样切分的归档。

# Split
t16a15383 = 切分

# Move {what}
tc168ca56 = 移动 { $what }

# Copy {what}
teda01f26 = 复制 { $what }

# Move here
t1aaf57aa = 移到此处

# Copy here
t85f0245a = 复制到此处

# Move
t7f2c2a34 = 移动

# If a name is taken
tc99c037f = 如果名称已被占用

# They are in this folder already. Choose another.
t5318c14e = 它们已在此文件夹中。请选择其他文件夹。

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = 所有内容都会经过此电脑，读取数据可能会产生服务商收费。每个原件会在其副本被读回并确认一致后删除。

# Each original goes once its copy has been read back and matches.
t8ecbab4e = 每个原件会在其副本被读回并确认一致后删除。

# The originals stay where they are.
t5937f050 = 原件保留在原处。

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = 另一个任务正在那里创建 { $name }：请为此任务换个名称，或等它结束。

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = 另一个任务正在解压到 { $folder }：请选择其他文件夹，或等它结束。

# no editor command
tbf2a5d3a = 未设置编辑器命令

# no text editor found
t6256b3da = 未找到文本编辑器

# Search this folder
t2c814039 = 搜索此文件夹

# This folder could not be read: {why}
t2c6017b2 = 无法读取此文件夹：{ $why }

# Could not load more: {why}
t466e87c4 = 无法加载更多：{ $why }

# Could not delete: {why}
te5951d38 = 无法删除：{ $why }

# the text editor
t91cdab0c = 文本编辑器

# Cannot start {who}: {why}
t8deb160a = 无法启动 { $who }：{ $why }

# Edit in text editor
td8d39fb8 = 在文本编辑器中编辑

# Edit in {editor}
te44cd57b = 在 { $editor } 中编辑

# Modified
tdd13a6b2 = 修改时间

# Loading…
tc4e2f181 = 正在加载…

# Only what is loaded was searched.
t39c5f536 = 只搜索了已加载的内容。

# Search the whole folder
t41f9a787 = 搜索整个文件夹

# Load {n} more
t30bed46a = 再加载 { $n } 项

# Refresh
t0815aad4 = 刷新

# Hide hidden files
t3375c9c0 = 隐藏隐藏文件

# Show hidden files
tf18c7057 = 显示隐藏文件

# View
t5e4baa18 = 查看

# Extract…
t4a09490e = 解压…

# Open with default app
t5ef80ec5 = 用默认应用打开

# Open as archive
t3373ce55 = 作为压缩包打开

# Compress…
t690f9553 = 压缩…

# Copy to…
t543cf15b = 复制到…

# Move to…
t7a32320b = 移动到…

# Sign…
td6d61b10 = 签名…

# Checksums…
t2891102d = 校验和…

# Convert…
tae85a7c4 = 转换…

# Split into parts…
t281db403 = 切分为多个分卷…

# Join the parts
t0ad56db8 = 合并分卷

# Copy path
tf165cb2b = 复制路径

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
       *[other] 压缩 { $n } 项…
    }

# {n} loaded, more in this folder
t29a52578 = 已加载 { $n } 项，此文件夹中还有更多

# Sorted among those loaded
t037d01ae = 仅在已加载的项目中排序

# {n} selected
tebda8468 = 已选择 { $n } 项

# Try again
t213e90fa = 再试一次

# Nothing here matches that.
t0f9a8a58 = 没有符合的内容。

# Add a storage to browse it.
t0b82c83a = 添加存储后即可浏览。

# No folders or archives here.
t6a090cc3 = 这里没有文件夹或压缩包。

# No folders here.
tc7731c8f = 这里没有文件夹。

# This folder is empty.
td9e47db6 = 此文件夹为空。

# Show in Finder
ta9442aa9 = 在访达中显示

# Show in File Explorer
t23c89eda = 在文件资源管理器中显示

# Show in the file manager
tb347ece9 = 在文件管理器中显示

# Open in Finder
te2a467e0 = 在访达中打开

# Open in File Explorer
t3794d625 = 在文件资源管理器中打开

# Open in the file manager
td1ceabc4 = 在文件管理器中打开

# Copy all
t6aef5c37 = 全部复制

# This system has no folder for settings.
tfd0ff0a1 = 此系统没有用于存放设置的文件夹。

# Archive
t73690b65 = 压缩包

# Search inside this archive
tb35c0c27 = 在此压缩包中搜索

# Open another…
tff74a552 = 打开其他…

# From a storage…
tcfba8fd5 = 从存储…

# Safe to extract
t3c93bb70 = 可安全解压

# Reading… {percent}%
t83223c27 = 正在读取… { $percent }%

# Read {fetched} of {whole}
t613dd932 = 已读取 { $fetched } / { $whole }

# Add files
tdd45c7bb = 添加文件

# Test
t2ffcbe05 = 测试

# Reading what’s inside…
t1bbda9f5 = 正在读取内容…

# Nothing in this archive matches “{query}”.
t36c3e2f5 = 此压缩包中没有与“{ $query }”匹配的内容。

# Nothing in this folder matches “{query}”.
teecdfa3d = 此文件夹中没有与“{ $query }”匹配的内容。

# And {more} more. Type more to narrow it down.
t843253d0 = 还有 { $more } 项。多输入几个字可缩小范围。

# Open an archive
t8382bd3e = 打开压缩包

# ZIP, 7z and tar archives, disc images, packages, snaps and AppImages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .deb, .rpm, .pkg, .snap, .AppImage, .zst.age…
t2cb379b7 = ZIP、7z 和 tar 压缩包、光盘映像、软件包、snap 和 AppImage，以及用 age 压缩或加密的文件：.zip、.7z、.tar.gz、.tar.zst、.iso、.deb、.rpm、.pkg、.snap、.AppImage、.zst.age…

# Back to {name}
t539b9c10 = 返回 { $name }

# Open an archive…
t2cf2a8ee = 打开压缩包…

# It could not be shown
tb60fb288 = 无法显示

# Choose a file to see it here
t37854216 = 选择一个文件即可在此查看

# Symbolic link
tc7c20187 = 符号链接

# Leads to {target}
te7ddae48 = 指向 { $target }

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
       *[other] 此文件夹中的 { $count } 个可执行文件解压后将保留互联网标记。
    }

# Today
te7c0775e = 今天

# Last 7 days
t35b9d98f = 最近 7 天

# Last 30 days
td0778b33 = 最近 30 天

# All days
tabb197ef = 全部日期

# All results
t6235c0b2 = 全部结果

# Not done
t0f88a146 = 未完成

# Partly done
t3355a2a5 = 部分完成

# Failed
t5579c3d4 = 失败

# Cancelled
ta62b09bc = 已取消

# No end recorded
t38ab1371 = 未记录结束

# Make an archive
t418bd6f0 = 创建压缩包

# Open a file from an archive
tc2dd3e41 = 从压缩包打开文件

# Preview a file
t1f706562 = 预览文件

# Read a file from an archive
t4229020d = 从压缩包读取文件

# Add to an archive
t4e28d1fa = 添加到压缩包

# Delete from an archive
t158044d3 = 从压缩包删除

# Rename in an archive
tb3eef763 = 在压缩包中重命名

# Check a signature
t6f441e5e = 检查签名

# Make a signing key
tbdf6bcd0 = 创建签名密钥

# Join parts
t4eee8c5f = 合并分卷

# Checksums
t47dd48d3 = 校验和

# Audit log started
tfb8df56f = 审计日志已开始

# Audit log stopped
t54654bc3 = 审计日志已停止

# Audit log changed
td6b7bc4a = 审计日志已更改

# Old records taken away
t5f337ebf = 已移除旧记录

# All actions
t4ff15157 = 全部操作

# Find a path, a name or an error
t30cb1a6c = 查找路径、名称或错误

# Read again
t2f0dcc5d = 重新读取

# Save as CSV…
tf103c3cc = 另存为 CSV…

# Checking that no record was changed…
t00deba79 = 正在检查记录是否被更改…

# The audit log cannot be read: {why}
t8783f4cf = 无法读取审计日志：{ $why }

# A record was changed, taken out or put in by hand: in {file}, line {line}. | Records were changed, taken out or put in by hand, in {n} places: the first in {file}, line {line}.
t3cd011bd =
    { $n ->
       *[other] 有 { $n } 处记录被手动更改、删除或插入：第一处在 { $file } 第 { $line } 行。
    }

# The oldest record follows one that is gone, and no record says it was taken away as past its time.
t4579f8a7 = 最早的记录之前的记录已不存在，且没有记录说明它因超过保留期而被移除。

# {n} record, and none was changed or taken out. | {n} records, and none was changed or taken out.
tefc82060 =
    { $n ->
       *[other] { $n } 条记录，没有任何记录被更改或删除。
    }

# Only the latest {most} records are shown: choose fewer days, or read all with pkw audit list.
t5fabfc51 = 仅显示最新的 { $most } 条记录：请选择更少的天数，或用 pkw audit list 读取全部。

# Saved to {path}.
te8346fa4 = 已保存到 { $path }。

# Could not save: {why}
t733a7cae = 无法保存：{ $why }

# Record
t9f4565ec = 记录

# Started
t97b5823e = 开始

# Ended
tec2b41db = 结束

# Reading the audit log…
t403ccdff = 正在读取审计日志…

# No records in these days.
t98687757 = 这些天没有记录。

# No record is as chosen.
ta9ffcb9f = 没有符合所选条件的记录。

# This computer
tfa0d176a = 此计算机

# Home
t52f50eae = 主目录

# Desktop
t64215c53 = 桌面

# Downloads
t539307de = 下载

# Drop to open
tb255877b = 拖放即可打开

# Browse
t0eac1f07 = 浏览

# Recent
t50e888fe = 最近

# Audit log
tcb83f9ca = 审计日志

# Choose a file to see its information here
t50aac02b = 选择一个文件即可在此查看其信息

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
       *[other] 已选择 { $n } 项
    }

# Files in them come to {size}
t0cca93f1 = 其中的文件共计 { $size }

# Delete…
t2fed07aa = 删除…

# Link to a folder
t4b408baf = 指向文件夹的链接

# Link
t8e08e589 = 链接

# {tag} file
ta56bbbe9 = { $tag } 文件

# {files}, {folders}
t72e46dc5 = { $files }，{ $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
       *[other] { $n } 个文件
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
       *[other] { $n } 个文件夹
    }

# {size} so far
tc87daf59 = 目前 { $size }

# Kind
t3ff50c63 = 类型

# Holds
tffe19c31 = 包含

# Permissions
t8700fd83 = 权限

# Path
teb66e456 = 路径

# Signature
tc049050d = 签名

# Count size and files
t0f23db17 = 统计大小和文件数

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
       *[other] 其中有 { $n } 个文件夹无法读取。
    }

# Check the signature
t9d44730e = 验证签名

# Extracted
t3250471f = 已解压

# Archive created
t22c69419 = 已创建归档

# Moved
t82869bf0 = 已移动

# Signed
t27c3ab35 = 已签名

# Joined
tf8f2b670 = 已合并

# Converted
tc7b76e99 = 已转换

# Waiting for a password or a key
t0e83ae32 = 正在等待密码或密钥

# Enter it…
tddb7844a = 输入…

# Show
t29742e5c = 显示

# Jobs
t4ae6a79d = 任务

# Clear finished
t30c178e8 = 清除已完成

# Nothing runs in the background.
t2573f39e = 后台没有正在运行的任务。

# Add to the right-click menu
t116d3868 = 添加到右键菜单

# Extract or compress from any folder.
t20864983 = 在任意文件夹中解压或压缩。

# Welcome to packwing
tb38a86a6 = 欢迎使用 packwing

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = 打开任何压缩包，制作更小的文件便于分享，并远离有害下载。

# Set things up
t63f9c1cd = 进行设置

# You can change these later in Settings.
t95f41ffb = 你之后可以在设置中更改这些。

# Get started
te83ca5ee = 开始使用

# Remove from Recent
t26cc8dba = 从最近中移除

# {location} · not there any more
t7d0c09c7 = { $location } · 已不存在

# The archives you open will show here.
tc0cc82d5 = 你打开过的压缩包会显示在这里。

# Copy version info
t3d37483c = 复制版本信息

# Version {version} · {os} {arch}
t6bead5ae = 版本 { $version } · { $os } { $arch }

# A file archiver that puts every core to work and opens archives safely.
t5d2a9fa0 = 让每个核心都投入工作、安全打开压缩包的文件压缩工具。

# Updates
t483075a5 = 更新

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = 这是测试版：会通知你更新的测试版，以及其后的正式版。

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = 这是正式版：会通知你更新的正式版，但不通知测试版。

# Check now
t4f0e0677 = 立即检查

# Look for updates once a day
tcc27ace1 = 每天检查一次更新

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = 从 GitHub 读取版本列表，仅在其签名验证通过时才采信。未经你的同意，不会下载或安装任何内容。

# Skip this version
t2c33d756 = 跳过此版本

# This build has no release keys, so it cannot tell of updates.
t378182cb = 此版本没有发布密钥，因此无法提示更新。

# packwing {version} is ready
tf02650f8 = packwing { $version } 已就绪

# It is installed when you quit, or now with Restart.
t6fecac86 = 退出时安装，或点击“重新启动”立即安装。

# Restart now
t54001a7c = 立即重新启动

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = 正在检查新版本…

# Release notes
tc52ab62f = 发行说明

# packwing {version} is out
t36a30d8b = packwing { $version } 已发布

# Your license covers versions released until {date}. This one came out after: with it, packwing is Free until you renew. The version you have stays Pro.
tfd489117 = 你的许可证涵盖 { $date } 之前发布的版本。此版本在那之后发布：使用它时，packwing 为 Free，直到你续订。你当前的版本仍为 Pro。

# Renew
t9a23808e = 续订

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = 请在终端中运行这些命令。它们会根据已签名的版本列表校验下载内容，sudo 会询问你的密码。

# {by} updates it: {command}
tdf2cab9c = 通过 { $by } 更新：{ $command }

# {by} updates it.
t84b27295 = 通过 { $by } 更新。

# It is downloaded and checked, then installed when you quit: Windows asks for an administrator's permission, as packwing is installed for everyone.
tc816fc67 = 下载并校验后，在退出时安装：由于 packwing 是为所有用户安装的，Windows 会请求管理员权限。

# It is downloaded and checked, then installed when you quit.
tb46c0678 = 下载并校验后，在退出时安装。

# Download it, and install it in place of this one.
t39312eae = 下载后安装，替换当前版本。

# The download failed: {why}
t5e96c075 = 下载失败：{ $why }

# Download and install
t1a99173f = 下载并安装

# Download
t6964bb19 = 下载

# The last look failed: {why}
td122a092 = 上次检查失败：{ $why }

# Up to date (looked {when}).
t4665ef60 = 已是最新版本（上次检查：{ $when }）。

# Not looked yet.
t5a3ba058 = 尚未检查。

# Free software
t731220a4 = 自由软件

# Under the GNU General Public License, version 3 or later
t4e98bf3c = 基于 GNU 通用公共许可证第 3 版或更高版本

# View license
tfc21d38b = 查看许可证

# Made by HivePaaS
t10c7007a = 由 HivePaaS 出品

# On hivepack, our library for reading and writing archives
tfae09b61 = 基于 hivepack，我们用于读写压缩包的库

# Source code
tf6a49f4d = 源代码

# Something wrong?
t610dfb6f = 遇到问题？

# Tell us, with the version above
ta3e29034 = 请连同上方的版本号告诉我们

# Report a problem
t7b1728e7 = 报告问题

# What changed
te8345543 = 更新内容

# Every release, and what is new in it
t8e6f2052 = 每个版本及其新增内容

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS。packwing 名称和徽标是 HivePaaS 的商标。

# More from HivePaaS
tb25cf729 = HivePaaS 的更多产品

# Free and open source, like packwing.
tc1ece130 = 与 packwing 一样，免费且开源。

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = 适用于 Linux、macOS 和 Windows 的快速原生 SSH 客户端。

# SSH and SFTP
t0f8af34d = SSH 和 SFTP

# Follows your ssh_config
tc1c4446d = 遵循你的 ssh_config

# A real terminal
tae9bcfa8 = 真正的终端

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = 访问 shellwing.app

# A native HTTP(S) debugging proxy that keeps up with big sessions.
t86e5ad3e = 原生 HTTP(S) 调试代理，大型会话也能跟上。

# HTTP/2, WebSocket and SSE
t2c3c2377 = HTTP/2、WebSocket 和 SSE

# A million requests, smooth
t60de9f6c = 百万请求依然流畅

# Breakpoints and rewrites
t00c623a3 = 断点与改写

# Phones and emulators
t8c26f435 = 手机与模拟器

# Visit tracewing.app
t4668641f = 访问 tracewing.app

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = 基于 Docker Swarm 的自托管应用平台：Heroku、Render 和 Coolify 的开源替代品。

# 300+ one-click apps
t6a770b0a = 300+ 一键应用

# Deploy on push
tb0bfc3e9 = 推送即部署

# Pull request previews
t4e4180fc = 拉取请求预览

# Automatic TLS
t9e9776ed = 自动 TLS

# Live logs and metrics
tebe3a172 = 实时日志和指标

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = 访问 hivepaas.com

# Always
tad4bbd04 = 总是

# Extensions, apart by spaces
tc709375f = 扩展名，用空格分隔

# Format of new archives
t3b50d238 = 新压缩包的格式

# New archive starts with it; you can pick another there.
t7b361b55 = 新建压缩包时默认使用它；你也可以在那里选择其他格式。

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = 仅存储会原样放入 ZIP 和 7z 压缩包中的文件。tar 格式使用“最低”，因为只有 .tar 不压缩。

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = 级别越高，压缩包越小，但生成时间更长，打开时间不变。

# Write straight to a storage above
ta4975128 = 直接写入上方的存储

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = 较大的文件不占用此电脑的空间，但连接中断就得重新压缩。S3 以这种方式支持 tar 和 ZIP，SFTP 则支持所有格式。

# Don’t compress what already is
t940ec31d = 不再压缩已压缩的文件

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = ZIP 和 7z：图片、视频、音频和压缩包原样放入，更快且不会变大

# What goes in as it is
tc0104b29 = 原样放入的文件

# Files whose names end in these, whatever their case
ta3acef9b = 名称以这些结尾的文件，不区分大小写

# Reset to default
t0db9f7dc = 恢复默认

# Leave out Mac-only files
t27eec14f = 排除仅限 Mac 的文件

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store、__MACOSX 和资源分支：在 Mac 以外都是多余的文件

# Hide file names behind the password
t93ff7715 = 用密码隐藏文件名

# For 7z: nobody sees what is inside without it
t0f5354d7 = 适用于 7z：没有密码就看不到里面有什么

# Yours
t2635ea3b = 你的

# By name
t983e37f7 = 按名称

# By number
tcbbada8d = 按编号

# No limit
t6de7f47d = 不限制

# Where the files go
t77e60aba = 文件放到哪里

# Where Extract starts: you can pick another there.
t4b73082f = “解压”的默认位置：你可以在那里选择其他位置。

# One item right here, more in a new folder
ta5a4d31d = 单个项目放在此处，多个项目放入新文件夹

# A new folder next to it
t2095cbe9 = 旁边的新文件夹

# Named after the archive
tb991d4c1 = 以压缩包命名

# Next to the archive
t4691c928 = 压缩包旁边

# Straight into its folder
t99d1df43 = 直接放入其文件夹

# When a file is already there
tca514dda = 当文件已存在时

# A file of the same name in the folder the files go to
t296ad686 = 文件要放入的文件夹中已有同名文件

# Largest an archive may unpack to
t5957c426 = 压缩包解压后的最大大小

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = 防止压缩炸弹：解压后会占满磁盘的小文件

# Only names every system can use
tf47f4968 = 仅接受所有系统都能使用的名称

# Names Windows can’t use, such as a:b.txt, and names differing only in case are asked about on this computer too
t44f108af = Windows 无法使用的名称（如 a:b.txt）以及仅大小写不同的名称，在这台电脑上也会询问

# Owners from a backup
t38a7cf66 = 备份中的所有者

# Where a tar archive kept them: yours, or the archive’s by name or by number. Only an administrator gives files to others
t46908bee = tar 归档有保留时：你的，或按名称或编号采用归档中的。只有管理员能把文件交给他人

# Extended attributes from a backup
t764eed3a = 备份中的扩展属性

# Where a tar archive kept them, ACLs among them
ta28f3ae5 = tar 归档有保留时，包括 ACL

# Light
t8ad2f8cf = 浅色

# Dark
t45570065 = 深色

# System
t8f3184bc = 跟随系统

# Same as system ({language})
t5bd525d3 = 与系统相同（{ $language }）

# The system’s
t1d68c60b = 系统的

# Custom…
ta391dade = 自定义…

# The system’s ({size} px)
ta51f1877 = 跟随系统（{ $size } px）

# All ({count})
tf65ec196 = 全部 ({ $count })

# e.g. code -n, or /path/to/editor --flag
t455dc834 = 例如 code -n，或 /path/to/editor --flag

# Language
t9a73db9b = 语言

# Appearance
t31018b9f = 外观

# Text size
t2c041143 = 文字大小

# Processor cores to use
t2f9c3d17 = 使用的处理器核心数

# Text editor
t1ca82aab = 文本编辑器

# What Edit opens a file of a storage in
t62506ced = “编辑”存储中的文件时使用的应用

# The file’s path is added at the end.
teba749d4 = 文件路径会添加在末尾。

# Paste your license key
t3c4da5cf = 粘贴你的许可证密钥

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = 除文件签名外的所有功能，以及 { $count } 个网络存储。此电脑上的文件夹不受限制，任何签名都可验证。

# Every feature, and any number of storages.
tc3e75ecf = 所有功能，存储数量不限。

# Every feature, and any number of storages, for your organization.
t69b29f66 = 所有功能，供你的组织使用，存储数量不限。

# Plan
t1eab5e32 = 套餐

# Issued to
t44ef5cad = 授权给

# Who this license is for
td514fb0d = 此许可证的使用者

# Seats
t8f706321 = 席位

# What your organization’s agreement covers
t60b0c6c9 = 贵组织的协议所涵盖的内容

# Ended {date}
tfccb9305 = 已于 { $date } 结束

# This version came out after your updates ended, so it is Free. Renew, or use a version released by then: those stay Pro.
t545a1cd3 = 此版本在你的更新期结束后发布，因此为 Free。请续订，或使用在那之前发布的版本：那些版本仍为 Pro。

# This version stays Pro for good, with its fixes. Versions released after need a renewal.
tbf1ce5b8 = 此版本及其修复永久保持 Pro。之后发布的版本需要续订。

# Until {date}
t4d9bac91 = 截至 { $date }

# Every version released by then is yours for good, with its fixes. Renew for a year more of new versions.
t5d8058ff = 在那之前发布的每个版本及其修复永久归你所有。续订可再获得一年的新版本。

# Trial, until {date}
t9ffc2b63 = 试用，截至 { $date }

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = 已于 { $date } 结束。仍可继续使用 { $days } 天。

# Valid
td2661011 = 有效

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = 过期后，packwing 将变为免费版。除无法添加存储外，其他功能都不受影响。

# Your organization manages this license.
te42a0ee6 = 此许可证由你的组织管理。

# Activate
t3b8f0190 = 激活

# Buy
tcada780b = 购买

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = 密钥在此电脑上验证。不会向任何地方发送任何内容，许可证也不会改变你所在组织的策略效果。

# On
t60e47df0 = 开

# They show when you right-click files, folders and archives.
te216d656 = 当你右键单击文件、文件夹和压缩包时显示。

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = packwing 目前不在菜单中：加入菜单后，它们才会出现。

# Changing the menu…
tc7f1a2f0 = 正在修改菜单…

# Good to know
t8d7b3a90 = 须知

# The menu could not be changed: {why}
t9fac5c06 = 无法修改菜单：{ $why }

# Show packwing in the right-click menu
t5d0d23cc = 在右键菜单中显示 packwing

# Extract and compress from any folder
t94a46acb = 在任意文件夹中解压和压缩

# Items in the menu
t67537dc4 = 菜单中的项目

# Put them in a submenu
te3449a65 = 放入子菜单

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11 会把它们放在一个 Packwing 项下，无论在其菜单中还是“显示更多选项”中

# One packwing item, with these under it
tb5fdc584 = 只显示一个 packwing 项，这些项目在其下

# 30 days
t7a572af9 = 30 天

# 90 days
t2d8a9817 = 90 天

# 1 year
t4d0a98a5 = 1 年

# Keep all
te0b1775b = 全部保留

# Keep the “downloaded from the Internet” mark
tfe574fa2 = 保留“从互联网下载”标记

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = 运行来自已下载压缩包的文件前，系统会发出警告

# This system has no such mark, so this changes nothing here
t314d60c1 = 此系统没有这种标记，因此在这里不会有任何改变

# License
t38e70ee4 = 许可证

# Your organization’s, in Settings › License
tff7c594e = 由你的组织管理，位于 设置 › 许可证

# Treat files from storages as from the Internet
t0af8d466 = 将来自存储的文件视为来自互联网

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = 来自 S3、WebDAV 和 SFTP 的文件也会被加上标记

# Require encryption for new archives
t0c3e1337 = 要求新压缩包必须加密

# Applies to archives made and files added here, not to other apps
tc889f3a2 = 适用于在此创建的压缩包和添加的文件，不影响其他应用

# Kinds of encryption allowed
t1b8b2116 = 允许的加密方式

# New archives can be encrypted only these ways
t71dcd29c = 新压缩包只能使用这些方式加密

# Minimum password length
tb7d64a44 = 最短密码长度

# In characters, for every new archive with a password
tca6d6982 = 以字符计，适用于每个设有密码的新压缩包

# Keep an audit log
tf7fd8762 = 保留审计日志

# A record of what packwing does, whatever the license
t34fe4e25 = 记录 packwing 所做的事，不论许可证如何

# Where the audit log is kept
tbd999abe = 审计日志的位置

# A folder your organization chose
td4e1f74a = 你的组织选定的文件夹

# How long the audit log is kept
td504acaf = 审计日志保留多久

# In days; 0 keeps every record
t127a3c54 = 以天计；0 表示保留所有记录

# Most the audit log may take
ta334bb40 = 审计日志的大小上限

# In MB; the oldest records go first
t03a543d2 = 以 MB 计；最旧的记录先删除

# What the audit log says of archives
t8d6fca43 = 审计日志对压缩包记录什么

# “entries” names each file extracted, and those found damaged or left out
t1e33aba6 = “entries” 会记录每个解压出的文件，以及发现损坏或被跳过的文件

# Every job recorded
t42369815 = 每项任务都记录

# A job whose start the audit log cannot write does not start
tda48e1c9 = 审计日志无法写入其开始的任务不会开始

# The audit log in the system’s log
t4a2fcd04 = 系统日志中的审计日志

# Each record goes to {log} as well, where your organization’s tools read it
t3a4c104f = 每条记录也会写入 { $log }，供您组织的工具读取

# packwing has no policy of this name.
t7e03649c = packwing 没有此名称的策略。

# Ask before opening files without extracting
t76d5d951 = 不解压直接打开文件前先询问

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = 打开压缩包内的文件时，会在临时文件夹中放一份副本。程序则始终会先询问。

# Paste a public key
t0bf72f73 = 粘贴公钥

# What packwing does, kept on this computer: archives extracted, made, converted and copied, when and by whom. Never a password or what is in a file.
t2162955d = packwing 所做的事，保存在这台电脑上：解压、创建、转换和复制了哪些压缩包，何时、由谁。绝不记录密码或文件内容。

# Keep its records for
tf423722f = 记录保留

# Older ones are taken away, and that is recorded too
t7073f273 = 较旧的记录会被删除，删除本身也会记录

# Takes a Pro or Enterprise license: Settings › License.
t2774cee6 = 需要 Pro 或 Enterprise 许可证：设置 › 许可证。

# Where the audit log is
t2702fcc6 = 审计日志的位置

# View records
t27c5284a = 查看记录

# Open folder
t713a842f = 打开文件夹

# The last record could not be written: {why}
t542148a1 = 无法写入上一条记录：{ $why }

# pkw audit verify checks that no record was changed or taken out. Whoever runs this computer can still remove the log: it tells what packwing did, it does not stop anyone.
t482581ff = pkw audit verify 会检查没有记录被修改或删除。管理这台电脑的人仍可删除日志：它说明 packwing 做了什么，并不能阻止任何人。

# This is not a public key packwing can check with: {why}.
td57933b9 = 这不是 packwing 可用于验证的公钥：{ $why }。

# This line cannot be read, so the key is not used
t90e7dd7b = 无法读取此行，因此不会使用该密钥

# Keys you trust
ted99ced8 = 你信任的密钥

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = 这些密钥中任何一个的签名，都表明文件由谁签署。其他密钥的签名即使匹配也不能证明任何事。

# Choose a file…
t28a576c9 = 选择文件…

# {name} (weak)
tdb06c037 = { $name }（弱）

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = 文件菜单中“校验和…”显示的内容，用于与文件来源处提供的校验和进行比较。弱校验和能发现损坏的文件，但发现不了被故意修改的文件。

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = 无法读取其策略，因此受其保护的操作一律拒绝：{ $why }

# Some settings are managed by your organization
t7db7e7ec = 部分设置由你的组织管理

# Get help
t4a1ed54a = 获取帮助

# Not understood
t068eea1b = 无法识别

# Off
t0dc6b4aa = 关闭

# General
t5561fd0b = 通用

# Extracting
tc82af3f2 = 解压

# Compressing
t6d183c73 = 压缩

# Security
tc2fe21db = 安全

# Right-click menu
t1287ae97 = 右键菜单

# About
t58e096de = 关于

# Settings
t4b058728 = 设置

# This computer has no way to ask that it is you.
t0c1b4d43 = 此电脑无法要求确认是你本人。

# open a storage
t780455bf = 打开存储

# Open a storage in packwing
tf861e014 = 在 packwing 中打开存储

# That was not it. It stays locked.
tbb2a9838 = 不正确。仍保持锁定。

# The system could not ask: {why}
td6905afe = 系统无法发出验证请求：{ $why }

# This copy of packwing cannot open it.
tfb56d8cf = 此 packwing 无法打开它。

# Storages
tc776c52f = 存储

# Add storage
te858d6b5 = 添加存储

# Type the password you log in to this computer with to open it.
tb544cd9b = 输入你登录此电脑的密码即可打开。

# Show this computer it is you to open it.
tfb4c635b = 向此电脑确认是你本人即可打开。

# Checking…
t87f62cb7 = 正在检查…

# It asks again after 5 minutes left unused.
t69c22a3b = 闲置 5 分钟后会再次询问。

# Edit…
t8e7efcd7 = 编辑…

# Remove from this list
t8c7c7c57 = 从此列表中移除

# Extract here with packwing
tc5e54e4a = 用 packwing 解压到此处

# Extract to new folder
tda62691f = 解压到新文件夹

# Extract with packwing…
tda9255a0 = 用 packwing 解压…

# Open with packwing
teb657427 = 用 packwing 打开

# Compress to ZIP
t344b1175 = 压缩为 ZIP

# Compress with packwing…
t9ec9b77d = 用 packwing 压缩…

# packwing cannot find its program: {e}
t1934ce08 = packwing 找不到自己的程序：{ $e }

# Signed by {signer}
t8eb35b10 = 由 { $signer } 签名

# Signed by {signer}, not checked yet
t88b7186a = 由 { $signer } 签名，尚未验证

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = 由你不信任的密钥签名：{ $fingerprint }

# It does not match its signature
tb7d0e1ed = 与其签名不匹配

# Its signature cannot be read
tadf2a401 = 无法读取其签名

# Its signature could not be checked
t69cbbb51 = 无法验证其签名

# Checking its signature…
tda8c2e03 = 正在验证其签名…

# the system's package manager
t11465c20 = 系统包管理器

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = 正在下载… { $got } / { $total }

# Downloading… {got}
t4873b921 = 正在下载… { $got }

# just now
ta715ef0f = 刚刚

# {n} minute ago | {n} minutes ago
t6c7302d6 = { $n } 分钟前

# {n} hour ago | {n} hours ago
t137b052e = { $n } 小时前

# {n} day ago | {n} days ago
tb9fa7744 = { $n } 天前

# No folder to download to.
t2bfc1370 = 没有可用于下载的文件夹。

# packwing can’t show this kind of file
tb909bf3c = packwing 无法显示这种文件

# Too big to show here
t1b339d35 = 太大，无法在此显示

# It needs the password to show it
t81f5dbce = 需要密码才能显示

# It could not be read: {why}
t26005fe3 = 无法读取：{ $why }

# Find in text
t9cbe4be4 = 在文本中查找

# Invalid pattern
t5b9dcaca = 无效的模式

# No matches
td29421ef = 无匹配项

# Find
t3e1cd17a = 查找

# Fit
t116f13cc = 适应窗口

# {count} line | {count} lines
t9fbb1078 =
    { $n ->
       *[other] { $count } 行
    }

# Its first 1 MB is shown
t329dac80 = 仅显示前 1 MB

# Update available
t676409df = 有可用更新

# Restart to update
tbbbbda26 = 重启以更新

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
       *[other] 有 { $n } 个文件因名称已被占用而被跳过。
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
       *[other] 有 { $n } 个链接保持原样。
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = 此文件已用密码短语加密。

# This archive needs its password.
td46b69a1 = 此压缩包需要密码。

# This file is encrypted to a key.
tbd048b4e = 此文件已加密到某个密钥。

# This key file needs its passphrase.
t81f4c775 = 此密钥文件需要密码短语。

# {paths} archives
t27f3b860 = { $paths } 个压缩包

# {folder}, a folder for each
t75e7db7c = { $folder }，每个压缩包一个文件夹

# New
t8b242db1 = 新建

# Your organization requires each job to be recorded, and the audit log cannot be written: {why}
t3f254f09 = 您的组织要求记录每项任务，但无法写入审计日志：{ $why }

# There is no file to save it beside.
ta9906e3b = 没有可在其旁边保存的文件。

# Nothing in the archive matches the patterns given.
t90b19482 = 归档中没有与给定模式匹配的内容。

# Nothing to compress.
t47339885 = 没有可压缩的内容。

# {path} is too big.
te9ab879e = { $path } 太大。

# This format holds one file, and no more.
t75c197d9 = 此格式只能存放一个文件。

# Store
t6da32dce = 仅存储

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
tad0120eb = 无法读取 { $path }：{ $e }

# {path} has no name.
tbd0e16dc = { $path } 没有名称。

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = { $path } 的名称不是有效的 Unicode，因此无法放入压缩包。

# Could not write {path}: {e}
t8dd055c6 = 无法写入 { $path }：{ $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted } 及其后的名称都已被占用。

# Could not encrypt: {e}
t485d71ab = 无法加密：{ $e }

# {level} takes about {needed} of memory, and {available} is free: it is made at {lower}, which takes about {less}.
t87cf6e96 = { $level } 约需 { $needed } 内存，当前可用 { $available }：将以约需 { $less } 的 { $lower } 创建。

# This level takes about {needed} of memory, and {available} is free: the computer may slow down, or packwing stop.
tbee8032a = 此级别约需 { $needed } 内存，当前可用 { $available }：电脑可能变慢，或 packwing 停止。

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = ZIP 或 7z 压缩包只有在由 age 整体加密时才能使用 age 密钥。

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age 只能加密到密码短语或密钥，不能同时使用两者。

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = 后量子密钥不能与其他密钥一起使用：其他密钥会使文件在量子计算机面前暴露。

# There are no keys in this file to encrypt to.
tc2809b2c = 此文件中没有可用于加密的密钥。

# {path} got shorter while it was being read.
t2e221d1f = 读取过程中 { $path } 变短了。

# A password has to be valid Unicode.
t5ef729bc = 密码必须是有效的 Unicode。

# Could not read the extended attributes of {path}: {e}
t4f1421d7 = 无法读取 { $path } 的扩展属性：{ $e }

# Could not start compressing: {e}
tc4faa70c = 无法开始压缩：{ $e }

# Could not compress: {e}
tf6ff23f2 = 无法压缩：{ $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path } 是链接，而链接仅在 Unix 上保留。

# Changing this archive writes a new one beside it, which may take up to {need}. Only {free} is free in {folder}.
ta087fe64 = 修改此压缩包会在旁边写入一个新的压缩包，可能需要多达 { $need }。{ $folder } 中只有 { $free } 可用。

# Nothing changed.
tedfc294c = 没有任何更改。

# The archive did not come out whole.
t2f015c79 = 压缩包未能完整生成。

# packwing can change only a ZIP or 7z archive that is one file.
t675882a8 = packwing 只能修改单文件的 ZIP 或 7z 压缩包。

# That is not in the archive.
t7530c419 = 压缩包中没有它。

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = 这已是压缩包中的全部内容。请改为删除压缩包本身。

# “{to}” cannot be a name.
td53e55bf = “{ $to }”不能作为名称。

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = “{ $to }”已在该文件夹中。未做任何更改。

# packwing can add files only to a ZIP or 7z archive that is one file.
t8d1ef729 = packwing 只能向单文件的 ZIP 或 7z 压缩包添加文件。

# An archive can’t go inside itself.
tff43ce74 = 压缩包不能放入它自身。

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = “{ $bare }”已在压缩包中。未做任何更改。

# Nothing chosen matches the patterns given.
te132289a = 所选内容中没有与这些模式匹配的项。

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = 上传前，此压缩包会先在 { $folder } 中生成，可能需要多达 { $need }。那里只有 { $free } 可用。

# The upload stopped.
t528232e6 = 上传已停止。

# “{pattern}” names no file or folder.
te17b2bfa = “{ $pattern }”没有指向任何文件或文件夹。

# a colon, which Windows does not allow
t3ba6a1aa = 含有 Windows 不允许的冒号

# a character Windows does not allow (\ < > " | ? *)
t0b11ab21 = 含有 Windows 不允许的字符 (\ < > " | ? *)

# a name Windows keeps for a device
t8cac89ee = 是 Windows 为设备保留的名称

# ends in a space or a dot, which Windows does not allow
t7ae9a209 = 以空格或句点结尾，Windows 不允许

# longer than 255 bytes
t9218e5c0 = 长于 255 字节

# a control character
t378bba3a = 含有控制字符

# not a name this system can read
t72b79e1d = 不是本系统能读取的名称

# the same file here as another, its name differing only in case
t1eead41f = 在这里与另一个文件相同，名称仅大小写不同

# a link that leads outside the folder
t00fdc25e = 指向文件夹外的链接

# {name} is already in the folder.
tbe7fab8a = { $name } 已在该文件夹中。

# Could not make a folder in {parent}.
tec7ca3a9 = 无法在 { $parent } 中创建文件夹。

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to } 是文件夹，而文件会将其替换。

# {name} is already in the folder
t5af584a0 = { $name } 已在该文件夹中

# This file is not an archive packwing can open.
t660f149b = 此文件不是 packwing 能打开的压缩包。

# This is an ar archive, as static libraries are. packwing can’t open those yet.
t9c5564b8 = 这是 ar 归档，如静态库。packwing 暂时还无法打开此类文件。

# This is a macOS disk image (.dmg). packwing can’t open those yet.
t080d0cb2 = 这是 macOS 磁盘映像 (.dmg)。packwing 暂时还无法打开此类文件。

# The password is wrong, or the archive is damaged.
ta4d3227d = 密码错误，或压缩包已损坏。

# The passphrase is wrong.
t7826c649 = 密码短语错误。

# The key file’s passphrase is wrong.
t8ad75e3a = 密钥文件的密码短语错误。

# This file is encrypted to a key packwing does not have.
te162af76 = 此文件已加密到 packwing 没有的密钥。

# Cancelled.
t27c010d6 = 已取消。

# the archive could not be read
t2267c3b1 = 无法读取压缩包

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = 无法解压，因为某个文件名在此处无法使用：“{ $name }”（{ $why }）。

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
       *[other] { $n } 字节
    }

# A volume of this archive is not there.
t3803103c = 此压缩包缺少一个分卷。

# {named}: its data is damaged.
t066227f9 = { $named }：其数据已损坏。

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = 压缩包已损坏：UnRAR 将以 { $number } 退出。

# There is no public key here.
t9fd24078 = 这里没有公钥。

# Could not make the key: {e}
t73610ecc = 无法生成密钥：{ $e }

# There is no file to sign.
tc95e3a9c = 没有可签名的文件。

# {name} is not there any more.
t07c1f2bc = { $name } 已不存在。

# A part has to hold at least one byte.
te779b111 = 每个分卷至少要有一个字节。

# {path} is not a file.
tdfee38b2 = { $path } 不是文件。

# That would make more parts than can be numbered: make them bigger.
t7ce88e46 = 这样分出的分卷会多到无法编号：请把分卷设大一些。

# {part} is there already.
t2577cbb6 = { $part } 已存在。

# {path} is not a part of a file cut as 7-Zip cuts it, as name.001.
t64c1c1d0 = { $path } 不是按 7-Zip 方式切分的文件的分卷（如 name.001）。

# {part} is missing: the parts start from it.
t5e65a10a = 缺少 { $part }：分卷从它开始。

# {part} is missing, and the parts after it are there.
t24165c89 = 缺少 { $part }，而它之后的分卷都在。

# {part} holds {got}, not {size} as the first part does: it may not be whole.
tb472eff0 = { $part } 有 { $got }，不是第一个分卷的 { $size }：它可能不完整。

# The name is empty.
t1a9d8e4e = 名称为空。

# That name is kept for the system.
t74c8683a = 该名称被系统保留。

# A name cannot hold / or other special characters.
t4c0596ec = 名称不能包含 / 或其他特殊字符。

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = 在 Windows 上，名称不能以点或空格结尾。

# That name is taken in this folder.
t43a2bce1 = 此文件夹中已有该名称。

# {name} is already there.
tb5e7b134 = { $name } 已存在。

# {name} cannot go inside itself.
t99f437a6 = { $name } 不能放入它自身。

# Stopped.
t15558940 = 已停止。

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name } 在一处是文件，在另一处是文件夹。

# The copy of {name} does not match. The original was kept.
t779a64f3 = { $name } 的副本不一致。已保留原件。

# This is not a valid packwing license key.
td5ae978f = 这不是有效的 packwing 许可证密钥。

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = 无法读取你的组织对此项的策略，因此 packwing 不会执行。请咨询你的 IT 部门。

# Your organization requires new archives to be encrypted.
t83acf110 = 你的组织要求新压缩包必须加密。

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = 你的组织不允许这种加密方式：{ $method }。

# Your organization requires a password of at least {least} characters.
t2d21b125 = 你的组织要求密码至少 { $least } 个字符。

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = 该密码太容易被猜到，而你的组织要求使用更强的密码。

# Expected true or false, not {other}.
tcadcff7a = 应为 true 或 false，而不是 { $other }。

# Expected a number from 1 to {most}, not {other}.
tda498fad = 应为 1 到 { $most } 之间的数字，而不是 { $other }。

# Expected some text, not {other}.
t9386ce3b = 应为文本，而不是 { $other }。

# Expected a whole path, such as /var/log/packwing or \\server\audit, not {other}.
t00d3565d = 应为完整路径，例如 /var/log/packwing 或 \\server\audit，而不是 { $other }。

# Expected a number of days from 0 (keep all) to {most}, not {other}.
tf8fba675 = 应为 0（全部保留）到 { $most } 的天数，而不是 { $other }。

# Expected a size in MB from 0 (no limit) to {most}, not {other}.
t9f7e002d = 应为 0（无上限）到 { $most } 的 MB 大小，而不是 { $other }。

# Expected summary or entries, not {other}.
t27e9b96f = 应为 summary 或 entries，而不是 { $other }。

# Expected an address starting with https://, not {other}.
t93bfd19e = 应为以 https:// 开头的地址，而不是 { $other }。

# Expected a list, not {other}.
tc945cc93 = 应为列表，而不是 { $other }。

# No kind of encryption is called {name}.
tfddb7ba6 = 没有名为 { $name } 的加密方式。

# The list names no kind of encryption.
t5f8076f5 = 该列表没有列出任何加密方式。

# The file is too big to be a list of policies.
taf68e582 = 该文件太大，不可能是策略列表。

# The file has to hold one JSON object.
tf483aa65 = 该文件必须包含一个 JSON 对象。

# {path} has to belong to root.
tfc77ea2b = { $path } 必须属于 root。

# {path} can be changed by others than root.
t0d4a4498 = { $path } 可被 root 以外的用户修改。

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = 此存储目前只接受最大 { $size } 的文件，因为每个文件在发送时都要保存在内存中。

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = 此存储无法直接重命名文件夹，因为其下所有内容都要复制。请使用“移动到…”。

# S3 bucket
tc5e013b3 = S3 存储桶

# Bucket
te34e3557 = 存储桶

# Region
t492e789d = 区域

# Endpoint
t27e7d69c = 端点

# Empty for AWS, or https://…
td3a94af8 = AWS 请留空，或填 https://…

# Start in
t5eb91ea6 = 起始位置

# A prefix, if not the whole bucket
t75255a6c = 前缀；若是整个存储桶则留空

# Access key ID
tf9408d75 = 访问密钥 ID

# Secret access key
te1c3d3a2 = 秘密访问密钥

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = 主机或 ssh_config 名称

# Empty for the home folder
t67b5f42b = 留空则使用主目录

# Empty to use the SSH agent and keys
tf10e6293 = 留空则使用 SSH 代理和密钥

# WebDAV folder
tb2401428 = WebDAV 文件夹

# Address
t700560f3 = 地址

# User
te0a63e12 = 用户

# Password or app token
t1c18cb9b = 密码或应用令牌
