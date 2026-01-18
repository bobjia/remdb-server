@echo off
REM High Performance JDBC Server Start Script

REM Default Configuration
set JDBC_PORT=6666
set MAX_CONNECTIONS=10000
set AUTH_ENABLED=false
set USERNAME=
set PASSWORD_HASH=
set LOG_LEVEL=info

REM Show Help
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help

REM Parse Command Line Arguments
:parse
if "%1"=="" goto :end_parse

if "%1"=="--port" (
    set JDBC_PORT=%2
    shift
    shift
    goto :parse
)

if "%1"=="-p" (
    set JDBC_PORT=%2
    shift
    shift
    goto :parse
)

if "%1"=="--connections" (
    set MAX_CONNECTIONS=%2
    shift
    shift
    goto :parse
)

if "%1"=="-c" (
    set MAX_CONNECTIONS=%2
    shift
    shift
    goto :parse
)

if "%1"=="--auth-enabled" (
    set AUTH_ENABLED=true
    shift
    goto :parse
)

if "%1"=="--username" (
    set USERNAME=%2
    shift
    shift
    goto :parse
)

if "%1"=="--password-hash" (
    set PASSWORD_HASH=%2
    shift
    shift
    goto :parse
)

if "%1"=="--log-level" (
    set LOG_LEVEL=%2
    shift
    shift
    goto :parse
)

if "%1"=="-l" (
    set LOG_LEVEL=%2
    shift
    shift
    goto :parse
)

:end_parse

REM Build Project
echo Building JDBC Server...
cargo build --release
if errorlevel 1 (
    echo Build Failed!
    exit /b 1
)

REM Set Environment Variables
set RUST_LOG=%LOG_LEVEL%
set RUST_BACKTRACE=1

REM Start Server
if "%AUTH_ENABLED%"=="true" (
    echo Starting JDBC Server with Authentication...
    target\release\remdb-server.exe --jdbc-enabled true --jdbc-port %JDBC_PORT% --max-connections %MAX_CONNECTIONS% --jdbc-auth-enabled true --jdbc-username "%USERNAME%" --jdbc-password-hash "%PASSWORD_HASH%"
) else (
    echo Starting JDBC Server...
    target\release\remdb-server.exe --jdbc-enabled true --jdbc-port %JDBC_PORT% --max-connections %MAX_CONNECTIONS%
)

goto :end

:help
echo Usage: %~nx0 [OPTIONS]
echo.
echo Options:
echo   --port PORT           JDBC server port (default: 6666)
echo   --connections N       Maximum connections (default: 10000)
echo   --auth-enabled        Enable authentication
echo   --username NAME       Authentication username
echo   --password-hash HASH  Authentication password hash (SHA-256)
echo   --log-level LEVEL     Log level (default: info)
echo   -h, --help            Show this help message
echo.
echo Examples:
echo   %~nx0 --port 8888 --connections 5000
echo   %~nx0 --auth-enabled --username admin --password-hash 8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918

goto :end

:end
