#include "gui/tray_icon.h"
#include "gui/main_window.h"
#include "service/download/manager.h"
#include <wx/artprov.h>

wxBEGIN_EVENT_TABLE(AppTrayIcon, wxTaskBarIcon)
    EVT_TASKBAR_LEFT_DCLICK(AppTrayIcon::OnLeftDoubleClick)
        EVT_MENU(ID_TRAY_RESTORE, AppTrayIcon::OnMenuRestore)
            EVT_MENU(ID_TRAY_ADD_DOWNLOAD, AppTrayIcon::OnMenuAddDownload)
                EVT_MENU(ID_TRAY_PAUSE_ALL, AppTrayIcon::OnMenuPauseAll)
                    EVT_MENU(ID_TRAY_RESUME_ALL, AppTrayIcon::OnMenuResumeAll)
                        EVT_MENU(ID_TRAY_EXIT, AppTrayIcon::OnMenuExit)
                            wxEND_EVENT_TABLE()

                                AppTrayIcon::AppTrayIcon(MainWindow *mainWindow)
    : m_mainWindow(mainWindow)
{
    wxIcon icon;
    wxBitmap bmp = wxArtProvider::GetBitmap(wxART_HARDDISK, wxART_OTHER, wxSize(16, 16));
    if (bmp.IsOk())
    {
        icon.CopyFromBitmap(bmp);
        SetIcon(icon, "Internet Downloader");
    }
}

AppTrayIcon::~AppTrayIcon()
{
    RemoveIcon();
}

wxMenu *AppTrayIcon::CreatePopupMenu()
{
    wxMenu *menu = new wxMenu;
    menu->Append(ID_TRAY_RESTORE, "&Open Internet Downloader");
    menu->Append(ID_TRAY_ADD_DOWNLOAD, "&Add Download...");
    menu->AppendSeparator();
    menu->Append(ID_TRAY_PAUSE_ALL, "&Pause All Downloads");
    menu->Append(ID_TRAY_RESUME_ALL, "&Resume All Downloads");
    menu->AppendSeparator();
    menu->Append(ID_TRAY_EXIT, "E&xit");
    return menu;
}

void AppTrayIcon::ShowNotification(const wxString &title, const wxString &message, int flags)
{
#if wxUSE_TASKBARICON_BALLOONS
    ShowBalloon(title, message, 5000, flags);
#else
    wxUnusedVar(title);
    wxUnusedVar(message);
    wxUnusedVar(flags);
#endif
}

void AppTrayIcon::OnLeftDoubleClick(wxTaskBarIconEvent &WXUNUSED(event))
{
    if (m_mainWindow)
    {
        if (!m_mainWindow->IsShown())
        {
            m_mainWindow->Show(true);
        }
        if (m_mainWindow->IsIconized())
        {
            m_mainWindow->Iconize(false);
        }
        m_mainWindow->Raise();
        m_mainWindow->SetFocus();
    }
}

void AppTrayIcon::OnMenuRestore(wxCommandEvent &WXUNUSED(event))
{
    if (m_mainWindow)
    {
        m_mainWindow->Show(true);
        if (m_mainWindow->IsIconized())
        {
            m_mainWindow->Iconize(false);
        }
        m_mainWindow->Raise();
    }
}

void AppTrayIcon::OnMenuAddDownload(wxCommandEvent &WXUNUSED(event))
{
    if (m_mainWindow)
    {
        m_mainWindow->Show(true);
        m_mainWindow->Raise();
        wxCommandEvent evt(wxEVT_COMMAND_TOOL_CLICKED, ID_AddDownload);
        m_mainWindow->GetEventHandler()->ProcessEvent(evt);
    }
}

void AppTrayIcon::OnMenuPauseAll(wxCommandEvent &WXUNUSED(event))
{
    idr::download::DownloadManager::GetInstance().PauseAll();
    if (m_mainWindow)
    {
        m_mainWindow->RefreshList();
    }
}

void AppTrayIcon::OnMenuResumeAll(wxCommandEvent &WXUNUSED(event))
{
    idr::download::DownloadManager::GetInstance().ResumeAll();
    if (m_mainWindow)
    {
        m_mainWindow->RefreshList();
    }
}

void AppTrayIcon::OnMenuExit(wxCommandEvent &WXUNUSED(event))
{
    if (m_mainWindow)
    {
        m_mainWindow->ForceExit();
    }
    else
    {
        wxExit();
    }
}
