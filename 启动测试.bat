@echo off
setlocal
title Desktop Schedule - Test Launcher
cd /d "%~dp0desktop-schedule"

REM Put cargo/rustc on PATH (needed for Tauri build)
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

:menu
cls
echo ============================================
echo         Desktop Schedule - Test Launcher
echo ============================================
echo.
echo   [1] Dev mode        (npm run tauri dev)
echo       Hot reload, for daily testing
echo.
echo   [2] Rust check      (cargo check)
echo       Verify backend compiles, fastest
echo.
echo   [3] Type check      (vue-tsc)
echo       Verify TypeScript types
echo.
echo   [4] Release build   (tauri build, installer)
echo       Slower, for release
echo.
echo   [5] Portable exe    (tauri build --no-bundle)
echo       +crt-static, zip-and-run
echo.
echo   [6] Clean build     (cargo clean, free space)
echo.
echo   [0] Exit
echo.
set /p choice=Select:

if "%choice%"=="1" goto dev
if "%choice%"=="2" goto check
if "%choice%"=="3" goto tsc
if "%choice%"=="4" goto build
if "%choice%"=="5" goto portable
if "%choice%"=="6" goto clean
if "%choice%"=="0" exit /b 0
goto menu

:dev
echo.
echo [Dev mode] First build ~2-3 min, please wait...
echo "Running target\debug\desktop-schedule.exe" means success.
echo Close window or Ctrl+C to stop.
echo ------------------------------------------------------------
call npm run tauri dev
echo.
echo --- Dev mode exited ---
pause
goto menu

:check
echo.
cd src-tauri
cargo check
cd ..
echo.
pause
goto menu

:tsc
echo.
call npx vue-tsc --noEmit
echo Exit code: %errorlevel%  (0 = pass)
echo.
pause
goto menu

:build
echo.
echo [Release build] This takes a while (5-10 min)...
call npm run tauri build
echo.
echo Output: src-tauri\target\release\bundle\
echo.
pause
goto menu

:portable
echo.
echo [Portable exe build] ...
call npm run tauri build -- --no-bundle
echo.
echo Portable exe: src-tauri\target\release\desktop-schedule.exe
echo CRT statically linked, runs on Win10/11 with WebView2.
echo.
pause
goto menu

:clean
echo.
echo Cleaning target dir...
cd src-tauri
cargo clean
cd ..
echo Done.
echo.
pause
goto menu
