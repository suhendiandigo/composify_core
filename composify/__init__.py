"""Composify framework, simplify object construction using set of rules."""

from .composify import *  # noqa

__doc__ = composify.__doc__
if hasattr(composify, "__all__"):
    __all__ = composify.__all__
