@echo off
REM MinGW/Clang linker wrapper for Frozen v1 (dev only)
REM Usage:
REM   tools\aot\windows\link_with_clang_mingw.bat out.exe obj1.o [obj2.o ...] [--nyrt path\to\libhako_kernel.a] [--allow-multi]

setlocal ENABLEDELAYEDEXPANSION

if "%~1"=="" (
  echo Usage: %~nx0 out.exe obj1.o [obj2.o ...] [--nyrt path\to\libhako_kernel.a] [--allow-multi]
  exit /b 2
)

set OUT=%~1
shift

set NYRT=
set ALLOW=
set OBJS=

:loop
if "%~1"=="" goto run
if "%~1"=="--nyrt" (
  set NYRT=%~2
  shift
  shift
  goto loop
)
if "%~1"=="--allow-multi" (
  set ALLOW=-Wl,--allow-multiple-definition
  shift
  goto loop
)
set OBJS=!OBJS! %~1
shift
goto loop

:run
set CMD=clang %OBJS% -o %OUT%
if not "%NYRT%"=="" set CMD=%CMD% -Wl,--whole-archive "%NYRT%" -Wl,--no-whole-archive
if not "%ALLOW%"=="" set CMD=%CMD% %ALLOW%
echo [link] %CMD%
%CMD%
if errorlevel 1 exit /b %ERRORLEVEL%
echo [link] done: %OUT%

endlocal
exit /b 0

