# Puts a folder in the user's PATH, or takes it out, for installer.nsi: pkw.exe
# is in packwing's folder, and a terminal opened after finds it.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File path.ps1 add|remove DIR
#
# The PATH is read and written in the registry as it is kept, its %VARIABLES%
# left as they are, and of the kind it is (REG_EXPAND_SZ, or REG_SZ): never
# through NSIS, whose strings end at 1024 characters, which a PATH passes.
# Its other folders stay as they are, in their order. The installer tells
# Windows it changed (WM_SETTINGCHANGE) after.

param(
    [Parameter(Mandatory)][ValidateSet('add', 'remove')][string]$Do,
    [Parameter(Mandatory)][string]$Dir
)

$ErrorActionPreference = 'Stop'
$key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
try {
    $there = $key.GetValueNames() -contains 'Path'
    $kind = if ($there) { $key.GetValueKind('Path') } else { [Microsoft.Win32.RegistryValueKind]::ExpandString }
    $path = if ($there) {
        $key.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
    } else { '' }
    # The same folder, written with a \ at its end or not, in any case.
    $same = { param($a) $a.TrimEnd('\') -ieq $Dir.TrimEnd('\') }
    $folders = @($path -split ';' | Where-Object { $_ -ne '' })
    $others = @($folders | Where-Object { -not (& $same $_) })
    if ($Do -eq 'add') {
        if ($others.Count -eq $folders.Count) {
            $key.SetValue('Path', (($folders + $Dir) -join ';'), $kind)
        }
    } elseif ($others.Count -ne $folders.Count) {
        if ($others.Count -eq 0) {
            $key.DeleteValue('Path')
        } else {
            $key.SetValue('Path', ($others -join ';'), $kind)
        }
    }
} finally {
    $key.Close()
}
