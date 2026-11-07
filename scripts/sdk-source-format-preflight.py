#!/usr/bin/env python3
"""Negative-only CI hint, never a replacement for LexLean or SDK acceptance.

Recognize a small, anchored native-declaration signature in language 1.1.
Unknown syntax, malformed configuration and exhausted inspection budgets all
continue to the real compiler. Do not extend this into a source parser.
"""
import errno
import os
from pathlib import Path
import re
import stat
import sys
import tomllib
from typing import NamedTuple


class Failure(NamedTuple):
    code: str
    path: str
    line: int
    reason: str


class Delegate(Exception):
    """The full SDK, not this optional inspection, decides the input."""


class Unsafe(Exception):
    def __init__(self, path):
        self.path = path


# This deliberately recognizes only the complete conventional header used by
# the failing source. No searching inside strings, comments or arbitrary text.
# LexLean SPEC 17.21.1: language 1.1 admits only the semanticdata form.
_NAME = rb'[A-Z][A-Za-z0-9_]*(?:\.[A-Z][A-Za-z0-9_]*)*'
_NATIVE_START = re.compile(
    rb'\A\\begin\{lexlean\}\{' + _NAME + rb'\}\n'
    rb'(?:\\importmodule\{' + _NAME + rb'\}\n)*'
    rb'\\title\{Parameters\}\n\\begin\{semanticmodule\}\n[ \n]*'
    rb'(?P<declaration>inductive|structure|definition)\{'
)
_DIRECTORY = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_NONBLOCK
_FILE = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK


def _parts(path):
    if (not isinstance(path, str) or not path or '\\' in path or '\x00' in path
            or any(part in ('', '.', '..') for part in path.split('/'))):
        raise Unsafe(str(path))
    return path.split('/')


def _open(name, flags, parent, display):
    try:
        return os.open(name, flags, dir_fd=parent)
    except OSError as error:
        if error.errno in (errno.ELOOP, errno.ENOTDIR):
            raise Unsafe(display) from error
        raise Delegate from error


def _directory(root, relative):
    current = os.dup(root)
    try:
        prefix = []
        for part in _parts(relative):
            prefix.append(part)
            child = _open(part, _DIRECTORY, current, '/'.join(prefix))
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def inspect_project(root=Path('.'), *, max_entries=4096, max_bytes=4194304,
                    prefix_bytes=16384, max_depth=64):
    """Return only a known rejection or unsafe-path failure; None delegates.

    Budgets cap this hint's work, never the product's accepted input. A file
    larger than its inspected prefix is not rejected for being large.
    """
    remaining_entries, remaining_bytes = max_entries, max_bytes
    root_fd = None

    def read_prefix(parent, name, relative, amount, configuration=False):
        nonlocal remaining_bytes
        descriptor = _open(name, _FILE, parent, relative)
        try:
            metadata = os.fstat(descriptor)
            if not stat.S_ISREG(metadata.st_mode):
                raise Unsafe(relative)
            if configuration and metadata.st_size > amount:
                raise Delegate
            if amount > remaining_bytes:
                raise Delegate
            content = bytearray()
            while len(content) < amount:
                block = os.read(descriptor, min(4096, amount - len(content)))
                if not block:
                    break
                content.extend(block)
                remaining_bytes -= len(block)
            return bytes(content)
        finally:
            os.close(descriptor)

    def walk(directory, relative, depth):
        nonlocal remaining_entries
        if depth > max_depth:
            raise Delegate
        # Materialize only the bounded directory inventory, never an unbounded
        # rglob/listdir. Descriptor-relative opens refuse symlink ancestors.
        names = []
        with os.scandir(directory) as entries:
            for entry in entries:
                if remaining_entries <= 0:
                    raise Delegate
                remaining_entries -= 1
                names.append(entry.name)
        for name in sorted(names):
            display = f'{relative}/{name}'
            metadata = os.stat(name, dir_fd=directory, follow_symlinks=False)
            if stat.S_ISLNK(metadata.st_mode):
                raise Unsafe(display)
            if stat.S_ISDIR(metadata.st_mode):
                child = _open(name, _DIRECTORY, directory, display)
                try:
                    failure = walk(child, display, depth + 1)
                    if failure:
                        return failure
                finally:
                    os.close(child)
            elif not stat.S_ISREG(metadata.st_mode):
                raise Unsafe(display)
            elif name.endswith('.lex.tex'):
                prefix = read_prefix(directory, name, display, prefix_bytes)
                match = _NATIVE_START.match(prefix)
                if match:
                    line = prefix.count(b'\n', 0, match.start('declaration')) + 1
                    declaration = match['declaration'].decode('ascii')
                    return Failure('UCF001', display, line,
                                   r'language 1.1 requires \semanticdata; '
                                   f'found native authoring declaration {declaration}{{ '
                                   '(known LLP2003 incompatibility; full SDK acceptance has not run)')
        return None

    try:
        root_fd = _open(os.fspath(root), _DIRECTORY, None, '.')
        config_bytes = read_prefix(root_fd, 'lexlean.toml', 'lexlean.toml', 65536, True)
        try:
            config = tomllib.loads(config_bytes.decode('utf-8'))
        except (tomllib.TOMLDecodeError, UnicodeDecodeError):
            raise Delegate
        if config.get('spec') != 'lexlean/project/1' or config.get('language') != '1.1':
            raise Delegate
        roots = config.get('source_roots')
        if not isinstance(roots, list) or not roots or not all(isinstance(item, str) for item in roots):
            raise Delegate
        # Validate every configured path before opening any of them.
        for relative in roots:
            if len(_parts(relative)) > max_depth:
                raise Delegate
        for relative in roots:
            directory = _directory(root_fd, relative)
            try:
                failure = walk(directory, relative, 1)
                if failure:
                    return failure
            finally:
                os.close(directory)
    except Delegate:
        return None
    except Unsafe as error:
        return Failure('UCF002', error.path, 1,
                       'cannot safely inspect path (symlink, non-regular file, or unsafe path); nothing followed')
    except OSError:
        # A concurrent filesystem change or unreadable inventory is not a
        # format prediction. The unchanged SDK still decides the project.
        return None
    finally:
        if root_fd is not None:
            os.close(root_fd)
    return None


def main():
    if len(sys.argv) != 1:
        print('UCF003: source-format preflight takes no arguments', file=sys.stderr)
        return 2
    failure = inspect_project()
    if failure:
        print(f'{failure.path}:{failure.line}: {failure.code}: {failure.reason}', file=sys.stderr)
        return 1 if failure.code == 'UCF001' else 2
    print('SDK source-format preflight: no rejection predicted; full SDK checks remain required.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
