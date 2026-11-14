newPackage(
    "BSDLibCommandPurse",
    Version => "1.2",
    Date => "November 14th, 2026",
    Authors => {
        {Name => "Mason Hughes", Email => "mase505x@gmail.com"}
    },
    Headline => "Command index purse and CLI stability validator for Windows Subsystem for BSD",
    DebuggingMode => true
)

export {
    "wsbVersion",
    "bsdCommandTable",
    "commandCount",
    "getCommandByIndex",
    "validateCommandIndex"
}

-- Synchronized with C++ (constexpr string_view version = "1.2")
wsbVersion = "1.2"

-- Indexed table of allowed BSD and WSB subsystem commands
bsdCommandTable = hashTable {
    0 => "help",
    1 => "start",
    2 => "shell",
    3 => "status",
    4 => "cr-server",
    5 => "server-start",
    6 => "uninstall",
    7 => "version",
    8 => "freebsd-version",
    9 => "pkg",
    10 => "service",
    11 => "sysrc",
    12 => "bsdinstall",
    13 => "dmesg",
    14 => "jls",
    15 => "jexec"
}

-- Returns the total number of indexed commands available
commandCount = () -> #bsdCommandTable

-- Retrieves a command by its deterministic index (replaces C-style enum loop)
getCommandByIndex = method()
getCommandByIndex ZZ := (idx) -> (
    if bsdCommandTable#?idx then (
        return bsdCommandTable#idx;
    ) else (
        error("WSB Index Exceeded: index " | toString idx | " out of bounds (max " | toString(commandCount() - 1) | ")");
    )
)

-- Validates concurrent index alignment between C++, Logtalk, and M2
validateCommandIndex = method()
validateCommandIndex (ZZ, String) := (idx, expectedCmd) -> (
    if not bsdCommandTable#?idx then return false;
    bsdCommandTable#idx === expectedCmd
)

end
