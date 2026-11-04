$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$taskProject=Split-Path -Parent $PSScriptRoot
$taskWorkspace=Split-Path -Parent $taskProject
$taskPackage=Join-Path $taskProject 'server-browser-2.1.0.hnmod'
$taskExpected='BB6BB8E5E89299A0D853CE85DB520205564DA8B77576B61FBF6054EB200BFD58'
if ((Get-FileHash -LiteralPath $taskPackage -Algorithm SHA256).Hash -ne $taskExpected) {throw 'The tested release changed'}
$taskPlayerRoot=[IO.Path]::GetFullPath((Join-Path $env:APPDATA 'horizon'))
$taskBackup=[IO.Path]::GetFullPath((Join-Path $taskWorkspace ('horizon-server-browser\.local\native-browser\installed-backup-'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss'))))
if (-not $taskBackup.StartsWith([IO.Path]::GetFullPath($taskWorkspace)+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Backup escaped workspace'}
$taskFolders=@(
    @{Name='default';Path=(Join-Path $taskPlayerRoot 'horizon-native-mods')},
    @{Name='server-browser-test';Path=(Join-Path $taskPlayerRoot 'instances\server-browser-test\horizon-native-mods')},
    @{Name='server-browser-2-1';Path=(Join-Path $taskPlayerRoot 'instances\server-browser-2-1\horizon-native-mods')}
)
$taskReport=@()
foreach($taskFolder in $taskFolders) {
    $taskResolved=[IO.Path]::GetFullPath($taskFolder.Path)
    if (-not $taskResolved.StartsWith($taskPlayerRoot+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Target escaped Horizon'}
    if (-not (Test-Path -LiteralPath $taskResolved -PathType Container)) {throw 'Expected existing native-mod folder'}
    $taskModFiles=@()
    foreach($taskFile in (Get-ChildItem -LiteralPath $taskResolved -File -Filter '*.hnmod')) {
        $taskZip=[IO.Compression.ZipFile]::OpenRead($taskFile.FullName)
        try {
            $taskEntry=$taskZip.GetEntry('horizon-native.json')
            if (-not $taskEntry) {continue}
            $taskReader=[IO.StreamReader]::new($taskEntry.Open())
            try {$taskManifest=$taskReader.ReadToEnd() | ConvertFrom-Json} finally {$taskReader.Dispose()}
            if ($taskManifest.id -eq 'server_browser') {$taskModFiles += $taskFile}
        } finally {$taskZip.Dispose()}
    }
    if ($taskModFiles.Count -eq 1 -and $taskModFiles[0].Name -eq 'server-browser-2.1.0.hnmod' -and (Get-FileHash -LiteralPath $taskModFiles[0].FullName).Hash -eq $taskExpected) {
        $taskReport += @{Profile=$taskFolder.Name;Status='Already current';Path=$taskModFiles[0].FullName};continue
    }
    $taskDestination=Join-Path $taskResolved 'server-browser-2.1.0.hnmod'
    if ((Test-Path -LiteralPath $taskDestination) -and $taskDestination -notin $taskModFiles.FullName) {throw 'Destination belongs to another package'}
    $taskArchive=Join-Path $taskBackup $taskFolder.Name
    New-Item -ItemType Directory -Path $taskArchive -Force | Out-Null
    foreach($taskFile in $taskModFiles) {
        $taskSource=[IO.Path]::GetFullPath($taskFile.FullName)
        $taskSaved=[IO.Path]::GetFullPath((Join-Path $taskArchive $taskFile.Name))
        if (-not $taskSource.StartsWith($taskResolved+'\',[StringComparison]::OrdinalIgnoreCase) -or -not $taskSaved.StartsWith($taskBackup+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Move target escaped checked folders'}
        Move-Item -LiteralPath $taskSource -Destination $taskSaved
    }
    Copy-Item -LiteralPath $taskPackage -Destination $taskDestination
    if ((Get-FileHash -LiteralPath $taskDestination).Hash -ne $taskExpected) {throw 'Installed checksum mismatch'}
    $taskReport += @{Profile=$taskFolder.Name;Status='Updated to 2.1.0';Path=$taskDestination;Archived=@($taskModFiles.Name)}
}
@{Profiles=$taskReport;Backup=$taskBackup;Sha256=$taskExpected;BuildStarted=$false} | ConvertTo-Json -Depth 5
