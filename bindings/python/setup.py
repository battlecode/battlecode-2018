# setup.py (requires CFFI to be installed first)
from setuptools import setup

import bc_build

setup(
    ext_modules=[bc_build.ffibuilder.distutils_extension()],
    test_suite = 'nose.collector',py_modules = ['battlecode']
)
