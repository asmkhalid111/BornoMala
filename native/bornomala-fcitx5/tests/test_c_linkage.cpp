#include <cassert>
#include <cstring>
#include <iostream>
#include "../../../crates/bornomala-core/include/bornomala.h"

int main() {
    std::cout << "[Layer 2] Testing C-ABI Static Linkage & Ownership...\n";

    // 1. Create engine instance
    BornoEngineHandle* engine = borno_engine_new();
    assert(engine != nullptr && "borno_engine_new() returned null");

    // 2. Query initial status
    BornoStatusC status{};
    int32_t rc = borno_engine_get_status(engine, &status);
    assert(rc == BORNO_OK && "borno_engine_get_status failed");
    assert(status.ready == true && "Engine status not ready");
    assert(status.active_mode != nullptr && std::strcmp(status.active_mode, "Phonetic") == 0);
    std::cout << "  Status verified: Engine=" << status.name 
              << " v" << status.version 
              << ", Mode=" << status.active_mode << "\n";
    borno_status_free_members(&status);

    // 3. Process char 'a' -> should compose 'আ' in Phonetic mode
    BornoStateC state{};
    rc = borno_engine_process_char(engine, 'a', &state);
    assert(rc == BORNO_OK && "borno_engine_process_char('a') failed");
    assert(state.is_composing == true && "Expected active composition");
    assert(state.preedit != nullptr && std::strcmp(state.preedit, "আ") == 0);
    assert(state.candidates_count > 0 && "Expected candidate suggestions");
    std::cout << "  Process char 'a' -> preedit: \"" << state.preedit 
              << "\", candidates: " << state.candidates_count << "\n";
    borno_state_free_members(&state);

    // 4. Process physical key 'm' -> should continue composition
    BornoProcessResultC result{};
    rc = borno_engine_process_key(engine, BORNO_KEY_M, 0, &result);
    assert(rc == BORNO_OK && "borno_engine_process_key(BORNO_KEY_M) failed");
    assert(result.handled == true && "Expected key to be handled");
    assert(result.state.is_composing == true && "Expected active composition");
    std::cout << "  Process key 'm' -> preedit: \"" << (result.state.preedit ? result.state.preedit : "") << "\"\n";
    borno_process_result_free_members(&result);

    // 5. Query candidate kind
    int32_t kind = borno_engine_get_candidate_kind(engine, 0);
    assert(kind >= 0 && kind <= 4 && "Candidate kind out of expected enum range");
    std::cout << "  Candidate 0 kind: " << kind << "\n";

    // 6. Reset composition
    BornoStateC reset_state{};
    rc = borno_engine_reset(engine, &reset_state);
    assert(rc == BORNO_OK && "borno_engine_reset failed");
    assert(reset_state.is_composing == false && "Expected composition to be cleared");
    borno_state_free_members(&reset_state);
    std::cout << "  Reset composition verified\n";

    // 7. Destroy engine
    borno_engine_free(engine);
    std::cout << "  Engine destroyed cleanly\n";

    // 8. Test UTF-8 echo FFI helper
    char* echo_str = nullptr;
    size_t echo_len = 0;
    rc = borno_ffi_echo_utf8("বাংলা", &echo_str, &echo_len);
    assert(rc == BORNO_OK && echo_str != nullptr);
    assert(std::strcmp(echo_str, "বাংলা") == 0);
    assert(echo_len == std::strlen("বাংলা"));
    borno_string_free(echo_str);
    std::cout << "  UTF-8 FFI echo verified\n";

    std::cout << "[Layer 2] C-ABI Static Linkage & Ownership verification PASSED!\n";
    return 0;
}
