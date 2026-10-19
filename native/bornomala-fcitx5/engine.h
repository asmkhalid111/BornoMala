#ifndef BORNOMALA_FCITX5_ENGINE_H
#define BORNOMALA_FCITX5_ENGINE_H

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/event.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/text.h>
#include "bornomala.h"
#include "key_mapper.h"
#include "utf8_utils.h"

namespace bornomala {

class BornoEngine;

class BornoCandidateWord : public fcitx::CandidateWord {
public:
    BornoCandidateWord(fcitx::Text text, size_t index, BornoEngineHandle* engine, std::string comment = "");
    void select(fcitx::InputContext* inputContext) const override;

private:
    size_t m_index;
    BornoEngineHandle* m_engine;
};

class BornoContextProperty : public fcitx::InputContextProperty {
public:
    explicit BornoContextProperty(fcitx::InputContext* ic);
    ~BornoContextProperty() override;

    BornoEngineHandle* engine() const { return m_engine; }
    int32_t mode() const { return m_mode; }
    void setMode(int32_t mode) { m_mode = mode; }
    bool isComposing() const { return m_isComposing; }
    void setComposing(bool composing) { m_isComposing = composing; }
    size_t candidatesCount() const { return m_candidatesCount; }
    void setCandidatesCount(size_t count) { m_candidatesCount = count; }

    void syncFromState(const BornoStateC& state);

private:
    fcitx::InputContext* m_ic;
    BornoEngineHandle* m_engine{nullptr};
    int32_t m_mode{BORNO_MODE_PHONETIC};
    bool m_isComposing{false};
    size_t m_candidatesCount{0};
};

class BornoEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit BornoEngine(fcitx::Instance* instance);
    ~BornoEngine() override = default;

    fcitx::Instance* instance() { return m_instance; }

    std::vector<fcitx::InputMethodEntry> listInputMethods() override;
    void keyEvent(const fcitx::InputMethodEntry& entry, fcitx::KeyEvent& keyEvent) override;
    void activate(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) override;
    void deactivate(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) override;
    void reset(const fcitx::InputMethodEntry& entry, fcitx::InputContextEvent& event) override;

    std::string subMode(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) override;
    std::string subModeLabelImpl(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) override;
    std::string subModeIconImpl(const fcitx::InputMethodEntry& entry, fcitx::InputContext& ic) override;

    int32_t globalMode() const { return m_globalMode; }
    void setGlobalMode(int32_t mode) { m_globalMode = mode; }

private:
    void syncContextAndUI(fcitx::InputContext* ic, BornoContextProperty* prop, const BornoProcessResultC& res);

    fcitx::Instance* m_instance;
    fcitx::FactoryFor<BornoContextProperty> m_factory;
    int32_t m_globalMode{BORNO_MODE_PHONETIC};
};

class BornoEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance* create(fcitx::AddonManager* manager) override {
        return new BornoEngine(manager->instance());
    }
};

} // namespace bornomala

#endif // BORNOMALA_FCITX5_ENGINE_H
