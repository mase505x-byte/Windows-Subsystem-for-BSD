#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <string_view>

namespace {
constexpr std::string_view version = "0.2.0";
void help() {
    std::cout << "Usage: wsb <command> [options]\n"
                 "Commands:\n"
                 "  help                  Show this help\n"
                 "  version               Show version\n"
                 "  status                Check whether the local bridge socket exists\n"
                 "  settings <IP> <PORT>  Validate a bind address and port\n"
                 "  collect               Print basic diagnostic information\n"
                 "  shell                 Explain shell runtime availability\n"
                 "  server-start          Explain host server availability\n";
}
int status() {
#if defined(_WIN32)
    std::cerr << "wsb: status currently supports Unix-domain socket installations only\n";
    return 2;
#else
    const char* configured = std::getenv("WSB_SOCKET");
    const std::filesystem::path socket = configured ? configured : "/var/run/wsb_server.sock";
    if (std::filesystem::exists(socket)) {
        std::cout << "WSB bridge socket exists: " << socket.string() << '\n';
        return 0;
    }
    std::cout << "WSB bridge is not running (socket not found: " << socket.string() << ")\n";
    return 1;
#endif
}
int settings(int argc, char** argv) {
    if (argc != 4) {
        std::cerr << "usage: wsb settings <IPv4-or-IPv6-address> <port 1..65535>\n";
        return 2;
    }
    char* end = nullptr;
    const auto port = std::strtoul(argv[3], &end, 10);
    if (end == argv[3] || *end != '\0' || port == 0 || port > 65535) {
        std::cerr << "wsb: port must be an integer from 1 to 65535\n";
        return 2;
    }
    // Deliberately validate only the port here; binding is not attempted by this CLI.
    std::cout << "Configuration accepted for " << argv[2] << ':' << port << " (not persisted)\n";
    return 0;
}
}

int main(int argc, char** argv) {
    if (argc < 2) { help(); return 2; }
    const std::string_view command(argv[1]);
    if (command == "help" || command == "--help" || command == "-h") { help(); return 0; }
    if (command == "version" || command == "--version") {
        std::cout << "Windows Subsystem for BSD (WSB) " << version << '\n'; return 0;
    }
    if (command == "status") return status();
    if (command == "settings") return settings(argc, argv);
    if (command == "collect") {
        std::cout << "WSB version: " << version << "\nPlatform: ";
#ifdef _WIN32
        std::cout << "Windows\n";
#elif defined(__FreeBSD__)
        std::cout << "FreeBSD\n";
#elif defined(__linux__)
        std::cout << "Linux\n";
#else
        std::cout << "unknown\n";
#endif
        return 0;
    }
    if (command == "shell" || command == "server-start") {
        std::cerr << "wsb: " << command << " is not available yet; no BSD runtime/server is bundled\n";
        return 2;
    }
    if (command == "uninstall") {
        std::cerr << "wsb: refusing to remove files; this build does not install or manage a runtime\n";
        return 2;
    }
    std::cerr << "wsb: unknown command: " << command << '\n';
    help();
    return 2;
}
