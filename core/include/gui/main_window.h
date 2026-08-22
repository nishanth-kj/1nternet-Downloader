#pragma once

#include <wx/wx.h>
#include <wx/listctrl.h>
#include <wx/splitter.h>
#include <wx/listbox.h>

class MainWindow : public wxFrame
{
public:
    MainWindow(const wxString& title);

private:
    void OnExit(wxCommandEvent& event);
    void OnAbout(wxCommandEvent& event);
    void OnAddDownload(wxCommandEvent& event);
    
    // UI Elements
    wxSplitterWindow* m_splitter;
    wxListBox* m_sidebar;
    wxListCtrl* m_downloadList;
    wxToolBar* m_toolbar;
    wxTimer m_progressTimer;

    void SetupToolbar();
    void SetupListCtrl();
    void OnTimer(wxTimerEvent& event);
    void RefreshList();

    // Any class wishing to process wxWidgets events must use this macro
    wxDECLARE_EVENT_TABLE();
};

enum
{
    ID_AddDownload = 1,
    ID_PauseAll = 2,
    ID_ResumeAll = 3,
    ID_ProgressTimer = 4
};
