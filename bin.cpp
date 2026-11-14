constexpr std::string_view kVersion = "1.2";

enum class WsbCommandIndex : size_t {
    Help = 0, Start, Shell, Status, CrServer, ServerStart, Uninstall, Version,
    FreebsdVersion, Pkg, Service, Sysrc, Bsdinstall, Dmesg, Jls, Jexec,
    Count // Concurrent index ceiling (16)
};
