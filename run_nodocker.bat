echo off
echo === STARTING THE MANAGER (no docker) ===
echo === ensuring dependencies ===
echo $ py -3 -m pip install --user cffi eel tqdm werkzeug psutil requests
py -3 -m pip install --user cffi eel tqdm werkzeug psutil requests
@if %errorlevel% neq 0 echo "Warning: pip3 install failed"

set PYTHONPATH=%~dp0\battlecode\python
echo %PYTHONPATH%
set NODOCKER=1
py -3 %~dp0\battlecode-manager\gui.py
