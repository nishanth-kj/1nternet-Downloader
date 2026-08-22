#pragma once

#include <wx/wx.h>
#include <wx/listctrl.h>

class MainWindow : public wxFrame
{
public:
    MainWindow(const wxString& title);

private:
    void OnExit(wxCommandEvent& event);
    void OnAbout(wxCommandEvent& event);
    void OnAddDownload(wxCommandEvent& event);
    
    // UI Elements
    wxListCtrl* m_downloadList;
    wxToolBar* m_toolbar;

    void SetupToolbar();
    void SetupListCtrl();

    // Any class wishing to process wxWidgets events must use this macro
    wxDECLARE_EVENT_TABLE();
};

enum
{
    ID_AddDownload = 1,
    ID_PauseAll = 2,
    ID_ResumeAll = 3
};
