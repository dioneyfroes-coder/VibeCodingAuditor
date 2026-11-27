# Instala o MSEdgeDriver compativel com o WebView2 do runner do Windows.
# O download automatico do @wdio/tauri-service 1.5.1 usa Expand-Archive -Force,
# que exige a sobrecarga de 3 argumentos de ExtractToDirectory (ausente no
# Windows PowerShell 5.1). Aqui a extracao usa a sobrecarga de 2 argumentos.
param(
  [Parameter(Mandatory = $true)]
  [string] $EdgeVersion,

  [string] $InstallDir = (Join-Path $env:RUNNER_TEMP 'vca-msedgedriver')
)

$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($EdgeVersion)) {
  throw 'Versao do WebView2 nao informada; nao e possivel escolher o MSEdgeDriver.'
}

$major = ($EdgeVersion -split '\.')[0]
if ($major -notmatch '^\d+$') {
  throw "Versao do WebView2 invalida: $EdgeVersion"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$driverPath = Join-Path $InstallDir 'msedgedriver.exe'

if (Test-Path -LiteralPath $driverPath) {
  $existing = (& $driverPath --version 2>&1 | Out-String)
  if ($existing -match "MSEdgeDriver\s+$major\.") {
    Write-Host "MSEdgeDriver $major reaproveitado de $InstallDir"
    if ($env:GITHUB_PATH) { Add-Content -Path $env:GITHUB_PATH -Value $InstallDir }
    else { $env:PATH = "$InstallDir;$env:PATH" }
    exit 0
  }
  Remove-Item -LiteralPath $driverPath -Force
}

$releaseUrl = "https://msedgedriver.microsoft.com/LATEST_RELEASE_$major"
$release = Invoke-WebRequest -Uri $releaseUrl -UseBasicParsing -TimeoutSec 30
if ($release.Content -is [byte[]]) {
  $bytes = $release.Content
  if ($bytes.Length -ge 2 -and $bytes[0] -eq 0xFF -and $bytes[1] -eq 0xFE) {
    $driverVersion = [System.Text.Encoding]::Unicode.GetString($bytes)
  }
  else {
    $driverVersion = [System.Text.Encoding]::UTF8.GetString($bytes)
  }
}
else {
  $driverVersion = [string]$release.Content
}
$driverVersion = $driverVersion.Trim([char]0xFEFF).Trim()
if ($driverVersion -notmatch '^\d+\.\d+\.\d+\.\d+$') {
  throw "Versao inesperada do MSEdgeDriver para $major : $driverVersion"
}

$zipPath = Join-Path $InstallDir 'edgedriver.zip'
$zipUrl = "https://msedgedriver.microsoft.com/$driverVersion/edgedriver_win64.zip"
Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing -TimeoutSec 120

Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction SilentlyContinue
$extractDir = Join-Path $InstallDir "extract-$driverVersion"
if (Test-Path -LiteralPath $extractDir) { Remove-Item -LiteralPath $extractDir -Recurse -Force }
[System.IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $extractDir)
Copy-Item -LiteralPath (Join-Path $extractDir 'msedgedriver.exe') -Destination $driverPath -Force
Remove-Item -LiteralPath $extractDir -Recurse -Force
Remove-Item -LiteralPath $zipPath -Force

if ($env:GITHUB_PATH) { Add-Content -Path $env:GITHUB_PATH -Value $InstallDir }
else { $env:PATH = "$InstallDir;$env:PATH" }
Write-Host "MSEdgeDriver $driverVersion instalado em $driverPath"
