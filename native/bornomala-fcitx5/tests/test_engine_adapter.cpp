#include <cassert>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>
#include "../../../crates/bornomala-core/include/bornomala.h"
#include "../utf8_utils.h"

using namespace bornomala;

void TestUtf8CodepointToByteOffset() {
    std::cout << "  Testing UTF-8 codepoint to byte offset conversion boundaries...\n";

    // 1. Empty String
    assert(CodepointToByteOffset("", 0) == 0);
    assert(CodepointToByteOffset("", 5) == 0);

    // 2. ASCII String: "ami" (3 bytes, 3 codepoints)
    assert(CodepointToByteOffset("ami", 0) == 0);
    assert(CodepointToByteOffset("ami", 1) == 1);
    assert(CodepointToByteOffset("ami", 2) == 2);
    assert(CodepointToByteOffset("ami", 3) == 3);

    // 3. Single Bengali Character: "ক" (U+0995, 3 bytes)
    std::string ko = "ক";
    assert(ko.length() == 3);
    assert(CodepointToByteOffset(ko, 0) == 0);
    assert(CodepointToByteOffset(ko, 1) == 3);

    // 4. Consonant + Dependent Vowel Sign: "কা" (ক U+0995 + া U+09BE, 6 bytes, 2 codepoints)
    std::string ka = "কা";
    assert(ka.length() == 6);
    assert(CodepointToByteOffset(ka, 0) == 0);
    assert(CodepointToByteOffset(ka, 1) == 3);
    assert(CodepointToByteOffset(ka, 2) == 6);

    // 5. Virama Conjunct Sequence: "ক্ষ" (ক U+0995 + ্ U+09CD + ষ U+09B7, 9 bytes, 3 codepoints)
    std::string ksha = "ক্ষ";
    assert(ksha.length() == 9);
    assert(CodepointToByteOffset(ksha, 0) == 0);
    assert(CodepointToByteOffset(ksha, 1) == 3);
    assert(CodepointToByteOffset(ksha, 2) == 6);
    assert(CodepointToByteOffset(ksha, 3) == 9);

    // 6. Complex Cluster with Matras: "শ্রী" (শ U+09B6 + ্ U+09CD + র U+09B0 + ী U+09C0, 12 bytes, 4 codepoints)
    std::string shri = "শ্রী";
    assert(shri.length() == 12);
    assert(CodepointToByteOffset(shri, 0) == 0);
    assert(CodepointToByteOffset(shri, 1) == 3);
    assert(CodepointToByteOffset(shri, 2) == 6);
    assert(CodepointToByteOffset(shri, 3) == 9);
    assert(CodepointToByteOffset(shri, 4) == 12);

    // 7. End-of-String Boundary: "বাংলা" (ব U+09AC + া U+09BE + ং U+0982 + ল U+09B2 + া U+09BE, 15 bytes, 5 codepoints)
    std::string bangla = "বাংলা";
    assert(bangla.length() == 15);
    assert(CodepointToByteOffset(bangla, 0) == 0);
    assert(CodepointToByteOffset(bangla, 5) == 15);

    // 8. Out-of-Bounds Clamp: "কা" (6 bytes), cursor_pos = 99 -> Clamps to 6
    assert(CodepointToByteOffset(ka, 99) == 6);

    std::cout << "    CodepointToByteOffset tests passed successfully.\n";
}

void TestMultiContextIsolation() {
    std::cout << "  Testing multi-context state lifecycle & isolation...\n";

    BornoEngineHandle* ctxA = borno_engine_new();
    BornoEngineHandle* ctxB = borno_engine_new();
    assert(ctxA != nullptr && ctxB != nullptr);

    // Type "ami" in Context A
    BornoProcessResultC resA{};
    for (char c : std::string("ami")) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = static_cast<uint32_t>(c);
        int32_t rc = borno_engine_process_event(ctxA, &ev, &resA);
        assert(rc == BORNO_OK);
        borno_process_result_free_members(&resA);
    }

    // Verify Context A is composing "আমি"
    BornoStateC stateA{};
    borno_engine_reset(ctxA, &stateA); // check state
    // Let's re-type "ami" in ctxA
    borno_state_free_members(&stateA);

    for (char c : std::string("ami")) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = static_cast<uint32_t>(c);
        int32_t rc = borno_engine_process_event(ctxA, &ev, &resA);
        assert(rc == BORNO_OK);
        if (c == 'i') {
            assert(resA.state.is_composing == true);
            assert(resA.state.preedit != nullptr && std::strcmp(resA.state.preedit, "আমি") == 0);
        }
        borno_process_result_free_members(&resA);
    }

    // Verify Context B is completely empty and NOT composing
    BornoProcessResultC resB{};
    BornoEventC check_ev{};
    check_ev.event_type = BORNO_EVENT_INPUT_CHAR;
    check_ev.char_codepoint = 't';
    int32_t rc = borno_engine_process_event(ctxB, &check_ev, &resB);
    assert(rc == BORNO_OK);
    assert(resB.state.is_composing == true);
    assert(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "ত") == 0);
    borno_process_result_free_members(&resB);

    // Type rest of "umi" in Context B -> "তুমি"
    for (char c : std::string("umi")) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = static_cast<uint32_t>(c);
        rc = borno_engine_process_event(ctxB, &ev, &resB);
        assert(rc == BORNO_OK);
        if (c == 'i') {
            assert(resB.state.is_composing == true);
            assert(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "তুমি") == 0);
        }
        borno_process_result_free_members(&resB);
    }

    // Commit Context A
    BornoEventC commit_ev{};
    commit_ev.event_type = BORNO_EVENT_COMMIT;
    rc = borno_engine_process_event(ctxA, &commit_ev, &resA);
    assert(rc == BORNO_OK);
    assert(resA.committed_text != nullptr && std::strcmp(resA.committed_text, "আমি") == 0);
    assert(resA.state.is_composing == false);
    borno_process_result_free_members(&resA);

    // Context B must STILL be composing "তুমি"
    BornoEventC bsp_ev{};
    bsp_ev.event_type = BORNO_EVENT_BACKSPACE;
    rc = borno_engine_process_event(ctxB, &bsp_ev, &resB);
    assert(rc == BORNO_OK);
    assert(resB.state.is_composing == true);
    assert(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "তুম") == 0);
    borno_process_result_free_members(&resB);

    borno_engine_free(ctxA);
    borno_engine_free(ctxB);

    std::cout << "    Multi-context isolation tests passed successfully.\n";
}

void TestModeCycleAndResetPreservation() {
    std::cout << "  Testing 5-mode cycling and mode preservation on reset...\n";

    BornoEngineHandle* engine = borno_engine_new();
    assert(engine != nullptr);

    const int32_t modes[] = {
        BORNO_MODE_PHONETIC,
        BORNO_MODE_PASSTHROUGH,
        BORNO_MODE_FIXED_NATIONAL,
        BORNO_MODE_FIXED_PROBHAT,
        BORNO_MODE_FIXED_BIJOY
    };

    BornoProcessResultC res{};
    for (int32_t target_mode : modes) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_SWITCH_MODE;
        ev.mode_payload = target_mode;
        int32_t rc = borno_engine_process_event(engine, &ev, &res);
        assert(rc == BORNO_OK);
        assert(res.handled == true);
        assert(res.state.mode == target_mode);
        borno_process_result_free_members(&res);
    }

    // Stay in National mode (index 2)
    BornoEventC ev_nat{};
    ev_nat.event_type = BORNO_EVENT_SWITCH_MODE;
    ev_nat.mode_payload = BORNO_MODE_FIXED_NATIONAL;
    int32_t rc = borno_engine_process_event(engine, &ev_nat, &res);
    assert(rc == BORNO_OK && res.state.mode == BORNO_MODE_FIXED_NATIONAL);
    borno_process_result_free_members(&res);

    // Type a key in National mode (e.g. BORNO_KEY_K -> 'ক')
    rc = borno_engine_process_key(engine, BORNO_KEY_K, 0, &res);
    assert(rc == BORNO_OK);
    borno_process_result_free_members(&res);

    // Reset engine composition
    BornoStateC reset_state{};
    rc = borno_engine_reset(engine, &reset_state);
    assert(rc == BORNO_OK);
    assert(reset_state.is_composing == false);
    // Mode must be strictly preserved!
    assert(reset_state.mode == BORNO_MODE_FIXED_NATIONAL && "Reset must preserve active mode");
    borno_state_free_members(&reset_state);

    borno_engine_free(engine);
    std::cout << "    Mode cycling and preservation tests passed successfully.\n";
}

void TestCandidateSelectionAndCancel() {
    std::cout << "  Testing candidate suggestions, selection and cancel...\n";

    BornoEngineHandle* engine = borno_engine_new();
    assert(engine != nullptr);

    // Type "k" -> Phonetic suggests candidates (e.g. ক, কে, কি, etc.)
    BornoProcessResultC res{};
    BornoEventC ev{};
    ev.event_type = BORNO_EVENT_INPUT_CHAR;
    ev.char_codepoint = 'k';
    int32_t rc = borno_engine_process_event(engine, &ev, &res);
    assert(rc == BORNO_OK);
    assert(res.state.is_composing == true);
    assert(res.state.candidates_count > 0);
    std::cout << "    Suggestions for 'k': count=" << res.state.candidates_count 
              << ", first=\"" << (res.state.candidates ? res.state.candidates[0] : "") << "\"\n";
    borno_process_result_free_members(&res);

    // Select candidate 0
    BornoEventC sel_ev{};
    sel_ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
    sel_ev.index_payload = 0;
    rc = borno_engine_process_event(engine, &sel_ev, &res);
    assert(rc == BORNO_OK);
    assert(res.committed_text != nullptr);
    assert(res.state.is_composing == false);
    std::cout << "    Committed candidate: \"" << res.committed_text << "\"\n";
    borno_process_result_free_members(&res);

    // Type "ami" then Cancel (Escape)
    for (char c : std::string("ami")) {
        BornoEventC inp{};
        inp.event_type = BORNO_EVENT_INPUT_CHAR;
        inp.char_codepoint = static_cast<uint32_t>(c);
        borno_engine_process_event(engine, &inp, &res);
        borno_process_result_free_members(&res);
    }

    BornoEventC cancel_ev{};
    cancel_ev.event_type = BORNO_EVENT_CANCEL;
    rc = borno_engine_process_event(engine, &cancel_ev, &res);
    assert(rc == BORNO_OK);
    assert(res.cancelled == true);
    assert(res.state.is_composing == false);
    assert(res.committed_text == nullptr);
    borno_process_result_free_members(&res);

    // Type "ami" + Space -> Should commit "আমি " with space included
    for (char c : std::string("ami")) {
        BornoEventC inp{};
        inp.event_type = BORNO_EVENT_INPUT_CHAR;
        inp.char_codepoint = static_cast<uint32_t>(c);
        borno_engine_process_event(engine, &inp, &res);
        borno_process_result_free_members(&res);
    }
    BornoEventC sp_ev{};
    sp_ev.event_type = BORNO_EVENT_INPUT_CHAR;
    sp_ev.char_codepoint = ' ';
    rc = borno_engine_process_event(engine, &sp_ev, &res);
    assert(rc == BORNO_OK && res.handled == true);
    assert(res.committed_text != nullptr && std::strcmp(res.committed_text, "আমি ") == 0);
    assert(res.state.is_composing == false);
    std::cout << "    Space commit verified: \"" << res.committed_text << "\"\n";
    borno_process_result_free_members(&res);

    borno_engine_free(engine);
    std::cout << "    Candidate selection, cancel, and space commit tests passed successfully.\n";
}

int main() {
    std::cout << "[Layer 4] Running Headless Engine Adapter Tests...\n";
    TestUtf8CodepointToByteOffset();
    TestMultiContextIsolation();
    TestModeCycleAndResetPreservation();
    TestCandidateSelectionAndCancel();
    std::cout << "[Layer 4] Headless Engine Adapter Tests PASSED!\n";
    return 0;
}
