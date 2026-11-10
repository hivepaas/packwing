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
; It installs for everyone on the computer, into Program Files, where only
; an administrator changes it, as an organization's policies and audit log
; need; or for the user alone, with no administrator rights, into
; %LOCALAPPDATA%\Programs\packwing. Both are where the updater knows the
; installer's (crates/update/src/install.rs), and it runs the next one
; quietly (/S) over it. A page asks which, for everyone by default; /ALLUSERS
; and /CURRENTUSER say it on the command line. Where packwing is installed
; already, it goes where it is, as it was: to change, uninstall it first.
;
; It runs as the user. For everyone, it starts itself again as an
; administrator (Windows asks), waits for that one, and ends: so the updater,
; which waits for it, starts the app after the install is done. The app it
; starts at the end is the user's, never the administrator's.
;
; The settings are each user's, under %APPDATA%: neither installing nor
; uninstalling touches them.
;
; pkw.exe, the command line, goes beside packwing.exe, and the folder in the
; PATH (path.ps1), the user's or, for everyone, the computer's, so a terminal
; opened after runs `pkw`; uninstalling takes it out again.
;
; The right-click menu comes in two kinds (context-menu-rs's docs/windows.md).
; packwing writes the classic one ("Show more options") itself, when it starts.
; For Windows 11's own menu the installer registers a package beside it,
; declaring the native extension, context-menu-packwing.dll: not signed, it is
; taken where Developer Mode is on, and refused elsewhere, where it is let go
; and the classic menu stays. Registered, packwing moves its menu to the
; package the next time it starts. Both are each user's: for everyone, the
; package is registered for the user who installs, and each other user has
; the classic menu, which packwing writes when it starts.

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma

!include "MUI2.nsh"
!include "WinMessages.nsh"
!include "LogicLib.nsh"
!include "nsDialogs.nsh"
!include "FileFunc.nsh"

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
; Where, as the mode says: .onInit and the mode's page set it.
InstallDir ""
RequestExecutionLevel user
BrandingText "packwing ${VERSION}"

; 1 for everyone, 0 for the user alone.
Var Everyone
; 1 once the mode is known (said, or packwing installed already): no page.
Var Known
; 1 when /D= said where.
Var Told
; 1 in the installer started again as an administrator.
Var Elevated
Var ForEveryone
Var ForMe

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
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Start packwing"
!define MUI_FINISHPAGE_RUN_FUNCTION StartApp

!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipWhenElevated
!insertmacro MUI_PAGE_WELCOME
Page custom ModePage ModeLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

; The installer's 32 bits would see the registry's 32-bit view: packwing runs
; on 64-bit Windows alone (x86-64 and ARM64), where the keys are in the other.
!macro VIEW
  SetRegView 64
!macroend

; Where, and in whose name, as $Everyone says.
!macro MODE
  ${If} $Everyone == 1
    SetShellVarContext all
    ${If} $Told != 1
      StrCpy $INSTDIR "$PROGRAMFILES64\packwing"
    ${EndIf}
  ${Else}
    SetShellVarContext current
    ${If} $Told != 1
      StrCpy $INSTDIR "$LOCALAPPDATA\Programs\packwing"
    ${EndIf}
  ${EndIf}
!macroend

; Whether this runs as an administrator: the user's token, under UAC, says
; "User" until Windows raises it.
!macro IS_ADMIN out
  UserInfo::GetAccountType
  Pop ${out}
!macroend

Function .onInit
  !insertmacro VIEW
  ${If} $INSTDIR != ""
    StrCpy $Told 1
  ${EndIf}
  ${GetParameters} $R0
  ClearErrors
  ${GetOptions} $R0 "/ELEVATED" $R1
  ${IfNot} ${Errors}
    StrCpy $Elevated 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/ALLUSERS" $R1
  ${IfNot} ${Errors}
    StrCpy $Everyone 1
    StrCpy $Known 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $R0 "/CURRENTUSER" $R1
  ${IfNot} ${Errors}
    StrCpy $Everyone 0
    StrCpy $Known 1
  ${EndIf}
  ; Installed already: there again, as it was, as the updater runs it.
  ${If} $Known != 1
    ReadRegStr $R1 HKLM "${UNINSTALL_KEY}" "InstallLocation"
    ReadRegStr $R2 HKCU "${UNINSTALL_KEY}" "InstallLocation"
    ${If} $R1 != ""
      StrCpy $Everyone 1
      StrCpy $Known 1
    ${ElseIf} $R2 != ""
      StrCpy $Everyone 0
      StrCpy $Known 1
    ${Else}
      StrCpy $Everyone 1
    ${EndIf}
  ${EndIf}
  ; Quiet, nothing is asked: for everyone where nothing says otherwise.
  ${If} ${Silent}
    StrCpy $Known 1
  ${EndIf}
  ${If} $Known == 1
  ${AndIf} $Everyone == 1
    Call Elevate
  ${EndIf}
  !insertmacro MODE
FunctionEnd

; For everyone, as an administrator: this installer started again so, with
; what it was told, and waited for; then this one ends.
Function Elevate
  !insertmacro IS_ADMIN $0
  ${If} $0 == "Admin"
    Return
  ${EndIf}
  ${If} $Elevated == 1
    MessageBox MB_ICONSTOP "packwing could not be installed for everyone: it needs an administrator's rights." /SD IDOK
    Quit
  ${EndIf}
  ${GetParameters} $R0
  ; /D= comes last, as NSIS reads it.
  ClearErrors
  ExecShellWait "runas" "$EXEPATH" '/ELEVATED /ALLUSERS $R0'
  ${If} ${Errors}
    MessageBox MB_ICONEXCLAMATION "packwing was not installed: Windows did not give it an administrator's rights. Install it for yourself alone to do without them." /SD IDOK
  ${EndIf}
  Quit
FunctionEnd

Function SkipWhenElevated
  ${If} $Elevated == 1
    Abort
  ${EndIf}
FunctionEnd

Function ModePage
  ${If} $Known == 1
    Abort
  ${EndIf}
  !insertmacro MUI_HEADER_TEXT "Install for whom?" "packwing can be installed for everyone on this computer, or for you alone."
  nsDialogs::Create 1018
  Pop $0
  ${NSD_CreateRadioButton} 0 0 100% 12u "For everyone on this computer (recommended)"
  Pop $ForEveryone
  ${NSD_CreateLabel} 12u 14u -12u 34u "Into Program Files, where only an administrator can change it, as an organization's policies need. Windows asks for an administrator's permission now, and for each update."
  Pop $0
  ${NSD_CreateRadioButton} 0 56u 100% 12u "Only for me"
  Pop $ForMe
  ${NSD_CreateLabel} 12u 70u -12u 34u "Into your own folder, with no administrator's permission: updates install without asking."
  Pop $0
  ${If} $Everyone == 1
    ${NSD_Check} $ForEveryone
  ${Else}
    ${NSD_Check} $ForMe
  ${EndIf}
  nsDialogs::Show
FunctionEnd

Function ModeLeave
  ${NSD_GetState} $ForEveryone $0
  ${If} $0 == ${BST_CHECKED}
    StrCpy $Everyone 1
    Call Elevate
  ${Else}
    StrCpy $Everyone 0
  ${EndIf}
  !insertmacro MODE
FunctionEnd

; The app, started as the user who is signed in, never as the administrator
; this may run as: Explorer starts it.
Function StartApp
  Exec '"$WINDIR\explorer.exe" "$INSTDIR\packwing.exe"'
FunctionEnd

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

  ; pkw in the PATH, once: the updater's install over it adds none.
  SetOutPath "$PLUGINSDIR"
  File "path.ps1"
  SetOutPath "$INSTDIR"
  DetailPrint "Putting pkw in the PATH"
  ${If} $Everyone == 1
    nsExec::ExecToLog '${PATH_PS1} add "$INSTDIR" machine'
  ${Else}
    nsExec::ExecToLog '${PATH_PS1} add "$INSTDIR" user'
  ${EndIf}
  Pop $0
  StrCmp $0 "0" +2
    DetailPrint "The PATH was not changed: pkw is in $INSTDIR"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  ; In the user's, or for everyone the computer's (SHCTX, as the mode set it).
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "DisplayName" "packwing"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "Publisher" "HivePaaS"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/hivepaas/packwing"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\packwing.exe"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr SHCTX "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD SHCTX "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD SHCTX "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

; The uninstaller knows the mode from where it is: the computer's key names
; its folder for an install for everyone, which it removes as an
; administrator, started again so as the installer is.
Function un.onInit
  !insertmacro VIEW
  ReadRegStr $R1 HKLM "${UNINSTALL_KEY}" "InstallLocation"
  ${If} $R1 == $INSTDIR
    StrCpy $Everyone 1
    !insertmacro IS_ADMIN $0
    ${If} $0 != "Admin"
      ${GetParameters} $R0
      ClearErrors
      ${GetOptions} $R0 "/ELEVATED" $R2
      ${IfNot} ${Errors}
        MessageBox MB_ICONSTOP "packwing could not be uninstalled: it needs an administrator's rights." /SD IDOK
        Quit
      ${EndIf}
      ClearErrors
      ExecShellWait "runas" "$EXEPATH" '/ELEVATED $R0'
      ${If} ${Errors}
        MessageBox MB_ICONEXCLAMATION "packwing was not uninstalled: Windows did not give it an administrator's rights." /SD IDOK
      ${EndIf}
      Quit
    ${EndIf}
    SetShellVarContext all
  ${Else}
    StrCpy $Everyone 0
    SetShellVarContext current
  ${EndIf}
FunctionEnd

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
  ${If} $Everyone == 1
    nsExec::ExecToLog '${PATH_PS1} remove "$INSTDIR" machine'
  ${Else}
    nsExec::ExecToLog '${PATH_PS1} remove "$INSTDIR" user'
  ${EndIf}
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
  DeleteRegKey SHCTX "${UNINSTALL_KEY}"
SectionEnd
