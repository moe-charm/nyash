@echo off
echo === NyaMesh Editor Clean Script ===
echo.

echo Removing build directories...
if exist build rmdir /s /q build
if exist build-* rmdir /s /q build-*

echo Removing CMake cache files...
if exist CMakeCache.txt del CMakeCache.txt
if exist CMakeFiles rmdir /s /q CMakeFiles

echo Removing temporary files...
if exist *.log del *.log
if exist _deps rmdir /s /q _deps

echo.
echo === Clean complete ===