@echo off
setlocal
set PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\HostX64\x86;C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE;%PATH%

echo Compiling wrapper...
cl.exe /c /O2 /MD /Fo:wrapper.o wrapper.c
if errorlevel 1 goto error

echo Creating DLL...
link.exe /DLL /DEF:exports.def /OUT:goldsrc_metamod_backend.dll wrapper.o target\i686-pc-windows-msvc\release\libgoldsrc_metamod_backend.a /MACHINE:X86 /DEFAULTLIB:msvcrt /DEFAULTLIB:kernel32
if errorlevel 1 goto error

echo Success!
goto end

:error
echo Failed!

:end
endlocal
