echo off
echo === RUNNING A SINGLE GAME (no docker) ===
echo === ensuring dependencies ===
echo $ pip3 install --user cffi tqdm werkzeug psutil
pip3 install --user cffi tqdm werkzeug psutil
@if %errorlevel% neq 0 echo "Warning: pip3 install failed"

set PYTHONPATH=%~dp0\battlecode\python
echo %PYTHONPATH%
set NODOCKER=1
python %~dp0\battlecode-manager\simple_cli.py %1 %2 %3 %4 %5 %6 %7 %8 %9
