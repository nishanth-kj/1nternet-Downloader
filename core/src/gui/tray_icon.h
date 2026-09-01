#pragma once

#include <wx/wx.h>
#include <wx/taskbar.h>

class MainWindow;

class AppTrayIcon : public wxTaskBarIcon
{
public:
    explicit AppTrayIcon(MainWindow *mainWindow);
    virtual ~AppTrayIcon();

    virtual wxMenu *CreatePopupMenu() override;
    void ShowNotification(const wxString &title, const wxString &message, int flags = wxICON_INFORMATION);

private:
    void OnLeftDoubleClick(wxTaskBarIconEvent &event);
    void OnMenuRestore(wxCommandEvent &event);
    void OnMenuAddDownload(wxCommandEvent &event);
    void OnMenuPauseAll(wxCommandEvent &event);
    void OnMenuResumeAll(wxCommandEvent &event);
    void OnMenuExit(wxCommandEvent &event);

    MainWindow *m_mainWindow;

    wxDECLARE_EVENT_TABLE();
};

enum
{
    ID_TRAY_RESTORE = 10001,
    ID_TRAY_ADD_DOWNLOAD,
    ID_TRAY_PAUSE_ALL,
    ID_TRAY_RESUME_ALL,
    ID_TRAY_EXIT
};
