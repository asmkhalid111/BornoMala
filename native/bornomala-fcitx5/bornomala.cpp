#include "engine.h"
#include <cstring>
#include <cstdlib>
#include <fstream>
#include <sstream>
#include <sys/stat.h>
#include <fcitx-utils/keysymgen.h>
#include <fcitx-utils/log.h>

namespace bornomala {

static std::string GetLinuxConfigPath() {
    const char* home = std::getenv("HOME");
    if (home) {
        return std::string(home) + "/.config/com.bornomala.desktop/config.json";
    }
    return "";
}

// Platform Boundary Note (Linux Fcitx 5):
// Candidate window themes (Dark Glass, Light Glass, High Contrast), font sizes, and layout
// orientations are strictly host-controlled by the desktop Fcitx 5 theme (e.g. Kimpanel,
// Classic UI theme, or KDE Plasma theme) under the Freedesktop / XDG desktop specification.
// The Rust core engine configuration (auto_convert_period_to_dari, linguistic ambiguity,
// candidate display limits, and fixed layouts) is fully honored and synchronized via
// borno_engine_configure_json.
static void CheckAndReloadLinuxConfig(BornoEngineHandle* engine, LinuxConfigMetadata& meta) {
    std::string path = GetLinuxConfigPath();
    if (path.empty()) return;

    struct stat st;
    if (stat(path.c_str(), &st) != 0) return;

    if (st.st_mtime == meta.lastMtime && st.st_size == meta.fileSize) {
        return;
    }

    std::ifstream file(path);
    if (!file.is_open()) return;

    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string content = buffer.str();

    uint32_t hash = 5381;
    for (unsigned char c : content) {
        hash = ((hash << 5) + hash) + c;
    }

    if (hash == meta.simpleHash) {
        meta.lastMtime = st.st_mtime;
        meta.fileSize = st.st_size;
        return;
    }

    if (engine) {
        int32_t rc = borno_engine_configure_json(engine, content.c_str());
        if (rc == BORNO_OK) {
            meta.lastMtime = st.st_mtime;
            meta.fileSize = st.st_size;
            meta.simpleHash = hash;
            FCITX_INFO() << "[Borno] Synchronized engine configuration from " << path;
        }
    }
}

static const char* GetCandidateComment(int32_t kind) {
    switch (kind) {
        case BORNO_CANDIDATE_PRIMARY_TRANSLITERATION: return "transliteration";
        case BORNO_CANDIDATE_PHONETIC_ALTERNATIVE:   return "alternative";
        case BORNO_CANDIDATE_EXACT_LEXICON_MATCH:    return "lexicon";
        case BORNO_CANDIDATE_PREFIX_COMPLETION:      return "completion";
        case BORNO_CANDIDATE_SPELLING_CORRECTION:    return "spelling";
        default: return "";
    }
}

// ============================================================================
// BornoCandidateWord Implementation
// ============================================================================

BornoCandidateWord::BornoCandidateWord(fcitx::Text text, size_t index, BornoEngineHandle* engine, std::string comment)
    : fcitx::CandidateWord(std::move(text)), m_index(index), m_engine(engine) {
    if (!comment.empty()) {
        setComment(fcitx::Text(std::move(comment)));
    }
}

void BornoCandidateWord::select(fcitx::InputContext* inputContext) const {
    if (!inputContext || !m_engine) {
        return;
    }
    BornoEventC ev{};
    ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
    ev.index_payload = m_index;
    BornoProcessResultC res{};
    int32_t rc = borno_engine_process_event(m_engine, &ev, &res);
    if (rc == BORNO_OK && res.handled && res.committed_text) {
        inputContext->commitString(res.committed_text);
    }
    borno_process_result_free_members(&res);

    inputContext->inputPanel().reset();
    inputContext->updatePreedit();
    inputContext->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

// ============================================================================
// BornoContextProperty Implementation
// ============================================================================

BornoContextProperty::BornoContextProperty(fcitx::InputContext* ic)
    : m_ic(ic), m_engine(borno_engine_new()), m_mode(BORNO_MODE_PHONETIC),
      m_isComposing(false), m_candidatesCount(0) {
}

BornoContextProperty::~BornoContextProperty() {
    if (m_engine) {
        borno_engine_free(m_engine);
        m_engine = nullptr;
    }
}

void BornoContextProperty::syncFromState(const BornoStateC& state) {
    m_isComposing = state.is_composing;
    m_candidatesCount = state.candidates_count;
    m_mode = state.mode;
}

// ============================================================================
// BornoEngine Implementation
// ============================================================================

BornoEngine::BornoEngine(fcitx::Instance* instance)
    : m_instance(instance),
      m_factory([this](fcitx::InputContext& ic) {
          auto* prop = new BornoContextProperty(&ic);
          prop->setMode(m_globalMode);
          return prop;
      }),
      m_globalMode(BORNO_MODE_PHONETIC) {
    m_instance->inputContextManager().registerProperty("bornoProperty", &m_factory);
}

std::vector<fcitx::InputMethodEntry> BornoEngine::listInputMethods() {
    std::vector<fcitx::InputMethodEntry> entries;
    fcitx::InputMethodEntry entry("bornomala", "BornoMala", "bn", "bornomala");
    entry.setNativeName("বর্ণমালা");
    entry.setLabel("বর্ণমালা");
    entry.setIcon("bornomala");
    entries.push_back(std::move(entry));
    return entries;
}

void BornoEngine::keyEvent(const fcitx::InputMethodEntry& entry, fcitx::KeyEvent& keyEvent) {
    FCITX_UNUSED(entry);

    // Step 1: Release filter
    if (keyEvent.isRelease()) {
        return;
    }

    // Step 2: Modifier-only filter
    if (keyEvent.key().isModifier()) {
        return;
    }

    auto* ic = keyEvent.inputContext();
    if (!ic) {
        return;
    }

    auto* prop = ic->propertyFor(&m_factory);
    if (!prop || !prop->engine()) {
        return;
    }

    // Step 3: Shortcut / Accelerator filter (Ctrl, Alt, Super without AltGr)
    if (IsShortcutModifier(keyEvent.rawKey().states())) {
        return;
    }

    auto sym = keyEvent.key().sym();
    FCITX_INFO() << "[Borno] keyEvent: sym=0x" << std::hex << sym << std::dec 
                 << ", mode=" << prop->mode() 
                 << ", isComposing=" << prop->isComposing();

    // Step 4: Mode switch via unmodified F12 (or F4 for laptop keyboards with Airplane Mode on F12)
    if (sym == FcitxKey_F12 || sym == FcitxKey_F4) {
        auto states = keyEvent.rawKey().states();
        if (states.test(fcitx::KeyState::Shift) ||
            states.test(fcitx::KeyState::Ctrl) ||
            states.test(fcitx::KeyState::Alt) ||
            states.test(fcitx::KeyState::Super)) {
            return; // Explicit pass-through: modified shortcut reaches application unconsumed
        }

        int32_t currentMode = prop->mode();
        int32_t nextMode;
        switch (currentMode) {
            case BORNO_MODE_PHONETIC:       nextMode = BORNO_MODE_FIXED_NATIONAL; break;
            case BORNO_MODE_FIXED_NATIONAL: nextMode = BORNO_MODE_FIXED_PROBHAT;  break;
            case BORNO_MODE_FIXED_PROBHAT:  nextMode = BORNO_MODE_FIXED_BIJOY;    break;
            case BORNO_MODE_FIXED_BIJOY:    nextMode = BORNO_MODE_PASSTHROUGH;     break;
            case BORNO_MODE_PASSTHROUGH:    nextMode = BORNO_MODE_PHONETIC;       break;
            default:                        nextMode = BORNO_MODE_PHONETIC;       break;
        }

        BornoEventC ev{};
        ev.event_type = BORNO_EVENT_SWITCH_MODE;
        ev.mode_payload = nextMode;
        BornoProcessResultC res{};
        int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
        if (rc == BORNO_OK && res.handled) {
            prop->setMode(nextMode);
            m_globalMode = nextMode;
            syncContextAndUI(ic, prop, res);
        }
        borno_process_result_free_members(&res);
        keyEvent.filterAndAccept();
        return;
    }

    // Step 5: English Passthrough mode
    if (prop->mode() == BORNO_MODE_PASSTHROUGH) {
        return;
    }

    // Step 6: Active composition handling
    if (prop->isComposing()) {
        // 6A: Escape -> Cancel composition
        if (sym == FcitxKey_Escape) {
            BornoEventC ev{};
            ev.event_type = BORNO_EVENT_CANCEL;
            BornoProcessResultC res{};
            int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
            }
            borno_process_result_free_members(&res);
            keyEvent.filterAndAccept();
            return;
        }

        // 6B: Enter / KP_Enter -> Commit composition
        if (sym == FcitxKey_Return || sym == FcitxKey_KP_Enter) {
            BornoEventC ev{};
            ev.event_type = BORNO_EVENT_COMMIT;
            BornoProcessResultC res{};
            int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
            }
            borno_process_result_free_members(&res);
            keyEvent.filterAndAccept();
            return;
        }

        // 6C: Space (Phonetic: InputChar ' ', Fixed: Physical Space Key)
        if (sym == FcitxKey_space) {
            BornoProcessResultC res{};
            int32_t rc;
            if (prop->mode() == BORNO_MODE_PHONETIC) {
                BornoEventC ev{};
                ev.event_type = BORNO_EVENT_INPUT_CHAR;
                ev.char_codepoint = ' ';
                rc = borno_engine_process_event(prop->engine(), &ev, &res);
            } else {
                rc = borno_engine_process_key(prop->engine(), BORNO_KEY_SPACE, 0, &res);
            }
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
            }
            borno_process_result_free_members(&res);
            keyEvent.filterAndAccept();
            return;
        }

        // 6D: Backspace
        if (sym == FcitxKey_BackSpace) {
            BornoEventC ev{};
            ev.event_type = BORNO_EVENT_BACKSPACE;
            BornoProcessResultC res{};
            int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
            }
            borno_process_result_free_members(&res);
            keyEvent.filterAndAccept();
            return;
        }

        // 6E: Candidate Navigation & Main-Row Digits (Phonetic only)
        if (prop->mode() == BORNO_MODE_PHONETIC) {
            if (sym == FcitxKey_Up) {
                BornoEventC ev{};
                ev.event_type = BORNO_EVENT_PREV_CANDIDATE;
                BornoProcessResultC res{};
                int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
                if (rc == BORNO_OK && res.handled) {
                    syncContextAndUI(ic, prop, res);
                }
                borno_process_result_free_members(&res);
                keyEvent.filterAndAccept();
                return;
            }
            if (sym == FcitxKey_Down) {
                BornoEventC ev{};
                ev.event_type = BORNO_EVENT_NEXT_CANDIDATE;
                BornoProcessResultC res{};
                int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
                if (rc == BORNO_OK && res.handled) {
                    syncContextAndUI(ic, prop, res);
                }
                borno_process_result_free_members(&res);
                keyEvent.filterAndAccept();
                return;
            }
            if (sym == FcitxKey_Left) {
                BornoEventC ev{};
                ev.event_type = BORNO_EVENT_MOVE_CURSOR_LEFT;
                BornoProcessResultC res{};
                int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
                if (rc == BORNO_OK && res.handled) {
                    syncContextAndUI(ic, prop, res);
                }
                borno_process_result_free_members(&res);
                keyEvent.filterAndAccept();
                return;
            }
            if (sym == FcitxKey_Right) {
                BornoEventC ev{};
                ev.event_type = BORNO_EVENT_MOVE_CURSOR_RIGHT;
                BornoProcessResultC res{};
                int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
                if (rc == BORNO_OK && res.handled) {
                    syncContextAndUI(ic, prop, res);
                }
                borno_process_result_free_members(&res);
                keyEvent.filterAndAccept();
                return;
            }

            // Main-row pure digits 1..9 (guard against Shift, AltGr, Ctrl, Alt, Super)
            if (sym >= FcitxKey_1 && sym <= FcitxKey_9) {
                auto states = keyEvent.rawKey().states();
                if (!states.test(fcitx::KeyState::Shift) &&
                    !states.test(fcitx::KeyState::Mod5) &&
                    !states.test(fcitx::KeyState::Ctrl) &&
                    !states.test(fcitx::KeyState::Alt) &&
                    !states.test(fcitx::KeyState::Super)) {
                    size_t cand_idx = static_cast<size_t>(sym - FcitxKey_1);
                    if (cand_idx < prop->candidatesCount()) {
                        BornoEventC ev{};
                        ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
                        ev.index_payload = cand_idx;
                        BornoProcessResultC res{};
                        int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
                        if (rc == BORNO_OK && res.handled) {
                            syncContextAndUI(ic, prop, res);
                        }
                        borno_process_result_free_members(&res);
                        keyEvent.filterAndAccept();
                        return;
                    }
                }
            }
        }
    }

    // Step 7: Ordinary Character & Physical Key Input
    if (prop->mode() == BORNO_MODE_PHONETIC) {
        uint32_t ucs = KeySymToPhoneticChar(sym);
        if (ucs > 0) {
            BornoEventC ev{};
            ev.event_type = BORNO_EVENT_INPUT_CHAR;
            ev.char_codepoint = ucs;
            BornoProcessResultC res{};
            int32_t rc = borno_engine_process_event(prop->engine(), &ev, &res);
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
                keyEvent.filterAndAccept();
            }
            borno_process_result_free_members(&res);
            return;
        }
    } else {
        // Fixed layout modes: National, Probhat, Bijoy
        uint32_t pkey = XkbKeycodeToBornoPhysicalKey(keyEvent.origKey().code());
        if (pkey != BORNO_KEY_INVALID) {
            uint32_t mods = ExtractBornoModifiers(keyEvent.rawKey().states());
            BornoProcessResultC res{};
            int32_t rc = borno_engine_process_key(prop->engine(), pkey, mods, &res);
            if (rc == BORNO_OK && res.handled) {
                syncContextAndUI(ic, prop, res);
                keyEvent.filterAndAccept();
            }
            borno_process_result_free_members(&res);
            return;
        }
    }
}

void BornoEngine::syncContextAndUI(fcitx::InputContext* ic, BornoContextProperty* prop, const BornoProcessResultC& res) {
    FCITX_INFO() << "[Borno] syncContextAndUI: handled=" << res.handled
                 << ", isComposing=" << res.state.is_composing
                 << ", preedit='" << (res.state.preedit ? res.state.preedit : "") << "'"
                 << ", committed='" << (res.committed_text ? res.committed_text : "") << "'";
    if (res.committed_text && std::strlen(res.committed_text) > 0) {
        ic->commitString(res.committed_text);
    }

    prop->syncFromState(res.state);

    if (res.state.is_composing && res.state.preedit) {
        size_t byte_cursor = CodepointToByteOffset(res.state.preedit, res.state.cursor_pos);

        if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
            fcitx::Text clientPreedit;
            clientPreedit.append(res.state.preedit, fcitx::TextFormatFlag::Underline);
            clientPreedit.setCursor(static_cast<int>(byte_cursor));
            ic->inputPanel().setClientPreedit(clientPreedit);
            ic->inputPanel().setPreedit(fcitx::Text());
        } else {
            fcitx::Text preedit;
            preedit.append(res.state.preedit, fcitx::TextFormatFlag::Underline);
            preedit.setCursor(static_cast<int>(byte_cursor));
            ic->inputPanel().setPreedit(preedit);
            ic->inputPanel().setClientPreedit(fcitx::Text());
        }

        if (prop->mode() == BORNO_MODE_PHONETIC && res.state.candidates_count > 0) {
            auto candList = std::make_unique<fcitx::CommonCandidateList>();
            candList->setLayoutHint(fcitx::CandidateLayoutHint::Vertical);
            candList->setPageSize(9);
            for (size_t i = 0; i < res.state.candidates_count; ++i) {
                std::string text = res.state.candidates[i] ? res.state.candidates[i] : "";
                int32_t kind = borno_engine_get_candidate_kind(prop->engine(), i);
                candList->append(std::make_unique<BornoCandidateWord>(
                    fcitx::Text(text),
                    i,
                    prop->engine(),
                    GetCandidateComment(kind)
                ));
            }
            if (res.state.selected_candidate_index >= 0) {
                candList->setGlobalCursorIndex(res.state.selected_candidate_index);
            } else {
                candList->setGlobalCursorIndex(-1);
            }
            ic->inputPanel().setCandidateList(std::move(candList));
        } else {
            ic->inputPanel().setCandidateList(nullptr);
        }
    } else {
        ic->inputPanel().reset();
    }

    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void BornoEngine::activate(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) {
    FCITX_UNUSED(entry);
    auto* ic = event.inputContext();
    if (!ic) return;
    auto* prop = ic->propertyFor(&m_factory);
    if (prop) {
        if (prop->engine()) {
            CheckAndReloadLinuxConfig(prop->engine(), m_configMeta);
        }
        if (!prop->isComposing()) {
            prop->setMode(m_globalMode);
        }
    }
}

void BornoEngine::deactivate(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) {
    FCITX_UNUSED(entry);
    reset(entry, event);
}

void BornoEngine::reset(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) {
    FCITX_UNUSED(entry);
    FCITX_INFO() << "[Borno] reset called";
    auto* ic = event.inputContext();
    if (!ic) return;
    auto* prop = ic->propertyFor(&m_factory);
    if (prop && prop->engine()) {
        BornoStateC reset_state{};
        borno_engine_reset(prop->engine(), &reset_state);
        prop->syncFromState(reset_state);
        borno_state_free_members(&reset_state);
    }
    ic->inputPanel().reset();
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

std::string BornoEngine::subMode(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) {
    FCITX_UNUSED(entry);
    auto* prop = ic.propertyFor(&m_factory);
    int32_t mode = prop ? prop->mode() : m_globalMode;
    switch (mode) {
        case BORNO_MODE_PHONETIC:       return "phonetic";
        case BORNO_MODE_PASSTHROUGH:    return "english";
        case BORNO_MODE_FIXED_NATIONAL: return "national";
        case BORNO_MODE_FIXED_PROBHAT:  return "probhat";
        case BORNO_MODE_FIXED_BIJOY:    return "bijoy";
        default:                        return "phonetic";
    }
}

std::string BornoEngine::subModeLabelImpl(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) {
    FCITX_UNUSED(entry);
    auto* prop = ic.propertyFor(&m_factory);
    int32_t mode = prop ? prop->mode() : m_globalMode;
    switch (mode) {
        case BORNO_MODE_PHONETIC:       return "ফোনেটিক";
        case BORNO_MODE_PASSTHROUGH:    return "English";
        case BORNO_MODE_FIXED_NATIONAL: return "জাতীয়";
        case BORNO_MODE_FIXED_PROBHAT:  return "প্রভাত";
        case BORNO_MODE_FIXED_BIJOY:    return "বিজয়";
        default:                        return "ফোনেটিক";
    }
}

std::string BornoEngine::subModeIconImpl(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) {
    FCITX_UNUSED(entry);
    FCITX_UNUSED(ic);
    return "bornomala";
}

} // namespace bornomala

FCITX_ADDON_FACTORY_V2_BACKWARDS(bornomala, bornomala::BornoEngineFactory)
