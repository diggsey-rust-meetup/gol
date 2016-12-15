@echo off
set FXC="%DXSDK_DIR%\Utilities\bin\x64\fxc.exe" -nologo
if not exist data mkdir data
%FXC% /T vs_4_0 /E VS /Fo data/vs.fx shader/gol.hlsl
%FXC% /T ps_4_0 /E PS /Fo data/ps.fx shader/gol.hlsl
%FXC% /T ps_4_0 /E PS_Display /Fo data/ps_display.fx shader/gol.hlsl
