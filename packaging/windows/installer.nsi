; packwing's installer for Windows (NSIS 3, Unicode), made by
; scripts/package_windows.sh, as shellwing's is:
;
;   makensis /DVERSION=0.1.0 /DFILEVERSION=0.1.0.0 /DEXE=path\to\packwing.exe
;            /DCLI=path\to\pkw.exe
;            /DEXTENSION=path\to\context-menu-windows-x86_64.dll
;            /DMANIFEST=path\to\AppxManifest.xml /DLICENSES=path\to\THIRD-PARTY-LICENSES
;            /DOUT=setup.exe installer.nsi
;
; FILEVERSION is VERSION as Windows writes a file's version: four numbers, no
; beta (0.3.0-beta2 is 0.3.0.0). MANIFEST is AppxManifest.xml with it written in.
;
; It installs for the user, with no administrator rights, into
; %LOCALAPPDATA%\Programs\packwing: the place the updater knows as the
; installer's (crates/update/src/install.rs), and runs the next one quietly
; (/S) over it. The settings are the user's, under %APPDATA%: neither
; installing nor uninstalling touches them.
;
; pkw.exe, the command line, goes beside packwing.exe, and the folder in the
; user's PATH (path.ps1), so a terminal opened after runs `pkw`; uninstalling
; takes it out again.
;
; The right-click menu comes in two kinds (context-menu-rs's docs/windows.md).
; packwing writes the classic one ("Show more options") itself, when it starts.
; For Windows 11's own menu the installer registers a package beside it,
; declaring the native extension, context-menu-packwing.dll: not signed, it is
; taken where Developer Mode is on, and refused elsewhere, where it is let go
; and the classic menu stays. Registered, packwing moves its menu to the
; package the next time it starts.

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "WinMessages.nsh"

!ifndef VERSION
  !error "VERSION is not defined"
!endif
!ifndef FILEVERSION
  !error "FILEVERSION is not defined"
!endif
!ifndef EXE
  !error "EXE is not defined"
!endif
!ifndef CLI
  !error "CLI is not defined"
!endif
!ifndef EXTENSION
  !error "EXTENSION is not defined"
!endif
!ifndef MANIFEST
  !error "MANIFEST is not defined"
!endif
!ifndef LICENSES
  !error "LICENSES is not defined"
!endif
!ifndef OUT
  !error "OUT is not defined"
!endif

!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\packwing"
; The package's name, as AppxManifest.xml's Identity has it.
!define PACKAGE "HivePaaS.packwing"
; context_menu::windows::native_name("packwing").
!define DLL "context-menu-packwing.dll"
; PowerShell, quiet, for the package.
!define PS 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command'
; path.ps1, with the folder as it is, whatever is in its name.
!define PATH_PS1 'powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\path.ps1"'

Name "packwing"
OutFile "${OUT}"
InstallDir "$LOCALAPPDATA\Programs\packwing"
RequestExecutionLevel user
BrandingText "packwing ${VERSION}"

VIProductVersion "${FILEVERSION}"
VIAddVersionKey "ProductName" "packwing"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "packwing installer"
VIAddVersionKey "CompanyName" "HivePaaS"
VIAddVersionKey "LegalCopyright" "Copyright (C) 2026 HivePaaS"

!define MUI_ICON "..\icons\packwing.ico"
!define MUI_UNICON "..\icons\packwing.ico"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\packwing.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Start packwing"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "packwing"
  ; The app may still be closing (the updater starts this as it quits): the
  ; new one is unpacked aside, then copied in, tried again for five seconds,
  ; then asked about (Retry / Cancel; Cancel when quiet).
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File "/oname=packwing.exe" "${EXE}"
  SetOutPath "$INSTDIR"
  StrCpy $0 0
  retry:
    ClearErrors
    CopyFiles /SILENT "$PLUGINSDIR\packwing.exe" "$INSTDIR\packwing.exe"
    IfErrors 0 copied
    IntOp $0 $0 + 1
    IntCmp $0 10 ask
    Sleep 500
    Goto retry
  ask:
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "packwing is still running. Quit it, then choose Retry." /SD IDCANCEL IDRETRY retry
    Abort
  copied:

  ; The package of an earlier install goes first: the one registered below is
  ; this one's, whatever its version.
  nsExec::ExecToLog '${PS} "Get-AppxPackage -Name ${PACKAGE} | Remove-AppxPackage"'
  Pop $0

  ; Explorer may hold the extension still: a file in use cannot be replaced,
  ; but it can be moved aside, and goes when Windows restarts.
  Delete "$INSTDIR\${DLL}.old"
  ClearErrors
  Delete "$INSTDIR\${DLL}"
  IfErrors 0 +3
    Rename "$INSTDIR\${DLL}" "$INSTDIR\${DLL}.old"
    Delete /REBOOTOK "$INSTDIR\${DLL}.old"
  File "/oname=${DLL}" "${EXTENSION}"

  File "/oname=pkw.exe" "${CLI}"
  File "logo44.png"
  File "logo50.png"
  File "logo150.png"
  File "/oname=LICENSE.txt" "..\..\LICENSE"
  File "/oname=GPL-ADDITIONAL-PERMISSION.txt" "..\..\GPL-ADDITIONAL-PERMISSION.md"
  File "/oname=THIRD-PARTY-LICENSES.txt" "${LICENSES}"
  SetOutPath "$INSTDIR\package"
  File "/oname=AppxManifest.xml" "${MANIFEST}"
  SetOutPath "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Windows 11's own menu, where Windows takes the package; where it does not,
  ; nothing more is done, and the classic menu stays.
  DetailPrint "Registering packwing for Windows 11's right-click menu"
  nsExec::ExecToLog '${PS} "Add-AppxPackage -Register \"$INSTDIR\package\AppxManifest.xml\" -ExternalLocation \"$INSTDIR\""'
  Pop $0
  StrCmp $0 "0" +2
    DetailPrint "Windows did not take it (Developer Mode is off, or it needs signing): the menu stays under Show more options"

  CreateShortcut "$SMPROGRAMS\packwing.lnk" "$INSTDIR\packwing.exe"

  ; pkw in the user's PATH, once: the updater's install over it adds none.
  SetOutPath "$PLUGINSDIR"
  File "path.ps1"
  SetOutPath "$INSTDIR"
  DetailPrint "Putting pkw in the PATH"
  nsExec::ExecToLog '${PATH_PS1} add "$INSTDIR"'
  Pop $0
  StrCmp $0 "0" +2
    DetailPrint "The PATH was not changed: pkw is in $INSTDIR"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "packwing"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "HivePaaS"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/hivepaas/packwing"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\packwing.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  ; The menu, the classic one and the package's, and Open With: packwing's
  ; own, in the registry, and the package.
  nsExec::ExecToLog '"$INSTDIR\packwing.exe" --uninstall'
  Pop $0
  nsExec::ExecToLog '${PS} "Get-AppxPackage -Name ${PACKAGE} | Remove-AppxPackage"'
  Pop $0
  ; pkw's folder out of the user's PATH.
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File "path.ps1"
  nsExec::ExecToLog '${PATH_PS1} remove "$INSTDIR"'
  Pop $0
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  ; Out of the folder, which is taken away below.
  SetOutPath "$TEMP"

  Delete "$INSTDIR\packwing.exe"
  Delete "$INSTDIR\pkw.exe"
  Delete "$INSTDIR\${DLL}.old"
  ClearErrors
  Delete "$INSTDIR\${DLL}"
  IfErrors 0 +2
    Delete /REBOOTOK "$INSTDIR\${DLL}"
  Delete "$INSTDIR\logo44.png"
  Delete "$INSTDIR\logo50.png"
  Delete "$INSTDIR\logo150.png"
  Delete "$INSTDIR\LICENSE.txt"
  Delete "$INSTDIR\GPL-ADDITIONAL-PERMISSION.txt"
  Delete "$INSTDIR\THIRD-PARTY-LICENSES.txt"
  ; The package's folder, with what Windows puts in it as it registers it.
  RMDir /r "$INSTDIR\package"
  Delete "$INSTDIR\uninstall.exe"
  RMDir /REBOOTOK "$INSTDIR"
  Delete "$SMPROGRAMS\packwing.lnk"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd
