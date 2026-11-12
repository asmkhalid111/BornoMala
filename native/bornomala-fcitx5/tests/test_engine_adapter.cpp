#include <cstring>
#include <fcntl.h>
#include <fstream>
#include <iostream>
#include <string>
#include <sys/stat.h>
#include <unistd.h>
#include <vector>
#include "../../../crates/bornomala-core/include/bornomala.h"
#include "../adapter_helpers.h"
#include "../utf8_utils.h"
#include "test_check.h"

using namespace bornomala;

namespace {

std::string MakeTempConfigPath(const std::string& suffix) {
    return "/tmp/bornomala_m8_wp1_test_" + std::to_string(static_cast<long>(getpid())) + "_" + suffix + ".json";
}

void WriteFileText(const std::string& path, const std::string& content) {
    std::ofstream out(path, std::ios::binary | std::ios::trunc);
    BORNO_CHECK(out.is_open());
    out << content;
    out.flush();
    BORNO_CHECK(out.good());
}

void BumpFileTimestamp(const std::string& path, time_t sec, long nsec) {
    struct timespec times[2];
    times[0].tv_sec = sec;
    times[0].tv_nsec = nsec;
    times[1].tv_sec = sec;
    times[1].tv_nsec = nsec;
    BORNO_CHECK(utimensat(AT_FDCWD, path.c_str(), times, 0) == 0);
}

} // namespace

void TestUtf8CodepointToByteOffset() {
    std::cout << "  Testing UTF-8 codepoint to byte offset conversion boundaries...\n";

    // 1. Empty String
    BORNO_CHECK(CodepointToByteOffset("", 0) == 0);
    BORNO_CHECK(CodepointToByteOffset("", 5) == 0);

    // 2. ASCII String: "ami" (3 bytes, 3 codepoints)
    BORNO_CHECK(CodepointToByteOffset("ami", 0) == 0);
    BORNO_CHECK(CodepointToByteOffset("ami", 1) == 1);
    BORNO_CHECK(CodepointToByteOffset("ami", 2) == 2);
    BORNO_CHECK(CodepointToByteOffset("ami", 3) == 3);

    // 3. Single Bengali Character: "ক" (U+0995, 3 bytes)
    std::string ko = "ক";
    BORNO_CHECK(ko.length() == 3);
    BORNO_CHECK(CodepointToByteOffset(ko, 0) == 0);
    BORNO_CHECK(CodepointToByteOffset(ko, 1) == 3);

    // 4. Consonant + Dependent Vowel Sign: "কা" (ক U+0995 + া U+09BE, 6 bytes, 2 codepoints)
    std::string ka = "কা";
    BORNO_CHECK(ka.length() == 6);
    BORNO_CHECK(CodepointToByteOffset(ka, 0) == 0);
    BORNO_CHECK(CodepointToByteOffset(ka, 1) == 3);
    BORNO_CHECK(CodepointToByteOffset(ka, 2) == 6);

    // 5. Virama Conjunct Sequence: "ক্ষ" (ক U+0995 + ্ U+09CD + ষ U+09B7, 9 bytes, 3 codepoints)
    std::string ksha = "ক্ষ";
    BORNO_CHECK(ksha.length() == 9);
    BORNO_CHECK(CodepointToByteOffset(ksha, 0) == 0);
    BORNO_CHECK(CodepointToByteOffset(ksha, 1) == 3);
    BORNO_CHECK(CodepointToByteOffset(ksha, 2) == 6);
    BORNO_CHECK(CodepointToByteOffset(ksha, 3) == 9);

    // 6. Complex Cluster with Matras: "শ্রী" (শ + ্ + র + ী, 12 bytes, 4 codepoints)
    std::string shri = "শ্রী";
    BORNO_CHECK(shri.length() == 12);
    BORNO_CHECK(CodepointToByteOffset(shri, 0) == 0);
    BORNO_CHECK(CodepointToByteOffset(shri, 1) == 3);
    BORNO_CHECK(CodepointToByteOffset(shri, 2) == 6);
    BORNO_CHECK(CodepointToByteOffset(shri, 3) == 9);
    BORNO_CHECK(CodepointToByteOffset(shri, 4) == 12);

    // 7. End-of-String Boundary: "বাংলা" (15 bytes, 5 codepoints)
    std::string bangla = "বাংলা";
    BORNO_CHECK(bangla.length() == 15);
    BORNO_CHECK(CodepointToByteOffset(bangla, 0) == 0);
    BORNO_CHECK(CodepointToByteOffset(bangla, 5) == 15);

    // 8. Out-of-Bounds Clamp: "কা" (6 bytes), cursor_pos = 99 -> Clamps to 6
    BORNO_CHECK(CodepointToByteOffset(ka, 99) == 6);

    std::cout << "    CodepointToByteOffset tests passed successfully.\n";
}

void TestMultiContextIsolation() {
    std::cout << "  Testing multi-context state lifecycle & isolation...\n";

    BornoEngineHandle* ctxA = borno_engine_new();
    BornoEngineHandle* ctxB = borno_engine_new();
    BORNO_CHECK(ctxA != nullptr && ctxB != nullptr);

    // Type "ami" in Context A -> "আমি"
    BornoProcessResultC resA{};
    for (char c : std::string("ami")) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = static_cast<uint32_t>(c);
        int32_t rc = borno_engine_process_event(ctxA, &ev, &resA);
        BORNO_CHECK(rc == BORNO_OK);
        if (c == 'i') {
            BORNO_CHECK(resA.state.is_composing == true);
            BORNO_CHECK(resA.state.preedit != nullptr && std::strcmp(resA.state.preedit, "আমি") == 0);
        }
        borno_process_result_free_members(&resA);
    }

    // Type "tumi" in Context B -> "তুমি"
    BornoProcessResultC resB{};
    for (char c : std::string("tumi")) {
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = static_cast<uint32_t>(c);
        int32_t rc = borno_engine_process_event(ctxB, &ev, &resB);
        BORNO_CHECK(rc == BORNO_OK);
        if (c == 't') {
            BORNO_CHECK(resB.state.is_composing == true);
            BORNO_CHECK(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "ত") == 0);
        }
        if (c == 'i') {
            BORNO_CHECK(resB.state.is_composing == true);
            BORNO_CHECK(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "তুমি") == 0);
        }
        borno_process_result_free_members(&resB);
    }

    // Commit Context A
    BornoEventC commit_ev{};
    commit_ev.event_type = BORNO_EVENT_COMMIT;
    int32_t rc = borno_engine_process_event(ctxA, &commit_ev, &resA);
    BORNO_CHECK(rc == BORNO_OK);
    BORNO_CHECK(resA.committed_text != nullptr && std::strcmp(resA.committed_text, "আমি") == 0);
    BORNO_CHECK(resA.state.is_composing == false);
    borno_process_result_free_members(&resA);

    // Context B must STILL be composing "তুমি"
    BornoEventC bsp_ev{};
    bsp_ev.event_type = BORNO_EVENT_BACKSPACE;
    rc = borno_engine_process_event(ctxB, &bsp_ev, &resB);
    BORNO_CHECK(rc == BORNO_OK);
    BORNO_CHECK(resB.state.is_composing == true);
    BORNO_CHECK(resB.state.preedit != nullptr && std::strcmp(resB.state.preedit, "তুম") == 0);
    borno_process_result_free_members(&resB);

    borno_engine_free(ctxA);
    borno_engine_free(ctxB);

    std::cout << "    Multi-context isolation tests passed successfully.\n";
}

void TestModeSynchronization() {
    std::cout << "  Testing Correction A: per-context engine mode synchronization...\n";

    // 1. Cycle order check matching F12/F4
    BORNO_CHECK(NextCycleMode(BORNO_MODE_PHONETIC) == BORNO_MODE_FIXED_NATIONAL);
    BORNO_CHECK(NextCycleMode(BORNO_MODE_FIXED_NATIONAL) == BORNO_MODE_FIXED_PROBHAT);
    BORNO_CHECK(NextCycleMode(BORNO_MODE_FIXED_PROBHAT) == BORNO_MODE_FIXED_BIJOY);
    BORNO_CHECK(NextCycleMode(BORNO_MODE_FIXED_BIJOY) == BORNO_MODE_PASSTHROUGH);
    BORNO_CHECK(NextCycleMode(BORNO_MODE_PASSTHROUGH) == BORNO_MODE_PHONETIC);

    // 2. Context A starts in Phonetic, transitions globalMode to Fixed Bijoy
    BornoContextState ctxA{};
    BORNO_CHECK(InitContextState(ctxA) == true);
    BORNO_CHECK(ctxA.mode == BORNO_MODE_PHONETIC);
    BORNO_CHECK(ctxA.modeSynced == true);

    int32_t globalMode = NextCycleMode(ctxA.mode); // National
    BORNO_CHECK(SyncContextEngineMode(ctxA, globalMode) == true);
    globalMode = NextCycleMode(globalMode); // Probhat
    BORNO_CHECK(SyncContextEngineMode(ctxA, globalMode) == true);
    globalMode = NextCycleMode(globalMode); // Bijoy
    BORNO_CHECK(SyncContextEngineMode(ctxA, globalMode) == true);
    BORNO_CHECK(ctxA.mode == BORNO_MODE_FIXED_BIJOY);

    // 3. Create Context B while globalMode == BORNO_MODE_FIXED_BIJOY (mirrors m_factory init)
    BornoContextState ctxB{};
    BORNO_CHECK(InitContextState(ctxB) == true);
    BORNO_CHECK(ctxB.mode == BORNO_MODE_PHONETIC); // raw borno_engine_new starts in Phonetic
    BORNO_CHECK(SyncContextEngineMode(ctxB, globalMode) == true);
    BORNO_CHECK(ctxB.mode == BORNO_MODE_FIXED_BIJOY);
    BORNO_CHECK(ctxB.modeSynced == true);

    // Verify Context B's actual Rust engine state and typing layout match Bijoy-Compatible (semicolon -> preedit "ম")
    BornoStatusC statusB{};
    BORNO_CHECK(borno_engine_get_status(ctxB.engine, &statusB) == BORNO_OK);
    BORNO_CHECK(statusB.active_mode != nullptr && std::strcmp(statusB.active_mode, "Bijoy-Compatible") == 0);
    borno_status_free_members(&statusB);

    BornoProcessResultC keyRes{};
    BORNO_CHECK(borno_engine_process_key(ctxB.engine, BORNO_KEY_SEMICOLON, 0, &keyRes) == BORNO_OK);
    BORNO_CHECK(keyRes.handled == true);
    BORNO_CHECK(keyRes.state.mode == BORNO_MODE_FIXED_BIJOY);
    BORNO_CHECK(keyRes.state.is_composing == true);
    BORNO_CHECK(keyRes.state.preedit != nullptr && std::strcmp(keyRes.state.preedit, "ম") == 0);
    borno_process_result_free_members(&keyRes);

    // 4. Existing Context C initialized earlier in Phonetic; simulate globalMode changed to National before activate()
    BornoContextState ctxC{};
    BORNO_CHECK(InitContextState(ctxC) == true);
    BORNO_CHECK(ctxC.mode == BORNO_MODE_PHONETIC);

    globalMode = BORNO_MODE_FIXED_NATIONAL;
    BORNO_CHECK(SyncContextEngineMode(ctxC, globalMode) == true);
    BORNO_CHECK(ctxC.mode == BORNO_MODE_FIXED_NATIONAL);

    // Verify actual Context C output is National (BORNO_KEY_K -> "ত", BORNO_KEY_SEMICOLON -> ";")
    BORNO_CHECK(borno_engine_process_key(ctxC.engine, BORNO_KEY_K, 0, &keyRes) == BORNO_OK);
    BORNO_CHECK(keyRes.state.preedit != nullptr && std::strcmp(keyRes.state.preedit, "ত") == 0);
    borno_process_result_free_members(&keyRes);

    BORNO_CHECK(borno_engine_process_key(ctxC.engine, BORNO_KEY_SEMICOLON, 0, &keyRes) == BORNO_OK);
    BORNO_CHECK(keyRes.state.preedit != nullptr && std::strcmp(keyRes.state.preedit, "ত;") == 0);
    borno_process_result_free_members(&keyRes);

    // 5. Repeated activation idempotency check (must NOT advance/cycle mode or break Phonetic preedit)
    BORNO_CHECK(SyncContextEngineMode(ctxC, globalMode) == true);
    BORNO_CHECK(SyncContextEngineMode(ctxC, globalMode) == true);
    BORNO_CHECK(ctxC.mode == BORNO_MODE_FIXED_NATIONAL);

    BORNO_CHECK(SyncContextEngineMode(ctxC, BORNO_MODE_PHONETIC) == true);
    BornoEventC inputA{};
    inputA.event_type = BORNO_EVENT_INPUT_CHAR;
    inputA.char_codepoint = 'a';
    BORNO_CHECK(borno_engine_process_event(ctxC.engine, &inputA, &keyRes) == BORNO_OK);
    BORNO_CHECK(ApplyEngineResultToState(BORNO_OK, keyRes, ctxC) == true);
    borno_process_result_free_members(&keyRes);
    BORNO_CHECK(ctxC.isComposing == true);
    // Repeated sync with same target mode preserves ongoing composition
    BORNO_CHECK(SyncContextEngineMode(ctxC, BORNO_MODE_PHONETIC) == true);
    BORNO_CHECK(ctxC.mode == BORNO_MODE_PHONETIC);
    BORNO_CHECK(ctxC.isComposing == true);

    // 6. Destroy Context B and verify Context A & C survive cleanly; verify Context D re-creates cleanly
    DestroyContextState(ctxB);
    BORNO_CHECK(ctxB.engine == nullptr);
    BORNO_CHECK(ctxB.modeSynced == false);

    BornoContextState ctxD{};
    BORNO_CHECK(InitContextState(ctxD) == true);
    BORNO_CHECK(SyncContextEngineMode(ctxD, BORNO_MODE_PASSTHROUGH) == true);
    BORNO_CHECK(ctxD.mode == BORNO_MODE_PASSTHROUGH);

    // 7. Error handling: invalid mode & null handle never report false success or corrupt state
    BORNO_CHECK(SyncContextEngineMode(ctxD, 99) == false);
    BORNO_CHECK(ctxD.mode == BORNO_MODE_PASSTHROUGH);
    BORNO_CHECK(ctxD.modeSynced == true);

    BornoContextState nullCtx{};
    BORNO_CHECK(SyncContextEngineMode(nullCtx, BORNO_MODE_FIXED_NATIONAL) == false);
    BORNO_CHECK(nullCtx.modeSynced == false);

    DestroyContextState(ctxA);
    DestroyContextState(ctxC);
    DestroyContextState(ctxD);

    std::cout << "    Mode synchronization tests passed successfully.\n";
}

void TestXdgPathAndIndependentConfigMetadata() {
    std::cout << "  Testing Correction B: XDG path resolution and per-context config metadata...\n";

    // 1. XDG_CONFIG_HOME resolution cases (using injected env values, never reading/mutating real user ~/.config)
    BORNO_CHECK(ResolveLinuxConfigPath("/custom/xdg", "/home/alice") ==
                "/custom/xdg/com.bornomala.desktop/config.json");
    BORNO_CHECK(ResolveLinuxConfigPath("/custom/xdg/", "/home/alice/") ==
                "/custom/xdg/com.bornomala.desktop/config.json");
    BORNO_CHECK(ResolveLinuxConfigPath(nullptr, "/home/alice") ==
                "/home/alice/.config/com.bornomala.desktop/config.json");
    BORNO_CHECK(ResolveLinuxConfigPath("", "/home/alice") ==
                "/home/alice/.config/com.bornomala.desktop/config.json");
    // Relative XDG paths must be rejected and fall back to HOME/.config
    BORNO_CHECK(ResolveLinuxConfigPath("relative/xdg", "/home/alice") ==
                "/home/alice/.config/com.bornomala.desktop/config.json");
    BORNO_CHECK(ResolveLinuxConfigPath("./local-cfg", "/home/alice") ==
                "/home/alice/.config/com.bornomala.desktop/config.json");
    BORNO_CHECK(ResolveLinuxConfigPath("relative", "relative_home") == "");
    BORNO_CHECK(ResolveLinuxConfigPath(nullptr, nullptr) == "");

    // 2. Two independent contexts observing config updates without blocking one another
    const std::string cfgPath = MakeTempConfigPath("cfg_reload");
    std::remove(cfgPath.c_str());

    BornoContextState ctxA{};
    BornoContextState ctxB{};
    BORNO_CHECK(InitContextState(ctxA) == true);
    BORNO_CHECK(InitContextState(ctxB) == true);

    // Missing config returns false safely without breaking engines
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxA, cfgPath) == false);
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxB, cfgPath) == false);

    // Write valid config with auto_convert_period_to_dari = true
    const std::string cfgTrue = R"({
        "schema_version": "1.2.0",
        "auto_convert_period_to_dari": true
    })";
    WriteFileText(cfgPath, cfgTrue);

    // Context A loads first -> advances ONLY ctxA.configMeta
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxA, cfgPath) == true);
    // Repeated check on ctxA without change is no-op
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxA, cfgPath) == false);
    // Context B STILL MUST load the same config independently!
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxB, cfgPath) == true);
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxB, cfgPath) == false);

    // Verify both engines honor auto_convert_period_to_dari = true ("ami." + commit) -> "আমি।"
    auto verifyPeriodOutput = [](BornoContextState& ctx, const char* expectedCommitted) {
        BornoProcessResultC res{};
        for (char c : std::string("ami.")) {
            BornoEventC ev{};
            ev.event_type = BORNO_EVENT_INPUT_CHAR;
            ev.char_codepoint = static_cast<uint32_t>(c);
            BORNO_CHECK(borno_engine_process_event(ctx.engine, &ev, &res) == BORNO_OK);
            borno_process_result_free_members(&res);
        }
        BornoEventC commitEv{};
        commitEv.event_type = BORNO_EVENT_COMMIT;
        BORNO_CHECK(borno_engine_process_event(ctx.engine, &commitEv, &res) == BORNO_OK);
        BORNO_CHECK(res.committed_text != nullptr && std::strcmp(res.committed_text, expectedCommitted) == 0);
        borno_process_result_free_members(&res);
    };
    verifyPeriodOutput(ctxA, "আমি।");
    verifyPeriodOutput(ctxB, "আমি।");

    // Subsequent update (auto_convert_period_to_dari = false) observed by BOTH contexts
    const std::string cfgFalse = R"({
        "schema_version": "1.2.0",
        "auto_convert_period_to_dari": false
    })";
    WriteFileText(cfgPath, cfgFalse);
    BumpFileTimestamp(cfgPath, 1700000100L, 123456789L);

    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxA, cfgPath) == true);
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxB, cfgPath) == true);
    verifyPeriodOutput(ctxA, "আমি.");
    verifyPeriodOutput(ctxB, "আমি.");

    // Malformed / unsupported JSON must fail safely and preserve existing settings + metadata
    WriteFileText(cfgPath, "{ invalid json");
    BumpFileTimestamp(cfgPath, 1700000200L, 1000000L);
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxA, cfgPath) == false);
    BORNO_CHECK(CheckAndReloadLinuxConfig(ctxB, cfgPath) == false);
    verifyPeriodOutput(ctxA, "আমি.");

    std::remove(cfgPath.c_str());
    DestroyContextState(ctxA);
    DestroyContextState(ctxB);

    std::cout << "    XDG path and independent config metadata tests passed successfully.\n";
}

void TestKeyPassthroughDecisionMatrix() {
    std::cout << "  Testing Correction C: key-event passthrough decision matrix...\n";

    BornoContextState ctx{};
    BORNO_CHECK(InitContextState(ctx) == true);

    // 1. Matrix row: (BORNO_OK, handled=true) -> Consumed
    {
        BornoProcessResultC res{};
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = 'a';
        int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
        BORNO_CHECK(rc == BORNO_OK && res.handled == true);
        BORNO_CHECK(ShouldConsumeEngineResult(rc, res) == true);
        BORNO_CHECK(ApplyEngineResultToState(rc, res, ctx) == true);
        BORNO_CHECK(ctx.isComposing == true);
        borno_process_result_free_members(&res);
    }

    // Cancel composition cleanly
    {
        BornoProcessResultC res{};
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_CANCEL;
        int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
        BORNO_CHECK(ShouldConsumeEngineResult(rc, res) == true);
        BORNO_CHECK(ApplyEngineResultToState(rc, res, ctx) == true);
        BORNO_CHECK(ctx.isComposing == false);
        borno_process_result_free_members(&res);
    }

    // 2. Matrix row: (BORNO_OK, handled=false) -> Passed through (real engine idle Backspace/Cancel)
    {
        BornoProcessResultC res{};
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_CANCEL;
        int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
        BORNO_CHECK(rc == BORNO_OK && res.handled == false);
        BORNO_CHECK(ShouldConsumeEngineResult(rc, res) == false);
        ctx.isComposing = true; // marker to ensure ApplyEngineResultToState leaves state untouched
        BORNO_CHECK(ApplyEngineResultToState(rc, res, ctx) == false);
        BORNO_CHECK(ctx.isComposing == true);
        ctx.isComposing = false;
        borno_process_result_free_members(&res);
    }

    // 3. Matrix row: (Error, handled=false) -> Passed through
    {
        BornoProcessResultC res{};
        res.handled = false;
        BornoEventC badEv{};
        badEv.event_type = 9999; // invalid event
        int32_t rc = borno_engine_process_event(ctx.engine, &badEv, &res);
        BORNO_CHECK(rc == BORNO_ERR_INVALID_ARGUMENT);
        BORNO_CHECK(ShouldConsumeEngineResult(rc, res) == false);
        BORNO_CHECK(ApplyEngineResultToState(rc, res, ctx) == false);
    }

    // 4. Matrix row: (Error, handled=true / corrupt output state) -> MUST NEVER consume accidentally!
    {
        BornoProcessResultC poisonRes{};
        poisonRes.handled = true;
        poisonRes.state.is_composing = true;
        poisonRes.state.mode = BORNO_MODE_FIXED_BIJOY;

        BORNO_CHECK(ShouldConsumeEngineResult(BORNO_ERR_NULL_PTR, poisonRes) == false);
        BORNO_CHECK(ShouldConsumeEngineResult(BORNO_ERR_INVALID_ARGUMENT, poisonRes) == false);
        BORNO_CHECK(ShouldConsumeEngineResult(BORNO_ERR_PANIC, poisonRes) == false);
        BORNO_CHECK(ApplyEngineResultToState(BORNO_ERR_PANIC, poisonRes, ctx) == false);
        BORNO_CHECK(ctx.isComposing == false);
        BORNO_CHECK(ctx.mode == BORNO_MODE_PHONETIC);
    }

    DestroyContextState(ctx);
    std::cout << "    Key-event passthrough matrix tests passed successfully.\n";
}

void TestPointerCandidateSelectionSync() {
    std::cout << "  Testing Correction D: pointer candidate selection state synchronization...\n";

    BornoContextState ctx{};
    BORNO_CHECK(InitContextState(ctx) == true);

    // 1. Type "k" -> active composition with multiple candidates
    {
        BornoProcessResultC res{};
        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_INPUT_CHAR;
        ev.char_codepoint = 'k';
        int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
        BORNO_CHECK(ApplyEngineResultToState(rc, res, ctx) == true);
        BORNO_CHECK(ctx.isComposing == true);
        BORNO_CHECK(ctx.candidatesCount > 0);
        borno_process_result_free_members(&res);
    }

    // 2. Invalid out-of-range selection must fail WITHOUT clearing composition or emitting commit text
    {
        BornoProcessResultC badSel{};
        BORNO_CHECK(SelectCandidateInContext(ctx, 9999, &badSel) == false);
        BORNO_CHECK(badSel.committed_text == nullptr);
        BORNO_CHECK(ctx.isComposing == true);
        BORNO_CHECK(ctx.candidatesCount > 0);
        borno_process_result_free_members(&badSel);
    }

    // 3. Valid pointer candidate selection clears composing state and commits text
    {
        BornoProcessResultC selRes{};
        BORNO_CHECK(SelectCandidateInContext(ctx, 0, &selRes) == true);
        BORNO_CHECK(selRes.handled == true);
        BORNO_CHECK(selRes.committed_text != nullptr && std::strlen(selRes.committed_text) > 0);
        BORNO_CHECK(selRes.state.is_composing == false);
        BORNO_CHECK(selRes.state.candidates_count == 0);
        BORNO_CHECK(selRes.state.cursor_pos == 0);
        BORNO_CHECK(selRes.state.preedit != nullptr && std::strcmp(selRes.state.preedit, "") == 0);
        BORNO_CHECK(ctx.isComposing == false);
        BORNO_CHECK(ctx.candidatesCount == 0);
        borno_process_result_free_members(&selRes);
    }

    // 4. Subsequent Escape / Backspace / Enter must NOT be consumed as composition events now that ctx.isComposing == false
    {
        const int32_t idleEvents[] = {BORNO_EVENT_CANCEL, BORNO_EVENT_BACKSPACE, BORNO_EVENT_COMMIT};
        for (int32_t evtType : idleEvents) {
            BornoProcessResultC res{};
            BornoEventC ev{};
            ev.event_type = evtType;
            int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
            BORNO_CHECK(ShouldConsumeEngineResult(rc, res) == false);
            borno_process_result_free_members(&res);
        }
    }

    // 5. Failed call with null handle does not falsely clear ctx.isComposing
    {
        BornoContextState deadCtx{};
        deadCtx.isComposing = true;
        BornoProcessResultC res{};
        BORNO_CHECK(SelectCandidateInContext(deadCtx, 0, &res) == false);
        BORNO_CHECK(deadCtx.isComposing == true);
    }

    DestroyContextState(ctx);
    std::cout << "    Pointer candidate selection state synchronization tests passed successfully.\n";
}

int main() {
    std::cout << "[Layer 4] Running Headless Engine Adapter Tests...\n";
    TestUtf8CodepointToByteOffset();
    TestMultiContextIsolation();
    TestModeSynchronization();
    TestXdgPathAndIndependentConfigMetadata();
    TestKeyPassthroughDecisionMatrix();
    TestPointerCandidateSelectionSync();
    std::cout << "[Layer 4] Headless Engine Adapter Tests PASSED!\n";
    return 0;
}
