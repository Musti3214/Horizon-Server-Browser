$ErrorActionPreference='Stop'
$taskProject=Split-Path -Parent $PSScriptRoot
$taskPackage=Join-Path $taskProject 'server-browser-2.1.0.hnmod'
$taskExpected='BB6BB8E5E89299A0D853CE85DB520205564DA8B77576B61FBF6054EB200BFD58'
if ((Get-FileHash -LiteralPath $taskPackage).Hash -ne $taskExpected) {throw 'Release candidate changed'}
$taskProfile=Join-Path $env:APPDATA 'horizon\instances\server-browser-2-1'
if (-not (Test-Path -LiteralPath $taskProfile -PathType Container)) {throw 'The official Server Browser 2.1 instance must exist first'}
$taskNative=Join-Path $taskProfile 'horizon-native-mods'
New-Item -ItemType Directory -Path $taskNative -Force | Out-Null
$taskTargets=@((Join-Path ([Environment]::GetFolderPath('Desktop')) 'server-browser-2.1.0.hnmod'),(Join-Path $taskNative 'server-browser-2.1.0.hnmod'))
foreach ($taskTarget in $taskTargets) {
    if ((Test-Path -LiteralPath $taskTarget) -and (Get-FileHash -LiteralPath $taskTarget).Hash -ne $taskExpected) {throw 'Preserve an existing different file before replacing it'}
    Copy-Item -LiteralPath $taskPackage -Destination $taskTarget -Force
    if ((Get-FileHash -LiteralPath $taskTarget).Hash -ne $taskExpected) {throw 'Delivery checksum mismatch'}
}
[pscustomobject]@{Desktop=$taskTargets[0];PreparedProfile=$taskTargets[1];Sha256=$taskExpected;BuildStarted=$false} | ConvertTo-Json
