# -*- mode: python -*-

block_cipher = None


a = Analysis(['battlecode_run.py'],
             pathex=['/Users/joshuagruenstein/Documents/Projects/battlecode-2018/battlecode-manager'],
             binaries=[],
             datas=[],
             hiddenimports=[],
             hookspath=[],
             runtime_hooks=[],
             excludes=[],
             win_no_prefer_redirects=False,
             win_private_assemblies=False,
             cipher=block_cipher)
pyz = PYZ(a.pure, a.zipped_data,
             cipher=block_cipher)
exe = EXE(pyz,
          a.scripts,
          a.binaries,
          a.zipfiles,
          a.datas,
          name='battlecode_run',
          debug=False,
          strip=False,
          upx=True,
          runtime_tmpdir=None,
          console=False )
app = BUNDLE(exe,
             name='battlecode_run.app',
             icon=None,
             bundle_identifier=None)
