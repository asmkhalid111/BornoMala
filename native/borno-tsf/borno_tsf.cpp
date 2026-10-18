#define WIN32_LEAN_AND_MEAN
#define INITGUID
#include <windows.h>
#include <ole2.h>
#include <olectl.h>
#include <msctf.h>
#include <new>
#include <cstdio>
#include <string>
#include <vector>

// Include the BornoMala Core C-ABI and components
#include "../../crates/bornomala-core/include/bornomala.h"
#include "key_mapper.h"
#include "candidate_window.h"
#include "lang_bar_item.h"
#include "edit_session.h"

// ============================================================================
// GUID Definitions
// ============================================================================

// {8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}
DEFINE_GUID(CLSID_BornoTextService,
    0x8a57f89c, 0x3bf7, 0x4b9e, 0xa7, 0x9b, 0x86, 0xdf, 0x37, 0x2e, 0x4f, 0x1);

// {A79D2034-7D5C-4BE8-8BA6-3BC212A803B1}
DEFINE_GUID(GUID_Profile_BornoBangla_BD,
    0xa79d2034, 0x7d5c, 0x4be8, 0x8b, 0xa6, 0x3b, 0xc2, 0x12, 0xa8, 0x3, 0xb1);

// {D8628E95-2BE3-4C07-9E11-209E385D36A2}
DEFINE_GUID(GUID_Profile_BornoBangla_IN,
    0xd8628e95, 0x2be3, 0x4c07, 0x9e, 0x11, 0x20, 0x9e, 0x38, 0x5d, 0x36, 0xa2);

// {B4C85382-72EF-4F55-9B7C-A4D8D7B0927B}
DEFINE_GUID(GUID_HOTKEY_MODE_TOGGLE,
    0xb4c85382, 0x72ef, 0x4f55, 0x9b, 0x7c, 0xa4, 0xd8, 0xd7, 0xb0, 0x92, 0x7b);

static HINSTANCE g_hInst = nullptr;
static LONG g_serverLocks = 0;
static LONG g_objectCount = 0;

#ifndef __ITfTextInputProcessorEx_INTERFACE_DEFINED__
#define __ITfTextInputProcessorEx_INTERFACE_DEFINED__
MIDL_INTERFACE("aa80e801-2021-11d2-93e0-0060b067b86e")
ITfTextInputProcessorEx : public ITfTextInputProcessor {
    virtual HRESULT STDMETHODCALLTYPE ActivateEx(ITfThreadMgr *ptim, TfClientId tid, DWORD dwFlags) = 0;
};
#endif
DEFINE_GUID(IID_ITfTextInputProcessorEx,
    0xaa80e801, 0x2021, 0x11d2, 0x93, 0xe0, 0x00, 0x60, 0xb0, 0x67, 0xb8, 0x6e);

// ============================================================================
// Borno Text Service Implementation
// ============================================================================

class BornoTextService : public ITfTextInputProcessorEx,
                         public ITfKeyEventSink,
                         public ITfCompositionSink,
                         public ITfThreadMgrEventSink
{
    LONG m_refCount;
    ITfThreadMgr* m_pThreadMgr;
    TfClientId m_clientId;
    DWORD m_threadMgrEventSinkCookie;
    BornoEngineHandle* m_pEngine;
    ITfComposition* m_pComposition;
    borno::CandidateWindow* m_pCandidateWindow;
    BornoLangBarItemButton* m_pLangBarItem;
    int32_t m_currentMode;
    bool m_isComposing;
    TfGuidAtom m_attrInputAtom;

public:
    BornoTextService()
        : m_refCount(1),
          m_pThreadMgr(nullptr),
          m_clientId(0),
          m_threadMgrEventSinkCookie(0),
          m_pEngine(nullptr),
          m_pComposition(nullptr),
          m_pCandidateWindow(nullptr),
          m_pLangBarItem(nullptr),
          m_currentMode(BORNO_MODE_PHONETIC),
          m_isComposing(false),
          m_attrInputAtom(0)
    {
        InterlockedIncrement(&g_objectCount);
        m_pEngine = borno_engine_new();
    }

    virtual ~BornoTextService() {
        if (m_pComposition) {
            m_pComposition->Release();
            m_pComposition = nullptr;
        }
        if (m_pCandidateWindow) {
            delete m_pCandidateWindow;
            m_pCandidateWindow = nullptr;
        }
        if (m_pLangBarItem) {
            m_pLangBarItem->Release();
            m_pLangBarItem = nullptr;
        }
        if (m_pEngine) {
            borno_engine_free(m_pEngine);
            m_pEngine = nullptr;
        }
        InterlockedDecrement(&g_objectCount);
    }

    // --- IUnknown (Strict COM Identity Invariant) ---
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (IsEqualIID(riid, IID_IUnknown)) {
            *ppvObj = static_cast<ITfTextInputProcessorEx*>(this);
        } else if (IsEqualIID(riid, IID_ITfTextInputProcessor)) {
            *ppvObj = static_cast<ITfTextInputProcessor*>(static_cast<ITfTextInputProcessorEx*>(this));
        } else if (IsEqualIID(riid, IID_ITfTextInputProcessorEx)) {
            *ppvObj = static_cast<ITfTextInputProcessorEx*>(this);
        } else if (IsEqualIID(riid, IID_ITfKeyEventSink)) {
            *ppvObj = static_cast<ITfKeyEventSink*>(this);
        } else if (IsEqualIID(riid, IID_ITfCompositionSink)) {
            *ppvObj = static_cast<ITfCompositionSink*>(this);
        } else if (IsEqualIID(riid, IID_ITfThreadMgrEventSink)) {
            *ppvObj = static_cast<ITfThreadMgrEventSink*>(this);
        } else {
            return E_NOINTERFACE;
        }

        AddRef();
        return S_OK;
    }

    STDMETHODIMP_(ULONG) AddRef() override {
        return InterlockedIncrement(&m_refCount);
    }

    STDMETHODIMP_(ULONG) Release() override {
        LONG count = InterlockedDecrement(&m_refCount);
        if (count == 0) delete this;
        return count;
    }

    // --- ITfTextInputProcessor ---
    STDMETHODIMP Activate(ITfThreadMgr* ptim, TfClientId tid) override {
        return ActivateEx(ptim, tid, 0);
    }

    // --- ITfTextInputProcessorEx ---
    STDMETHODIMP ActivateEx(ITfThreadMgr* ptim, TfClientId tid, DWORD dwFlags) override {
        if (!ptim) return E_INVALIDARG;
        m_pThreadMgr = ptim;
        m_pThreadMgr->AddRef();
        m_clientId = tid;

        // 1. Advise KeyEventSink via ITfKeystrokeMgr
        ITfKeystrokeMgr* pKeystrokeMgr = nullptr;
        if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr))) {
            pKeystrokeMgr->AdviseKeyEventSink(m_clientId, static_cast<ITfKeyEventSink*>(this), TRUE);

            // Register preserved hotkey: Ctrl + ` (VK_OEM_3)
            TF_PRESERVEDKEY pk = {};
            pk.uVKey = VK_OEM_3;
            pk.uModifiers = TF_MOD_CONTROL;
            pKeystrokeMgr->PreserveKey(m_clientId, GUID_HOTKEY_MODE_TOGGLE, &pk, L"Borno Mode Toggle", 17);

            pKeystrokeMgr->Release();
        }

        // 2. Advise ThreadMgrEventSink via ITfSource
        ITfSource* pSource = nullptr;
        if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource))) {
            pSource->AdviseSink(IID_ITfThreadMgrEventSink, static_cast<ITfThreadMgrEventSink*>(this), &m_threadMgrEventSinkCookie);
            pSource->Release();
        }

        // 3. Register Language Bar Item Button
        ITfLangBarItemMgr* pLangBarItemMgr = nullptr;
        if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfLangBarItemMgr, (void**)&pLangBarItemMgr))) {
            m_pLangBarItem = new (std::nothrow) BornoLangBarItemButton();
            if (m_pLangBarItem) {
                m_pLangBarItem->SetModeChangeCallback([this](int32_t mode) {
                    SetEngineMode(mode);
                });
                pLangBarItemMgr->AddItem(m_pLangBarItem);
            }
            pLangBarItemMgr->Release();
        }

        // 4. Initialize Candidate Window
        m_pCandidateWindow = new (std::nothrow) borno::CandidateWindow(g_hInst);
        if (m_pCandidateWindow) {
            m_pCandidateWindow->Create();
            m_pCandidateWindow->SetSelectCallback([this](size_t index) {
                SelectCandidateByIndex(index);
            });
        }

        // 5. Query Display Attribute Atom
        ITfCategoryMgr* pCatMgr = nullptr;
        if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                                      IID_ITfCategoryMgr, (void**)&pCatMgr))) {
            pCatMgr->RegisterGUID(GUID_PROP_ATTRIBUTE, &m_attrInputAtom);
            pCatMgr->Release();
        }

        return S_OK;
    }

    STDMETHODIMP Deactivate() override {
        // Abort any active composition
        if (m_pComposition) {
            m_pComposition->Release();
            m_pComposition = nullptr;
        }

        // Hide and destroy candidate window
        if (m_pCandidateWindow) {
            m_pCandidateWindow->Destroy();
            delete m_pCandidateWindow;
            m_pCandidateWindow = nullptr;
        }

        // Remove Language Bar item
        if (m_pThreadMgr && m_pLangBarItem) {
            ITfLangBarItemMgr* pLangBarItemMgr = nullptr;
            if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfLangBarItemMgr, (void**)&pLangBarItemMgr))) {
                pLangBarItemMgr->RemoveItem(m_pLangBarItem);
                pLangBarItemMgr->Release();
            }
            m_pLangBarItem->Release();
            m_pLangBarItem = nullptr;
        }

        // Unadvise ThreadMgrEventSink
        if (m_pThreadMgr && m_threadMgrEventSinkCookie != 0) {
            ITfSource* pSource = nullptr;
            if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfSource, (void**)&pSource))) {
                pSource->UnadviseSink(m_threadMgrEventSinkCookie);
                pSource->Release();
            }
            m_threadMgrEventSinkCookie = 0;
        }

        // Unadvise KeyEventSink and Unpreserve Hotkey
        if (m_pThreadMgr) {
            ITfKeystrokeMgr* pKeystrokeMgr = nullptr;
            if (SUCCEEDED(m_pThreadMgr->QueryInterface(IID_ITfKeystrokeMgr, (void**)&pKeystrokeMgr))) {
                TF_PRESERVEDKEY pk = {};
                pk.uVKey = VK_OEM_3;
                pk.uModifiers = TF_MOD_CONTROL;
                pKeystrokeMgr->UnpreserveKey(GUID_HOTKEY_MODE_TOGGLE, &pk);

                pKeystrokeMgr->UnadviseKeyEventSink(m_clientId);
                pKeystrokeMgr->Release();
            }

            m_pThreadMgr->Release();
            m_pThreadMgr = nullptr;
        }

        m_clientId = 0;
        m_isComposing = false;
        return S_OK;
    }

    // --- ITfCompositionSink ---
    STDMETHODIMP OnCompositionTerminated(TfEditCookie ecWrite, ITfComposition* pComposition) override {
        // Invariant: NEVER call EndComposition() recursively from inside OnCompositionTerminated!
        if (m_pComposition) {
            m_pComposition->Release();
            m_pComposition = nullptr;
        }
        m_isComposing = false;

        // Reset engine to idle
        if (m_pEngine) {
            BornoStateC state = {};
            borno_engine_reset(m_pEngine, &state);
            borno_state_free_members(&state);
        }

        // Hide candidate window
        if (m_pCandidateWindow) {
            m_pCandidateWindow->Show(false);
        }

        return S_OK;
    }

    // --- ITfThreadMgrEventSink ---
    STDMETHODIMP OnInitDocumentMgr(ITfDocumentMgr* pdim) override { return S_OK; }
    STDMETHODIMP OnUninitDocumentMgr(ITfDocumentMgr* pdim) override { return S_OK; }
    STDMETHODIMP OnSetFocus(ITfDocumentMgr* pdimFocus, ITfDocumentMgr* pdimPrevFocus) override {
        // If composition is active on focus change, reset cleanly
        if (m_isComposing && m_pComposition) {
            m_pComposition->Release();
            m_pComposition = nullptr;
            m_isComposing = false;
            if (m_pEngine) {
                BornoStateC state = {};
                borno_engine_reset(m_pEngine, &state);
                borno_state_free_members(&state);
            }
            if (m_pCandidateWindow) {
                m_pCandidateWindow->Show(false);
            }
        }
        return S_OK;
    }
    STDMETHODIMP OnPushContext(ITfContext* pic) override { return S_OK; }
    STDMETHODIMP OnPopContext(ITfContext* pic) override { return S_OK; }

    // --- ITfKeyEventSink ---
    STDMETHODIMP OnSetFocus(BOOL fForeground) override {
        return S_OK;
    }

    STDMETHODIMP OnTestKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override {
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;

        // English passthrough mode does not intercept typing
        if (m_currentMode == BORNO_MODE_PASSTHROUGH) {
            return S_OK;
        }

        // Check if context is read-only
        if (pic) {
            TF_STATUS status = {};
            if (SUCCEEDED(pic->GetStatus(&status)) && (status.dwDynamicFlags & TS_SD_READONLY)) {
                return S_OK;
            }
        }

        // If in composition: intercept alphanumeric, backspace, escape, enter, space, arrows, candidate digits
        if (m_isComposing) {
            if (wParam == VK_BACK || wParam == VK_ESCAPE || wParam == VK_RETURN ||
                wParam == VK_SPACE || wParam == VK_DELETE ||
                wParam == VK_LEFT || wParam == VK_RIGHT ||
                wParam == VK_UP || wParam == VK_DOWN) {
                *pfEaten = TRUE;
                return S_OK;
            }
            if (wParam >= '1' && wParam <= '9') {
                *pfEaten = TRUE;
                return S_OK;
            }
            uint32_t key = 0;
            if (borno::VirtualKeyToPhysicalKey(wParam, lParam, &key)) {
                *pfEaten = TRUE;
                return S_OK;
            }
        } else {
            // Not composing: check if mapped physical key
            uint32_t key = 0;
            if (borno::VirtualKeyToPhysicalKey(wParam, lParam, &key)) {
                // Ignore unmapped control combinations (Ctrl+C, Ctrl+V, etc.) in Phonetic mode
                uint32_t mods = borno::GetCurrentModifiers();
                if ((mods & (BORNO_MOD_CTRL | BORNO_MOD_ALT)) && !(mods & BORNO_MOD_ALTGR)) {
                    *pfEaten = FALSE;
                    return S_OK;
                }
                *pfEaten = TRUE;
                return S_OK;
            }
        }

        return S_OK;
    }

    STDMETHODIMP OnTestKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override {
        if (pfEaten) *pfEaten = FALSE;
        return S_OK;
    }

    STDMETHODIMP OnKeyDown(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override {
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;

        if (!pic || !m_pEngine || m_currentMode == BORNO_MODE_PASSTHROUGH) {
            return S_OK;
        }

        // Request synchronous edit session
        BornoEditSession* pSession = new (std::nothrow) BornoEditSession(
            this, pic, EditAction::ProcessKey, wParam, lParam
        );
        if (!pSession) return E_OUTOFMEMORY;

        HRESULT hrSession = S_OK;
        HRESULT hr = pic->RequestEditSession(m_clientId, pSession, TF_ES_READWRITE | TF_ES_SYNC, &hrSession);
        if (SUCCEEDED(hr) && SUCCEEDED(hrSession)) {
            *pfEaten = pSession->m_fHandled;
        } else {
            // Synchronous lock denied or edit failed: do not mutate engine, pass key to host safely
            *pfEaten = FALSE;
        }

        pSession->Release();
        return S_OK;
    }

    STDMETHODIMP OnKeyUp(ITfContext* pic, WPARAM wParam, LPARAM lParam, BOOL* pfEaten) override {
        if (pfEaten) *pfEaten = FALSE;
        return S_OK;
    }

    STDMETHODIMP OnPreservedKey(ITfContext* pic, REFGUID rguid, BOOL* pfEaten) override {
        if (!pfEaten) return E_INVALIDARG;
        *pfEaten = FALSE;

        if (IsEqualGUID(rguid, GUID_HOTKEY_MODE_TOGGLE)) {
            int32_t newMode = (m_currentMode == BORNO_MODE_PASSTHROUGH) ? BORNO_MODE_PHONETIC : BORNO_MODE_PASSTHROUGH;
            SetEngineMode(newMode);
            *pfEaten = TRUE;
            return S_OK;
        }

        return S_OK;
    }

    // --- Mode Control ---
    void SetEngineMode(int32_t mode) {
        m_currentMode = mode;
        if (m_pEngine) {
            BornoEventC ev = {};
            ev.event_type = BORNO_EVENT_SWITCH_MODE;
            ev.mode_payload = mode;
            BornoProcessResultC res = {};
            borno_engine_process_event(m_pEngine, &ev, &res);
            borno_process_result_free_members(&res);
        }
        if (m_pLangBarItem) {
            m_pLangBarItem->SetMode(mode);
        }
    }

    // --- Candidate Click Handler ---
    void SelectCandidateByIndex(size_t index) {
        if (!m_pThreadMgr) return;
        ITfDocumentMgr* pDocMgr = nullptr;
        if (SUCCEEDED(m_pThreadMgr->GetFocus(&pDocMgr)) && pDocMgr) {
            ITfContext* pContext = nullptr;
            if (SUCCEEDED(pDocMgr->GetTop(&pContext)) && pContext) {
                BornoEditSession* pSession = new (std::nothrow) BornoEditSession(
                    this, pContext, EditAction::SelectCandidate, 0, 0, index
                );
                if (pSession) {
                    HRESULT hrSession = S_OK;
                    pContext->RequestEditSession(m_clientId, pSession, TF_ES_READWRITE | TF_ES_SYNC, &hrSession);
                    pSession->Release();
                }
                pContext->Release();
            }
            pDocMgr->Release();
        }
    }

    // --- Display Attribute Helpers ---
    void SetDisplayAttribute(TfEditCookie ec, ITfContext* pic, ITfRange* pRange) {
        if (!pic || !pRange || m_attrInputAtom == 0) return;
        ITfProperty* pProp = nullptr;
        if (SUCCEEDED(pic->GetProperty(GUID_PROP_ATTRIBUTE, &pProp)) && pProp) {
            VARIANT var;
            VariantInit(&var);
            var.vt = VT_I4;
            var.lVal = static_cast<LONG>(m_attrInputAtom);
            pProp->SetValue(ec, pRange, &var);
            pProp->Release();
        }
    }

    void ClearDisplayAttribute(TfEditCookie ec, ITfContext* pic, ITfRange* pRange) {
        if (!pic || !pRange) return;
        ITfProperty* pProp = nullptr;
        if (SUCCEEDED(pic->GetProperty(GUID_PROP_ATTRIBUTE, &pProp)) && pProp) {
            pProp->Clear(ec, pRange);
            pProp->Release();
        }
    }

    // --- Document Edit Session Execution ---
    HRESULT ExecuteEditSession(TfEditCookie ec, ITfContext* pic, EditAction action,
                               WPARAM wParam, LPARAM lParam, size_t candidateIndex, BOOL* pfHandled)
    {
        if (!pfHandled) return E_INVALIDARG;
        *pfHandled = FALSE;

        if (!m_pEngine || !pic) return E_FAIL;

        BornoProcessResultC result = {};
        int32_t rc = BORNO_OK;

        if (action == EditAction::SelectCandidate) {
            BornoEventC ev = {};
            ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
            ev.index_payload = candidateIndex;
            rc = borno_engine_process_event(m_pEngine, &ev, &result);
        } else if (action == EditAction::ProcessKey) {
            // Check candidate digit selection (guarded by candidate count)
            size_t candIdx = 0;
            if (m_isComposing && m_pCandidateWindow && m_pCandidateWindow->IsVisible() &&
                borno::IsCandidateSelectionDigit(wParam, m_pCandidateWindow->GetCandidateCount(), &candIdx))
            {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_SELECT_CANDIDATE;
                ev.index_payload = candIdx;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_UP) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_PREV_CANDIDATE;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_DOWN) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_NEXT_CANDIDATE;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_LEFT) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_MOVE_CURSOR_LEFT;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_RIGHT) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_MOVE_CURSOR_RIGHT;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_BACK) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_BACKSPACE;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_DELETE) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_DELETE;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_ESCAPE) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_CANCEL;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else if (wParam == VK_RETURN) {
                BornoEventC ev = {};
                ev.event_type = BORNO_EVENT_COMMIT;
                rc = borno_engine_process_event(m_pEngine, &ev, &result);
            } else {
                uint32_t physKey = 0;
                if (borno::VirtualKeyToPhysicalKey(wParam, lParam, &physKey)) {
                    uint32_t mods = borno::GetCurrentModifiers();
                    rc = borno_engine_process_key(m_pEngine, physKey, mods, &result);
                } else {
                    return S_OK; // Unmapped
                }
            }
        }

        if (rc != BORNO_OK) {
            return E_FAIL;
        }

        *pfHandled = result.handled;

        // Apply document mutations:
        // 1. Text committed
        if (result.committed_text != nullptr) {
            std::wstring wCommit = borno::Utf8ToUtf16(result.committed_text);
            if (m_pComposition) {
                ITfRange* pRange = nullptr;
                if (SUCCEEDED(m_pComposition->GetRange(&pRange)) && pRange) {
                    ClearDisplayAttribute(ec, pic, pRange);
                    pRange->SetText(ec, 0, wCommit.c_str(), static_cast<LONG>(wCommit.length()));
                    pRange->Collapse(ec, TF_ANCHOR_END);

                    TF_SELECTION sel = {};
                    sel.range = pRange;
                    sel.style.ase = TF_AE_END;
                    sel.style.fInterimChar = FALSE;
                    pic->SetSelection(ec, 1, &sel);

                    m_pComposition->EndComposition(ec);
                    m_pComposition->Release();
                    m_pComposition = nullptr;
                    pRange->Release();
                }
            } else {
                // Direct insert at document selection
                TF_SELECTION sel = {};
                ULONG cFetched = 0;
                if (SUCCEEDED(pic->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &sel, &cFetched)) && cFetched == 1) {
                    sel.range->SetText(ec, 0, wCommit.c_str(), static_cast<LONG>(wCommit.length()));
                    sel.range->Collapse(ec, TF_ANCHOR_END);
                    pic->SetSelection(ec, 1, &sel);
                    sel.range->Release();
                }
            }
            m_isComposing = false;
            if (m_pCandidateWindow) m_pCandidateWindow->Show(false);
        }

        // 2. Composition cancelled
        if (result.cancelled) {
            if (m_pComposition) {
                ITfRange* pRange = nullptr;
                if (SUCCEEDED(m_pComposition->GetRange(&pRange)) && pRange) {
                    ClearDisplayAttribute(ec, pic, pRange);
                    pRange->SetText(ec, 0, L"", 0);
                    m_pComposition->EndComposition(ec);
                    m_pComposition->Release();
                    m_pComposition = nullptr;
                    pRange->Release();
                }
            }
            m_isComposing = false;
            if (m_pCandidateWindow) m_pCandidateWindow->Show(false);
        }

        // 3. Active composition in progress
        if (result.state.is_composing) {
            std::wstring wPreedit = borno::Utf8ToUtf16(result.state.preedit);

            if (!m_pComposition) {
                // Start new composition
                ITfContextComposition* pContextComp = nullptr;
                if (SUCCEEDED(pic->QueryInterface(IID_ITfContextComposition, (void**)&pContextComp))) {
                    TF_SELECTION sel = {};
                    ULONG cFetched = 0;
                    if (SUCCEEDED(pic->GetSelection(ec, TF_DEFAULT_SELECTION, 1, &sel, &cFetched)) && cFetched == 1) {
                        pContextComp->StartComposition(ec, sel.range, static_cast<ITfCompositionSink*>(this), &m_pComposition);
                        sel.range->Release();
                    }
                    pContextComp->Release();
                }
            }

            if (m_pComposition) {
                ITfRange* pRange = nullptr;
                if (SUCCEEDED(m_pComposition->GetRange(&pRange)) && pRange) {
                    pRange->SetText(ec, 0, wPreedit.c_str(), static_cast<LONG>(wPreedit.length()));
                    SetDisplayAttribute(ec, pic, pRange);

                    // Map cursor_pos (Unicode codepoints) to UTF-16 code units offset
                    size_t utf16Offset = 0;
                    size_t cpCount = 0;
                    for (size_t i = 0; i < wPreedit.length() && cpCount < result.state.cursor_pos; ) {
                        wchar_t ch = wPreedit[i];
                        if (ch >= 0xD800 && ch <= 0xDBFF && i + 1 < wPreedit.length()) {
                            utf16Offset += 2;
                            i += 2;
                        } else {
                            utf16Offset += 1;
                            i += 1;
                        }
                        cpCount++;
                    }

                    // Position caret inside preedit range
                    ITfRange* pCaretRange = nullptr;
                    if (SUCCEEDED(pRange->Clone(&pCaretRange)) && pCaretRange) {
                        pCaretRange->Collapse(ec, TF_ANCHOR_START);
                        LONG cch = 0;
                        pCaretRange->ShiftEnd(ec, static_cast<LONG>(utf16Offset), &cch, nullptr);
                        pCaretRange->Collapse(ec, TF_ANCHOR_END);

                        TF_SELECTION sel = {};
                        sel.range = pCaretRange;
                        sel.style.ase = TF_AE_END;
                        sel.style.fInterimChar = FALSE;
                        pic->SetSelection(ec, 1, &sel);
                        pCaretRange->Release();
                    }

                    // Update candidate window
                    if (m_pCandidateWindow) {
                        std::vector<borno::CandidateItem> items;
                        for (size_t i = 0; i < result.state.candidates_count; ++i) {
                            borno::CandidateItem item;
                            item.text = borno::Utf8ToUtf16(result.state.candidates[i]);
                            item.kind = borno_engine_get_candidate_kind(m_pEngine, i);
                            switch (item.kind) {
                                case BORNO_CANDIDATE_PRIMARY_TRANSLITERATION: item.badge = L"Main"; break;
                                case BORNO_CANDIDATE_PHONETIC_ALTERNATIVE:   item.badge = L"Alt"; break;
                                case BORNO_CANDIDATE_EXACT_LEXICON_MATCH:    item.badge = L"Word"; break;
                                case BORNO_CANDIDATE_PREFIX_COMPLETION:      item.badge = L"Comp"; break;
                                case BORNO_CANDIDATE_SPELLING_CORRECTION:    item.badge = L"Corr"; break;
                                default: item.badge = L""; break;
                            }
                            items.push_back(item);
                        }

                        m_pCandidateWindow->UpdatePosition(pic, ec, pRange);
                        m_pCandidateWindow->SetCandidates(items, result.state.selected_candidate_index);
                    }

                    pRange->Release();
                }
            }
            m_isComposing = true;
        } else if (result.committed_text == nullptr && !result.cancelled) {
            // In link-pending fixed layout or single key commit
            m_isComposing = false;
            if (m_pCandidateWindow) m_pCandidateWindow->Show(false);
        }

        borno_process_result_free_members(&result);
        return S_OK;
    }
};

// Implement BornoEditSession::DoEditSession
STDMETHODIMP BornoEditSession::DoEditSession(TfEditCookie ec) {
    if (!m_pTextService || !m_pContext) return E_FAIL;
    return m_pTextService->ExecuteEditSession(ec, m_pContext, m_action, m_wParam, m_lParam, m_candidateIndex, &m_fHandled);
}

// ============================================================================
// Class Factory Implementation
// ============================================================================

class BornoClassFactory : public IClassFactory {
    LONG m_refCount;
public:
    BornoClassFactory() : m_refCount(1) {
        InterlockedIncrement(&g_objectCount);
    }
    virtual ~BornoClassFactory() {
        InterlockedDecrement(&g_objectCount);
    }

    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;

        if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IClassFactory)) {
            *ppvObj = static_cast<IClassFactory*>(this);
            AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }

    STDMETHODIMP_(ULONG) AddRef() override {
        return InterlockedIncrement(&m_refCount);
    }

    STDMETHODIMP_(ULONG) Release() override {
        LONG count = InterlockedDecrement(&m_refCount);
        if (count == 0) delete this;
        return count;
    }

    STDMETHODIMP CreateInstance(IUnknown* pUnkOuter, REFIID riid, void** ppvObj) override {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;
        if (pUnkOuter) return CLASS_E_NOAGGREGATION;

        BornoTextService* pTip = new (std::nothrow) BornoTextService();
        if (!pTip) return E_OUTOFMEMORY;

        HRESULT hr = pTip->QueryInterface(riid, ppvObj);
        pTip->Release();
        return hr;
    }

    STDMETHODIMP LockServer(BOOL fLock) override {
        if (fLock) {
            InterlockedIncrement(&g_serverLocks);
        } else {
            InterlockedDecrement(&g_serverLocks);
        }
        return S_OK;
    }
};

// ============================================================================
// COM DLL Standard Export Entry Points
// ============================================================================

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    if (fdwReason == DLL_PROCESS_ATTACH) {
        g_hInst = hinstDLL;
        DisableThreadLibraryCalls(hinstDLL);
    }
    return TRUE;
}

STDAPI DllGetClassObject(REFCLSID rclsid, REFIID riid, void** ppv) {
    if (!ppv) return E_INVALIDARG;
    *ppv = nullptr;

    if (!IsEqualCLSID(rclsid, CLSID_BornoTextService)) {
        return CLASS_E_CLASSNOTAVAILABLE;
    }

    BornoClassFactory* pFactory = new (std::nothrow) BornoClassFactory();
    if (!pFactory) return E_OUTOFMEMORY;

    HRESULT hr = pFactory->QueryInterface(riid, ppv);
    pFactory->Release();
    return hr;
}

STDAPI DllCanUnloadNow(void) {
    return (g_serverLocks == 0 && g_objectCount == 0) ? S_OK : S_FALSE;
}

static const wchar_t* CLSID_STRING = L"{8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}";

STDAPI DllRegisterServer(void) {
    wchar_t path[MAX_PATH] = {};
    if (!GetModuleFileNameW(g_hInst, path, MAX_PATH)) {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    // 1. Register COM InprocServer32: try HKLM (elevated) first, fallback to HKCU
    HKEY rootKey = HKEY_LOCAL_MACHINE;
    HKEY hKey = nullptr;
    std::wstring subkey = L"Software\\Classes\\CLSID\\";
    subkey += CLSID_STRING;

    LSTATUS status = RegCreateKeyExW(rootKey, subkey.c_str(), 0, nullptr,
                                     REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr);
    if (status != ERROR_SUCCESS) {
        rootKey = HKEY_CURRENT_USER;
        status = RegCreateKeyExW(rootKey, subkey.c_str(), 0, nullptr,
                                 REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hKey, nullptr);
    }
    if (status != ERROR_SUCCESS) {
        return HRESULT_FROM_WIN32(status);
    }

    const wchar_t* desc = L"Borno Bangla Text Service";
    RegSetValueExW(hKey, nullptr, 0, REG_SZ, (const BYTE*)desc, (DWORD)((wcslen(desc) + 1) * sizeof(wchar_t)));

    HKEY hInproc = nullptr;
    status = RegCreateKeyExW(hKey, L"InprocServer32", 0, nullptr,
                             REG_OPTION_NON_VOLATILE, KEY_WRITE, nullptr, &hInproc, nullptr);
    if (status == ERROR_SUCCESS) {
        RegSetValueExW(hInproc, nullptr, 0, REG_SZ, (const BYTE*)path, (DWORD)((wcslen(path) + 1) * sizeof(wchar_t)));
        const wchar_t* model = L"Apartment";
        RegSetValueExW(hInproc, L"ThreadingModel", 0, REG_SZ, (const BYTE*)model, (DWORD)((wcslen(model) + 1) * sizeof(wchar_t)));
        RegCloseKey(hInproc);
    }
    RegCloseKey(hKey);

    // 2. Register TSF Category & Language Profiles
    ITfCategoryMgr* pCatMgr = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                                  IID_ITfCategoryMgr, (void**)&pCatMgr))) {
        pCatMgr->RegisterCategory(CLSID_BornoTextService, GUID_TFCAT_TIP_KEYBOARD, CLSID_BornoTextService);
        pCatMgr->Release();
    }

    ITfInputProcessorProfiles* pProfiles = nullptr;
    HRESULT hr = CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr,
                                  CLSCTX_INPROC_SERVER, IID_ITfInputProcessorProfiles,
                                  (void**)&pProfiles);
    if (SUCCEEDED(hr)) {
        hr = pProfiles->Register(CLSID_BornoTextService);
        if (SUCCEEDED(hr)) {
            // Profile 1: Bengali (Bangladesh) - 0x0845
            const wchar_t* descBD = L"Borno Bangla (Bangladesh)";
            pProfiles->AddLanguageProfile(
                CLSID_BornoTextService,
                0x0845,
                GUID_Profile_BornoBangla_BD,
                descBD,
                static_cast<ULONG>(wcslen(descBD)),
                path,
                static_cast<ULONG>(wcslen(path)),
                0
            );
            pProfiles->EnableLanguageProfile(CLSID_BornoTextService, 0x0845, GUID_Profile_BornoBangla_BD, TRUE);

            // Profile 2: Bengali (India) - 0x0445
            const wchar_t* descIN = L"Borno Bangla (India)";
            pProfiles->AddLanguageProfile(
                CLSID_BornoTextService,
                0x0445,
                GUID_Profile_BornoBangla_IN,
                descIN,
                static_cast<ULONG>(wcslen(descIN)),
                path,
                static_cast<ULONG>(wcslen(path)),
                0
            );
            pProfiles->EnableLanguageProfile(CLSID_BornoTextService, 0x0445, GUID_Profile_BornoBangla_IN, TRUE);
        } else {
            // Rollback on failure
            DllUnregisterServer();
        }
        pProfiles->Release();
    } else {
        DllUnregisterServer();
    }

    return hr;
}

STDAPI DllUnregisterServer(void) {
    // 1. Unregister TSF Profiles
    ITfInputProcessorProfiles* pProfiles = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_InputProcessorProfiles, nullptr,
                                  CLSCTX_INPROC_SERVER, IID_ITfInputProcessorProfiles,
                                  (void**)&pProfiles))) {
        pProfiles->Unregister(CLSID_BornoTextService);
        pProfiles->Release();
    }

    // 2. Unregister Category
    ITfCategoryMgr* pCatMgr = nullptr;
    if (SUCCEEDED(CoCreateInstance(CLSID_TF_CategoryMgr, nullptr, CLSCTX_INPROC_SERVER,
                                  IID_ITfCategoryMgr, (void**)&pCatMgr))) {
        pCatMgr->UnregisterCategory(CLSID_BornoTextService, GUID_TFCAT_TIP_KEYBOARD, CLSID_BornoTextService);
        pCatMgr->Release();
    }

    // 3. Clean up COM registry (both HKLM and HKCU for full idempotency)
    std::wstring subkeyInproc = L"Software\\Classes\\CLSID\\" + std::wstring(CLSID_STRING) + L"\\InprocServer32";
    std::wstring subkeyClsid = L"Software\\Classes\\CLSID\\" + std::wstring(CLSID_STRING);

    RegDeleteKeyW(HKEY_LOCAL_MACHINE, subkeyInproc.c_str());
    RegDeleteKeyW(HKEY_LOCAL_MACHINE, subkeyClsid.c_str());

    RegDeleteKeyW(HKEY_CURRENT_USER, subkeyInproc.c_str());
    RegDeleteKeyW(HKEY_CURRENT_USER, subkeyClsid.c_str());

    return S_OK;
}
