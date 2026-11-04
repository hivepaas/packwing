# packwing in Korean. Each message is found by its English, the
# comment above it; scripts/i18n.py keeps this file in step with the code.

# Appearance: {name}. Click for {name2}.
t17634ee0 = 모양: { $name }. 클릭하면 { $name2 }(으)로 바뀝니다.

# Stop and say so
tdc8b782b = 중지하고 알림

# Replace it
t1e17fc0c = 교체

# Skip it
tad9909cb = 건너뛰기

# Add
t9dc3aa14 = 추가

# The top of {name}
t369f9d4d = { $name }의 맨 위

# Choose what to add
t48b969a8 = 추가할 항목 선택

# {count} item | {count} items
t534e422e =
    { $n ->
       *[other] 항목 { $count }개
    }

# {count} file | {count} files
tde674bd8 =
    { $n ->
       *[other] 파일 { $count }개
    }

# Add to {name}
t0d53ac5e = { $name }에 추가

# Goes in
t36e95c22 = 추가 위치

# If a file of that name is there
t34c69cec = 같은 이름의 파일이 있을 때

# This archive is signed. With more in it, it will no longer match its signature, until it is signed again.
t55ed1cfd = 이 압축 파일은 서명되어 있습니다. 항목을 더 추가하면 서명과 일치하지 않게 되며, 다시 서명하기 전까지 그대로입니다.

# The new files are encrypted with the archive’s password.
t52b1d223 = 새 파일은 압축 파일의 비밀번호로 암호화됩니다.

# Cancel
t35afca3b = 취소

# Name
t0fe07306 = 이름

# Folder
t06357afd = 폴더

# Its secrets could not be read: {why}
t2af0c8ab = 보안 정보를 읽을 수 없습니다: { $why }

# That folder is not on this computer.
ta4073fff = 이 컴퓨터에 없는 폴더입니다.

# Its secrets could not be kept: {why}
ta213827f = 보안 정보를 저장할 수 없습니다: { $why }

# Local folder
te075bcbc = 로컬 폴더

# Trying it…
t214cdad5 = 시도하는 중…

# It answers.
t3bd5bc9f = 응답합니다.

# It did not answer: {why}
t0ac8fa44 = 응답하지 않습니다: { $why }

# Edit storage
t7bae68d0 = 스토리지 편집

# Add a storage
t4fe8f806 = 스토리지 추가

# Secrets go in the system’s keychain, not with the settings.
t3f9544ac = 보안 정보는 설정이 아닌 시스템 키체인에 저장됩니다.

# Its files stay where they are.
t4e4acf9a = 파일은 있던 자리에 그대로 남습니다.

# Ask for this computer’s password to open it
t7a7d61e9 = 열 때 이 컴퓨터의 비밀번호 요구

# For a computer left unlocked: what unlocks this computer opens it, and it asks again after 5 minutes left unused.
t1b5ad6b1 = 잠금이 해제된 채 자리를 비운 컴퓨터를 위한 설정입니다. 이 컴퓨터의 잠금을 해제하는 방법으로 열 수 있으며, 5분간 사용하지 않으면 다시 묻습니다.

# packwing Free keeps one storage across a network. A Pro or Enterprise license lifts this: Settings › License. Folders on this computer are not limited.
t89f1bf95 = packwing Free는 네트워크 너머의 스토리지를 하나만 사용할 수 있습니다. Pro 또는 Enterprise 라이선스로 이 제한을 풀 수 있습니다: 설정 › 라이선스. 이 컴퓨터의 폴더에는 제한이 없습니다.

# Test connection
te0f3b6b7 = 연결 테스트

# Enter a license key
ta24f28a2 = 라이선스 키 입력

# Save
t4d2d5d68 = 저장

# Paste the checksum given with the file
t8333758c = 파일과 함께 제공된 체크섬을 붙여 넣으세요

# They could not be saved: {why}
tb45ae457 = 저장할 수 없습니다: { $why }

# The file could not be read: {why}
t5daa8455 = 파일을 읽을 수 없습니다: { $why }

# Reading…
t6aad8485 = 읽는 중…

# Copied
t8dc21305 = 복사됨

# Copy
t658f3664 = 복사

# weak
tedbc2ec9 = 약함

# There is no checksum in this.
t231f838a = 여기에는 체크섬이 없습니다.

# This is not a checksum packwing makes: it has {digits} digits.
t65dd697c = packwing이 만드는 체크섬이 아닙니다: { $digits }자리입니다.

# It matches the file’s {name} checksum.
t4f0350e4 = 파일의 { $name } 체크섬과 일치합니다.

# It is compared once the file is read.
tcb5c1745 = 파일을 다 읽으면 비교합니다.

# It does not match the file’s {name} checksum: this is not the file it was made of, or the file changed on the way.
t55d6aeb7 = 파일의 { $name } 체크섬과 일치하지 않습니다: 이 체크섬을 만든 파일이 아니거나, 오는 도중에 파일이 바뀌었습니다.

# It may be a {name} checksum, which is not made yet.
t3d713e04 = 아직 만들지 않은 { $name } 체크섬일 수 있습니다.

# Make it
tcbefe458 = 만들기

# Compare with
tbddcfc14 = 비교 대상

# Reading… {size} of {size2}
t80036d1c = 읽는 중… { $size } / { $size2 }

# Reading… {size}
t1569c388 = 읽는 중… { $size }

# Checksums of {name}
te88f3ed9 = { $name }의 체크섬

# All of the file passes through this computer, and your provider may charge for what is read from it.
t8cca3c05 = 파일 전체가 이 컴퓨터를 거치므로, 제공업체에서 읽은 양에 따라 요금이 부과될 수 있습니다.

# A weak checksum ({names}) still tells a damaged file, but not one changed on purpose, which can be made to match.
t164fdec6 = 약한 체크섬({ $names })은 손상된 파일은 알아내지만, 일치하도록 일부러 바꾼 파일은 알아내지 못합니다.

# Saved beside it as {files}, where sha256sum -c and the like check it without packwing.
t1d91952b = 옆에 { $files }(으)로 저장했습니다. sha256sum -c 등으로 packwing 없이 확인할 수 있습니다.

# Compare…
tec85be80 = 비교…

# Saving…
tbd79b37d = 저장 중…

# Save beside it
t7f4d85e3 = 옆에 저장

# Read it
t80fb8482 = 읽기

# Done
t8dd31791 = 완료

# Nothing chosen yet
tcf390b0a = 아직 선택하지 않음

# {join} and {more} more
t8399ef70 = { $join } 외 { $more }개

# Hide what goes in
t5a420780 = 추가할 항목 숨기기

# Show what goes in
t9a20eb53 = 추가할 항목 보기

# Take it out
t90a93ec7 = 빼기

# Drop them to add them
t6a4aedbe = 여기에 놓으면 추가됩니다

# What goes in
t3b7c797a = 추가할 항목

# + Files
tba8a1e71 = + 파일

# + Folder
tcef8b0ce = + 폴더

# Add the files or a folder to put in the archive.
t2e5c8c3e = 압축 파일에 넣을 파일이나 폴더를 추가하세요.

# Windows 11 and Linux open 7z on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tf7f64b21 = Windows 11과 Linux는 7z를 기본으로 열 수 있습니다. macOS, 휴대폰, 이전 버전의 Windows는 압축 앱이 필요할 수 있습니다. 누구에게나 공유하려면 .zip을 선택하세요.

# Linux, macOS and Windows 11 open it on their own; phones and older Windows may need an archiver app. To share with anyone, pick .zip.
t051031b1 = Linux, macOS, Windows 11은 기본으로 열 수 있습니다. 휴대폰과 이전 버전의 Windows는 압축 앱이 필요할 수 있습니다. 누구에게나 공유하려면 .zip을 선택하세요.

# Linux and Windows 11 open it on their own; macOS, phones and older Windows may need an archiver app. To share with anyone, pick .zip.
tc2c29183 = Linux와 Windows 11은 기본으로 열 수 있습니다. macOS, 휴대폰, 이전 버전의 Windows는 압축 앱이 필요할 수 있습니다. 누구에게나 공유하려면 .zip을 선택하세요.

# Most computers need an archiver app or the lz4 tool to open it. To share with anyone, pick .zip.
te9afe1c2 = 대부분의 컴퓨터에서 열려면 압축 앱이나 lz4 도구가 필요합니다. 누구에게나 공유하려면 .zip을 선택하세요.

# It holds this one file. Command-line tools open it; Windows and macOS may need an archiver app. To share with anyone, pick .zip.
t16463509 = 파일 하나만 담습니다. 명령줄 도구로 열 수 있으며, Windows와 macOS는 압축 앱이 필요할 수 있습니다. 누구에게나 공유하려면 .zip을 선택하세요.

# Store
t6da32dce = 저장만

# Lowest
t5d7dbfdb = 최저

# Low
tb1dd9521 = 낮음

# Medium
t09be5f96 = 보통

# High
tbe0a061d = 높음

# Highest
t9487b22f = 최고

# Don’t split
t56ee6829 = 나누지 않음

# 25 MB (email)
tc6ba1c9c = 25MB (이메일)

# 4 GB (USB stick)
tfc6b3ea5 = 4GB (USB 메모리)

# Custom size…
t0fcf9e2d = 직접 지정…

# Size
ta6478e7c = 크기

# Password
t2cc30838 = 비밀번호

# age1…, age1pq1… or ssh-ed25519 …
t76cb89a8 = age1…, age1pq1… 또는 ssh-ed25519 …

# Save here
ta1f147a6 = 여기에 저장

# Use these keys
t1a01e439 = 이 키 사용

# No compression
t2e9c9e78 = 압축 안 함

# {name} compression
te2db10f6 = { $name } 압축

# Parts of 25 MB
ta999efe2 = 25MB 단위로 분할

# Parts of 4 GB
t760df467 = 4GB 단위로 분할

# Parts of {size}
tdd286479 = { $size } 단위로 분할

# Parts of a size to type
tafd4b17d = 직접 입력한 크기로 분할

# Public key
t9981f45f = 공개 키

# Beside the files
t757989af = 파일과 같은 위치

# Save in
tf2d34837 = 저장 위치

# Folder…
tc8abcd43 = 폴더…

# Storage…
td0d8c806 = 스토리지…

# Parts are made in this computer’s temporary folder first, then uploaded, so they need room there.
tdc045fbd = 분할 파일은 먼저 이 컴퓨터의 임시 폴더에 만든 뒤 업로드하므로, 그곳에 여유 공간이 필요합니다.

# {ext} is made in this computer’s temporary folder first, then uploaded, so it needs room there. .zip and tar formats go straight to this storage.
tb6b5d19b = { $ext }는 먼저 이 컴퓨터의 임시 폴더에 만든 뒤 업로드하므로, 그곳에 여유 공간이 필요합니다. .zip과 tar 형식은 이 스토리지에 바로 저장됩니다.

# It is made in this computer’s temporary folder first, then uploaded, so it needs room there.
ted6f948e = 먼저 이 컴퓨터의 임시 폴더에 만든 뒤 업로드하므로, 그곳에 여유 공간이 필요합니다.

# {place} takes files of up to {most}. An archive larger than that fails when it starts to upload.
t91729834 = { $place }은(는) 최대 { $most }까지의 파일을 받습니다. 이보다 큰 압축 파일은 업로드를 시작할 때 실패합니다.

# Choose a storage, and a folder in it
td7627e48 = 스토리지와 그 안의 폴더를 선택하세요

# New folder
t1926d127 = 새 폴더

# Give it a name, without / or \ in it.
ta598b15b = 이름을 입력하세요. /나 \는 사용할 수 없습니다.

# A file of that name is there already: this one will be {name}.
t9451c3b1 = 이름이 같은 파일이 이미 있습니다. 이 파일은 { $name }(으)로 저장됩니다.

# Hide advanced
tae2a96e9 = 고급 옵션 숨기기

# Advanced options
t701d5ed9 = 고급 옵션

# Compression level
td386c9ef = 압축 수준

# Store puts the files in as they are: the quickest to make, and the largest.
t076ef350 = 저장만은 파일을 그대로 넣습니다. 만들기는 가장 빠르고 크기는 가장 큽니다.

# Higher makes a smaller archive, which takes longer to make, not to open.
t273a832f = 높을수록 압축 파일이 작아지며, 만드는 시간이 길어지지만 여는 시간은 그대로입니다.

# Split into parts
tca4436bd = 분할

# Each part
t642c444d = 분할 크기

# Parts open in 7-Zip and archivers like it, not in the tools built into Windows and macOS.
t2fd629a2 = 분할 파일은 7-Zip 및 비슷한 압축 앱에서 열 수 있으며, Windows와 macOS에 내장된 도구에서는 열 수 없습니다.

# A part takes a size of 1 MB or more, such as 100 MB.
t804e2099 = 분할 크기는 100MB처럼 1MB 이상이어야 합니다.

# Encrypt with age
t22784ad9 = age로 암호화

# Protect with a password
tae8d8740 = 비밀번호로 보호

# Suggest
ta6e44ce5 = 추천

# Keep this somewhere safe: without it, this archive can never be opened.
t8eb937cf = 안전한 곳에 보관하세요. 이것이 없으면 이 압축 파일을 열 수 없습니다.

# Type a password, or turn this off.
t62b19318 = 비밀번호를 입력하거나 이 옵션을 끄세요.

# Hide file names too
t2198d77b = 파일 이름도 숨기기

# With a password, the built-in Windows and macOS tools can’t open this ZIP. The recipient needs an archiver app. File names also stay visible; pick .7z to hide them.
t659d3827 = 비밀번호를 설정하면 Windows와 macOS의 내장 도구로는 이 ZIP을 열 수 없으며, 받는 사람에게 압축 앱이 필요합니다. 파일 이름도 그대로 보입니다. 이름을 숨기려면 .7z를 선택하세요.

# Saved as {name}. The names inside are encrypted too. Windows and macOS may need the age tool or an archiver app to open it.
tdc4a7daf = { $name }(으)로 저장했습니다. 안의 이름도 암호화되어 있습니다. Windows와 macOS에서는 age 도구나 압축 앱이 필요할 수 있습니다.

# {n} key from {name} | {n} keys from {name}
t575a1a6c =
    { $n ->
       *[other] { $name }의 키 { $n }개
    }

# Remove
t21a5901d = 제거

# {n} key in it was left out, of a kind age does not encrypt to. | {n} keys in it were left out, of a kind age does not encrypt to.
tbb501800 =
    { $n ->
       *[other] age가 암호화에 쓰지 않는 종류의 키 { $n }개는 제외했습니다.
    }

# From a file…
t40d7c944 = 파일에서 가져오기…

# Type a public key, or choose a file of them.
tcb028b06 = 공개 키를 입력하거나 키가 담긴 파일을 선택하세요.

# A post-quantum key: to open this, the age tool must be version 1.3 or newer.
ta856ab34 = 양자 내성 키입니다. 이 파일을 열려면 age 도구가 1.3 이상 버전이어야 합니다.

# Only the private key that goes with it opens this, and nothing can if that key is lost.
t39644118 = 이와 짝이 되는 개인 키로만 열 수 있으며, 그 키를 잃어버리면 아무도 열 수 없습니다.

# Weak
t6de87ae9 = 약함

# Fair
t82fa0d53 = 보통

# Strong
t0dad409a = 강함

# Choose what to put in it
t5461cfa6 = 넣을 항목 선택

# New archive
tf352befb = 새 압축 파일

# Create
t990de47d = 만들기

# {n} item | {n} items
tfff7fd6c =
    { $n ->
       *[other] 항목 { $n }개
    }

# Move {what} to the Trash?
tdf5a8d53 = { $what }을(를) 휴지통으로 이동하시겠습니까?

# Delete {what}?
ta297d6c5 = { $what }을(를) 삭제하시겠습니까?

# It can be put back from the system’s Trash.
te5e137a1 = 시스템 휴지통에서 복원할 수 있습니다.

# This cannot be undone from packwing.
t3947dab4 = packwing에서는 되돌릴 수 없습니다.

# and {n} more | and {n} more
t6f358d35 =
    { $n ->
       *[other] 외 { $n }개
    }

# Move to Trash
ta036661b = 휴지통으로 이동

# Delete
t5797ea6a = 삭제

# {many} items
t49912bd4 = 항목 { $many }개

# Rename {what}
t83401013 = { $what } 이름 바꾸기

# New name
taf4cad88 = 새 이름

# It goes from the archive for good, folders with all they hold. This can’t be undone.
t33cb3e89 = 압축 파일에서 완전히 삭제되며, 폴더는 안의 모든 항목과 함께 삭제됩니다. 되돌릴 수 없습니다.

# This archive is signed. After this change it will no longer match its signature, until it is signed again.
t8f575534 = 이 압축 파일은 서명되어 있습니다. 이 변경 후에는 서명과 일치하지 않게 되며, 다시 서명하기 전까지 그대로입니다.

# Rename
tc80631f3 = 이름 바꾸기

# The archive’s password
t8a168799 = 압축 파일의 비밀번호

# one folder
td571ce35 = 폴더 하나

# one file
t95c9ec7b = 파일 하나

# {location} › {one} · {kind}, so right here
t63f6caf1 = { $location } › { $one } · { $kind }이므로 바로 여기에

# {new_folder} · {counted}, so in a new folder
t2380eff7 = { $new_folder } · { $counted }이므로 새 폴더에

# Extract here
t354232ec = 여기에 압축 풀기

# {picked} selected
t9addc37e = { $picked } 선택됨

# All {counted}
t649feaf8 = 전체 { $counted }

# What should it extract?
td12e23bf = 무엇을 압축 풀까요?

# Smart
t54535972 = 스마트

# Recommended
t7865024c = 추천

# Where should the files go?
td35f540c = 파일을 어디에 풀까요?

# Right here
t900a8a39 = 바로 여기

# Another folder
t9f128ba8 = 다른 폴더

# Choose…
t58daae9c = 선택…

# Goes to {destination}
t8195aed2 = { $destination }에 풀기

# Checked as it goes
tb6c9570b = 풀면서 검사함

# · Nothing leaves the folder or passes the size limit
t11424797 = · 폴더를 벗어나거나 크기 제한을 넘는 항목 없음

# {name} came from the Internet. It will stay marked, so you’ll be asked before it runs.
t60cb2adf = { $name }은(는) 인터넷에서 받은 파일입니다. 표시가 유지되므로 실행하기 전에 확인을 요청합니다.

# Extract archive
t834ba3c0 = 압축 풀기

# Extract
t30518e1e = 압축 풀기

# Open {name}?
tcfe84c2f = { $name }을(를) 여시겠습니까?

# packwing puts a copy in a temporary folder and opens that. Changes to it do not go back in the archive.
t5b9fc10b = packwing이 임시 폴더에 사본을 만들어 엽니다. 사본을 바꿔도 압축 파일에는 반영되지 않습니다.

# This is a program: opening it runs it. Only do so if you trust where the archive came from.
tdc4d48bd = 프로그램입니다. 열면 실행됩니다. 압축 파일의 출처를 신뢰하는 경우에만 여세요.

# Don’t ask again
t00c5d168 = 다시 묻지 않기

# Run it
t72430853 = 실행

# Open
t538b10e9 = 열기

# Open from a storage
t45a8991d = 스토리지에서 열기

# Choose a storage, and an archive in it
t3c900950 = 스토리지와 그 안의 압축 파일을 선택하세요

# Choose
t73c66160 = 선택

# This is not a key file packwing can use: {why}.
tcf6986a7 = packwing에서 사용할 수 있는 키 파일이 아닙니다: { $why }.

# Could not read {path}: {e}.
t1dc6961f = { $path }을(를) 읽을 수 없습니다: { $e }.

# The passphrase
t1e921d00 = 암호 문구

# The key file’s passphrase
t4051f4d7 = 키 파일의 암호 문구

# Enter the passphrase
tbd0672da = 암호 문구 입력

# Passphrase
t191a2dc5 = 암호 문구

# This file is encrypted with age. packwing keeps the passphrase only while the archive is open, and never writes it down.
t663ac047 = 이 파일은 age로 암호화되어 있습니다. packwing은 압축 파일을 열어 둔 동안에만 암호 문구를 보관하며 어디에도 기록하지 않습니다.

# Enter the password
t513ea7ab = 비밀번호 입력

# This archive is encrypted. packwing keeps its password only while the archive is open, and never writes it down.
t0f356f7c = 이 압축 파일은 암호화되어 있습니다. packwing은 압축 파일을 열어 둔 동안에만 비밀번호를 보관하며 어디에도 기록하지 않습니다.

# Choose the key file
t3c521a4e = 키 파일 선택

# Key file
t6ed7c612 = 키 파일

# This file is encrypted with age to a key: an age key file or an SSH key. packwing keeps what it reads from it only while the archive is open, and never writes it down.
tcda9a50a = 이 파일은 age 키(age 키 파일 또는 SSH 키)로 암호화되어 있습니다. packwing은 압축 파일을 열어 둔 동안에만 키에서 읽은 내용을 보관하며 어디에도 기록하지 않습니다.

# Enter the key file’s passphrase
t808b85c5 = 키 파일의 암호 문구 입력

# The key file is itself protected. packwing keeps its passphrase only while the archive is open, and never writes it down.
t2a39d1e8 = 키 파일 자체가 보호되어 있습니다. packwing은 압축 파일을 열어 둔 동안에만 암호 문구를 보관하며 어디에도 기록하지 않습니다.

# The job stopped before it was done.
t57a78885 = 작업이 끝나기 전에 멈췄습니다.

# Waiting for other jobs…
t16ec36d9 = 다른 작업을 기다리는 중…

# A file is damaged
t8b1230ed = 파일이 손상되었습니다

# Extracted, but some files are damaged
t8226d624 = 압축을 풀었지만 일부 파일이 손상되었습니다

# Extracting…
t94279f82 = 압축을 푸는 중…

# Uploading…
tc75fe478 = 업로드하는 중…

# Creating…
t01fcd896 = 만드는 중…

# Adding…
t2060c750 = 추가하는 중…

# Changing the archive…
t186fbef5 = 압축 파일을 변경하는 중…

# Opening…
tc40f2321 = 여는 중…

# Testing…
t19d1cb65 = 테스트하는 중…

# Copying…
t02dd17e0 = 복사하는 중…

# Moving…
t890f7ed1 = 이동하는 중…

# Signing…
t7b8f8780 = 서명하는 중…

# Couldn’t sign
t4489f97c = 서명하지 못했습니다

# Couldn’t copy
t613e4e9c = 복사하지 못했습니다

# Couldn’t move
t9796adec = 이동하지 못했습니다

# Couldn’t extract
tacd22c56 = 압축을 풀지 못했습니다

# Couldn’t create the archive
t9d0e01bc = 압축 파일을 만들지 못했습니다

# Couldn’t add the files
tfbd989b0 = 파일을 추가하지 못했습니다

# Couldn’t change the archive
tcdc4a49c = 압축 파일을 변경하지 못했습니다

# Couldn’t open the file
t041198a4 = 파일을 열지 못했습니다

# The test failed
tfcd070a5 = 테스트에 실패했습니다

# Starts when one of the others ends
tbe2a0509 = 다른 작업 하나가 끝나면 시작합니다

# Paused
tde281feb = 일시 중지됨

# Starting…
t73727cef = 시작하는 중…

# Almost done
ta550a6a9 = 거의 완료

# About {secs} seconds left
t3045da16 = 약 { $secs }초 남음

# About a minute left
tf727815c = 약 1분 남음

# About {div_ceil} minutes left
tbd7a3305 = 약 { $div_ceil }분 남음

# {name} · checking that every file reads
t0d56ec7c = { $name } · 모든 파일을 읽을 수 있는지 확인하는 중

# The test passed
tdf0ce240 = 테스트를 통과했습니다

# {count} file read through, {size}, with no errors. | {count} files read through, {size}, with no errors.
tc1421590 =
    { $n ->
       *[other] 파일 { $count }개({ $size })를 오류 없이 모두 읽었습니다.
    }

# Close
tcd86acc3 = 닫기

# “{path}” is damaged: {why}. Stop, leave it out, or keep what could be read of it?
t145d7a09 = “{ $path }” 파일이 손상되었습니다: { $why }. 중지할까요, 이 파일을 건너뛸까요, 읽은 부분을 남길까요?

# Do the same for the other damaged files
t798693ad = 다른 손상된 파일에도 똑같이 하기

# Stop
t4b7f7705 = 중지

# Leave it out
taaff8ec9 = 건너뛰기

# Keep what was read
tc4387d25 = 읽은 부분 남기기

# kept, {size}
t001f6992 = 남김, { $size }

# left out
t73aeedd4 = 건너뜀

# {count} file’s data was damaged. The rest was extracted. | {count} files’ data was damaged. The rest was extracted.
t9d82f78b =
    { $n ->
       *[other] 파일 { $count }개의 데이터가 손상되었습니다. 나머지는 압축을 풀었습니다.
    }

# and {count} more | and {count} more
te728628b =
    { $n ->
       *[other] 외 { $count }개
    }

# Copy the list
t3c49b269 = 목록 복사

# {count} of {count2} files · {size} of {size2}
t27f1ab7d = 파일 { $count }/{ $count2 }개 · { $size }/{ $size2 }

# Open the folder when it’s done
tf0f1017b = 완료되면 폴더 열기

# Run in background
tb800c0ef = 백그라운드에서 실행

# Resume
t454d1d8a = 다시 시작

# Pause
t44f9bf2d = 일시 중지

# {n} job is still running | {n} jobs are still running
tf2047916 =
    { $n ->
       *[other] 작업 { $n }개가 아직 실행 중입니다
    }

# Closing packwing stops them
t2e6491b3 = packwing을 닫으면 중지됩니다

# What they have not finished stays unfinished: an archive being made is removed, and an extraction stays as far as it got.
tf2a64f8b = 끝나지 않은 작업은 미완성으로 남습니다. 만들던 아카이브는 삭제되고, 압축 해제는 진행된 곳까지 남습니다.

# Keep running
t79c1b9c7 = 계속 실행

# Stop them and quit
tf2707857 = 중지하고 종료

# Its files stay where they are. What packwing signs in to it with is taken out of the system’s keychain.
td68bff45 = 파일은 그대로 남습니다. packwing이 로그인에 쓰는 정보는 시스템 키체인에서 삭제됩니다.

# The folder and its files stay where they are.
t4850aaa8 = 폴더와 파일은 그대로 남습니다.

# Remove “{name}” from the list?
t5dd9cfcf = 목록에서 “{ $name }”을(를) 제거할까요?

# It can be added again at any time.
t80d51a99 = 언제든지 다시 추가할 수 있습니다.

# A passphrase for the new key
tf5593f05 = 새 키의 암호 문구

# The same passphrase again
tad5c08e6 = 암호 문구 다시 입력

# Signing takes a Pro or Enterprise license: Settings › License. Checking signatures is free, on every plan.
t29db17cb = 서명은 Pro 또는 Enterprise 라이선스가 필요합니다: 설정 › 라이선스. 서명 확인은 모든 플랜에서 무료입니다.

# This is one part of a file cut in several. A signature covers one file, and would seem to cover them all, so packwing does not sign parts.
t020be41a = 여러 조각으로 나눈 파일의 한 부분입니다. 서명은 파일 하나에 적용되지만 전체에 적용되는 것처럼 보일 수 있으므로, packwing은 분할 파일에는 서명하지 않습니다.

# That passphrase does not open this key.
tc5357c97 = 이 암호 문구로는 이 키를 열 수 없습니다.

# This is not a key packwing can sign with: {why}.
t662c29f9 = packwing에서 서명에 사용할 수 있는 키가 아닙니다: { $why }.

# The two passphrases are not the same.
t335fe90b = 두 암호 문구가 일치하지 않습니다.

# Keep the key here
t9306dafe = 키를 여기에 보관

# No key chosen
te58c13f5 = 선택한 키 없음

# Key
tcd1ac90c = 키

# New key…
t9e1134da = 새 키…

# Again
t58fb56ef = 다시 입력

# A new key holds Ed25519 and ML-DSA-65 together, so that a signature stands even if one of them falls. Only packwing and hivepack check these: for ssh-keygen, sign with an SSH Ed25519 key. There is no way back into a key whose passphrase is lost.
tbdc850a3 = 새 키는 Ed25519와 ML-DSA-65를 함께 담고 있어, 둘 중 하나가 뚫려도 서명이 유효합니다. 이 서명은 packwing과 hivepack만 확인할 수 있습니다. ssh-keygen으로 확인하려면 SSH Ed25519 키로 서명하세요. 암호 문구를 잃어버린 키는 복구할 방법이 없습니다.

# The signature goes beside it as {signature}, in place of one that is there. All of the file passes through this computer, and your provider may charge for what is read from it.
t5e187bdd = 서명은 { $signature }(으)로 파일 옆에 저장되며, 이미 있는 서명은 대체됩니다. 파일 전체가 이 컴퓨터를 거치므로, 제공업체에서 읽은 양에 따라 요금이 부과될 수 있습니다.

# The signature goes beside it as {signature}, in place of one that is there.
t6035bc3f = 서명은 { $signature }(으)로 파일 옆에 저장되며, 이미 있는 서명은 대체됩니다.

# Sign {name}
ted250f31 = { $name } 서명

# Anyone can check it, with the key’s public half
t04ecc8be = 키의 공개 부분으로 누구나 확인할 수 있습니다

# Make the key…
t6bba28d5 = 키 만들기…

# Sign
t8f6340c4 = 서명

# Move {what}
tc168ca56 = { $what } 이동

# Copy {what}
teda01f26 = { $what } 복사

# Move here
t1aaf57aa = 여기로 이동

# Copy here
t85f0245a = 여기로 복사

# Move
t7f2c2a34 = 이동

# If a name is taken
tc99c037f = 이름이 겹칠 때

# They are in this folder already. Choose another.
t5318c14e = 이미 이 폴더에 있습니다. 다른 폴더를 선택하세요.

# All of it passes through this computer, and your provider may charge for what is read from it. Each moved original goes once its copy has been read back and matches.
t51019905 = 전체가 이 컴퓨터를 거치므로, 제공업체에서 읽은 양에 따라 요금이 부과될 수 있습니다. 이동한 원본은 복사본을 다시 읽어 일치하는 것을 확인한 뒤 삭제됩니다.

# Each original goes once its copy has been read back and matches.
t8ecbab4e = 원본은 복사본을 다시 읽어 일치하는 것을 확인한 뒤 삭제됩니다.

# The originals stay where they are.
t5937f050 = 원본은 그대로 남습니다.

# Too big to show here
t1b339d35 = 너무 커서 여기에 표시할 수 없습니다

# packwing can’t show this kind of file
tb909bf3c = packwing에서 표시할 수 없는 종류의 파일입니다

# It could not be read
tb7927177 = 읽을 수 없습니다

# It could not be read: {why}
t26005fe3 = 읽을 수 없습니다: { $why }

# It could not be shown
tb60fb288 = 표시할 수 없습니다

# Open with default app
t5ef80ec5 = 기본 앱으로 열기

# Another job is making {name} there now: give this one another name, or wait for it to end.
tc8c2bae0 = 다른 작업이 그곳에서 { $name }을(를) 만들고 있습니다. 다른 이름을 지정하거나 끝날 때까지 기다리세요.

# Another job is extracting into {folder} now: choose another folder, or wait for it to end.
t042cbb2a = 다른 작업이 { $folder }에 압축을 풀고 있습니다. 다른 폴더를 선택하거나 끝날 때까지 기다리세요.

# no editor command
tbf2a5d3a = 편집기 명령 없음

# no text editor found
t6256b3da = 텍스트 편집기를 찾을 수 없음

# Search this folder
t2c814039 = 이 폴더에서 검색

# This folder could not be read: {why}
t2c6017b2 = 이 폴더를 읽을 수 없습니다: { $why }

# Could not load more: {why}
t466e87c4 = 더 불러올 수 없습니다: { $why }

# Could not delete: {why}
te5951d38 = 삭제할 수 없습니다: { $why }

# the text editor
t91cdab0c = 텍스트 편집기

# Cannot start {who}: {why}
t8deb160a = { $who }을(를) 시작할 수 없습니다: { $why }

# Edit in text editor
td8d39fb8 = 텍스트 편집기에서 편집

# Edit in {editor}
te44cd57b = { $editor }에서 편집

# Modified
tdd13a6b2 = 수정한 날짜

# Loading…
tc4e2f181 = 불러오는 중…

# Only what is loaded was searched.
t39c5f536 = 불러온 항목만 검색했습니다.

# Search the whole folder
t41f9a787 = 폴더 전체 검색

# Load {n} more
t30bed46a = { $n }개 더 불러오기

# Refresh
t0815aad4 = 새로 고침

# Hide hidden files
t3375c9c0 = 숨김 파일 숨기기

# Show hidden files
tf18c7057 = 숨김 파일 표시

# View
t5e4baa18 = 보기

# Extract…
t4a09490e = 압축 풀기…

# Compress…
t690f9553 = 압축…

# Copy to…
t543cf15b = 복사…

# Move to…
t7a32320b = 이동…

# Sign…
td6d61b10 = 서명…

# Checksums…
t2891102d = 체크섬…

# Copy path
tf165cb2b = 경로 복사

# Compress {n} item… | Compress {n} items…
tad1b72f6 =
    { $n ->
       *[other] 항목 { $n }개 압축…
    }

# {n} loaded, more in this folder
t29a52578 = { $n }개 불러옴, 이 폴더에 더 있음

# Sorted among those loaded
t037d01ae = 불러온 항목 안에서 정렬됨

# {n} selected
tebda8468 = { $n }개 선택됨

# Try again
t213e90fa = 다시 시도

# Nothing here matches that.
t0f9a8a58 = 일치하는 항목이 없습니다.

# Add a storage to browse it.
t0b82c83a = 탐색하려면 스토리지를 추가하세요.

# No folders or archives here.
t6a090cc3 = 폴더나 압축 파일이 없습니다.

# No folders here.
tc7731c8f = 폴더가 없습니다.

# This folder is empty.
td9e47db6 = 이 폴더는 비어 있습니다.

# Show in Finder
ta9442aa9 = Finder에서 보기

# Show in File Explorer
t23c89eda = 파일 탐색기에서 보기

# Show in the file manager
tb347ece9 = 파일 관리자에서 보기

# Open in Finder
te2a467e0 = Finder에서 열기

# Open in File Explorer
t3794d625 = 파일 탐색기에서 열기

# Open in the file manager
td1ceabc4 = 파일 관리자에서 열기

# Copy all
t6aef5c37 = 모두 복사

# This system has no folder for settings.
tfd0ff0a1 = 이 시스템에는 설정 폴더가 없습니다.

# Archive
t73690b65 = 압축 파일

# Search inside this archive
tb35c0c27 = 이 압축 파일 안에서 검색

# Not text it can show
ta413eca5 = 표시할 수 있는 텍스트가 아닙니다

# It needs the password to show it
t81f5dbce = 표시하려면 비밀번호가 필요합니다

# Open another…
tff74a552 = 다른 파일 열기…

# From a storage…
tcfba8fd5 = 스토리지에서…

# Safe to extract
t3c93bb70 = 안전하게 압축을 풀 수 있음

# Read {fetched} of {whole}
t613dd932 = { $whole } 중 { $fetched } 읽음

# Add files
tdd45c7bb = 파일 추가

# Test
t2ffcbe05 = 테스트

# Nothing in this archive matches “{query}”.
t36c3e2f5 = 이 압축 파일에서 “{ $query }”와(과) 일치하는 항목이 없습니다.

# Nothing in this folder matches “{query}”.
teecdfa3d = 이 폴더에서 “{ $query }”와(과) 일치하는 항목이 없습니다.

# And {more} more. Type more to narrow it down.
t843253d0 = 그 외 { $more }개가 더 있습니다. 더 입력하여 좁혀 보세요.

# Reading what’s inside…
t1bbda9f5 = 내용을 읽는 중…

# Open an archive
t8382bd3e = 압축 파일 열기

# ZIP, 7z and tar archives, disc images and packages, compressed or encrypted with age: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…
t4a498ac8 = ZIP, 7z, tar 압축 파일, 디스크 이미지, 패키지, age로 압축하거나 암호화한 파일: .zip, .7z, .tar.gz, .tar.zst, .iso, .rpm, .pkg, .zst.age…

# Open an archive…
t2cf2a8ee = 압축 파일 열기…

# Choose a file to see it here
t37854216 = 파일을 선택하면 여기에 표시됩니다

# {count} executable in this folder will keep its Internet mark when extracted. | {count} executables in this folder will keep their Internet mark when extracted.
td7688f56 =
    { $n ->
       *[other] 이 폴더의 실행 파일 { $count }개는 압축을 풀어도 인터넷 표시가 유지됩니다.
    }

# This computer
tfa0d176a = 이 컴퓨터

# Home
t52f50eae = 홈

# Desktop
t64215c53 = 바탕 화면

# Downloads
t539307de = 다운로드

# Drop to open
tb255877b = 놓아서 열기

# Browse
t0eac1f07 = 찾아보기

# Recent
t50e888fe = 최근

# Choose a file to see its information here
t50aac02b = 파일을 선택하면 정보가 여기에 표시됩니다

# {n} selected | {n} selected
tbb5a2d5f =
    { $n ->
       *[other] { $n }개 선택됨
    }

# Files in them come to {size}
t0cca93f1 = 그 안의 파일은 모두 { $size }입니다

# Delete…
t2fed07aa = 삭제…

# Link to a folder
t4b408baf = 폴더 링크

# Link
t8e08e589 = 링크

# {tag} file
ta56bbbe9 = { $tag } 파일

# {files}, {folders}
t72e46dc5 = { $files }, { $folders }

# {n} file | {n} files
teee8d81a =
    { $n ->
       *[other] 파일 { $n }개
    }

# {n} folder | {n} folders
te89ea942 =
    { $n ->
       *[other] 폴더 { $n }개
    }

# {size} so far
tc87daf59 = 지금까지 { $size }

# Kind
t3ff50c63 = 종류

# Holds
tffe19c31 = 내용

# Permissions
t8700fd83 = 권한

# Path
teb66e456 = 경로

# Signature
tc049050d = 서명

# Count size and files
t0f23db17 = 크기와 파일 수 세기

# {n} folder in it could not be read. | {n} folders in it could not be read.
t6d165412 =
    { $n ->
       *[other] 안에 있는 폴더 { $n }개를 읽을 수 없습니다.
    }

# Check the signature
t9d44730e = 서명 확인

# Extracted
t3250471f = 압축 해제됨

# Archive created
t22c69419 = 아카이브 생성됨

# Moved
t82869bf0 = 이동됨

# Signed
t27c3ab35 = 서명됨

# Waiting for a password or a key
t0e83ae32 = 암호 또는 키를 기다리는 중

# Enter it…
tddb7844a = 입력…

# Show
t29742e5c = 보기

# Jobs
t4ae6a79d = 작업

# Clear finished
t30c178e8 = 완료된 항목 지우기

# Nothing runs in the background.
t2573f39e = 백그라운드에서 실행 중인 작업이 없습니다.

# Add to the right-click menu
t116d3868 = 마우스 오른쪽 버튼 메뉴에 추가

# Extract or compress from any folder.
t20864983 = 어느 폴더에서나 압축을 풀거나 압축할 수 있습니다.

# Welcome to packwing
tb38a86a6 = packwing에 오신 것을 환영합니다

# Open any archive, make smaller files to share, and stay safe from harmful downloads.
t0265387b = 어떤 압축 파일이든 열고, 공유하기 좋게 파일을 줄이고, 위험한 다운로드로부터 안전하게 지키세요.

# Set things up
t63f9c1cd = 설정하기

# You can change these later in Settings.
t95f41ffb = 나중에 설정에서 바꿀 수 있습니다.

# Get started
te83ca5ee = 시작하기

# Remove from Recent
t26cc8dba = 최근 항목에서 제거

# {location} · not there any more
t7d0c09c7 = { $location } · 더 이상 없음

# The archives you open will show here.
tc0cc82d5 = 연 압축 파일이 여기에 표시됩니다.

# Copy version info
t3d37483c = 버전 정보 복사

# Version {version} · {os} {arch}
t6bead5ae = 버전 { $version } · { $os } { $arch }

# A file archiver that opens archives safely.
t3c244df9 = 압축 파일을 안전하게 여는 파일 압축 프로그램입니다.

# Updates
t483075a5 = 업데이트

# This is a beta: you are told of newer betas, and of the release that follows them.
t1d6a96bc = 베타 버전입니다: 새 베타 버전과 그 뒤에 나오는 정식 버전의 알림을 받습니다.

# This is a release: you are told of newer releases, not of betas.
t9d2d8250 = 정식 버전입니다: 새 정식 버전의 알림을 받으며, 베타 버전 알림은 받지 않습니다.

# Check now
t4f0e0677 = 지금 확인

# Look for updates once a day
tcc27ace1 = 하루에 한 번 업데이트 확인

# It reads the list of releases from GitHub, and believes it only when its signatures verify. Nothing is downloaded or installed without you.
t258cdfd9 = GitHub에서 릴리스 목록을 읽고, 서명이 검증될 때만 신뢰합니다. 사용자 동의 없이는 아무것도 다운로드하거나 설치하지 않습니다.

# Skip this version
t2c33d756 = 이 버전 건너뛰기

# This build has no release keys, so it cannot tell of updates.
t378182cb = 이 빌드에는 릴리스 키가 없어 업데이트를 알릴 수 없습니다.

# packwing {version} is ready
tf02650f8 = packwing { $version } 사용 준비 완료

# It is installed when you quit, or now with Restart.
t6fecac86 = 종료할 때 설치되며, 다시 시작을 누르면 지금 설치됩니다.

# Restart now
t54001a7c = 지금 다시 시작

# packwing {version}
t20bb8a0d = packwing { $version }

# Looking for a newer version…
tf1905fe9 = 새 버전을 확인하는 중…

# Release notes
tc52ab62f = 릴리스 노트

# packwing {version} is out
t36a30d8b = packwing { $version } 출시

# Run these in a terminal. They check the download against the signed list of releases, and sudo asks for your password.
t98b9753c = 터미널에서 이 명령을 실행하세요. 서명된 릴리스 목록과 대조해 다운로드를 확인하며, sudo가 비밀번호를 묻습니다.

# {by} updates it: {command}
tdf2cab9c = { $by }에서 업데이트합니다: { $command }

# {by} updates it.
t84b27295 = { $by }에서 업데이트합니다.

# It is downloaded and checked, then installed when you quit.
tb46c0678 = 다운로드하고 확인한 뒤, 종료할 때 설치합니다.

# Download it, and install it in place of this one.
t39312eae = 다운로드한 뒤 현재 버전 대신 설치하세요.

# The download failed: {why}
t5e96c075 = 다운로드 실패: { $why }

# Download and install
t1a99173f = 다운로드 및 설치

# Download
t6964bb19 = 다운로드

# The last look failed: {why}
td122a092 = 마지막 확인에 실패했습니다: { $why }

# Up to date (looked {when}).
t4665ef60 = 최신 버전입니다({ $when } 확인).

# Not looked yet.
t5a3ba058 = 아직 확인하지 않았습니다.

# Free software
t731220a4 = 자유 소프트웨어

# Under the GNU General Public License, version 3 or later
t4e98bf3c = GNU 일반 공중 사용 허가서 버전 3 이상에 따름

# View license
tfc21d38b = 라이선스 보기

# Made by HivePaaS
t10c7007a = HivePaaS 제작

# On hivepack, our library for reading and writing archives
tfae09b61 = 압축 파일을 읽고 쓰는 라이브러리 hivepack 기반

# Source code
tf6a49f4d = 소스 코드

# Something wrong?
t610dfb6f = 문제가 있나요?

# Tell us, with the version above
ta3e29034 = 위의 버전과 함께 알려 주세요

# Report a problem
t7b1728e7 = 문제 신고

# What changed
te8345543 = 변경 내용

# Every release, and what is new in it
t8e6f2052 = 모든 릴리스와 새로운 기능

# © {year} HivePaaS. The packwing name and logo are trademarks of HivePaaS.
t11d91ba2 = © { $year } HivePaaS. packwing 이름과 로고는 HivePaaS의 상표입니다.

# More from HivePaaS
tb25cf729 = HivePaaS의 다른 제품

# Free and open source, like packwing.
tc1ece130 = packwing처럼 무료 오픈 소스입니다.

# A fast, native SSH client for Linux, macOS and Windows.
t0f4c758a = Linux, macOS, Windows용 빠른 네이티브 SSH 클라이언트입니다.

# SSH and SFTP
t0f8af34d = SSH와 SFTP

# Follows your ssh_config
tc1c4446d = ssh_config 지원

# A real terminal
tae9bcfa8 = 진짜 터미널

# GPL-3.0-or-later
t8622a5c1 = GPL-3.0-or-later

# Visit shellwing.app
t1d2ed540 = shellwing.app 방문

# A self-hosted platform for your apps, built on Docker Swarm: an open-source alternative to Heroku, Render and Coolify.
tc9d13419 = Docker Swarm 기반으로 앱을 직접 호스팅하는 플랫폼입니다. Heroku, Render, Coolify의 오픈 소스 대안입니다.

# 300+ one-click apps
t6a770b0a = 원클릭 앱 300개 이상

# Deploy on push
tb0bfc3e9 = push 시 자동 배포

# Pull request previews
t4e4180fc = 풀 리퀘스트 미리보기

# Automatic TLS
t9e9776ed = 자동 TLS

# Live logs and metrics
tebe3a172 = 실시간 로그와 메트릭

# Apache-2.0
t0274f036 = Apache-2.0

# Visit hivepaas.com
t0632975a = hivepaas.com 방문

# Always
tad4bbd04 = 항상

# Extensions, apart by spaces
tc709375f = 확장자, 공백으로 구분

# Format of new archives
t3b50d238 = 새 압축 파일의 형식

# New archive starts with it; you can pick another there.
t7b361b55 = 새 압축 파일은 이 형식으로 시작하며, 만들 때 다른 형식을 선택할 수 있습니다.

# Store puts the files of ZIP and 7z archives in as they are. tar formats take Lowest, as .tar alone has no compression.
t2595cbcb = 저장만은 ZIP과 7z 압축 파일의 파일을 그대로 넣습니다. tar 형식은 최저를 씁니다. 압축이 없는 것은 .tar뿐입니다.

# Higher makes smaller archives, which take longer to make, not to open.
t9184e778 = 높을수록 압축 파일이 작아지며, 만드는 시간이 길어지지만 여는 시간은 그대로입니다.

# Write straight to a storage above
ta4975128 = 위의 스토리지에 바로 쓰기

# Larger files take no room on this computer, but a lost connection means compressing again. S3 takes tar and ZIP this way, SFTP every format too.
t2b8f2459 = 큰 파일이 이 컴퓨터의 공간을 차지하지 않지만, 연결이 끊기면 처음부터 다시 압축해야 합니다. S3는 tar와 ZIP, SFTP는 모든 형식에서 이 방식을 사용할 수 있습니다.

# Don’t compress what already is
t940ec31d = 이미 압축된 것은 압축하지 않기

# For ZIP and 7z: pictures, video, sound and archives go in as they are, sooner and no larger
t4de1e81e = ZIP과 7z: 사진, 동영상, 오디오, 압축 파일을 그대로 넣어 더 빠르고 크기도 커지지 않습니다

# What goes in as it is
tc0104b29 = 그대로 넣을 파일

# Files whose names end in these, whatever their case
ta3acef9b = 이름이 이것으로 끝나는 파일(대소문자 무관)

# Reset to default
t0db9f7dc = 기본값으로 되돌리기

# Leave out Mac-only files
t27eec14f = Mac 전용 파일 제외

# .DS_Store, __MACOSX and resource forks, clutter anywhere but a Mac
t3bdd25aa = .DS_Store, __MACOSX, 리소스 포크: Mac 외에서는 불필요한 파일입니다

# Hide file names behind the password
t93ff7715 = 비밀번호로 파일 이름 숨기기

# For 7z: nobody sees what is inside without it
t0f5354d7 = 7z용: 비밀번호 없이는 안에 무엇이 있는지 볼 수 없습니다

# No limit
t6de7f47d = 제한 없음

# Where the files go
t77e60aba = 파일을 풀 위치

# Where Extract starts: you can pick another there.
t4b73082f = 압축 풀기가 시작되는 위치입니다. 풀 때 다른 위치를 선택할 수 있습니다.

# One item right here, more in a new folder
ta5a4d31d = 항목이 하나면 바로 여기에, 더 많으면 새 폴더에

# A new folder next to it
t2095cbe9 = 옆에 새 폴더

# Named after the archive
tb991d4c1 = 압축 파일 이름으로

# Next to the archive
t4691c928 = 압축 파일 옆에

# Straight into its folder
t99d1df43 = 해당 폴더에 바로

# When a file is already there
tca514dda = 이미 파일이 있을 때

# A file of the same name in the folder the files go to
t296ad686 = 파일을 풀 폴더에 같은 이름의 파일이 있을 때

# Largest an archive may unpack to
t5957c426 = 압축을 풀 수 있는 최대 크기

# Stops archive bombs: small files that unpack to fill the disk
t87a41242 = 작은 파일이 풀리면서 디스크를 가득 채우는 압축 폭탄을 막습니다

# Light
t8ad2f8cf = 라이트

# Dark
t45570065 = 다크

# System
t8f3184bc = 시스템

# Same as system ({language})
t5bd525d3 = 시스템과 동일 ({ $language })

# The system’s
t1d68c60b = 시스템 설정

# Custom…
ta391dade = 사용자 지정…

# The system’s ({size} px)
ta51f1877 = 시스템 설정({ $size } px)

# e.g. code -n, or /path/to/editor --flag
t455dc834 = 예: code -n 또는 /path/to/editor --flag

# Show packwing in the right-click menu
t5d0d23cc = 마우스 오른쪽 버튼 메뉴에 packwing 표시

# Extract and compress from any folder
t94a46acb = 어느 폴더에서나 압축을 풀고 압축할 수 있습니다

# Language
t9a73db9b = 언어

# Appearance
t31018b9f = 모양

# Text size
t2c041143 = 글자 크기

# Text editor
t1ca82aab = 텍스트 편집기

# What Edit opens a file of a storage in
t62506ced = 스토리지의 파일을 편집할 때 여는 프로그램

# The file’s path is added at the end.
teba749d4 = 파일 경로가 끝에 추가됩니다.

# Paste your license key
t3c4da5cf = 라이선스 키를 붙여넣으세요

# Every feature but signing files, and {count} storage across a network. Folders on this computer are not limited, and any signature can be checked.
tf9725115 = 파일 서명을 제외한 모든 기능과 네트워크 너머의 스토리지 { $count }개를 사용할 수 있습니다. 이 컴퓨터의 폴더에는 제한이 없으며, 모든 서명을 확인할 수 있습니다.

# Every feature, and any number of storages.
tc3e75ecf = 모든 기능과 무제한 스토리지를 사용할 수 있습니다.

# Every feature, and any number of storages, for your organization.
t69b29f66 = 조직을 위한 모든 기능과 무제한 스토리지를 사용할 수 있습니다.

# Plan
t1eab5e32 = 플랜

# Issued to
t44ef5cad = 발급 대상

# Who this license is for
td514fb0d = 이 라이선스를 사용하는 대상

# Seats
t8f706321 = 사용자 수

# What your organization’s agreement covers
t60b0c6c9 = 조직의 계약에 포함된 내용

# Trial, until {date}
t9ffc2b63 = 체험판, { $date }까지

# Until {date}
t4d9bac91 = { $date }까지

# Ended {date}. It keeps working for {days} days more.
t7b9eb499 = { $date }에 종료되었습니다. { $days }일 더 계속 사용할 수 있습니다.

# Ended {date}
tfccb9305 = { $date }에 종료되었습니다

# Valid
td2661011 = 유효함

# Past this, packwing is Free. Nothing is taken away but adding storages.
td7098137 = 이 기간이 지나면 packwing은 Free가 됩니다. 스토리지 추가만 제한되고 사라지는 기능은 없습니다.

# Your organization manages this license.
te42a0ee6 = 조직에서 이 라이선스를 관리합니다.

# Activate
t3b8f0190 = 활성화

# A key is checked on this computer. Nothing is sent anywhere, and no license changes what your organization’s policies do.
t9e6ecd3d = 키는 이 컴퓨터에서 확인합니다. 어디로도 전송되지 않으며, 라이선스가 조직 정책의 동작을 바꾸지 않습니다.

# On
t60e47df0 = 켜짐

# They show when you right-click files, folders and archives.
te216d656 = 파일, 폴더, 압축 파일을 마우스 오른쪽 버튼으로 클릭하면 표시됩니다.

# packwing is not in the menu now: these go in when it is.
t3ddc5e66 = 지금은 메뉴에 packwing이 없습니다. 추가하면 이 항목들이 들어갑니다.

# Changing the menu…
tc7f1a2f0 = 메뉴를 바꾸는 중…

# Good to know
t8d7b3a90 = 알아 두세요

# The menu could not be changed: {why}
t9fac5c06 = 메뉴를 바꿀 수 없습니다: { $why }

# Items in the menu
t67537dc4 = 메뉴 항목

# Put them in a submenu
te3449a65 = 하위 메뉴에 넣기

# Windows 11 puts them under one Packwing item, in its menu and under Show more options
t7730ec1a = Windows 11은 메뉴와 ‘더 많은 옵션 표시’ 모두에서 이 항목들을 하나의 Packwing 항목 아래에 둡니다

# One packwing item, with these under it
tb5fdc584 = packwing 항목 하나 아래에 이 항목들을 모읍니다

# Keep the “downloaded from the Internet” mark
tfe574fa2 = “인터넷에서 받음” 표시 유지

# The system warns before running files that came from a downloaded archive
t9fbe84c0 = 다운로드한 압축 파일에서 나온 파일을 실행하기 전에 시스템이 경고합니다

# This system has no such mark, so this changes nothing here
t314d60c1 = 이 시스템에는 이런 표시가 없으므로 여기서는 아무것도 바뀌지 않습니다

# License
t38e70ee4 = 라이선스

# Your organization’s, in Settings › License
tff7c594e = 조직의 설정: 설정 › 라이선스

# Treat files from storages as from the Internet
t0af8d466 = 스토리지의 파일을 인터넷에서 받은 것으로 처리

# What comes from S3, WebDAV and SFTP gets the mark too
tf02df9d4 = S3, WebDAV, SFTP에서 가져온 파일에도 표시를 붙입니다

# Require encryption for new archives
t0c3e1337 = 새 압축 파일에 암호화 필수

# Applies to archives made and files added here, not to other apps
tc889f3a2 = 여기서 만든 압축 파일과 추가한 파일에 적용되며, 다른 앱에는 적용되지 않습니다

# Kinds of encryption allowed
t1b8b2116 = 허용되는 암호화 방식

# New archives can be encrypted only these ways
t71dcd29c = 새 압축 파일은 이 방식으로만 암호화할 수 있습니다

# Minimum password length
tb7d64a44 = 최소 비밀번호 길이

# In characters, for every new archive with a password
tca6d6982 = 비밀번호가 있는 모든 새 압축 파일에 적용되며, 글자 수 기준입니다

# packwing has no policy of this name.
t7e03649c = packwing에는 이 이름의 정책이 없습니다.

# Ask before opening files without extracting
t76d5d951 = 압축을 풀지 않고 파일을 열기 전에 확인

# Opening a file inside an archive puts a copy in a temporary folder. A program always asks.
t9173caaa = 압축 파일 안의 파일을 열면 임시 폴더에 사본을 만듭니다. 프로그램은 항상 확인을 요청합니다.

# Paste a public key
t0bf72f73 = 공개 키 붙여넣기

# This is not a public key packwing can check with: {why}.
td57933b9 = packwing에서 확인에 사용할 수 있는 공개 키가 아닙니다: { $why }.

# This line cannot be read, so the key is not used
t90e7dd7b = 이 줄은 읽을 수 없으므로 이 키는 사용하지 않습니다

# Keys you trust
ted99ced8 = 신뢰하는 키

# A signature by one of these says who signed a file. One by any other key proves nothing, whatever it matches.
td25843b8 = 이 중 하나의 서명은 누가 파일에 서명했는지 알려 줍니다. 다른 키의 서명은 일치하더라도 아무것도 증명하지 않습니다.

# Choose a file…
t28a576c9 = 파일 선택…

# {name} (weak)
tdb06c037 = { $name } (약함)

# Checksums
t47dd48d3 = 체크섬

# What Checksums… in a file’s menu shows, to compare with the one given where the file came from. A weak one still tells a damaged file, but not one changed on purpose.
t6edeb650 = 파일 메뉴의 체크섬…에 표시되는 항목으로, 파일을 받은 곳에서 제공한 체크섬과 비교할 수 있습니다. 약한 체크섬은 손상된 파일은 알아내지만, 일부러 바꾼 파일은 알아내지 못합니다.

# Its policies cannot be read, so what they guard refuses: {why}
t7b4eb9c1 = 정책을 읽을 수 없어 정책이 지키는 항목은 거부됩니다: { $why }

# Some settings are managed by your organization
t7db7e7ec = 일부 설정은 조직에서 관리합니다

# Get help
t4a1ed54a = 도움말

# Not understood
t068eea1b = 이해할 수 없음

# Off
t0dc6b4aa = 끔

# General
t5561fd0b = 일반

# Extracting
tc82af3f2 = 압축 풀기

# Compressing
t6d183c73 = 압축

# Security
tc2fe21db = 보안

# Right-click menu
t1287ae97 = 마우스 오른쪽 버튼 메뉴

# About
t58e096de = 정보

# Settings
t4b058728 = 설정

# This computer has no way to ask that it is you.
t0c1b4d43 = 이 컴퓨터에는 본인 확인을 요청할 방법이 없습니다.

# open a storage
t780455bf = 스토리지 열기

# Open a storage in packwing
tf861e014 = packwing에서 스토리지 열기

# That was not it. It stays locked.
tbb2a9838 = 올바르지 않습니다. 잠긴 상태로 유지됩니다.

# The system could not ask: {why}
td6905afe = 시스템에서 요청하지 못했습니다: { $why }

# This copy of packwing cannot open it.
tfb56d8cf = 이 packwing으로는 열 수 없습니다.

# Storages
tc776c52f = 스토리지

# Add storage
te858d6b5 = 스토리지 추가

# Type the password you log in to this computer with to open it.
tb544cd9b = 열려면 이 컴퓨터에 로그인할 때 쓰는 비밀번호를 입력하세요.

# Show this computer it is you to open it.
tfb4c635b = 열려면 이 컴퓨터에 본인임을 확인하세요.

# Checking…
t87f62cb7 = 확인 중…

# It asks again after 5 minutes left unused.
t69c22a3b = 5분간 사용하지 않으면 다시 묻습니다.

# Edit…
t8e7efcd7 = 편집…

# Remove from this list
t8c7c7c57 = 목록에서 제거

# Extract here with packwing
tc5e54e4a = packwing으로 여기에 압축 풀기

# Extract to new folder
tda62691f = 새 폴더에 압축 풀기

# Extract with packwing…
tda9255a0 = packwing으로 압축 풀기…

# Open with packwing
teb657427 = packwing으로 열기

# Compress to ZIP
t344b1175 = ZIP으로 압축

# Compress with packwing…
t9ec9b77d = packwing으로 압축…

# packwing cannot find its program: {e}
t1934ce08 = packwing이 프로그램을 찾을 수 없습니다: { $e }

# Signed by {signer}
t8eb35b10 = { $signer }의 서명

# Signed by {signer}, not checked yet
t88b7186a = { $signer }의 서명, 아직 확인하지 않음

# Signed by a key you do not trust: {fingerprint}
tec0df1c3 = 신뢰하지 않는 키의 서명: { $fingerprint }

# It does not match its signature
tb7d0e1ed = 서명과 일치하지 않습니다

# Its signature cannot be read
tadf2a401 = 서명을 읽을 수 없습니다

# Its signature could not be checked
t69cbbb51 = 서명을 확인할 수 없습니다

# Checking its signature…
tda8c2e03 = 서명을 확인하는 중…

# the system's package manager
t11465c20 = 시스템 패키지 관리자

# the App Store
ta8cbd72e = App Store

# the Microsoft Store
t1df799a1 = Microsoft Store

# Downloading… {got} of {total}
taee668c2 = 다운로드 중… { $got } / { $total }

# Downloading… {got}
t4873b921 = 다운로드 중… { $got }

# just now
ta715ef0f = 방금

# {n} minute ago | {n} minutes ago
t6c7302d6 = { $n }분 전

# {n} hour ago | {n} hours ago
t137b052e = { $n }시간 전

# {n} day ago | {n} days ago
tb9fa7744 = { $n }일 전

# No folder to download to.
t2bfc1370 = 다운로드할 폴더가 없습니다.

# Update available
t676409df = 업데이트 가능

# Restart to update
tbbbbda26 = 다시 시작하여 업데이트

# {n} file was skipped, as its name was taken. | {n} files were skipped, as their names were taken.
t843cf8f1 =
    { $n ->
       *[other] 이름이 겹쳐 파일 { $n }개를 건너뛰었습니다.
    }

# {n} link stayed where it was. | {n} links stayed where they were.
tbdeb1b33 =
    { $n ->
       *[other] 링크 { $n }개는 원래 자리에 그대로 두었습니다.
    }

# This file is encrypted with a passphrase.
t23ab4fa0 = 이 파일은 암호 문구로 암호화되어 있습니다.

# This archive needs its password.
td46b69a1 = 이 압축 파일은 비밀번호가 필요합니다.

# This file is encrypted to a key.
tbd048b4e = 이 파일은 키로 암호화되어 있습니다.

# This key file needs its passphrase.
t81f4c775 = 이 키 파일은 암호 문구가 필요합니다.

# {paths} archives
t27f3b860 = 압축 파일 { $paths }개

# {folder}, a folder for each
t75e7db7c = { $folder }, 각각 폴더 하나씩

# New
t8b242db1 = 새로 만들기

# There is no file to save it beside.
ta9906e3b = 옆에 저장할 파일이 없습니다.

# Nothing to compress.
t47339885 = 압축할 항목이 없습니다.

# {path} is too big.
te9ab879e = { $path }이(가) 너무 큽니다.

# This format holds one file, and no more.
t75c197d9 = 이 형식은 파일 하나만 담을 수 있습니다.

# Could not read {path}: {e}
tad0120eb = { $path }을(를) 읽을 수 없습니다: { $e }

# {path} has no name.
tbd0e16dc = { $path }에는 이름이 없습니다.

# The name of {path} is not valid Unicode, so it cannot go in an archive.
t2588efb5 = { $path }의 이름이 올바른 유니코드가 아니어서 압축 파일에 넣을 수 없습니다.

# Could not write {path}: {e}
t8dd055c6 = { $path }에 쓸 수 없습니다: { $e }

# {wanted} and the names after it are taken.
t40554bf6 = { $wanted }와(과) 그 뒤의 이름은 이미 사용 중입니다.

# Could not encrypt: {e}
t485d71ab = 암호화할 수 없습니다: { $e }

# A ZIP or 7z archive takes age keys only when age encrypts it whole.
t582284c9 = ZIP 또는 7z 압축 파일은 age로 전체를 암호화할 때만 age 키를 쓸 수 있습니다.

# age encrypts to a passphrase or to keys, not to both.
t83582c17 = age는 암호 문구 또는 키 중 하나로만 암호화할 수 있으며, 둘을 함께 쓸 수 없습니다.

# Post-quantum keys can’t be used with other keys: the others would leave the file open to a quantum computer.
t500313af = 양자 내성 키는 다른 키와 함께 쓸 수 없습니다. 다른 키 때문에 파일이 양자 컴퓨터에 노출될 수 있습니다.

# There are no keys in this file to encrypt to.
tc2809b2c = 이 파일에는 암호화에 쓸 키가 없습니다.

# {path} got shorter while it was being read.
t2e221d1f = { $path }이(가) 읽는 도중 작아졌습니다.

# A password has to be valid Unicode.
t5ef729bc = 비밀번호는 올바른 유니코드여야 합니다.

# Could not start compressing: {e}
tc4faa70c = 압축을 시작할 수 없습니다: { $e }

# Could not compress: {e}
tf6ff23f2 = 압축할 수 없습니다: { $e }

# {path} is a link, which is kept only on Unix.
t030316d9 = { $path }은(는) 링크이며, 링크는 Unix에서만 보존됩니다.

# Nothing changed.
tedfc294c = 변경된 것이 없습니다.

# The archive did not come out whole.
t2f015c79 = 압축 파일이 온전하게 만들어지지 않았습니다.

# packwing can change only a ZIP archive that is one file.
t48628036 = packwing은 파일 하나로 된 ZIP 압축 파일만 변경할 수 있습니다.

# That is not in the archive.
t7530c419 = 압축 파일에 없는 항목입니다.

# That is all the archive holds. Delete the archive itself instead.
t495f94b1 = 압축 파일에 있는 전부입니다. 대신 압축 파일 자체를 삭제하세요.

# “{to}” cannot be a name.
td53e55bf = “{ $to }”은(는) 이름으로 쓸 수 없습니다.

# “{to}” is already in that folder. Nothing was changed.
t5516dd83 = “{ $to }”이(가) 이미 그 폴더에 있습니다. 변경된 것이 없습니다.

# packwing can add files only to a ZIP archive that is one file.
t11a330d3 = packwing은 파일 하나로 된 ZIP 압축 파일에만 파일을 추가할 수 있습니다.

# An archive can’t go inside itself.
tff43ce74 = 압축 파일을 자기 자신 안에 넣을 수 없습니다.

# “{bare}” is already in the archive. Nothing was changed.
t2c3e91c4 = “{ $bare }”이(가) 이미 압축 파일에 있습니다. 변경된 것이 없습니다.

# Before it is uploaded, this archive is made in {folder}, where it may take up to {need}. Only {free} is free there.
tdb9326d3 = 업로드하기 전에 이 압축 파일을 { $folder }에 만드는데, 최대 { $need }가 필요할 수 있습니다. 그곳의 여유 공간은 { $free }뿐입니다.

# The upload stopped.
t528232e6 = 업로드가 중단되었습니다.

# {name} is already in the folder.
tbe7fab8a = { $name }이(가) 이미 폴더에 있습니다.

# Could not make a folder in {parent}.
tec7ca3a9 = { $parent }에 폴더를 만들 수 없습니다.

# {to} is a folder, and a file would replace it.
tf606b3b3 = { $to }은(는) 폴더이며, 파일로 대체될 수 있습니다.

# {name} is already in the folder
t5af584a0 = { $name }이(가) 이미 폴더에 있습니다

# This file is not an archive packwing can open.
t660f149b = packwing에서 열 수 있는 압축 파일이 아닙니다.

# The password is wrong, or the archive is damaged.
ta4d3227d = 비밀번호가 올바르지 않거나 압축 파일이 손상되었습니다.

# The passphrase is wrong.
t7826c649 = 암호 문구가 올바르지 않습니다.

# The key file’s passphrase is wrong.
t8ad75e3a = 키 파일의 암호 문구가 올바르지 않습니다.

# This file is encrypted to a key packwing does not have.
te162af76 = 이 파일은 packwing에 없는 키로 암호화되어 있습니다.

# Cancelled.
t27c010d6 = 취소되었습니다.

# the archive could not be read
t2267c3b1 = 압축 파일을 읽을 수 없습니다

# This can’t be extracted, as a file’s name can’t be used here: “{name}” ({why}).
tcf18f6e9 = 파일 이름을 여기서 사용할 수 없어 압축을 풀 수 없습니다: “{ $name }” ({ $why }).

# {n} byte | {n} bytes
te8f231ba =
    { $n ->
       *[other] { $n }바이트
    }

# A volume of this archive is not there.
t3803103c = 이 압축 파일의 분할 파일 중 하나가 없습니다.

# {named}: its data is damaged.
t066227f9 = { $named }: 데이터가 손상되었습니다.

# The archive is damaged: UnRAR would exit with {number}.
t7923a417 = 압축 파일이 손상되었습니다. UnRAR가 종료 코드 { $number }(으)로 끝났을 것입니다.

# There is no public key here.
t9fd24078 = 여기에는 공개 키가 없습니다.

# Could not make the key: {e}
t73610ecc = 키를 만들 수 없습니다: { $e }

# There is no file to sign.
tc95e3a9c = 서명할 파일이 없습니다.

# {name} is not there any more.
t07c1f2bc = { $name }이(가) 더 이상 없습니다.

# The name is empty.
t1a9d8e4e = 이름이 비어 있습니다.

# That name is kept for the system.
t74c8683a = 시스템에서 사용하는 이름입니다.

# A name cannot hold / or other special characters.
t4c0596ec = 이름에는 /나 다른 특수 문자를 쓸 수 없습니다.

# On Windows a name cannot end with a dot or a space.
tf16ba7f1 = Windows에서는 이름이 마침표나 공백으로 끝날 수 없습니다.

# That name is taken in this folder.
t43a2bce1 = 이 폴더에 이미 있는 이름입니다.

# {name} is already there.
tb5e7b134 = { $name }이(가) 이미 있습니다.

# {name} cannot go inside itself.
t99f437a6 = { $name }은(는) 자기 자신 안에 넣을 수 없습니다.

# Stopped.
t15558940 = 중지되었습니다.

# {name} is a file in one place and a folder in the other.
t02a45af5 = { $name }은(는) 한쪽에서는 파일이고 다른 쪽에서는 폴더입니다.

# The copy of {name} does not match. The original was kept.
t779a64f3 = { $name }의 사본이 일치하지 않습니다. 원본은 그대로 두었습니다.

# This is not a valid packwing license key.
td5ae978f = 올바른 packwing 라이선스 키가 아닙니다.

# Your organization’s policy for this cannot be read, so packwing won’t do it. Ask your IT department.
t3c605280 = 이에 대한 조직의 정책을 읽을 수 없어 packwing이 실행하지 않습니다. IT 부서에 문의하세요.

# Your organization requires new archives to be encrypted.
t83acf110 = 조직에서 새 압축 파일을 반드시 암호화하도록 요구합니다.

# Your organization does not allow this kind of encryption: {method}.
te8eb49bf = 조직에서 이 암호화 방식을 허용하지 않습니다: { $method }.

# Your organization requires a password of at least {least} characters.
t2d21b125 = 조직에서 { $least }자 이상의 비밀번호를 요구합니다.

# That password is too easy to guess, and your organization requires a better one.
t7020f9d1 = 비밀번호를 너무 쉽게 추측할 수 있으며, 조직에서 더 안전한 비밀번호를 요구합니다.

# Expected true or false, not {other}.
tcadcff7a = true 또는 false여야 하며, { $other }은(는) 안 됩니다.

# Expected a number from 1 to {most}, not {other}.
tda498fad = 1에서 { $most } 사이의 숫자여야 하며, { $other }은(는) 안 됩니다.

# Expected some text, not {other}.
t9386ce3b = 텍스트여야 하며, { $other }은(는) 안 됩니다.

# Expected an address starting with https://, not {other}.
t93bfd19e = 예상한 값은 https://로 시작하는 주소이며, { $other }이(가) 아닙니다.

# Expected a list, not {other}.
tc945cc93 = 목록이어야 하며, { $other }은(는) 안 됩니다.

# No kind of encryption is called {name}.
tfddb7ba6 = { $name }이라는 암호화 방식은 없습니다.

# The list names no kind of encryption.
t5f8076f5 = 목록에 암호화 방식이 하나도 없습니다.

# The file is too big to be a list of policies.
taf68e582 = 정책 목록으로 보기에는 파일이 너무 큽니다.

# The file has to hold one JSON object.
tf483aa65 = 파일에는 JSON 객체가 하나 있어야 합니다.

# {path} has to belong to root.
tfc77ea2b = { $path }의 소유자는 root여야 합니다.

# {path} can be changed by others than root.
t0d4a4498 = { $path }을(를) root 외의 사용자가 변경할 수 있습니다.

# This storage takes files of at most {size} for now, as each is held in memory while it is sent.
t6485842e = 이 스토리지는 전송하는 동안 각 파일을 메모리에 보관하므로, 지금은 최대 { $size }까지의 파일만 받습니다.

# This storage cannot rename a folder at once, as everything under it would be copied. Use Move to….
ta87a5d75 = 이 스토리지는 폴더 이름을 한 번에 바꿀 수 없습니다. 그 아래의 모든 항목이 복사되기 때문입니다. 이동…을 사용하세요.

# S3 bucket
tc5e013b3 = S3 버킷

# Bucket
te34e3557 = 버킷

# Region
t492e789d = 리전

# Endpoint
t27e7d69c = 엔드포인트

# Empty for AWS, or https://…
td3a94af8 = AWS는 비워 두세요. 또는 https://…

# Start in
t5eb91ea6 = 시작 위치

# A prefix, if not the whole bucket
t75255a6c = 버킷 전체가 아니면 프리픽스

# Access key ID
tf9408d75 = 액세스 키 ID

# Secret access key
te1c3d3a2 = 시크릿 액세스 키

# SFTP
t62703ae8 = SFTP

# Host or ssh_config name
t7b943498 = 호스트 또는 ssh_config 이름

# Empty for the home folder
t67b5f42b = 홈 폴더는 비워 두세요

# Empty to use the SSH agent and keys
tf10e6293 = SSH 에이전트와 키를 사용하려면 비워 두세요

# WebDAV folder
tb2401428 = WebDAV 폴더

# Address
t700560f3 = 주소

# User
te0a63e12 = 사용자

# Password or app token
t1c18cb9b = 비밀번호 또는 앱 토큰
