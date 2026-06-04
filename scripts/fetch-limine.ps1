param(
    [string]$Version = "v12.2.0"
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Target = Join-Path $Root ".tools\limine\$Version"
$Archive = Join-Path $Root ".tools\limine\limine-$Version.zip"
$Url = "https://github.com/limine-bootloader/limine/releases/download/$Version/limine-binary.zip"

if ((Test-Path (Join-Path $Target "limine.exe")) -and (Test-Path (Join-Path $Target "limine-bios.sys"))) {
    Write-Host "Limine $Version is already available at $Target"
    exit 0
}

New-Item -ItemType Directory -Force -Path (Split-Path $Archive) | Out-Null
New-Item -ItemType Directory -Force -Path $Target | Out-Null

Write-Host "Downloading Limine $Version"
Invoke-WebRequest -Uri $Url -OutFile $Archive

$ExtractRoot = Join-Path $Root ".tools\limine\extract-$Version"
if (Test-Path $ExtractRoot) {
    Remove-Item -Recurse -Force $ExtractRoot
}
Expand-Archive -Path $Archive -DestinationPath $ExtractRoot

$Files = @("limine.exe", "limine-bios.sys")
foreach ($File in $Files) {
    $Match = Get-ChildItem -Path $ExtractRoot -Recurse -File -Filter $File | Select-Object -First 1
    if (-not $Match) {
        throw "Could not find $File in Limine release archive."
    }
    Copy-Item -Force $Match.FullName (Join-Path $Target $File)
}

Remove-Item -Recurse -Force $ExtractRoot
Write-Host "Limine $Version is ready at $Target"
