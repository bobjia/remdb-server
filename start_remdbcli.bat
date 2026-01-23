@echo off
chcp 65001 >nul

setlocal

set PROJECT_ROOT=%~dp0
set REMDBCLI_PATH=%PROJECT_ROOT%target\debug\remdbcli.exe

if not exist "%REMDBCLI_PATH%" (
    echo 错误: 未找到remdbcli可执行文件!
    echo 请先在项目根目录执行 cargo build 命令来构建项目
    pause
    exit /b 1
)

echo =======================================
echo          remdbcli 启动脚本          
echo =======================================
echo.
echo 启动remdbcli命令行工具...
echo.

"%REMDBCLI_PATH%" %*

endlocal