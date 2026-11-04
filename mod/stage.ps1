param(
    [Parameter(Mandatory=$true)][string]$SourceRoot,
    [Parameter(Mandatory=$true)][string]$WorkRoot,
    [Parameter(Mandatory=$true)][string]$MixinsExe,
    [string]$Python = 'python',
    [switch]$PrepareOnly
)
$ErrorActionPreference = 'Stop'
$taskProject = $PSScriptRoot
$taskRoot = [IO.Path]::GetFullPath($WorkRoot)
New-Item -ItemType Directory -Force -Path $taskRoot | Out-Null
$taskRoot = (Resolve-Path -LiteralPath $taskRoot).Path
$taskIncoming = Join-Path $taskRoot ('incoming-'+[guid]::NewGuid().ToString('N'))
$taskStage = Join-Path $taskRoot 'stage'
$taskBackup = Join-Path $taskRoot ('stage-old-'+[guid]::NewGuid().ToString('N'))
foreach ($taskPath in @($taskIncoming,$taskStage,$taskBackup)) {
    if (-not [IO.Path]::GetFullPath($taskPath).StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)) {
        throw 'Stage path must stay inside the managed work folder.'
    }
}
& $Python (Join-Path $taskProject 'build_bundle.py')
if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
& $MixinsExe apply --workspace $SourceRoot --mods (Join-Path $taskProject 'mixins') --out $taskIncoming
if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
if (Test-Path -LiteralPath $taskStage) {
    & $Python (Join-Path $taskProject 'carry_timestamps.py') $taskRoot $taskStage $taskIncoming
    if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
}
if ($PrepareOnly) {Write-Output $taskIncoming; exit 0}
if (Test-Path -LiteralPath $taskStage) {Move-Item -LiteralPath $taskStage -Destination $taskBackup}
Move-Item -LiteralPath $taskIncoming -Destination $taskStage
Write-Output $taskStage
