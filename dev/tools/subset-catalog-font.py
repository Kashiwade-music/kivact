"""Subset Noto Sans JP to the characters the catalogs and their tests use.

Run by hand, never by the build or CI, after adding text with characters the
catalogs did not use before (`examples/kivact-ui-basic`'s test
`every_page_character_is_in_the_catalog_fonts` fails until then):

    python dev/tools/subset-catalog-font.py

It reads the original Google Fonts file NotoSansJP[wght].ttf given with
`--source <file>` (download it from
https://github.com/google/fonts/tree/main/ofl/notosansjp), else from the
working tree or the git history. The original's SHA-256 is checked. The output
replaces
examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf. Needs
fontTools (`python -m pip install --user fonttools`).
"""

import argparse
import hashlib
import io
import pathlib
import subprocess
import sys

from fontTools import subset

ROOT = pathlib.Path(__file__).resolve().parents[2]
ORIGINAL = "examples/shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf"
OUTPUT = ROOT / "examples/shared-assets/fonts/notosansjp/NotoSansJP-subset[wght].ttf"
ORIGINAL_SHA256 = "c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f"
SCANNED = ["examples/kivact-ui-*", "crates/kivact-ui-*", "dev/tests/kivact-public-api"]
SUFFIXES = {".rs", ".ftl", ".json", ".toml", ".md"}
# Characters ICU writes for ja-JP dates and times (the catalogs show calendars
# in Japanese), whether or not a source uses them.
EXTRA = "年月日火水木金土曜午前後時分秒"
RANGES = [(0x0020, 0x007E), (0x00A0, 0x00FF), (0x2000, 0x206F), (0x3000, 0x30FF), (0xFF00, 0xFFEF)]


def git(*args):
    """The standard output of git run at the repository root."""
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True).stdout


def original(source):
    """The original font's bytes: `source`, the working tree's, or git's."""
    if source is not None:
        return source.read_bytes()
    path = ROOT / ORIGINAL
    if path.exists():
        return path.read_bytes()
    deleted = git("log", "--diff-filter=D", "--format=%H", "-1", "--", f":(literal){ORIGINAL}")
    deleted = deleted.decode().strip()
    if not deleted:
        sys.exit(f"{ORIGINAL} is neither in the working tree nor in the history; download NotoSansJP[wght].ttf from https://github.com/google/fonts/tree/main/ofl/notosansjp and pass --source <file> (SHA-256 {ORIGINAL_SHA256})")
    return git("show", f"{deleted}^:{ORIGINAL}")


def scanned_characters():
    """Every character above ASCII in the scanned sources."""
    found = set()
    for pattern in SCANNED:
        for top in sorted(ROOT.glob(pattern)):
            for path in sorted(top.rglob("*")):
                if path.suffix in SUFFIXES and path.is_file():
                    text = path.read_text(encoding="utf-8")
                    found.update(ord(c) for c in text if ord(c) >= 0x80)
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--source", type=pathlib.Path)
    args = parser.parse_args()
    data = original(args.source)
    if hashlib.sha256(data).hexdigest() != ORIGINAL_SHA256:
        sys.exit(f"not the original NotoSansJP[wght].ttf (SHA-256 {ORIGINAL_SHA256})")
    codepoints = scanned_characters()
    codepoints.update(ord(c) for c in EXTRA)
    for first, last in RANGES:
        codepoints.update(range(first, last + 1))
    options = subset.Options()
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.notdef_outline = True
    options.prune_unicode_ranges = False
    options.drop_tables = [t for t in options.drop_tables if t != "BASE"]
    font = subset.load_font(io.BytesIO(data), options)
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=sorted(codepoints))
    subsetter.subset(font)
    subset.save_font(font, str(OUTPUT), options)
    print(f"{OUTPUT.relative_to(ROOT)}: {OUTPUT.stat().st_size} bytes, {len(codepoints)} code points")


if __name__ == "__main__":
    main()
