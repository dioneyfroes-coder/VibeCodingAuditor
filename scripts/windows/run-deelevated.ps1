<#
.SYNOPSIS
  Executa um comando com integridade média no Windows sem dependências externas.

.DESCRIPTION
  Runners do GitHub Actions executam elevados. Desde o WebView2 150, um host
  elevado ignora --remote-debugging-port recebido por WEBVIEW2_* ou HKCU, o que
  impede o msedgedriver de criar a sessão do tauri-driver. O runas integrado ao
  Windows permite gerar um processo de integridade média com o mesmo usuário.
  Este script espera o filho, devolve a saída e propaga o código de saída.
#>
param(
  [Parameter(Mandatory = $true)]
  [string] $Command,

  [string] $WorkingDirectory = (Get-Location).Path,

  [int] $TimeoutMinutes = 30
)

$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $WorkingDirectory)) {
  throw "Diretório de trabalho inexistente: $WorkingDirectory"
}

$root = Join-Path $env:TEMP ('vca-medium-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root -Force | Out-Null
# O filho usa token médio: logs e status não podem herdar o rótulo alto do runner.
& icacls.exe $root /setintegritylevel '(CI)(OI)M' /T /Q | Out-Null

$batch = Join-Path $root 'run.cmd'
$outputLog = Join-Path $root 'output.log'
$exitFile = Join-Path $root 'exit.code'

$batchContent = @"
@echo off
chcp 65001 >nul
cd /d "$WorkingDirectory"
call $Command > "$outputLog" 2>&1
echo %ERRORLEVEL% > "$exitFile"
"@
Set-Content -LiteralPath $batch -Value $batchContent -Encoding ascii

& runas.exe /trustlevel:0x20000 "`"$batch`""
if ($LASTEXITCODE -ne 0) {
  throw "runas falhou ao iniciar o processo de integridade média (código $LASTEXITCODE)."
}

$deadline = (Get-Date).AddMinutes($TimeoutMinutes)
while (-not (Test-Path -LiteralPath $exitFile)) {
  if ((Get-Date) -ge $deadline) {
    throw "Tempo limite de $TimeoutMinutes min aguardando o processo de integridade média."
  }
  Start-Sleep -Milliseconds 250
}

if (Test-Path -LiteralPath $outputLog) {
  Get-Content -LiteralPath $outputLog -Encoding UTF8
}

$exitText = (Get-Content -LiteralPath $exitFile -Raw).Trim()
$exitCode = 0
if (-not [int]::TryParse($exitText, [ref] $exitCode)) {
  throw "Código de saída inválido do processo de integridade média: '$exitText'."
}
exit $exitCode
