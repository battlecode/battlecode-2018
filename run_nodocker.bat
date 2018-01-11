echo off
echo "=== STARTING THE MANAGER (no docker) ==="
echo "=== ensuring dependencies ==="
echo "$ pip3 install --user cffi eel tqdm werkzeug ujson psutil"
pip3 install --user cffi eel tqdm werkzeug ujson psutil
@if %errorlevel% neq 0 echo "Warning: pip3 install failed"

set PYTHONPATH="battlecode\python;%PYTHONPATH%"
set NODOCKER=1
python3 %~dp0\battlecode-manager\gui.py