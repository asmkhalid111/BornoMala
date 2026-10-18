#pragma once

#ifndef LANG_BAR_ITEM_H
#define LANG_BAR_ITEM_H

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <ole2.h>
#include <olectl.h>
#include <msctf.h>
#include <ctfutb.h>
#include <cstdint>
#include <string>
#include <functional>

#ifndef CONNECT_E_NOCONNECTION
#define CONNECT_E_NOCONNECTION  ((HRESULT)0x80040200L)
#endif
#ifndef CONNECT_E_ADVISELIMIT
#define CONNECT_E_ADVISELIMIT   ((HRESULT)0x80040201L)
#endif
#ifndef CONNECT_E_CANNOTCONNECT
#define CONNECT_E_CANNOTCONNECT ((HRESULT)0x80040202L)
#endif

#ifndef TF_LBMENUF_CHECKED
#define TF_LBMENUF_CHECKED 0x00000001
#endif
#ifndef TF_LBI_STYLE_BTN_BUTTON
#define TF_LBI_STYLE_BTN_BUTTON 0x00010000
#endif
#ifndef TF_LBI_STYLE_BTN_MENU
#define TF_LBI_STYLE_BTN_MENU   0x00020000
#endif

#ifndef TF_LBI_STATUS
#define TF_LBI_STATUS 0x00010000
#endif
#ifndef TF_LBI_TEXT
#define TF_LBI_TEXT   0x00020000
#endif

#ifndef __ITfMenu_INTERFACE_DEFINED__
#define __ITfMenu_INTERFACE_DEFINED__
MIDL_INTERFACE("07d5ab3b-e623-4db0-89b1-4348ec52f614")
ITfMenu : public IUnknown {
    virtual HRESULT STDMETHODCALLTYPE AddMenuItem(
        UINT uId,
        DWORD dwFlags,
        HBITMAP hbmp,
        HBITMAP hbmpMask,
        const WCHAR *pch,
        ULONG cch,
        ITfMenu **ppSubMenu) = 0;
};
#endif

#ifndef __ITfLangBarItemButton_INTERFACE_DEFINED__
#define __ITfLangBarItemButton_INTERFACE_DEFINED__
typedef enum {
    TF_LBI_CLK_RIGHT = 1,
    TF_LBI_CLK_LEFT  = 2
} TfLBIClick;

MIDL_INTERFACE("28c52652-25e0-4c60-ae71-aea2e70316a3")
ITfLangBarItemButton : public ITfLangBarItem {
    virtual HRESULT STDMETHODCALLTYPE OnClick(TfLBIClick click, POINT pt, const RECT *prcArea) = 0;
    virtual HRESULT STDMETHODCALLTYPE InitMenu(ITfMenu *pMenu) = 0;
    virtual HRESULT STDMETHODCALLTYPE OnMenuSelect(UINT uId) = 0;
    virtual HRESULT STDMETHODCALLTYPE GetIcon(HICON *phIcon) = 0;
    virtual HRESULT STDMETHODCALLTYPE GetText(BSTR *pbstrText) = 0;
};
#endif

// {3E4A28B1-7A3D-4C9E-9A52-8CF8D21389A4}
DEFINE_GUID(GUID_BornoLangBarItem,
    0x3e4a28b1, 0x7a3d, 0x4c9e, 0x9a, 0x52, 0x8c, 0xf8, 0xd2, 0x13, 0x89, 0xa4);

// {28C52652-25E0-4C60-AE71-AEA2E70316A3}
DEFINE_GUID(IID_ITfLangBarItemButton,
    0x28c52652, 0x25e0, 0x4c60, 0xae, 0x71, 0xae, 0xa2, 0xe7, 0x3, 0x16, 0xa3);

class BornoLangBarItemButton : public ITfLangBarItemButton, public ITfSource {
    LONG m_refCount;
    ITfLangBarItemSink* m_pItemSink;
    int32_t m_currentMode;
    std::function<void(int32_t)> m_onModeChange;

public:
    BornoLangBarItemButton();
    virtual ~BornoLangBarItemButton();

    void SetModeChangeCallback(std::function<void(int32_t)> cb) {
        m_onModeChange = cb;
    }

    // --- IUnknown ---
    STDMETHODIMP QueryInterface(REFIID riid, void** ppvObj) override;
    STDMETHODIMP_(ULONG) AddRef() override;
    STDMETHODIMP_(ULONG) Release() override;

    // --- ITfLangBarItem ---
    STDMETHODIMP GetInfo(TF_LANGBARITEMINFO* pInfo) override;
    STDMETHODIMP GetStatus(DWORD* pdwStatus) override;
    STDMETHODIMP Show(BOOL fShow) override;
    STDMETHODIMP GetTooltipString(BSTR* pbstrToolTip) override;

    // --- ITfLangBarItemButton ---
    STDMETHODIMP OnClick(TfLBIClick click, POINT pt, const RECT* prcArea) override;
    STDMETHODIMP InitMenu(ITfMenu* pMenu) override;
    STDMETHODIMP OnMenuSelect(UINT uID) override;
    STDMETHODIMP GetIcon(HICON* phIcon) override;
    STDMETHODIMP GetText(BSTR* pbstrText) override;

    // --- ITfSource ---
    STDMETHODIMP AdviseSink(REFIID riid, IUnknown* punk, DWORD* pdwCookie) override;
    STDMETHODIMP UnadviseSink(DWORD dwCookie) override;

    void SetMode(int32_t mode);
};

#endif // LANG_BAR_ITEM_H
