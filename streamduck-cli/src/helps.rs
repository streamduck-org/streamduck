pub const COMMANDS: &'static [(&'static str, &'static str)] = &[
    ("(d)evice (l)ist", "- lists all streamdeck devices"),
    ("(d)evice (a)dd", "<serial> - adds specified device to managed"),
    ("(d)evice (r)emove", "<serial> - removes specified device from managed"),
    ("-(s)elect", "<serial> - selects device to be used with device related operations, to unselect, enter 'select' alone"),
    ("(con)fig (r)eload (a)ll", "- reloads all device configs, all unsaved changes will lost"),
    ("(con)fig (r)eload", "[<serial>] - reloads device config for specific/selected device, all unsaved changes will lost"),
    ("(con)fig (s)ave (a)ll", "- saves all device configs"),
    ("(con)fig save", "[<serial>] - saves device config for specific/selected device"),
    ("(con)fig import", "[<serial>] <path> - imports device config from a file for specific/selected device"),
    ("(con)fig export", "[<serial>] <path> - exports device config into a file for specific/selected device"),
    ("(br)ightness", "[<serial>] <0-255> - sets brightness for device, different for each device, but usually 0-100"),
    ("(ba)ck", "[<serial>] - navigates back, even if there's no button for that"),
    ("(p)ress", "[<serial>] <key index> - simulates a press on a button"),
    ("(m)odule (l)ist", "- lists all loaded modules"),
    ("(m)odule (i)nfo", "<name> - prints information about module"),
    ("(m)odule (p)arams (a)dd", "<name> <parameter path> - adds a new element into parameter array"),
    ("(m)odule (p)arams (r)emove", "<name> <parameter path> <element index> - removes element from parameter array"),
    ("(m)odule (p)arams (s)et", "<name> <parameter path> <value> - sets value to module's parameter"),
    ("(m)odule (p)arams (l)ist", "<name> - lists parameters of the module along with values and paths"),
    ("(com)ponent (l)ist", "- lists available components from modules"),
    ("(com)ponent (i)nfo", "<name> - prints information about component"),
    ("(i)mage (l)ist", "[<serial>] [preview size] - lists all images used by a device, optionally sizes images according to provided size"),
    ("(i)mage (a)dd", "[<serial>] <file path> - adds image to device config"),
    ("(i)mage (r)emove", "[<serial>] <identifier> - removes image from device config"),
    ("(b)utton (l)ist", "[<serial>] - lists all buttons defined on current screen"),
    ("(b)utton (i)nfo", "[<serial>] <key index> - provides more detailed information about a button"),
    ("(b)utton (n)ew", "[<serial>] <key index> - creates an empty button on current screen"),
    ("(b)utton (f)rom", "[<serial>] <key index> <component name> - creates a button based on component's template"),
    ("(b)utton (c)o(p)y", "[<serial>] <key index> - saves button to internal clipboard"),
    ("(b)utton (p)aste", "[<serial>] <key index> - creates a new button from internal clipboard"),
    ("(b)utton (r)emove", "[<serial>] <key index> - removes a button on current screen"),
    ("(b)utton (c)omponent (a)dd", "[<serial>] <key index> <component name> - adds component on a button"),
    ("(b)utton (c)omponent (r)emove", "[<serial>] <key index> <component name> - removes component from a button"),
    ("(b)utton (c)omponent (p)arams (a)dd", "[<serial>] <key index> <component name> <parameter path> - adds a new element into parameter array"),
    ("(b)utton (c)omponent (p)arams (r)emove", "[<serial>] <key index> <component name> <parameter path> <element index> - removes element from parameter array"),
    ("(b)utton (c)omponent (p)arams (s)et", "[<serial>] <key index> <component name> <parameter path> <value> - sets value to component's parameter"),
    ("(b)utton (c)omponent (p)arams (u)pload", "[<serial>] <key index> <component name> <parameter path> <file path> - reads binary file and sets that as value to component's parameter"),
    ("(b)utton (c)omponent (p)arams (l)ist", "[<serial>] <key index> <component name> - lists parameters of the component along with values and paths"),
];