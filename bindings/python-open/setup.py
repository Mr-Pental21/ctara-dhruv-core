import os
import subprocess
import sys
from pathlib import Path

from setuptools import setup
from setuptools.dist import Distribution
from setuptools.command.build_py import build_py


def _prepare_native_lib():
    if os.environ.get("DHRUV_SKIP_NATIVE_PREP") == "1":
        return

    script = Path(__file__).resolve().parent / "scripts" / "prepare_native_lib.py"
    subprocess.run([sys.executable, str(script)], check=True)


class BinaryDistribution(Distribution):
    """Force platform-specific wheel tags (not py3-none-any)."""

    def has_ext_modules(self):
        return True


class BuildPyWithNativePrep(build_py):
    def run(self):
        _prepare_native_lib()
        super().run()


setup(
    distclass=BinaryDistribution,
    cmdclass={"build_py": BuildPyWithNativePrep},
)
