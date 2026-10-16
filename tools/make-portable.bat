@echo off
rem Build the portable program folder (07 > Program folder, task 2.13) without opening Git Bash
rem first: this wrapper finds Git for Windows' bash.exe and runs make-portable.sh, which holds the
rem steps (build, then Packing.exe + pdfium.dll into target\portable\Packing\).
rem
rem   make-portable.bat             build; wait for a key at the end (keeps a double-click's output)
rem   make-portable.bat --no-pause  build and return (for scripts)
rem
rem Double-click it, or from cmd:  pc\tools\make-portable.bat
setlocal
cd /d "%~dp0"

set "BASH="
for %%B in (
  "%ProgramFiles%\Git\bin\bash.exe"
  "%ProgramFiles(x86)%\Git\bin\bash.exe"
  "%LocalAppData%\Programs\Git\bin\bash.exe"
) do if not defined BASH if exist %%B set "BASH=%%~B"

if not defined BASH (
  echo error: Git for Windows' bash.exe was not found.
  echo   Looked in %%ProgramFiles%%\Git\bin, %%ProgramFiles(x86^)%%\Git\bin and
  echo   %%LocalAppData%%\Programs\Git\bin.
  echo   Install Git for Windows ^(https://git-scm.com/download/win^), or run
  echo   bash tools/make-portable.sh from a Git Bash window.
  if /i not "%~1"=="--no-pause" pause
  exit /b 1
)

echo ==^> Using "%BASH%"
echo.
"%BASH%" "%~dp0make-portable.sh"
set "CODE=%ERRORLEVEL%"

if not "%CODE%"=="0" (
  echo.
  echo error: the build failed, exit code %CODE%.
)
echo.
if /i not "%~1"=="--no-pause" pause
exit /b %CODE%
