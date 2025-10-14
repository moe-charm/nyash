@echo off
REM MSVC/clang-cl linker wrapper for Frozen v1 (dev only)
REM Usage:
REM   tools\aot\windows\link_with_clang_cl.bat out.exe obj1.obj [obj2.obj ...] [--libdir path\to\dir] [--nyrt hako_kernel.lib] [--allow-multi]

setlocal ENABLEDELAYEDEXPANSION

if "%~1"=="" (
  echo Usage: %~nx0 out.exe obj1.obj [obj2.obj ...] [--libdir path\to\dir] [--nyrt hako_kernel.lib] [--allow-multi]
  exit /b 2
)

set OUT=%~1
shift

set LIBDIR=
set NYRT=
set ALLOW=
set OBJS=

:loop
if "%~1"=="" goto run
if "%~1"=="--libdir" (
  set LIBDIR=%~2
  shift
  shift
  goto loop
)
if "%~1"=="--nyrt" (
  set NYRT=%~2
  shift
  shift
  goto loop
)
if "%~1"=="--allow-multi" (
  set ALLOW=/FORCE:MULTIPLE
  shift
  goto loop
)
set OBJS=!OBJS! %~1
shift
goto loop

:run
set CMD=clang-cl /Fe:%OUT% %OBJS%
if not "%LIBDIR%"=="" set CMD=%CMD% /link /LIBPATH:%LIBDIR%
if not "%NYRT%"=="" set CMD=%CMD% %NYRT%
if not "%ALLOW%"=="" set CMD=%CMD% %ALLOW%

echo [link] %CMD%
%CMD%
if errorlevel 1 exit /b %ERRORLEVEL%
echo [link] done: %OUT%

endlocal
exit /b 0

