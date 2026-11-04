param([Parameter(Mandatory=$true)][ValidatePattern('^[A-Fa-f0-9]{64}$')][string]$ExpectedSha256)
$ErrorActionPreference = 'Stop'
$taskProject = Split-Path -Parent $PSScriptRoot
$taskWorkspace = Split-Path -Parent $taskProject
$taskPackage = Join-Path $taskProject 'server-browser-2.0.0.hnmod'
$taskExpected = $ExpectedSha256.ToUpperInvariant()
if ((Get-FileHash -LiteralPath $taskPackage -Algorithm SHA256).Hash -ne $taskExpected) {
    throw 'The package differs from the validated delivery candidate.'
}
$taskDesktop = [Environment]::GetFolderPath('Desktop')
$taskHome = Join-Path $env:APPDATA 'horizon'
$taskInstance = Join-Path $taskHome 'instances\server-browser-test'
if (-not (Test-Path -LiteralPath $taskInstance -PathType Container)) {throw 'Create the official Server Browser Test instance first.'}
$taskNative = Join-Path $taskInstance 'horizon-native-mods'
$taskBackup = Join-Path $taskWorkspace ('horizon-server-browser\.local\legacy-host-patch-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss'))
if (-not [IO.Path]::GetFullPath($taskBackup).StartsWith([IO.Path]::GetFullPath($taskWorkspace)+'\',[StringComparison]::OrdinalIgnoreCase)) {
    throw 'Backup must remain inside the workspace.'
}
New-Item -ItemType Directory -Force -Path $taskBackup,$taskNative | Out-Null

# Only our previously delivered package leaves the default profile. Other mods
# and all player settings stay in their current instance.
$taskDefaultMod = Join-Path $taskHome 'horizon-native-mods\server_browser.hnmod'
if (Test-Path -LiteralPath $taskDefaultMod -PathType Leaf) {
    $taskPreviousHash=(Get-FileHash -LiteralPath $taskDefaultMod -Algorithm SHA256).Hash
    if ($taskPreviousHash -notin @('FDEC9486BCDB9F3A4B60C58D7E91240A68388345CDF74D664BA2995838C7F01C',$taskExpected)) {throw 'The default-profile browser package changed; preserve it for review.'}
    $taskCheckedSource=[IO.Path]::GetFullPath($taskDefaultMod)
    $taskCheckedBackup=[IO.Path]::GetFullPath((Join-Path $taskBackup 'default-server_browser.hnmod'))
    if (-not $taskCheckedSource.StartsWith([IO.Path]::GetFullPath($taskHome)+'\',[StringComparison]::OrdinalIgnoreCase) -or -not $taskCheckedBackup.StartsWith($taskBackup+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Archive path escaped its intended home.'}
    Move-Item -LiteralPath $taskCheckedSource -Destination $taskCheckedBackup
}

$taskOld = @(
    @{Path=(Join-Path $taskDesktop 'server-browser-1.1.0.hmod');Name='desktop-server-browser-1.1.0.hmod'},
    @{Path=(Join-Path $taskHome 'horizon-mods\server-browser-1.1.0.hmod');Name='installed-server-browser-1.1.0.hmod'}
)
foreach ($taskItem in $taskOld) {
    if (Test-Path -LiteralPath $taskItem.Path -PathType Leaf) {
        if ((Get-FileHash -LiteralPath $taskItem.Path -Algorithm SHA256).Hash -ne 'B34D5546DFA2510129FCB53DA510324AC71B7974065F3A28CCC2D6D89EA8CE22') {
            throw ('The old file changed; preserve it for review: '+$taskItem.Path)
        }
        Move-Item -LiteralPath $taskItem.Path -Destination (Join-Path $taskBackup $taskItem.Name)
    }
}
$taskLink = Join-Path $taskDesktop 'Horizon Server-Browser.lnk'
if (Test-Path -LiteralPath $taskLink -PathType Leaf) {
    $taskShell = New-Object -ComObject WScript.Shell
    $taskShortcut = $taskShell.CreateShortcut($taskLink)
    if ($taskShortcut.Arguments.Contains((Join-Path $taskWorkspace 'start-server-browser.cmd'))) {
        Move-Item -LiteralPath $taskLink -Destination (Join-Path $taskBackup 'Horizon Server-Browser.lnk')
    }
}
foreach ($taskRelative in @('start-server-browser.cmd','tools\start-server-browser-mod.ps1')) {
    $taskOldEntry = Join-Path $taskWorkspace $taskRelative
    if (Test-Path -LiteralPath $taskOldEntry -PathType Leaf) {
        Move-Item -LiteralPath $taskOldEntry -Destination (Join-Path $taskBackup ([IO.Path]::GetFileName($taskOldEntry)))
    }
}
$taskDeliveries = @(
    (Join-Path $taskDesktop 'server-browser-2.0.0.hnmod'),
    (Join-Path $taskNative 'server_browser.hnmod')
)
foreach ($taskDestination in $taskDeliveries) {
    if (Test-Path -LiteralPath $taskDestination -PathType Leaf) {
        Copy-Item -LiteralPath $taskDestination -Destination (Join-Path $taskBackup ('previous-'+[IO.Path]::GetFileName($taskDestination)))
    }
    Copy-Item -LiteralPath $taskPackage -Destination $taskDestination -Force
    if ((Get-FileHash -LiteralPath $taskDestination -Algorithm SHA256).Hash -ne $taskExpected) { throw 'Delivery hash mismatch.' }
}
# Native mods default to on. Preserve all player settings, including an explicit off setting.
[pscustomobject]@{Desktop=$taskDeliveries[0];Installed=$taskDeliveries[1];Backup=$taskBackup} | ConvertTo-Json
