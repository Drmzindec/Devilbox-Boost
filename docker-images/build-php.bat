@echo off
setlocal

REM Build custom PHP images for Devilbox (Windows)
REM Usage: build-php.bat [VERSION]
REM Example: build-php.bat 8.3

REM Get PHP version from argument or default to 8.3
set VERSION=%1
if "%VERSION%"=="" set VERSION=8.3

REM Determine script directory
set SCRIPT_DIR=%~dp0
set IMAGE_DIR=%SCRIPT_DIR%php-%VERSION%-work

REM Check if image directory exists
if not exist "%IMAGE_DIR%" (
    echo Error: Directory %IMAGE_DIR% does not exist
    echo.
    echo Available versions:
    for /d %%d in ("%SCRIPT_DIR%php-*-work") do (
        set "dirname=%%~nxd"
        set "dirname=!dirname:php-=!"
        set "dirname=!dirname:-work=!"
        echo   !dirname!
    )
    exit /b 1
)

echo Building PHP %VERSION% work image...
echo Image directory: %IMAGE_DIR%
echo.

REM Note: Windows doesn't have UID/GID like Linux
REM We'll use default values that work with Docker Desktop
set HOST_UID=1000
set HOST_GID=1000

echo Building with:
echo   UID: %HOST_UID%
echo   GID: %HOST_GID%
echo.
echo This may take 10-15 minutes...
echo.

REM Build the image
docker build ^
    --build-arg NEW_UID=%HOST_UID% ^
    --build-arg NEW_GID=%HOST_GID% ^
    -t devilbox-php-%VERSION%:work ^
    "%IMAGE_DIR%"

if errorlevel 1 (
    echo.
    echo ERROR: Build failed!
    exit /b 1
)

echo.
echo Successfully built devilbox-php-%VERSION%:work
echo.
echo To use this image with Devilbox:
echo 1. Set PHP_SERVER=%VERSION% in .env
echo 2. Modify docker-compose.yml to use: image: devilbox-php-%VERSION%:work
echo    (or use docker-compose.override.yml)
echo 3. Run: docker-compose up httpd php mysql
echo.
