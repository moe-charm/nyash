@echo off
echo Installing libffi using vcpkg...
cd C:\vcpkg
git pull
.\bootstrap-vcpkg.bat
.\vcpkg integrate install
.\vcpkg install libffi:x64-windows
echo Done!