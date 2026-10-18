#include "lang_bar_item.h"
#include <strsafe.h>

BornoLangBarItemButton::BornoLangBarItemButton()
    : m_refCount(1),
      m_pItemSink(nullptr),
      m_currentMode(0) // Default: Phonetic
{
}

BornoLangBarItemButton::~BornoLangBarItemButton() {
    if (m_pItemSink) {
        m_pItemSink->Release();
        m_pItemSink = nullptr;
    }
}

STDMETHODIMP BornoLangBarItemButton::QueryInterface(REFIID riid, void** ppvObj) {
    if (!ppvObj) return E_INVALIDARG;
    *ppvObj = nullptr;

    if (IsEqualIID(riid, IID_IUnknown) ||
        IsEqualIID(riid, IID_ITfLangBarItem) ||
        IsEqualIID(riid, IID_ITfLangBarItemButton))
    {
        *ppvObj = static_cast<ITfLangBarItemButton*>(this);
    } else if (IsEqualIID(riid, IID_ITfSource)) {
        *ppvObj = static_cast<ITfSource*>(this);
    } else {
        return E_NOINTERFACE;
    }

    AddRef();
    return S_OK;
}

STDMETHODIMP_(ULONG) BornoLangBarItemButton::AddRef() {
    return InterlockedIncrement(&m_refCount);
}

STDMETHODIMP_(ULONG) BornoLangBarItemButton::Release() {
    LONG count = InterlockedDecrement(&m_refCount);
    if (count == 0) delete this;
    return count;
}

// {8A57F89C-3BF7-4B9E-A79B-86DF372E4F01}
static const GUID CLSID_Borno = {
    0x8a57f89c, 0x3bf7, 0x4b9e, { 0xa7, 0x9b, 0x86, 0xdf, 0x37, 0x2e, 0x4f, 0x1 }
};

STDMETHODIMP BornoLangBarItemButton::GetInfo(TF_LANGBARITEMINFO* pInfo) {
    if (!pInfo) return E_INVALIDARG;

    pInfo->clsidService = CLSID_Borno;
    pInfo->guidItem = GUID_BornoLangBarItem;
    pInfo->dwStyle = TF_LBI_STYLE_BTN_BUTTON | TF_LBI_STYLE_BTN_MENU;
    pInfo->ulSort = 0;
    StringCchCopyW(pInfo->szDescription, 32, L"Borno Bangla Mode");
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::GetStatus(DWORD* pdwStatus) {
    if (!pdwStatus) return E_INVALIDARG;
    *pdwStatus = 0;
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::Show(BOOL fShow) {
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::GetTooltipString(BSTR* pbstrToolTip) {
    if (!pbstrToolTip) return E_INVALIDARG;
    *pbstrToolTip = SysAllocString(L"Borno Bangla Input (Click to toggle, menu for layouts)");
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::OnClick(TfLBIClick click, POINT pt, const RECT* prcArea) {
    if (click == TF_LBI_CLK_LEFT) {
        int32_t newMode = (m_currentMode == 0) ? 1 : 0;
        SetMode(newMode);
        if (m_onModeChange) {
            m_onModeChange(newMode);
        }
    }
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::InitMenu(ITfMenu* pMenu) {
    if (!pMenu) return E_INVALIDARG;

    pMenu->AddMenuItem(1, (m_currentMode == 0) ? TF_LBMENUF_CHECKED : 0, nullptr, nullptr, L"Phonetic (Avro-compatible)", 25, nullptr);
    pMenu->AddMenuItem(2, (m_currentMode == 2) ? TF_LBMENUF_CHECKED : 0, nullptr, nullptr, L"National (Jatiya)", 16, nullptr);
    pMenu->AddMenuItem(3, (m_currentMode == 3) ? TF_LBMENUF_CHECKED : 0, nullptr, nullptr, L"Probhat", 7, nullptr);
    pMenu->AddMenuItem(4, (m_currentMode == 4) ? TF_LBMENUF_CHECKED : 0, nullptr, nullptr, L"Bijoy (Unicode)", 15, nullptr);
    pMenu->AddMenuItem(5, (m_currentMode == 1) ? TF_LBMENUF_CHECKED : 0, nullptr, nullptr, L"English Passthrough", 19, nullptr);

    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::OnMenuSelect(UINT uID) {
    int32_t mode = 0;
    switch (uID) {
        case 1: mode = 0; break; // Phonetic
        case 2: mode = 2; break; // Fixed National
        case 3: mode = 3; break; // Fixed Probhat
        case 4: mode = 4; break; // Fixed Bijoy
        case 5: mode = 1; break; // English Passthrough
        default: return S_OK;
    }
    SetMode(mode);
    if (m_onModeChange) {
        m_onModeChange(mode);
    }
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::GetIcon(HICON* phIcon) {
    if (!phIcon) return E_INVALIDARG;
    *phIcon = nullptr;
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::GetText(BSTR* pbstrText) {
    if (!pbstrText) return E_INVALIDARG;
    switch (m_currentMode) {
        case 0: *pbstrText = SysAllocString(L"বাং"); break;
        case 1: *pbstrText = SysAllocString(L"EN"); break;
        case 2: *pbstrText = SysAllocString(L"জাতীয়"); break;
        case 3: *pbstrText = SysAllocString(L"প্রভাত"); break;
        case 4: *pbstrText = SysAllocString(L"বিজয়"); break;
        default: *pbstrText = SysAllocString(L"বাং"); break;
    }
    return S_OK;
}

STDMETHODIMP BornoLangBarItemButton::AdviseSink(REFIID riid, IUnknown* punk, DWORD* pdwCookie) {
    if (!punk || !pdwCookie) return E_INVALIDARG;
    if (!IsEqualIID(riid, IID_ITfLangBarItemSink)) return CONNECT_E_CANNOTCONNECT;

    if (m_pItemSink) return CONNECT_E_ADVISELIMIT;

    if (SUCCEEDED(punk->QueryInterface(IID_ITfLangBarItemSink, (void**)&m_pItemSink))) {
        *pdwCookie = 1;
        return S_OK;
    }
    return E_NOINTERFACE;
}

STDMETHODIMP BornoLangBarItemButton::UnadviseSink(DWORD dwCookie) {
    if (dwCookie != 1 || !m_pItemSink) return CONNECT_E_NOCONNECTION;
    m_pItemSink->Release();
    m_pItemSink = nullptr;
    return S_OK;
}

void BornoLangBarItemButton::SetMode(int32_t mode) {
    m_currentMode = mode;
    if (m_pItemSink) {
        m_pItemSink->OnUpdate(TF_LBI_STATUS | TF_LBI_TEXT);
    }
}
