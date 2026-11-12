#ifndef BORNOMALA_FCITX5_ADAPTER_HELPERS_H
#define BORNOMALA_FCITX5_ADAPTER_HELPERS_H

#include <cstddef>
#include <cstdint>
#include <string>
#include <sys/types.h>
#include "bornomala.h"

namespace bornomala {

/**
 * @brief Per-context metadata used to detect on-disk config.json updates.
 */
struct LinuxConfigMetadata {
    bool hasLoaded{false};
    time_t lastMtimeSec{0};
    long lastMtimeNsec{0};
    off_t fileSize{0};
    uint32_t simpleHash{0};
};

/**
 * @brief Per-context engine ownership and synchronized mirror state.
 */
struct BornoContextState {
    BornoEngineHandle* engine{nullptr};
    int32_t mode{BORNO_MODE_PHONETIC};
    bool modeSynced{false};
    bool isComposing{false};
    size_t candidatesCount{0};
    LinuxConfigMetadata configMeta{};
};

/**
 * @brief Initialize a BornoContextState with a newly created or supplied engine handle.
 *
 * Queries the initial C-ABI state via borno_engine_reset() so mirrored fields match
 * the Rust core's authoritative initial state rather than an assumed local constant.
 */
bool InitContextState(BornoContextState& ctx, BornoEngineHandle* engineHandle = nullptr);

/**
 * @brief Release owned engine resources and reset mirrored state fields.
 */
void DestroyContextState(BornoContextState& ctx);

/**
 * @brief Update mirrored context flags from a C-ABI BornoStateC snapshot.
 */
void SyncContextStateFromC(BornoContextState& ctx, const BornoStateC& state);

/**
 * @brief Return true if mode is a valid C-ABI BORNO_MODE_* value.
 */
bool IsValidBornoMode(int32_t mode);

/**
 * @brief Return the next input mode in the F12/F4 mode cycle order.
 */
int32_t NextCycleMode(int32_t currentMode);

/**
 * @brief Decide whether an engine C-ABI result requires consuming the event and updating UI/state.
 *
 * Consumes strictly when rc == BORNO_OK and res.handled is true.
 */
bool ShouldConsumeEngineResult(int32_t rc, const BornoProcessResultC& res);

/**
 * @brief Apply an engine C-ABI process result to ctx strictly when consumed.
 */
bool ApplyEngineResultToState(int32_t rc, const BornoProcessResultC& res, BornoContextState& ctx);

/**
 * @brief Deterministically synchronize ctx.engine and ctx fields to targetMode via BORNO_EVENT_SWITCH_MODE.
 *
 * Idempotent when ctx.modeSynced is true and ctx.mode == targetMode.
 * Never sets ctx.modeSynced = true unless the C-ABI call succeeds, handles the event,
 * and returns state.mode == targetMode.
 */
bool SyncContextEngineMode(
    BornoContextState& ctx,
    int32_t targetMode,
    std::string* outCommittedText = nullptr
);

/**
 * @brief Dispatch a candidate selection event for candidateIndex and synchronize ctx on success.
 */
bool SelectCandidateInContext(
    BornoContextState& ctx,
    size_t candidateIndex,
    BornoProcessResultC* outResult
);

/**
 * @brief Resolve the canonical Linux configuration file path using injected env values.
 *
 * Honors XDG_CONFIG_HOME only when non-empty and absolute (starts with '/'),
 * otherwise falls back to HOME/.config when HOME is non-empty and absolute.
 * Appends /com.bornomala.desktop/config.json.
 */
std::string ResolveLinuxConfigPath(const char* xdgConfigHome, const char* homeDir);

/**
 * @brief Resolve the canonical Linux configuration file path from process environment.
 */
std::string GetLinuxConfigPath();

/**
 * @brief Reload JSON configuration from configPath into ctx.engine if file contents changed.
 *
 * Returns true when borno_engine_configure_json(ctx.engine, ...) succeeds and metadata advances.
 */
bool CheckAndReloadLinuxConfig(BornoContextState& ctx, const std::string& configPath);

/**
 * @brief Convenience overload resolving config path from environment.
 */
bool CheckAndReloadLinuxConfig(BornoContextState& ctx, std::string* outResolvedPath = nullptr);

} // namespace bornomala

#endif // BORNOMALA_FCITX5_ADAPTER_HELPERS_H

