@echo off
set "VCA_CLI=%~dp0target\release\vibe-coding-auditor-cli.exe"
if not exist "%VCA_CLI%" set "VCA_CLI=%~dp0target\debug\vibe-coding-auditor-cli.exe"
"%VCA_CLI%" %*
if "%~1"=="" (
  echo.
  echo Pressione qualquer tecla para fechar esta janela.
  pause >nul
)
