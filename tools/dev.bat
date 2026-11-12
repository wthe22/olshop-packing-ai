@echo off
rem Run the app as a dev build: a debug build that compiles much faster than the release one
rem make-portable builds, with the front end on the Vite dev server (changes show without a rebuild).
rem pdfium.dll is taken from vendor\ (tools\get-pdfium.sh). Close the window to stop.
rem
rem Double-click it, or from cmd:  tools\dev.bat
setlocal
cd /d "%~dp0..\app"
call npm run tauri dev
set "CODE=%ERRORLEVEL%"
if not "%CODE%"=="0" (
  echo.
  echo error: the dev build failed, exit code %CODE%.
  pause
)
exit /b %CODE%
