"""GTIN parsing, canonicalisation, match keys and GS1 classification.

One implementation, shared with the TypeScript build and the Rust core. See
https://github.com/mommo-codes/barkod.

    >>> import barkod
    >>> barkod.store_form("7350053850019")
    '07350053850019'
    >>> barkod.key("7350053850019").as_key_str()
    '07350053850010'
    >>> barkod.parse("lek").reason
    'non_numeric'
"""

from ._barkod import (
    DOMAIN_MAX_DIGITS,
    DOMAIN_MIN_DIGITS,
    Gtin14,
    GtinKey,
    Parsed,
    key,
    key_strings,
    parse,
    parse_many,
    store_form,
    store_form_many,
)

__all__ = [
    "DOMAIN_MAX_DIGITS",
    "DOMAIN_MIN_DIGITS",
    "Gtin14",
    "GtinKey",
    "Parsed",
    "key",
    "key_strings",
    "parse",
    "parse_many",
    "store_form",
    "store_form_many",
]
