# setup.py (requires CFFI to be installed first)
from setuptools import setup

import bc_build

setup(
    setup_requires=["cffi>=1.0.0"],
    cffi_modules=["bc_build.py:ffibuilder"],
    install_requires=["cffi>=1.0.0"],
    ext_modules=[bc_build.ffibuilder.distutils_extension()],
    test_suite = 'nose.collector',
    py_modules = ['battlecode.__init__', 'battlecode.bc']
)
