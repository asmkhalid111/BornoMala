#include <fcitx-utils/standardpaths.h>
#include <iostream>

int main() {
    const auto& sp = fcitx::StandardPaths::global();
    std::cout << "--- ADDON DIRS ---\n";
    for (const auto& p : sp.directories(fcitx::StandardPathsType::Addon)) {
        std::cout << p << std::endl;
    }
    std::cout << "--- PKGDATA DIRS ---\n";
    for (const auto& p : sp.directories(fcitx::StandardPathsType::PkgData)) {
        std::cout << p << std::endl;
    }
    std::cout << "--- PKGCONFIG DIRS ---\n";
    for (const auto& p : sp.directories(fcitx::StandardPathsType::PkgConfig)) {
        std::cout << p << std::endl;
    }
    return 0;
}
