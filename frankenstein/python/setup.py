# setup.py (requires CFFI to be installed first)
from setuptools import setup

import engine_build

setup(
    ext_modules=[engine_build.ffibuilder.distutils_extension()],
    test_suite = 'nose.collector',
)