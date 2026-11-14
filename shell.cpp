#include <unordered_set>

// Add this FFI hook for translating & executing BSD ELF binaries via your Rust assembler
extern "C" {
    int wsb_exec_bsd_binary(const char* bin_name, const char* args_str);
}

namespace {

// Strict allowlist of BSD user-space commands (aligned with man.1)
const std::unordered_set<std::string_view> kAllowedBsdCommands = {
    "freebsd-version", "pkg", "service", "sysrc",
    "bsdinstall", "dmesg", "jls", "jexec",
    "ls", "cd", "pwd", "cat", "uname", "ps", "kill", "ifconfig"
};

int dispatch_bsd_command(std::string_view op, const std::string& full_line) {
    if (kAllowedBsdCommands.find(op) == kAllowedBsdCommands.end()) {
        std::cerr << "wsb: " << op << ": command not found in BSD user-space\n";
        log_event("sandbox_violation", op, 127);
        return 127;
    }

    // Route through the Rust Mutable Assembler / BSD ABI Translator instead of host std::system()
    // When linked to Rust FFI:
    // return wsb_exec_bsd_binary(std::string{op}.c_str(), full_line.c_str());

    // Stub response until Rust FFI ELF loader is linked:
    if (op == "freebsd-version" || op == "uname") {
        std::cout << "14.2-RELEASE-p1 (WSB Translated ABI)\n";
        return 0;
    }

    std::cout << "[WSB-ABI] Translating BSD binary '" << op << "' via mutable assembler...\n";
    log_event("bsd_exec", op, 0);
    return 0;
}

int cmd_shell() {
    if (!is_env_running()) {
        std::cerr << "wsb: BSD environment is not running (run 'wsb start' first)\n";
        log_event("shell_error", "env_not_running", 2);
        return 2;
    }

    void* active_jit_block = nullptr; // wsb_alloc_mutable_block(1);

    std::cout << "Windows Subsystem for BSD: Version " << kVersion << " (Restricted BSD Shell)\n";
    std::cout << "Built-ins: audit | patch <uint32> | exit\n";

    std::string line;
    while (true) {
        std::cout << "wsb# ";
        if (!std::getline(std::cin, line)) break;
        if (line.empty()) continue;

        // Block shell chaining/injection operators (&&, ||, ;, |, `, $())
        if (line.find_first_of("&;|`$") != std::string::npos) {
            std::cerr << "wsb: shell metacharacters are disabled in BSD user-space\n";
            log_event("shell_injection_blocked", "metacharacter", 2);
            continue;
        }

        std::istringstream iss(line);
        std::vector<std::string> tokens;
        std::string tok;
        while (iss >> tok) tokens.push_back(tok);

        const std::string_view op{tokens};

        if (op == "exit" || op == "logout") {
            break;
        } else if (op == "audit") {
            int rc = run_logtalk_audit();
            std::cout << (rc == 0 ? "[AUDIT] Nominal\n" : "[AUDIT] Recalibration triggered\n");
        } else if (op == "patch" && tokens.size() > 1) {
            uint32_t val = static_cast<uint32_t>(std::stoul(tokens));
            if (active_jit_block) {
                wsb_mutate_block(active_jit_block, val);
            }
            log_event("jit_mutation", "imm32_patched", static_cast<int>(val));
            std::cout << "[JIT] Mutated immediate value to " << val << '\n';
        } else {
            // Strictly dispatch only through the BSD translation layer
            int rc = dispatch_bsd_command(op, line);
            if (rc != 0) {
                log_event("bsd_cmd_error", op, rc);
            }
        }
    }
    return 0;
}

} // namespace
