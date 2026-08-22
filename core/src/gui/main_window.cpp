#include "gui/main_window.h"
#include "../../include/download/manager.h"
#include <wx/textdlg.h>
#include <wx/msgdlg.h>
#include <iomanip>
#include <sstream>

wxBEGIN_EVENT_TABLE(MainWindow, wxFrame)
    EVT_MENU(wxID_EXIT,  MainWindow::OnExit)
    EVT_MENU(wxID_ABOUT, MainWindow::OnAbout)
    EVT_TOOL(ID_AddDownload, MainWindow::OnAddDownload)
    EVT_TIMER(ID_ProgressTimer, MainWindow::OnTimer)
wxEND_EVENT_TABLE()

MainWindow::MainWindow(const wxString& title)
    : wxFrame(NULL, wxID_ANY, title, wxDefaultPosition, wxSize(1000, 700)),
      m_progressTimer(this, ID_ProgressTimer)
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

    // Create Splitter Window
    m_splitter = new wxSplitterWindow(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_3D | wxSP_LIVE_UPDATE);
    
    // Create Sidebar
    wxArrayString sidebarChoices;
    sidebarChoices.Add("All Downloads");
    sidebarChoices.Add("Active");
    sidebarChoices.Add("Completed");
    sidebarChoices.Add("Trash");
    m_sidebar = new wxListBox(m_splitter, wxID_ANY, wxDefaultPosition, wxDefaultSize, sidebarChoices, 0);

    // Setup List Control
    SetupListCtrl();

    // Split the window
    m_splitter->SplitVertically(m_sidebar, m_downloadList, 200);
    m_splitter->SetMinimumPaneSize(150);

    mainSizer->Add(m_splitter, 1, wxEXPAND | wxALL, 0);

    SetSizer(mainSizer);

    CreateStatusBar();
    SetStatusText("Ready");

    // Start UI refresh timer (1 second intervals)
    m_progressTimer.Start(1000);
}

#include <wx/artprov.h>

void MainWindow::SetupToolbar()
{
    m_toolbar = CreateToolBar();
    
    wxBitmap bmpAdd = wxArtProvider::GetBitmap(wxART_PLUS, wxART_TOOLBAR, wxSize(24, 24));
    wxBitmap bmpPause = wxArtProvider::GetBitmap(wxART_MINUS, wxART_TOOLBAR, wxSize(24, 24));
    wxBitmap bmpResume = wxArtProvider::GetBitmap(wxART_GO_FORWARD, wxART_TOOLBAR, wxSize(24, 24));

    m_toolbar->AddTool(ID_AddDownload, "Add Download", bmpAdd, "Add a new download");
    m_toolbar->AddSeparator();
    m_toolbar->AddTool(ID_PauseAll, "Pause All", bmpPause, "Pause all active downloads");
    m_toolbar->AddTool(ID_ResumeAll, "Resume All", bmpResume, "Resume all paused downloads");
    
    m_toolbar->Realize();
}

void MainWindow::SetupListCtrl()
{
    m_downloadList = new wxListCtrl(m_splitter, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxLC_REPORT);
    
    m_downloadList->InsertColumn(0, "Filename", wxLIST_FORMAT_LEFT, 350);
    m_downloadList->InsertColumn(1, "Size", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(2, "Progress", wxLIST_FORMAT_RIGHT, 150);
    m_downloadList->InsertColumn(3, "Speed", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(4, "Status", wxLIST_FORMAT_LEFT, 150);
}

static std::string FormatBytes(uint64_t bytes) {
    const char* suffixes[] = {"B", "KB", "MB", "GB", "TB"};
    int suffixIndex = 0;
    double dBytes = bytes;
    while (dBytes >= 1024 && suffixIndex < 4) {
        dBytes /= 1024;
        suffixIndex++;
    }
    std::stringstream stream;
    stream << std::fixed << std::setprecision(2) << dBytes << " " << suffixes[suffixIndex];
    return stream.str();
}

void MainWindow::RefreshList()
{
    auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
    
    // Ensure row count matches
    while (m_downloadList->GetItemCount() < downloads.size()) {
        m_downloadList->InsertItem(m_downloadList->GetItemCount(), "");
    }

    for (size_t i = 0; i < downloads.size(); ++i) {
        auto& dl = downloads[i];
        
        uint64_t total = dl->GetTotalBytes();
        uint64_t dled = dl->GetDownloadedBytes();
        double speed = dl->GetSpeedBytesPerSec();
        
        std::string progressStr = "0%";
        if (total > 0) {
            double percent = (static_cast<double>(dled) / total) * 100.0;
            std::stringstream stream;
            stream << std::fixed << std::setprecision(1) << percent << "%";
            progressStr = stream.str();
        }

        std::string statusStr = "Unknown";
        switch (dl->GetStatus()) {
            case idr::download::DownloadStatus::Queued: statusStr = "Queued"; break;
            case idr::download::DownloadStatus::Downloading: statusStr = "Downloading"; break;
            case idr::download::DownloadStatus::Paused: statusStr = "Paused"; break;
            case idr::download::DownloadStatus::Completed: statusStr = "Completed"; break;
            case idr::download::DownloadStatus::Error: statusStr = "Error"; break;
        }

        m_downloadList->SetItem(i, 0, dl->GetFilename());
        m_downloadList->SetItem(i, 1, FormatBytes(total));
        m_downloadList->SetItem(i, 2, progressStr);
        m_downloadList->SetItem(i, 3, FormatBytes(speed) + "/s");
        m_downloadList->SetItem(i, 4, statusStr);
    }
}

void MainWindow::OnTimer(wxTimerEvent& event)
{
    RefreshList();
}

void MainWindow::OnExit(wxCommandEvent& event)
{
    Close(true);
}

void MainWindow::OnAbout(wxCommandEvent& event)
{
    wxMessageBox("Internet Downloader - Built with wxWidgets and libcurl",
                 "About Internet Downloader", wxOK | wxICON_INFORMATION);
}

void MainWindow::OnAddDownload(wxCommandEvent& event)
{
    wxTextEntryDialog dlg(this, "Enter URL:", "Add New Download", "https://speed.hetzner.de/100MB.bin");
    if (dlg.ShowModal() == wxID_OK)
    {
        wxString url = dlg.GetValue();
        if (!url.IsEmpty())
        {
            // Just drop it into the current directory for now
            wxString dest = url.AfterLast('/');
            if (dest.IsEmpty()) dest = "download.dat";
            
            idr::download::DownloadManager::GetInstance().AddDownload(
                url.ToStdString(), 
                dest.ToStdString()
            );
            RefreshList();
        }
    }
}
