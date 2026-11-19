#!/usr/bin/env python3
"""Prints what a screen reader is told of packwing's window, on Linux.

packwing's controls say what they are, what they are called and whether they
are on (crates/app/src/widgets.rs); GPUI hands that to AccessKit, which
publishes it over AT-SPI, as Orca reads it. This walks that tree for the
packwing running now, the way Orca or Accerciser would see it:

    scripts/a11y_tree.py                 the tree, as it is now
    scripts/a11y_tree.py --click Settings
                                         presses the control called Settings
                                         first, as a screen reader's user does,
                                         then prints the tree

AT-SPI publishes an app's tree only while something asks for it: the script
turns the session's accessibility on for the time it reads, and back as it
was. Needs the GObject bindings of at-spi2 (Ubuntu: gir1.2-atspi-2.0).
"""

import argparse
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, Gio, GLib  # noqa: E402

STATES = [
    (Atspi.StateType.FOCUSED, "focused"),
    (Atspi.StateType.SELECTED, "selected"),
    (Atspi.StateType.CHECKED, "checked"),
    (Atspi.StateType.EXPANDED, "expanded"),
]


def accessibility(bus, on=None):
    """The session's accessibility switch: read, or set to `on`."""
    if on is None:
        reply = bus.call_sync(
            "org.a11y.Bus", "/org/a11y/bus", "org.freedesktop.DBus.Properties", "Get",
            GLib.Variant("(ss)", ("org.a11y.Status", "IsEnabled")), None, 0, -1, None,
        )
        return reply.unpack()[0]
    bus.call_sync(
        "org.a11y.Bus", "/org/a11y/bus", "org.freedesktop.DBus.Properties", "Set",
        GLib.Variant("(ssv)", ("org.a11y.Status", "IsEnabled", GLib.Variant("b", on))),
        None, 0, -1, None,
    )


def app_named(name):
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        if app is not None and app.get_name() == name:
            return app
    return None


def find(node, name):
    if node.get_name() == name and node.get_role_name() != "application":
        return node
    for i in range(node.get_child_count()):
        child = node.get_child_at_index(i)
        found = child and find(child, name)
        if found:
            return found
    return None


def show(node, depth=0):
    states = node.get_state_set()
    said = [label for state, label in STATES if states.contains(state)]
    line = f"{'  ' * depth}[{node.get_role_name()}] {node.get_name()!r}"
    if node.get_description():
        line += f" ({node.get_description()})"
    if said:
        line += f" {' '.join(said)}"
    print(line)
    for i in range(node.get_child_count()):
        child = node.get_child_at_index(i)
        if child is not None:
            show(child, depth + 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--app", default="packwing", help="the application's name")
    parser.add_argument("--click", metavar="NAME", help="press the control of this name first")
    parser.add_argument("--wait", type=float, default=3.0, help="seconds for the tree to come")
    args = parser.parse_args()

    bus = Gio.bus_get_sync(Gio.BusType.SESSION)
    was = accessibility(bus)
    accessibility(bus, True)
    try:
        time.sleep(args.wait)
        app = app_named(args.app)
        if app is None:
            raise SystemExit(f"{args.app} is not running, or tells AT-SPI nothing")
        if args.click:
            control = find(app, args.click)
            if control is None:
                raise SystemExit(f"nothing called {args.click!r}")
            Atspi.Action.do_action(control, 0)
            time.sleep(1.5)
        show(app)
    finally:
        accessibility(bus, was)


if __name__ == "__main__":
    main()
