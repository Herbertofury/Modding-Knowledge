"""Validate explicitly selected staging entries and read a bounded ordinary file."""
import os
import stat


def reparse(s):
    return getattr(s, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0)


def safe_dir(path):
    s = path.lstat()
    return not path.is_symlink() and not reparse(s) and stat.S_ISDIR(s.st_mode)


def read_checked(path, initial, limit):
    flags = (os.O_RDONLY | getattr(os, "O_BINARY", 0)
             | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0))
    with os.fdopen(os.open(path, flags), "rb") as handle:
        now = os.fstat(handle.fileno())
        if ((now.st_dev, now.st_ino) != (initial.st_dev, initial.st_ino)
                or not stat.S_ISREG(now.st_mode) or reparse(now)):
            raise ValueError("Changed while opening")
        raw = handle.read(limit + 1)
        if len(raw) > limit:
            raise ValueError("Changed beyond budget")
        return raw
