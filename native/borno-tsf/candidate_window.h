#pragma once

#ifndef CANDIDATE_WINDOW_H
#define CANDIDATE_WINDOW_H

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <msctf.h>
#include <cstdint>
#include <string>
#include <vector>
#include <functional>

namespace borno {

struct CandidateItem {
    std::wstring text;
    std::wstring badge;
    int32_t kind; // 0=Primary, 1=Alt, 2=Word, 3=Comp, 4=Corr
};

class CandidateWindow {
    HWND m_hwnd;
    HINSTANCE m_hInst;
    std::vector<CandidateItem> m_candidates;
    int32_t m_selectedIndex;
    std::function<void(size_t)> m_onSelectCallback;
    int m_dpi;

    static LRESULT CALLBACK WndProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);
    void OnPaint(HWND hwnd);
    void OnLButtonUp(int x, int y);

public:
    CandidateWindow(HINSTANCE hInst);
    ~CandidateWindow();

    bool Create();
    void Destroy();
    void SetCandidates(const std::vector<CandidateItem>& candidates, int32_t selectedIndex);
    void Show(bool show);
    bool IsVisible() const;
    size_t GetCandidateCount() const { return m_candidates.size(); }

    void SetSelectCallback(std::function<void(size_t)> callback) {
        m_onSelectCallback = callback;
    }

    void UpdatePosition(ITfContext* pContext, TfEditCookie ec, ITfRange* pRange);
};

} // namespace borno

#endif // CANDIDATE_WINDOW_H
