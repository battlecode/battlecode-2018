set -e

echo -e "--\e[32m Running rust tests \e[0m--"
echo ''
echo 'python3 setup.py build_ext --inplace'
python3 setup.py build_ext --inplace
