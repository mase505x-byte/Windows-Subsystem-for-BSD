#include <iostream>
#include <string>
#include <string_view>

namespace {
constexpr std::string_view version = "1.1.3";

void print_help() {
    std::cout
        << "Usage: wsb <command>\n"
        << "Commands:\n"
        << "  help       Show this help message\n"
        << "  version    Show the WSB version\n"
        << "  uninstall  Uninstall a configured BSD environment\n"
        << "  cr-server  Collect dedicated-server settings\n";
}

int uninstall() {
    std::cerr << "wsb: uninstall is not implemented yet\n";
    return 2;
}

int configure_server() {
    std::string address;
    unsigned int port = 0;

    std::cout << "Server bind address: ";
    if (!(std::cin >> address)) {
        std::cerr << "wsb: couldn't read the address\n";
        return 2;
    }

    std::cout << "Server port: ";
    if (!(std::cin >> port) || port == 0 || port > 65535) {
        std::cerr << "wsb: enter a port from 1 to 65535\n";
        return 2;
    }

    std::cout << "Server configuration: " << address << ':' << port << '\n';
    std::cout << "Note: this only collects settings; it does not start a server.\n";
    return 0;
}
} // namespace

int main(int argc, char* argv[]) {
    if (argc != 2) {
        print_help();
        return 2;
    }

    const std::string_view command{argv[1]};

    if (command == "help" || command == "--help") {
        print_help();
        return 0;
    }

    if (command == "version" || command == "--version") {
        std::cout << "Windows Subsystem for BSD: Version " << version << '\n';
        return 0;
    }

    if (command == "uninstall" || command == "--uninstall") {
        return uninstall();
    }

    if (command == "cr-server" || command == "--server-msg") {
        return configure_server();
    }

    std::cerr << "wsb: unknown command: " << command << '\n';
    print_help();
    return 2;
}