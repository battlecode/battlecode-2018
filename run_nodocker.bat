echo off
echo === STARTING THE MANAGER (no docker) ===
echo === ensuring dependencies ===
echo $ pip3 install --user cffi eel tqdm werkzeug psutil
pip3 install --user cffi eel tqdm werkzeug psutil
@if %errorlevel% neq 0 echo "Warning: pip3 install failed"

set PYTHONPATH=%~dp0\battlecode\python
echo %PYTHONPATH%
set NODOCKER=1
python %~dp0\battlecode-manager\gui.py
