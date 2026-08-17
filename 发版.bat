@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ============================================
echo   桌面日程 · 一键发版
echo   前提：版本号4处已同步、更新公告已写好
echo ============================================

REM 读取版本号（package.json 为准）
for /f %%v in ('powershell -NoProfile -Command "(Get-Content desktop-schedule\package.json | ConvertFrom-Json).version"') do set VER=%%v
echo 本次发版版本： v%VER%
echo.
set /p CONFIRM=确认发版？ (y=继续):
if /i not "%CONFIRM%"=="y" goto :cancelled

echo [1/5] 构建 release ...
cd desktop-schedule
call npx tauri build --no-bundle
if errorlevel 1 goto :fail
cd ..

echo [2/5] 重新生成 README（简介 + 更新公告）...
copy /b readme-intro.md + 更新公告.md README.md >nul

echo [3/5] 提交并推送 GitHub（走 Clash 代理）...
git add -A
git commit -m "release: v%VER%"
git push
if errorlevel 1 goto :fail

echo [4/5] 计算 SHA-256 并创建 GitHub Release ...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0发版.ps1" -Ver %VER%
if errorlevel 1 goto :fail

echo [5/5] 完成！
echo   https://github.com/AyanMeow/desktop-schedule/releases/latest
pause
goto :eof

:fail
echo.
echo 发版失败，请检查上方错误输出。
pause
goto :eof

:cancelled
echo 已取消。
pause
