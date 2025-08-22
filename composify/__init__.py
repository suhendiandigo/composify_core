"""Composify framework, simplify object construction using set of rules."""

from .core import *  # noqa

__doc__ = core.__doc__
if hasattr(core, "__all__"):
    __all__ = core.__all__
