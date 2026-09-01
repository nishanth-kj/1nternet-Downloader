#pragma once

#include <wx/wx.h>
#include <wx/listctrl.h>
#include <wx/splitter.h>
#include <wx/listbox.h>
#include <memory>
#include "gui/tray_icon.h"

class MainWindow : public wxFrame
{
public:
    explicit MainWindow(const wxString& title);
    virtual ~MainWindow();

    void ForceExit();
    void RefreshList();

private:
    // Event handlers
    void OnClose(wxCloseEvent& event);
    void OnExit(wxCommandEvent& event);
    void OnAbout(wxCommandEvent& event);
    void OnAddDownload(wxCommandEvent& event);
    void OnAddTorrent(wxCommandEvent& event);
    void OnPauseAll(wxCommandEvent& event);
    void OnResumeAll(wxCommandEvent& event);
    void OnClearCompleted(wxCommandEvent& event);
    void OnDeleteSelected(wxCommandEvent& event);
    void OnOpenDownloadFolder(wxCommandEvent& event);
    void OnSidebarSelected(wxCommandEvent& event);
    void OnTimer(wxTimerEvent& event);

    // List context menu events
    void OnListRightClick(wxListEvent& event);
    void OnListDoubleClick(wxListEvent& event);
    void OnCtxPause(wxCommandEvent& event);
    void OnCtxResume(wxCommandEvent& event);
    void OnCtxRestart(wxCommandEvent& event);
    void OnCtxOpenFile(wxCommandEvent& event);
    void OnCtxOpenFolder(wxCommandEvent& event);
    void OnCtxCopyUrl(wxCommandEvent& event);
    void OnCtxRemove(wxCommandEvent& event);
    void OnCtxDeleteFile(wxCommandEvent& event);
    void OnCtxViewSegments(wxCommandEvent& event);

    // UI Setup helpers
    void SetupMenuBar();
    void SetupToolbar();
    void SetupListCtrl();
    void SetupStatusBar();

    int GetSelectedDownloadId();

    // UI Elements
    wxSplitterWindow* m_splitter{nullptr};
    wxListBox* m_sidebar{nullptr};
    wxListCtrl* m_downloadList{nullptr};
    wxToolBar* m_toolbar{nullptr};
    wxTimer m_progressTimer;
    std::unique_ptr<AppTrayIcon> m_trayIcon;

    int m_selectedCategory{0};
    bool m_forceExit{false};

    wxDECLARE_EVENT_TABLE();
};

enum
{
    ID_AddDownload = 100,
    ID_AddTorrent,
    ID_PauseAll,
    ID_ResumeAll,
    ID_ClearCompleted,
    ID_DeleteSelected,
    ID_OpenDownloadFolder,
    ID_ProgressTimer,
    ID_Sidebar,
    ID_DownloadList,

    // Context menu IDs
    ID_CtxPause = 200,
    ID_CtxResume,
    ID_CtxRestart,
    ID_CtxOpenFile,
    ID_CtxOpenFolder,
    ID_CtxCopyUrl,
    ID_CtxRemove,
    ID_CtxDeleteFile,
    ID_CtxViewSegments
};
