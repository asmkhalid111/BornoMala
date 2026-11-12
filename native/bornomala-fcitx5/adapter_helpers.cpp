#include "adapter_helpers.h"

#include <cstdlib>
#include <fstream>
#include <sstream>
#include <sys/stat.h>

namespace bornomala {

namespace {

std::string NormalizeAbsoluteBaseDir(const char* dir) {
    if (!dir || dir[0] != '/') {
        return "";
    }
    std::string path(dir);
    while (path.size() > 1 && path.back() == '/') {
        path.pop_back();
    }
    return path;
}

std::string JoinCanonicalConfigSuffix(const std::string& baseDir) {
    if (baseDir == "/") {
        return "/com.bornomala.desktop/config.json";
    }
    return baseDir + "/com.bornomala.desktop/config.json";
}

} // namespace

bool IsValidBornoMode(int32_t mode) {
    switch (mode) {
        case BORNO_MODE_PHONETIC:
        case BORNO_MODE_PASSTHROUGH:
        case BORNO_MODE_FIXED_NATIONAL:
        case BORNO_MODE_FIXED_PROBHAT:
        case BORNO_MODE_FIXED_BIJOY:
            return true;
        default:
            return false;
    }
}

int32_t NextCycleMode(int32_t currentMode) {
    switch (currentMode) {
        case BORNO_MODE_PHONETIC:       return BORNO_MODE_FIXED_NATIONAL;
        case BORNO_MODE_FIXED_NATIONAL: return BORNO_MODE_FIXED_PROBHAT;
        case BORNO_MODE_FIXED_PROBHAT:  return BORNO_MODE_FIXED_BIJOY;
        case BORNO_MODE_FIXED_BIJOY:    return BORNO_MODE_PASSTHROUGH;
        case BORNO_MODE_PASSTHROUGH:    return BORNO_MODE_PHONETIC;
        default:                        return BORNO_MODE_PHONETIC;
    }
}

void SyncContextStateFromC(BornoContextState& ctx, const BornoStateC& state) {
    ctx.isComposing = state.is_composing;
    ctx.candidatesCount = state.candidates_count;
    ctx.mode = state.mode;
    ctx.modeSynced = IsValidBornoMode(state.mode);
}

bool InitContextState(BornoContextState& ctx, BornoEngineHandle* engineHandle) {
    DestroyContextState(ctx);
    ctx.engine = engineHandle ? engineHandle : borno_engine_new();
    if (!ctx.engine) {
        ctx.mode = BORNO_MODE_PHONETIC;
        ctx.modeSynced = false;
        ctx.isComposing = false;
        ctx.candidatesCount = 0;
        return false;
    }

    BornoStateC initState{};
    int32_t rc = borno_engine_reset(ctx.engine, &initState);
    if (rc == BORNO_OK) {
        SyncContextStateFromC(ctx, initState);
        borno_state_free_members(&initState);
        return true;
    }

    ctx.modeSynced = false;
    return false;
}

void DestroyContextState(BornoContextState& ctx) {
    if (ctx.engine) {
        borno_engine_free(ctx.engine);
        ctx.engine = nullptr;
    }
    ctx.mode = BORNO_MODE_PHONETIC;
    ctx.modeSynced = false;
    ctx.isComposing = false;
    ctx.candidatesCount = 0;
    ctx.configMeta = LinuxConfigMetadata{};
}

bool ShouldConsumeEngineResult(int32_t rc, const BornoProcessResultC& res) {
    return rc == BORNO_OK && res.handled;
}

bool ApplyEngineResultToState(int32_t rc, const BornoProcessResultC& res, BornoContextState& ctx) {
    if (!ShouldConsumeEngineResult(rc, res)) {
        return false;
    }
    SyncContextStateFromC(ctx, res.state);
    return true;
}

bool SyncContextEngineMode(
    BornoContextState& ctx,
    int32_t targetMode,
    std::string* outCommittedText
) {
    if (outCommittedText) {
        outCommittedText->clear();
    }
    if (!ctx.engine || !IsValidBornoMode(targetMode)) {
        return false;
    }
    if (ctx.modeSynced && ctx.mode == targetMode) {
        return true;
    }

    BornoEventC ev{};
    ev.event_type = BORNO_EVENT_SWITCH_MODE;
    ev.mode_payload = targetMode;

    BornoProcessResultC res{};
    int32_t rc = borno_engine_process_event(ctx.engine, &ev, &res);
    if (ShouldConsumeEngineResult(rc, res) && res.state.mode == targetMode) {
        SyncContextStateFromC(ctx, res.state);
        if (outCommittedText && res.committed_text && res.committed_text[0] != '\0') {
            *outCommittedText = res.committed_text;
        }
        borno_process_result_free_members(&res);
        return true;
    }

    if (ShouldConsumeEngineResult(rc, res)) {
        // Engine transitioned to an unexpected mode; keep mirrored state honest.
        SyncContextStateFromC(ctx, res.state);
    }
    ctx.modeSynced = false;
    borno_process_result_free_members(&res);
    return false;
}

bool SelectCandidateInContext(
    BornoContextState& ctx,
    size_t candidateIndex,
    BornoProcessResultC* outResult
) {
    if (!outResult) {
        return false;
    }
    *outResult = BornoProcessResultC{};
    if (!ctx.engine) {
        return false;
    }

    BornoEventC ev{};
    ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
    ev.index_payload = candidateIndex;

    int32_t rc = borno_engine_process_event(ctx.engine, &ev, outResult);
    return ApplyEngineResultToState(rc, *outResult, ctx);
}

std::string ResolveLinuxConfigPath(const char* xdgConfigHome, const char* homeDir) {
    std::string xdgBase = NormalizeAbsoluteBaseDir(xdgConfigHome);
    if (!xdgBase.empty()) {
        return JoinCanonicalConfigSuffix(xdgBase);
    }

    std::string homeBase = NormalizeAbsoluteBaseDir(homeDir);
    if (!homeBase.empty()) {
        std::string configBase = (homeBase == "/") ? "/.config" : (homeBase + "/.config");
        return JoinCanonicalConfigSuffix(configBase);
    }

    return "";
}

std::string GetLinuxConfigPath() {
    return ResolveLinuxConfigPath(std::getenv("XDG_CONFIG_HOME"), std::getenv("HOME"));
}

bool CheckAndReloadLinuxConfig(BornoContextState& ctx, const std::string& configPath) {
    if (!ctx.engine || configPath.empty()) {
        return false;
    }

    struct stat st {};
    if (stat(configPath.c_str(), &st) != 0 || !S_ISREG(st.st_mode)) {
        return false;
    }

    auto& meta = ctx.configMeta;
    if (meta.hasLoaded &&
        st.st_mtim.tv_sec == meta.lastMtimeSec &&
        st.st_mtim.tv_nsec == meta.lastMtimeNsec &&
        st.st_size == meta.fileSize) {
        return false;
    }

    std::ifstream file(configPath);
    if (!file.is_open()) {
        return false;
    }

    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string content = buffer.str();

    uint32_t hash = 5381;
    for (unsigned char c : content) {
        hash = ((hash << 5) + hash) + c;
    }

    if (meta.hasLoaded && hash == meta.simpleHash) {
        meta.lastMtimeSec = st.st_mtim.tv_sec;
        meta.lastMtimeNsec = st.st_mtim.tv_nsec;
        meta.fileSize = st.st_size;
        return false;
    }

    int32_t rc = borno_engine_configure_json(ctx.engine, content.c_str());
    if (rc == BORNO_OK) {
        meta.hasLoaded = true;
        meta.lastMtimeSec = st.st_mtim.tv_sec;
        meta.lastMtimeNsec = st.st_mtim.tv_nsec;
        meta.fileSize = st.st_size;
        meta.simpleHash = hash;
        return true;
    }

    return false;
}

bool CheckAndReloadLinuxConfig(BornoContextState& ctx, std::string* outResolvedPath) {
    std::string path = GetLinuxConfigPath();
    if (outResolvedPath) {
        *outResolvedPath = path;
    }
    return CheckAndReloadLinuxConfig(ctx, path);
}

} // namespace bornomala

