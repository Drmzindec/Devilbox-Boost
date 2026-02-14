@echo off
setlocal enabledelayedexpansion

REM ###############################################################################
REM Devilbox Boost Setup for Windows
REM Simplified setup script for Windows users
REM
REM NOTE: For the full interactive wizard, use WSL2 and run setup-devilbox.sh
REM ###############################################################################

echo.
echo ========================================================================
echo   Devilbox Boost - Windows Setup
echo ========================================================================
echo.
echo NOTE: This is a simplified setup for Windows.
echo For the best experience, we recommend using WSL2 and running:
echo   setup-devilbox.sh
echo.
pause

REM Check if Docker is running
echo.
echo [1/5] Checking Docker...
docker version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Docker is not running or not installed!
    echo.
    echo Please install Docker Desktop for Windows:
    echo https://www.docker.com/products/docker-desktop
    echo.
    echo Make sure Docker Desktop is running before continuing.
    pause
    exit /b 1
)
echo  - Docker is running

REM Check if .env exists
echo.
echo [2/5] Checking configuration...
if not exist ".env" (
    echo  - Creating .env from env-example...
    copy env-example .env >nul
    echo  - Created .env file
) else (
    echo  - .env file already exists
)

REM Configure basic settings
echo.
echo [3/5] Basic Configuration
echo.
echo Press ENTER to use defaults shown in [brackets]
echo.

REM PHP Version
set /p PHP_VERSION="Select PHP version (8.3 or 8.4) [8.4]: "
if "!PHP_VERSION!"=="" set PHP_VERSION=8.4

REM MySQL Password
set /p MYSQL_PASS="MySQL root password [root]: "
if "!MYSQL_PASS!"=="" set MYSQL_PASS=root

REM HTTP Port
set /p HTTP_PORT="HTTP port (80 for localhost, 8000 for localhost:8000) [80]: "
if "!HTTP_PORT!"=="" set HTTP_PORT=80

REM HTTPS Port
set /p HTTPS_PORT="HTTPS port [443]: "
if "!HTTPS_PORT!"=="" set HTTPS_PORT=443

REM TLD Suffix
set /p TLD="TLD suffix for projects (.local, .test, .dev) [local]: "
if "!TLD!"=="" set TLD=local

echo.
echo  - Updating .env with your settings...

REM Update .env file (using PowerShell for better file manipulation)
powershell -Command "(gc .env) -replace '^PHP_SERVER=.*', 'PHP_SERVER=%PHP_VERSION%' | Out-File -encoding ASCII .env.tmp"
move /y .env.tmp .env >nul

powershell -Command "(gc .env) -replace '^MYSQL_ROOT_PASSWORD=.*', 'MYSQL_ROOT_PASSWORD=%MYSQL_PASS%' | Out-File -encoding ASCII .env.tmp"
move /y .env.tmp .env >nul

powershell -Command "(gc .env) -replace '^HOST_PORT_HTTPD=.*', 'HOST_PORT_HTTPD=%HTTP_PORT%' | Out-File -encoding ASCII .env.tmp"
move /y .env.tmp .env >nul

powershell -Command "(gc .env) -replace '^HOST_PORT_HTTPS=.*', 'HOST_PORT_HTTPS=%HTTPS_PORT%' | Out-File -encoding ASCII .env.tmp"
move /y .env.tmp .env >nul

powershell -Command "(gc .env) -replace '^TLD_SUFFIX=.*', 'TLD_SUFFIX=%TLD%' | Out-File -encoding ASCII .env.tmp"
move /y .env.tmp .env >nul

echo  - Configuration updated

REM Build PHP images
echo.
echo [4/5] Build Custom PHP Images
echo.
echo Would you like to build custom PHP %PHP_VERSION% image now?
echo This includes: Laravel, WP-CLI, Bun, Vite, Pest, React, Vue, Angular
echo.
echo WARNING: This will take 10-15 minutes
echo.
set /p BUILD_IMAGE="Build PHP %PHP_VERSION% image? (y/n) [y]: "
if "!BUILD_IMAGE!"=="" set BUILD_IMAGE=y

if /i "!BUILD_IMAGE!"=="y" (
    echo.
    echo Building PHP %PHP_VERSION% image...
    cd docker-images
    call build-php.bat %PHP_VERSION%
    cd ..
    if errorlevel 1 (
        echo.
        echo WARNING: Image build failed. You can build it later with:
        echo   docker-images\build-php.bat %PHP_VERSION%
        echo.
    ) else (
        echo.
        echo  - PHP %PHP_VERSION% image built successfully
    )
) else (
    echo.
    echo Skipped. You can build the image later with:
    echo   docker-images\build-php.bat %PHP_VERSION%
    echo.
)

REM Start containers
echo.
echo [5/5] Starting Devilbox
echo.
set /p START_NOW="Start Devilbox containers now? (y/n) [y]: "
if "!START_NOW!"=="" set START_NOW=y

if /i "!START_NOW!"=="y" (
    echo.
    echo Starting containers...
    docker-compose up -d httpd php mysql

    if errorlevel 1 (
        echo.
        echo ERROR: Failed to start containers
        echo.
        echo Try running manually:
        echo   docker-compose up -d httpd php mysql
        echo.
        pause
        exit /b 1
    )

    echo.
    echo ========================================================================
    echo   Setup Complete!
    echo ========================================================================
    echo.
    echo  Dashboard:    http://localhost:%HTTP_PORT%
    echo  phpMyAdmin:   http://localhost:%HTTP_PORT%/vendor/phpmyadmin-5.2.3/
    echo  Adminer:      http://localhost:%HTTP_PORT%/vendor/adminer-5.4.2-devilbox.php
    echo.
    echo  Database Connection (from projects):
    echo    Host: 127.0.0.1
    echo    User: root
    echo    Pass: %MYSQL_PASS%
    echo.
    echo  Your projects go in: data\www\
    echo  Each project accessible at: http://project-name.%TLD%
    echo.
    echo  Enter PHP container:
    echo    shell.bat
    echo.
    echo  Create Laravel project:
    echo    docker-compose exec php laravel new my-project
    echo    Visit: http://my-project.%TLD%
    echo.
    echo ========================================================================
    echo.
    echo  Next Steps:
    echo  1. Read QUICKSTART.md for detailed usage
    echo  2. Create your first project in data\www\
    echo  3. Visit http://localhost:%HTTP_PORT% to see dashboard
    echo.
    echo ========================================================================
    echo.
) else (
    echo.
    echo Skipped. You can start containers later with:
    echo   docker-compose up -d httpd php mysql
    echo.
)

echo Setup wizard complete!
echo.
echo For advanced features and full wizard, consider using WSL2:
echo   wsl
echo   cd /path/to/devilbox
echo   ./setup-devilbox.sh
echo.
pause
