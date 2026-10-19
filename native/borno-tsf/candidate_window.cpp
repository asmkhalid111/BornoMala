#include "candidate_window.h"
#include <algorithm>
#include <cstdint>

namespace borno {

static const wchar_t *CANDIDATE_WINDOW_CLASS = L"BornoCandidateWindow";
static const int ROW_HEIGHT = 28;
static const int WINDOW_WIDTH = 260;
static const int PADDING = 6;

CandidateWindow::CandidateWindow(HINSTANCE hInst)
    : m_hwnd(nullptr), m_hInst(hInst), m_selectedIndex(-1), m_dpi(96) {}

CandidateWindow::~CandidateWindow() { Destroy(); }

bool CandidateWindow::Create() {
  if (m_hwnd)
    return true;

  WNDCLASSEXW wcex = {};
  wcex.cbSize = sizeof(WNDCLASSEXW);
  wcex.style = CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW;
  wcex.lpfnWndProc = WndProc;
  wcex.hInstance = m_hInst;
  wcex.hCursor = LoadCursor(nullptr, IDC_ARROW);
  wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
  wcex.lpszClassName = CANDIDATE_WINDOW_CLASS;

  RegisterClassExW(&wcex);

  m_hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                           CANDIDATE_WINDOW_CLASS, L"", WS_POPUP | WS_BORDER, 0,
                           0, WINDOW_WIDTH, ROW_HEIGHT, nullptr, nullptr,
                           m_hInst, this);

  return (m_hwnd != nullptr);
}

void CandidateWindow::Destroy() {
  if (m_hwnd) {
    DestroyWindow(m_hwnd);
    m_hwnd = nullptr;
  }
}

void CandidateWindow::SetCandidates(
    const std::vector<CandidateItem> &candidates, int32_t selectedIndex) {
  m_candidates = candidates;
  m_selectedIndex = selectedIndex;

  if (!m_hwnd)
    Create();

  if (m_candidates.empty()) {
    Show(false);
    return;
  }

  float scale = (m_dpi > 0) ? static_cast<float>(m_dpi) / 96.0f : 1.0f;
  int rowHeight = static_cast<int>(28 * scale);
  int windowWidth = static_cast<int>(260 * scale);
  int padding = static_cast<int>(6 * scale);
  int totalHeight =
      padding * 2 + static_cast<int>(m_candidates.size()) * rowHeight;

  SetWindowPos(m_hwnd, nullptr, 0, 0, windowWidth, totalHeight,
               SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);

  InvalidateRect(m_hwnd, nullptr, TRUE);
  Show(true);
}

void CandidateWindow::Show(bool show) {
  if (m_hwnd) {
    if (show) {
      ShowWindow(m_hwnd, SW_SHOWNOACTIVATE);
    } else {
      ShowWindow(m_hwnd, SW_HIDE);
    }
  }
}

bool CandidateWindow::IsVisible() const {
  return m_hwnd && IsWindowVisible(m_hwnd);
}

void CandidateWindow::UpdatePosition(ITfContext *pContext, TfEditCookie ec,
                                     ITfRange *pRange) {
  if (!m_hwnd || m_candidates.empty())
    return;

  // Update DPI from DC if available
  HDC hdcScreen = GetDC(m_hwnd);
  if (hdcScreen) {
    int dpiY = GetDeviceCaps(hdcScreen, LOGPIXELSY);
    if (dpiY > 0)
      m_dpi = dpiY;
    ReleaseDC(m_hwnd, hdcScreen);
  }

  float scale = (m_dpi > 0) ? static_cast<float>(m_dpi) / 96.0f : 1.0f;
  int rowHeight = static_cast<int>(28 * scale);
  int windowWidth = static_cast<int>(260 * scale);
  int padding = static_cast<int>(6 * scale);
  int totalHeight =
      padding * 2 + static_cast<int>(m_candidates.size()) * rowHeight;

  RECT rcCaret = {};
  BOOL hasCoord = FALSE;

  if (pContext && pRange) {
    ITfContextView *pView = nullptr;
    if (SUCCEEDED(pContext->GetActiveView(&pView)) && pView) {
      BOOL fClipped = FALSE;
      if (SUCCEEDED(pView->GetTextExt(ec, pRange, &rcCaret, &fClipped))) {
        if (rcCaret.left != 0 || rcCaret.top != 0 || rcCaret.right != 0 ||
            rcCaret.bottom != 0) {
          hasCoord = TRUE;
        }
      }
      pView->Release();
    }
  }

  // Fallback 1: Win32 Caret
  if (!hasCoord) {
    POINT pt = {};
    if (GetCaretPos(&pt)) {
      HWND hFocus = GetFocus();
      if (hFocus) {
        ClientToScreen(hFocus, &pt);
        rcCaret.left = pt.x;
        rcCaret.right = pt.x + 2;
        rcCaret.top = pt.y;
        rcCaret.bottom = pt.y + static_cast<int>(20 * scale);
        hasCoord = TRUE;
      }
    }
  }

  // Fallback 2: Active Window lower-left
  if (!hasCoord) {
    HWND hActive = GetActiveWindow();
    if (hActive) {
      RECT rcClient = {};
      GetClientRect(hActive, &rcClient);
      POINT pt = {rcClient.left + static_cast<int>(20 * scale),
                  rcClient.bottom - static_cast<int>(80 * scale)};
      ClientToScreen(hActive, &pt);
      rcCaret.left = pt.x;
      rcCaret.right = pt.x + 2;
      rcCaret.top = pt.y;
      rcCaret.bottom = pt.y + static_cast<int>(20 * scale);
      hasCoord = TRUE;
    }
  }

  int posX = rcCaret.left;
  int posY = rcCaret.bottom + static_cast<int>(4 * scale);

  // Monitor boundary clamping
  POINT ptCheck = {posX, posY};
  HMONITOR hMon = MonitorFromPoint(ptCheck, MONITOR_DEFAULTTONEAREST);
  if (hMon) {
    MONITORINFO mi = {sizeof(mi)};
    if (GetMonitorInfoW(hMon, &mi)) {
      if (posY + totalHeight > mi.rcWork.bottom) {
        // Flip above caret
        posY = rcCaret.top - totalHeight - static_cast<int>(4 * scale);
      }
      if (posX + windowWidth > mi.rcWork.right) {
        posX = mi.rcWork.right - windowWidth;
      }
      if (posX < mi.rcWork.left)
        posX = mi.rcWork.left;
      if (posY < mi.rcWork.top)
        posY = mi.rcWork.top;
    }
  }

  SetWindowPos(m_hwnd, HWND_TOPMOST, posX, posY, windowWidth, totalHeight,
               SWP_NOACTIVATE | SWP_SHOWWINDOW);
}

LRESULT CALLBACK CandidateWindow::WndProc(HWND hwnd, UINT uMsg, WPARAM wParam,
                                          LPARAM lParam) {
  CandidateWindow *pThis = nullptr;
  if (uMsg == WM_NCCREATE) {
    CREATESTRUCTW *pCS = reinterpret_cast<CREATESTRUCTW *>(lParam);
    pThis = static_cast<CandidateWindow *>(pCS->lpCreateParams);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(pThis));
  } else {
    pThis = reinterpret_cast<CandidateWindow *>(
        GetWindowLongPtrW(hwnd, GWLP_USERDATA));
  }

  switch (uMsg) {
  case WM_MOUSEACTIVATE:
    // Crucial: never steal focus from the target application
    return MA_NOACTIVATE;

  case WM_LBUTTONUP:
    if (pThis) {
      pThis->OnLButtonUp(LOWORD(lParam), HIWORD(lParam));
    }
    return 0;

  case WM_PAINT:
    if (pThis) {
      pThis->OnPaint(hwnd);
    }
    return 0;

  case WM_ERASEBKGND:
    return 1; // handled in double buffering

  default:
    break;
  }

  return DefWindowProcW(hwnd, uMsg, wParam, lParam);
}

void CandidateWindow::OnLButtonUp(int x, int y) {
  float scale = (m_dpi > 0) ? static_cast<float>(m_dpi) / 96.0f : 1.0f;
  int rowHeight = static_cast<int>(28 * scale);
  int padding = static_cast<int>(6 * scale);

  if (y < padding)
    return;
  int index = (y - padding) / rowHeight;
  if (index >= 0 && index < static_cast<int>(m_candidates.size())) {
    if (m_onSelectCallback) {
      m_onSelectCallback(static_cast<size_t>(index));
    }
  }
}

void CandidateWindow::OnPaint(HWND hwnd) {
  PAINTSTRUCT ps;
  HDC hdc = BeginPaint(hwnd, &ps);

  RECT rcClient;
  GetClientRect(hwnd, &rcClient);

  float scale = (m_dpi > 0) ? static_cast<float>(m_dpi) / 96.0f : 1.0f;
  int rowHeight = static_cast<int>(28 * scale);
  int padding = static_cast<int>(6 * scale);

  // Double buffer
  HDC memDC = CreateCompatibleDC(hdc);
  HBITMAP memBitmap =
      CreateCompatibleBitmap(hdc, rcClient.right, rcClient.bottom);
  HBITMAP oldBitmap = (HBITMAP)SelectObject(memDC, memBitmap);

  // Modern clean card background
  HBRUSH bgBrush = CreateSolidBrush(RGB(255, 255, 255));
  FillRect(memDC, &rcClient, bgBrush);
  DeleteObject(bgBrush);

  // Fonts: prefer Nirmala UI (native Windows Bengali font), with fallback to
  // Segoe UI
  int fontSize = -static_cast<int>(14 * scale);
  HFONT hFont = CreateFontW(fontSize, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
                            CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                            DEFAULT_PITCH | FF_DONTCARE, L"Nirmala UI");
  if (!hFont) {
    hFont = CreateFontW(fontSize, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
                        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
                        CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                        DEFAULT_PITCH | FF_DONTCARE, L"Segoe UI");
  }
  HFONT hOldFont = (HFONT)SelectObject(memDC, hFont);

  SetBkMode(memDC, TRANSPARENT);

  for (size_t i = 0; i < m_candidates.size(); ++i) {
    int y = padding + static_cast<int>(i) * rowHeight;
    RECT rcRow = {padding, y, rcClient.right - padding, y + rowHeight};

    bool isSelected = (static_cast<int32_t>(i) == m_selectedIndex);
    if (isSelected) {
      HBRUSH selBrush =
          CreateSolidBrush(RGB(232, 240, 254)); // Soft blue highlight
      FillRect(memDC, &rcRow, selBrush);
      DeleteObject(selBrush);
    }

    // Draw digit number
    SetTextColor(memDC, RGB(128, 128, 128));
    std::wstring numStr = std::to_wstring(i + 1) + L". ";
    int numWidth = static_cast<int>(22 * scale);
    RECT rcNum = {rcRow.left + static_cast<int>(4 * scale), rcRow.top,
                  rcRow.left + numWidth, rcRow.bottom};
    DrawTextW(memDC, numStr.c_str(), -1, &rcNum,
              DT_SINGLELINE | DT_VCENTER | DT_LEFT);

    // Draw candidate Bangla text
    int badgeWidth = static_cast<int>(52 * scale);
    SetTextColor(memDC, isSelected ? RGB(26, 115, 232) : RGB(32, 33, 36));
    RECT rcText = {rcNum.right + static_cast<int>(2 * scale), rcRow.top,
                   rcRow.right - badgeWidth - static_cast<int>(4 * scale),
                   rcRow.bottom};
    DrawTextW(memDC, m_candidates[i].text.c_str(), -1, &rcText,
              DT_SINGLELINE | DT_VCENTER | DT_LEFT);

    // Draw kind badge if present
    if (!m_candidates[i].badge.empty()) {
      RECT rcBadge = {rcRow.right - badgeWidth,
                      rcRow.top + static_cast<int>(3 * scale),
                      rcRow.right - static_cast<int>(4 * scale),
                      rcRow.bottom - static_cast<int>(3 * scale)};
      HBRUSH badgeBrush = CreateSolidBrush(RGB(241, 243, 244));
      FillRect(memDC, &rcBadge, badgeBrush);
      DeleteObject(badgeBrush);

      SetTextColor(memDC, RGB(95, 99, 104));
      DrawTextW(memDC, m_candidates[i].badge.c_str(), -1, &rcBadge,
                DT_SINGLELINE | DT_VCENTER | DT_CENTER);
    }
  }

  SelectObject(memDC, hOldFont);
  DeleteObject(hFont);

  // Blit to screen
  BitBlt(hdc, 0, 0, rcClient.right, rcClient.bottom, memDC, 0, 0, SRCCOPY);

  SelectObject(memDC, oldBitmap);
  DeleteObject(memBitmap);
  DeleteDC(memDC);

  EndPaint(hwnd, &ps);
}

} // namespace borno
