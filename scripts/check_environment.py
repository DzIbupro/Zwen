"""Read-only developer dependency check; does not connect to the desktop."""

import importlib.util
import json
import platform
import shutil
import sqlite3


def main():
    report = {
        "python": platform.python_version(),
        "sqlite": sqlite3.sqlite_version,
        "commands": {name: shutil.which(name) is not None for name in ("niri", "wl-paste", "git")},
        "typelibs": {},
    }
    if importlib.util.find_spec("gi") is None:
        report["gi"] = False
    else:
        import gi
        report["gi"] = True
        for name, version in (("Gtk", "4.0"), ("Soup", "3.0"), ("Adw", "1"), ("Gtk4LayerShell", "1.0")):
            try:
                gi.require_version(name, version)
                report["typelibs"][name] = True
            except ValueError:
                report["typelibs"][name] = False
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
