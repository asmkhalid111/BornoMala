#pragma once

#ifndef EDIT_SESSION_H
#define EDIT_SESSION_H

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <msctf.h>
#include <string>
#include "key_mapper.h"

// Forward declaration of BornoTextService
class BornoTextService;

enum class EditAction {
    ProcessKey,
    SelectCandidate,
    CommitComposition,
    CancelComposition,
    ResetComposition
};

class BornoEditSession : public ITfEditSession {
    LONG m_refCount;
    BornoTextService* m_pTextService;
    ITfContext* m_pContext;
    EditAction m_action;
    WPARAM m_wParam;
    LPARAM m_lParam;
    size_t m_candidateIndex;

public:
    BOOL m_fHandled;

    BornoEditSession(BornoTextService* pService, ITfContext* pContext, EditAction action,
                     WPARAM wParam = 0, LPARAM lParam = 0, size_t candidateIndex = 0)
        : m_refCount(1),
          m_pTextService(pService),
          m_pContext(pContext),
          m_action(action),
          m_wParam(wParam),
          m_lParam(lParam),
          m_candidateIndex(candidateIndex),
          m_fHandled(FALSE)
    {
        if (m_pContext) m_pContext->AddRef();
    }

    virtual ~BornoEditSession() {
        if (m_pContext) {
            m_pContext->Release();
            m_pContext = nullptr;
        }
    }

    // --- IUnknown ---
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override {
        if (!ppvObj) return E_INVALIDARG;
        *ppvObj = nullptr;
        if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_ITfEditSession)) {
            *ppvObj = static_cast<ITfEditSession*>(this);
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

    // --- ITfEditSession ---
    STDMETHODIMP DoEditSession(TfEditCookie ec) override;
};

#endif // EDIT_SESSION_H
