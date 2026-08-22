#include "gui/main_window.h"

wxBEGIN_EVENT_TABLE(MainWindow, wxFrame)
    EVT_MENU(wxID_EXIT,  MainWindow::OnExit)
    EVT_MENU(wxID_ABOUT, MainWindow::OnAbout)
    EVT_TOOL(ID_AddDownload, MainWindow::OnAddDownload)
wxEND_EVENT_TABLE()

MainWindow::MainWindow(const wxString& title)
    : wxFrame(NULL, wxID_ANY, title, wxDefaultPosition, wxSize(1000, 700))
{
    // Setup Menu
    wxMenu *menuFile = new wxMenu;
    menuFile->Append(wxID_EXIT);

    wxMenu *menuHelp = new wxMenu;
    menuHelp->Append(wxID_ABOUT);

    wxMenuBar *menuBar = new wxMenuBar;
    menuBar->Append(menuFile, "&File");
    menuBar->Append(menuHelp, "&Help");
    SetMenuBar(menuBar);

    // Create main sizer
    wxBoxSizer* mainSizer = new wxBoxSizer(wxVERTICAL);

    // Setup Toolbar
    SetupToolbar();

    // Setup List Control
    SetupListCtrl();
    mainSizer->Add(m_downloadList, 1, wxEXPAND | wxALL, 0);

    SetSizer(mainSizer);

    CreateStatusBar();
    SetStatusText("Ready");
}

void MainWindow::SetupToolbar()
{
    m_toolbar = CreateToolBar();
    
    // We could use bitmaps, but for now we'll just add simple text/icon tools
    m_toolbar->AddTool(ID_AddDownload, "Add Download", wxNullBitmap, "Add a new download");
    m_toolbar->AddSeparator();
    m_toolbar->AddTool(ID_PauseAll, "Pause All", wxNullBitmap, "Pause all active downloads");
    m_toolbar->AddTool(ID_ResumeAll, "Resume All", wxNullBitmap, "Resume all paused downloads");
    
    m_toolbar->Realize();
}

void MainWindow::SetupListCtrl()
{
    m_downloadList = new wxListCtrl(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxLC_REPORT);
    
    m_downloadList->InsertColumn(0, "Filename", wxLIST_FORMAT_LEFT, 350);
    m_downloadList->InsertColumn(1, "Size", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(2, "Progress", wxLIST_FORMAT_RIGHT, 150);
    m_downloadList->InsertColumn(3, "Speed", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(4, "Status", wxLIST_FORMAT_LEFT, 150);
    m_downloadList->InsertColumn(5, "ETA", wxLIST_FORMAT_RIGHT, 100);

    // Add some dummy data
    long itemIndex = m_downloadList->InsertItem(0, "Ubuntu-24.04-desktop-amd64.iso");
    m_downloadList->SetItem(itemIndex, 1, "5.3 GB");
    m_downloadList->SetItem(itemIndex, 2, "95%");
    m_downloadList->SetItem(itemIndex, 3, "2.3 MB/s");
    m_downloadList->SetItem(itemIndex, 4, "Downloading");
    m_downloadList->SetItem(itemIndex, 5, "1m 22s");

    itemIndex = m_downloadList->InsertItem(1, "VSCode-win32-x64-1.92.0.exe");
    m_downloadList->SetItem(itemIndex, 1, "94 MB");
    m_downloadList->SetItem(itemIndex, 2, "100%");
    m_downloadList->SetItem(itemIndex, 3, "0 KB/s");
    m_downloadList->SetItem(itemIndex, 4, "Completed");
    m_downloadList->SetItem(itemIndex, 5, "-");
}

void MainWindow::OnExit(wxCommandEvent& event)
{
    Close(true);
}

void MainWindow::OnAbout(wxCommandEvent& event)
{
    wxMessageBox("Internet Downloader - Built with wxWidgets",
                 "About Internet Downloader", wxOK | wxICON_INFORMATION);
}

void MainWindow::OnAddDownload(wxCommandEvent& event)
{
    wxTextEntryDialog dlg(this, "Enter URL or magnet link:", "Add New Download");
    if (dlg.ShowModal() == wxID_OK)
    {
        wxString url = dlg.GetValue();
        if (!url.IsEmpty())
        {
            long itemIndex = m_downloadList->InsertItem(m_downloadList->GetItemCount(), "New Download");
            m_downloadList->SetItem(itemIndex, 1, "Unknown");
            m_downloadList->SetItem(itemIndex, 2, "0%");
            m_downloadList->SetItem(itemIndex, 3, "0 KB/s");
            m_downloadList->SetItem(itemIndex, 4, "Queued");
            m_downloadList->SetItem(itemIndex, 5, "Pending");
        }
    }
}
