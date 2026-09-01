#include "gui/main_window.h"
#include "service/download/manager.h"
#include <wx/textdlg.h>
#include <wx/msgdlg.h>
#include <wx/filedlg.h>
#include <wx/clipbrd.h>
#include <wx/dataobj.h>
#include <wx/artprov.h>
#include <wx/filename.h>
#include <wx/utils.h>
#include <iomanip>
#include <sstream>
#include <filesystem>

wxBEGIN_EVENT_TABLE(MainWindow, wxFrame)
    EVT_CLOSE(MainWindow::OnClose)
    EVT_MENU(wxID_EXIT, MainWindow::OnExit)
    EVT_MENU(wxID_ABOUT, MainWindow::OnAbout)
    EVT_MENU(ID_AddDownload, MainWindow::OnAddDownload)
    EVT_MENU(ID_AddTorrent, MainWindow::OnAddTorrent)
    EVT_MENU(ID_PauseAll, MainWindow::OnPauseAll)
    EVT_MENU(ID_ResumeAll, MainWindow::OnResumeAll)
    EVT_MENU(ID_ClearCompleted, MainWindow::OnClearCompleted)
    EVT_MENU(ID_DeleteSelected, MainWindow::OnDeleteSelected)
    EVT_MENU(ID_OpenDownloadFolder, MainWindow::OnOpenDownloadFolder)

    EVT_TOOL(ID_AddDownload, MainWindow::OnAddDownload)
    EVT_TOOL(ID_AddTorrent, MainWindow::OnAddTorrent)
    EVT_TOOL(ID_PauseAll, MainWindow::OnPauseAll)
    EVT_TOOL(ID_ResumeAll, MainWindow::OnResumeAll)
    EVT_TOOL(ID_DeleteSelected, MainWindow::OnDeleteSelected)
    EVT_TOOL(ID_ClearCompleted, MainWindow::OnClearCompleted)
    EVT_TOOL(ID_OpenDownloadFolder, MainWindow::OnOpenDownloadFolder)

    EVT_LISTBOX(ID_Sidebar, MainWindow::OnSidebarSelected)
    EVT_LIST_ITEM_RIGHT_CLICK(ID_DownloadList, MainWindow::OnListRightClick)
    EVT_LIST_ITEM_ACTIVATED(ID_DownloadList, MainWindow::OnListDoubleClick)

    EVT_MENU(ID_CtxPause, MainWindow::OnCtxPause)
    EVT_MENU(ID_CtxResume, MainWindow::OnCtxResume)
    EVT_MENU(ID_CtxRestart, MainWindow::OnCtxRestart)
    EVT_MENU(ID_CtxOpenFile, MainWindow::OnCtxOpenFile)
    EVT_MENU(ID_CtxOpenFolder, MainWindow::OnCtxOpenFolder)
    EVT_MENU(ID_CtxCopyUrl, MainWindow::OnCtxCopyUrl)
    EVT_MENU(ID_CtxRemove, MainWindow::OnCtxRemove)
    EVT_MENU(ID_CtxDeleteFile, MainWindow::OnCtxDeleteFile)
    EVT_MENU(ID_CtxViewSegments, MainWindow::OnCtxViewSegments)

    EVT_TIMER(ID_ProgressTimer, MainWindow::OnTimer)
wxEND_EVENT_TABLE()

MainWindow::MainWindow(const wxString& title)
    : wxFrame(NULL, wxID_ANY, title, wxDefaultPosition, wxSize(1100, 680)),
      m_progressTimer(this, ID_ProgressTimer)
{
    SetMinSize(wxSize(800, 480));

    // Setup Window Icon
    wxIcon appIcon;
    wxBitmap appBmp = wxArtProvider::GetBitmap(wxART_HARDDISK, wxART_FRAME_ICON, wxSize(32, 32));
    if (appBmp.IsOk()) {
        appIcon.CopyFromBitmap(appBmp);
        SetIcon(appIcon);
    }

    // Initialize System Tray Icon
    m_trayIcon = std::make_unique<AppTrayIcon>(this);

    // Setup UI Components
    SetupMenuBar();
    SetupToolbar();

    // Create Main Layout with Splitter
    wxBoxSizer* mainSizer = new wxBoxSizer(wxVERTICAL);

    m_splitter = new wxSplitterWindow(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_3D | wxSP_LIVE_UPDATE);

    // Sidebar Categories (IDM Style)
    wxArrayString sidebarChoices;
    sidebarChoices.Add("All Downloads");
    sidebarChoices.Add("Active (Downloading)");
    sidebarChoices.Add("Completed");
    sidebarChoices.Add("Torrents & Magnets");
    sidebarChoices.Add("Videos (.mp4, .mkv)");
    sidebarChoices.Add("Compressed (.zip, .rar)");
    sidebarChoices.Add("Programs (.exe, .msi)");
    sidebarChoices.Add("Documents (.pdf, .doc)");

    m_sidebar = new wxListBox(m_splitter, ID_Sidebar, wxDefaultPosition, wxDefaultSize, sidebarChoices, 0);
    m_sidebar->SetSelection(0);

    // Download Table
    SetupListCtrl();

    m_splitter->SplitVertically(m_sidebar, m_downloadList, 210);
    m_splitter->SetMinimumPaneSize(160);

    mainSizer->Add(m_splitter, 1, wxEXPAND | wxALL, 0);
    SetSizer(mainSizer);

    SetupStatusBar();

    // Initial Refresh & Start Timer
    RefreshList();
    m_progressTimer.Start(500);
}

MainWindow::~MainWindow()
{
    m_progressTimer.Stop();
    if (m_trayIcon) {
        m_trayIcon->RemoveIcon();
    }
}

void MainWindow::SetupMenuBar()
{
    wxMenuBar* menuBar = new wxMenuBar;

    // File Menu
    wxMenu* menuFile = new wxMenu;
    menuFile->Append(ID_AddDownload, "&Add URL Download...\tCtrl+N", "Add a new URL to download");
    menuFile->Append(ID_AddTorrent, "Add &Torrent / Magnet Link...\tCtrl+T", "Add a .torrent file or magnet link");
    menuFile->AppendSeparator();
    menuFile->Append(ID_PauseAll, "Pause &All\tCtrl+P", "Pause all active downloads");
    menuFile->Append(ID_ResumeAll, "&Resume All\tCtrl+R", "Resume all downloads");
    menuFile->Append(ID_ClearCompleted, "Clear &Completed", "Remove finished downloads from the list");
    menuFile->AppendSeparator();
    menuFile->Append(wxID_EXIT, "E&xit\tCtrl+Q", "Exit the application completely");

    // View Menu
    wxMenu* menuView = new wxMenu;
    menuView->Append(ID_OpenDownloadFolder, "&Open Downloads Folder\tCtrl+O", "Open default downloads directory");

    // Help Menu
    wxMenu* menuHelp = new wxMenu;
    menuHelp->Append(wxID_ABOUT, "&About Internet Downloader", "About this software");

    menuBar->Append(menuFile, "&File");
    menuBar->Append(menuView, "&View");
    menuBar->Append(menuHelp, "&Help");
    SetMenuBar(menuBar);
}

void MainWindow::SetupToolbar()
{
    m_toolbar = CreateToolBar(wxTB_FLAT | wxTB_HORIZONTAL);

    wxBitmap bmpAdd = wxArtProvider::GetBitmap(wxART_PLUS, wxART_TOOLBAR, wxSize(20, 20));
    wxBitmap bmpTorrent = wxArtProvider::GetBitmap(wxART_FILE_OPEN, wxART_TOOLBAR, wxSize(20, 20));
    wxBitmap bmpPause = wxArtProvider::GetBitmap(wxART_MINUS, wxART_TOOLBAR, wxSize(20, 20));
    wxBitmap bmpResume = wxArtProvider::GetBitmap(wxART_GO_FORWARD, wxART_TOOLBAR, wxSize(20, 20));
    wxBitmap bmpDelete = wxArtProvider::GetBitmap(wxART_DELETE, wxART_TOOLBAR, wxSize(20, 20));
    wxBitmap bmpFolder = wxArtProvider::GetBitmap(wxART_FOLDER_OPEN, wxART_TOOLBAR, wxSize(20, 20));

    m_toolbar->AddTool(ID_AddDownload, "Add URL", bmpAdd, "Add a new URL to download");
    m_toolbar->AddTool(ID_AddTorrent, "Add Torrent", bmpTorrent, "Add a .torrent file or Magnet URI");
    m_toolbar->AddSeparator();
    m_toolbar->AddTool(ID_ResumeAll, "Resume All", bmpResume, "Resume all paused downloads");
    m_toolbar->AddTool(ID_PauseAll, "Pause All", bmpPause, "Pause all active downloads");
    m_toolbar->AddSeparator();
    m_toolbar->AddTool(ID_DeleteSelected, "Delete", bmpDelete, "Delete selected download");
    m_toolbar->AddTool(ID_OpenDownloadFolder, "Open Folder", bmpFolder, "Open download folder");

    m_toolbar->Realize();
}

void MainWindow::SetupListCtrl()
{
    m_downloadList = new wxListCtrl(m_splitter, ID_DownloadList, wxDefaultPosition, wxDefaultSize, wxLC_REPORT | wxLC_SINGLE_SEL);

    m_downloadList->InsertColumn(0, "#", wxLIST_FORMAT_RIGHT, 45);
    m_downloadList->InsertColumn(1, "Filename", wxLIST_FORMAT_LEFT, 240);
    m_downloadList->InsertColumn(2, "Size", wxLIST_FORMAT_RIGHT, 95);
    m_downloadList->InsertColumn(3, "Progress", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(4, "Speed", wxLIST_FORMAT_RIGHT, 100);
    m_downloadList->InsertColumn(5, "Threads / Peers", wxLIST_FORMAT_CENTER, 105);
    m_downloadList->InsertColumn(6, "Status", wxLIST_FORMAT_LEFT, 105);
    m_downloadList->InsertColumn(7, "URL / Source", wxLIST_FORMAT_LEFT, 220);
}

void MainWindow::SetupStatusBar()
{
    CreateStatusBar(3);
    int widths[] = {-1, 180, 180};
    SetStatusWidths(3, widths);
    SetStatusText("Ready", 0);
    SetStatusText("Speed: 0 B/s", 1);
    SetStatusText("Active: 0 | Total: 0", 2);
}

static std::string FormatBytes(uint64_t bytes)
{
    const char* suffixes[] = {"B", "KB", "MB", "GB", "TB"};
    int suffixIndex = 0;
    double dBytes = static_cast<double>(bytes);
    while (dBytes >= 1024.0 && suffixIndex < 4) {
        dBytes /= 1024.0;
        suffixIndex++;
    }
    std::stringstream stream;
    stream << std::fixed << std::setprecision(suffixIndex == 0 ? 0 : 2) << dBytes << " " << suffixes[suffixIndex];
    return stream.str();
}

void MainWindow::RefreshList()
{
    auto allDownloads = idr::download::DownloadManager::GetInstance().GetDownloads();

    std::vector<std::shared_ptr<idr::download::Download>> filtered;
    double totalSpeed = 0.0;
    int activeCount = 0;

    for (auto& dl : allDownloads) {
        auto st = dl->GetStatus();
        if (st == idr::download::DownloadStatus::Downloading) {
            activeCount++;
            totalSpeed += dl->GetSpeedBytesPerSec();
        }

        std::string fname = dl->GetFilename();
        std::transform(fname.begin(), fname.end(), fname.begin(), ::tolower);

        bool match = false;
        switch (m_selectedCategory) {
            case 0: match = true; break; // All
            case 1: match = (st == idr::download::DownloadStatus::Downloading || st == idr::download::DownloadStatus::Queued); break;
            case 2: match = (st == idr::download::DownloadStatus::Completed); break;
            case 3: match = dl->IsTorrent(); break; // Torrents
            case 4: match = (fname.rfind(".mp4") != std::string::npos || fname.rfind(".mkv") != std::string::npos || fname.rfind(".avi") != std::string::npos); break; // Video
            case 5: match = (fname.rfind(".zip") != std::string::npos || fname.rfind(".rar") != std::string::npos || fname.rfind(".7z") != std::string::npos || fname.rfind(".tar") != std::string::npos); break; // Compressed
            case 6: match = (fname.rfind(".exe") != std::string::npos || fname.rfind(".msi") != std::string::npos || fname.rfind(".iso") != std::string::npos); break; // Programs
            case 7: match = (fname.rfind(".pdf") != std::string::npos || fname.rfind(".doc") != std::string::npos || fname.rfind(".txt") != std::string::npos); break; // Documents
        }
        if (match) {
            filtered.push_back(dl);
        }
    }

    while (m_downloadList->GetItemCount() > static_cast<int>(filtered.size())) {
        m_downloadList->DeleteItem(m_downloadList->GetItemCount() - 1);
    }
    while (m_downloadList->GetItemCount() < static_cast<int>(filtered.size())) {
        m_downloadList->InsertItem(m_downloadList->GetItemCount(), "");
    }

    for (size_t i = 0; i < filtered.size(); ++i) {
        auto& dl = filtered[i];

        uint64_t total = dl->GetTotalBytes();
        uint64_t dled = dl->GetDownloadedBytes();
        double speed = dl->GetSpeedBytesPerSec();

        std::string progressStr = "0%";
        if (total > 0) {
            double percent = (static_cast<double>(dled) / static_cast<double>(total)) * 100.0;
            if (percent > 100.0) percent = 100.0;
            std::stringstream stream;
            stream << std::fixed << std::setprecision(1) << percent << "%";
            progressStr = stream.str();
        } else if (dled > 0) {
            progressStr = FormatBytes(dled);
        }

        std::string statusStr = "Unknown";
        switch (dl->GetStatus()) {
            case idr::download::DownloadStatus::Queued: statusStr = "Queued"; break;
            case idr::download::DownloadStatus::Downloading: statusStr = dl->IsTorrent() ? "Downloading" : "Accelerating"; break;
            case idr::download::DownloadStatus::Paused: statusStr = "Paused"; break;
            case idr::download::DownloadStatus::Completed: statusStr = dl->IsTorrent() ? "Seeding" : "Completed"; break;
            case idr::download::DownloadStatus::Error: statusStr = "Error"; break;
        }

        std::string connInfo = dl->IsTorrent()
            ? (std::to_string(dl->GetPeers()) + " peers")
            : (std::to_string(dl->GetSegmentCount()) + " threads");

        m_downloadList->SetItem(static_cast<long>(i), 0, wxString::Format("%d", dl->GetId()));
        m_downloadList->SetItem(static_cast<long>(i), 1, wxString::FromUTF8(dl->GetFilename()));
        m_downloadList->SetItem(static_cast<long>(i), 2, total > 0 ? FormatBytes(total) : (dled > 0 ? FormatBytes(dled) : "-"));
        m_downloadList->SetItem(static_cast<long>(i), 3, progressStr);
        m_downloadList->SetItem(static_cast<long>(i), 4, (dl->GetStatus() == idr::download::DownloadStatus::Downloading && speed > 0) ? FormatBytes(static_cast<uint64_t>(speed)) + "/s" : "-");
        m_downloadList->SetItem(static_cast<long>(i), 5, wxString::FromUTF8(connInfo));
        m_downloadList->SetItem(static_cast<long>(i), 6, statusStr);
        m_downloadList->SetItem(static_cast<long>(i), 7, wxString::FromUTF8(dl->GetUrl()));

        m_downloadList->SetItemData(static_cast<long>(i), dl->GetId());
    }

    SetStatusText(activeCount > 0 ? wxString::Format("Downloading %d item%s (IDM Multi-Stream Accelerated)...", activeCount, activeCount > 1 ? "s" : "") : "Ready", 0);
    SetStatusText(wxString::Format("Speed: %s/s", FormatBytes(static_cast<uint64_t>(totalSpeed))), 1);
    SetStatusText(wxString::Format("Active: %d | Total: %d", activeCount, static_cast<int>(allDownloads.size())), 2);
}

int MainWindow::GetSelectedDownloadId()
{
    long item = m_downloadList->GetNextItem(-1, wxLIST_NEXT_ALL, wxLIST_STATE_SELECTED);
    if (item != -1) {
        return static_cast<int>(m_downloadList->GetItemData(item));
    }
    return -1;
}

void MainWindow::OnClose(wxCloseEvent& event)
{
    if (event.CanVeto() && !m_forceExit) {
        event.Veto();
        Hide();
    } else {
        Destroy();
    }
}

void MainWindow::ForceExit()
{
    m_forceExit = true;
    Close(true);
}

void MainWindow::OnExit(wxCommandEvent& WXUNUSED(event))
{
    ForceExit();
}

void MainWindow::OnAbout(wxCommandEvent& WXUNUSED(event))
{
    wxMessageBox("Internet Downloader v1.0.0\n\nHigh-Performance Accelerated Download Manager\nSupports Multi-Segment HTTP/HTTPS & BitTorrent / Magnet links.\nBuilt with C++, wxWidgets, libcurl & sqlite-orm.",
                 "About Internet Downloader", wxOK | wxICON_INFORMATION, this);
}

void MainWindow::OnAddDownload(wxCommandEvent& WXUNUSED(event))
{
    wxString defaultUrl = "https://speed.hetzner.de/100MB.bin";

    if (wxTheClipboard->Open()) {
        if (wxTheClipboard->IsSupported(wxDF_TEXT)) {
            wxTextDataObject data;
            wxTheClipboard->GetData(data);
            wxString clipText = data.GetText().Trim().Trim(false);
            if (clipText.StartsWith("http://") || clipText.StartsWith("https://") || clipText.StartsWith("ftp://") || clipText.StartsWith("magnet:?")) {
                defaultUrl = clipText;
            }
        }
        wxTheClipboard->Close();
    }

    wxTextEntryDialog urlDlg(this, "Enter Download URL or Magnet Link:", "Add New Accelerated Download", defaultUrl);
    if (urlDlg.ShowModal() != wxID_OK) return;

    wxString url = urlDlg.GetValue().Trim().Trim(false);
    if (url.IsEmpty()) return;

    wxString suggestedName = url.AfterLast('/');
    if (suggestedName.Find('?') != wxNOT_FOUND) {
        suggestedName = suggestedName.BeforeFirst('?');
    }
    if (suggestedName.IsEmpty() || url.StartsWith("magnet:?")) {
        suggestedName = url.StartsWith("magnet:?") ? "torrent_download.dat" : "download.dat";
    }

    wxFileDialog saveDlg(this, "Save File As", "", suggestedName,
                         "All Files (*.*)|*.*", wxFD_SAVE | wxFD_OVERWRITE_PROMPT);
    if (saveDlg.ShowModal() != wxID_OK) return;

    wxString destPath = saveDlg.GetPath();

    idr::download::DownloadManager::GetInstance().AddDownload(url.ToStdString(), destPath.ToStdString(), true);
    RefreshList();
}

void MainWindow::OnAddTorrent(wxCommandEvent& WXUNUSED(event))
{
    wxFileDialog openDlg(this, "Select .torrent File", "", "",
                         "Torrent Files (*.torrent)|*.torrent|All Files (*.*)|*.*", wxFD_OPEN | wxFD_FILE_MUST_EXIST);

    if (openDlg.ShowModal() == wxID_OK) {
        wxString torrentPath = openDlg.GetPath();
        wxString defaultDest = openDlg.GetFilename().BeforeLast('.') + ".download";

        wxFileDialog saveDlg(this, "Save Torrent Payload To", "", defaultDest,
                             "All Files (*.*)|*.*", wxFD_SAVE | wxFD_OVERWRITE_PROMPT);
        if (saveDlg.ShowModal() == wxID_OK) {
            idr::download::DownloadManager::GetInstance().AddDownload(torrentPath.ToStdString(), saveDlg.GetPath().ToStdString(), true);
            RefreshList();
        }
    }
}

void MainWindow::OnPauseAll(wxCommandEvent& WXUNUSED(event))
{
    idr::download::DownloadManager::GetInstance().PauseAll();
    RefreshList();
}

void MainWindow::OnResumeAll(wxCommandEvent& WXUNUSED(event))
{
    idr::download::DownloadManager::GetInstance().ResumeAll();
    RefreshList();
}

void MainWindow::OnClearCompleted(wxCommandEvent& WXUNUSED(event))
{
    idr::download::DownloadManager::GetInstance().ClearCompleted();
    RefreshList();
}

void MainWindow::OnDeleteSelected(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id <= 0) {
        wxMessageBox("Please select a download to delete.", "No Selection", wxOK | wxICON_INFORMATION, this);
        return;
    }

    int res = wxMessageBox("Are you sure you want to remove this download?", "Confirm Delete", wxYES_NO | wxICON_QUESTION, this);
    if (res == wxYES) {
        idr::download::DownloadManager::GetInstance().RemoveDownload(id, false);
        RefreshList();
    }
}

void MainWindow::OnOpenDownloadFolder(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
        for (auto& dl : downloads) {
            if (dl->GetId() == id) {
                wxFileName fn(dl->GetDestination());
                wxString folderPath = fn.GetPath();
                if (!folderPath.IsEmpty() && wxDirExists(folderPath)) {
                    wxLaunchDefaultApplication(folderPath);
                    return;
                }
            }
        }
    }
    wxLaunchDefaultApplication(wxGetCwd());
}

void MainWindow::OnSidebarSelected(wxCommandEvent& event)
{
    m_selectedCategory = event.GetSelection();
    RefreshList();
}

void MainWindow::OnListRightClick(wxListEvent& event)
{
    int id = static_cast<int>(event.GetData());
    if (id <= 0) return;

    auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
    std::shared_ptr<idr::download::Download> targetDl = nullptr;
    for (auto& dl : downloads) {
        if (dl->GetId() == id) {
            targetDl = dl;
            break;
        }
    }
    if (!targetDl) return;

    wxMenu menu;
    auto status = targetDl->GetStatus();

    if (status == idr::download::DownloadStatus::Downloading) {
        menu.Append(ID_CtxPause, "&Pause");
    } else if (status == idr::download::DownloadStatus::Paused || status == idr::download::DownloadStatus::Error) {
        menu.Append(ID_CtxResume, "&Resume");
    }
    menu.Append(ID_CtxRestart, "Re&start");
    menu.Append(ID_CtxViewSegments, "View &Connections (IDM Segments)...");
    menu.AppendSeparator();

    if (status == idr::download::DownloadStatus::Completed) {
        menu.Append(ID_CtxOpenFile, "&Open File");
    }
    menu.Append(ID_CtxOpenFolder, "Open &Containing Folder");
    menu.Append(ID_CtxCopyUrl, "&Copy Download Link");
    menu.AppendSeparator();
    menu.Append(ID_CtxRemove, "&Remove from List");
    menu.Append(ID_CtxDeleteFile, "&Delete File from Disk");

    PopupMenu(&menu);
}

void MainWindow::OnListDoubleClick(wxListEvent& event)
{
    int id = static_cast<int>(event.GetData());
    if (id <= 0) return;

    auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
    for (auto& dl : downloads) {
        if (dl->GetId() == id) {
            if (dl->GetStatus() == idr::download::DownloadStatus::Completed) {
                wxLaunchDefaultApplication(dl->GetDestination());
            } else if (dl->GetStatus() == idr::download::DownloadStatus::Downloading) {
                dl->Pause();
            } else {
                dl->Resume();
            }
            RefreshList();
            break;
        }
    }
}

void MainWindow::OnCtxPause(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        idr::download::DownloadManager::GetInstance().PauseDownload(id);
        RefreshList();
    }
}

void MainWindow::OnCtxResume(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        idr::download::DownloadManager::GetInstance().ResumeDownload(id);
        RefreshList();
    }
}

void MainWindow::OnCtxRestart(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        idr::download::DownloadManager::GetInstance().RestartDownload(id);
        RefreshList();
    }
}

void MainWindow::OnCtxOpenFile(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
        for (auto& dl : downloads) {
            if (dl->GetId() == id) {
                wxLaunchDefaultApplication(dl->GetDestination());
                break;
            }
        }
    }
}

void MainWindow::OnCtxOpenFolder(wxCommandEvent& event)
{
    OnOpenDownloadFolder(event);
}

void MainWindow::OnCtxCopyUrl(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
        for (auto& dl : downloads) {
            if (dl->GetId() == id) {
                if (wxTheClipboard->Open()) {
                    wxTheClipboard->SetData(new wxTextDataObject(dl->GetUrl()));
                    wxTheClipboard->Close();
                }
                break;
            }
        }
    }
}

void MainWindow::OnCtxRemove(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        idr::download::DownloadManager::GetInstance().RemoveDownload(id, false);
        RefreshList();
    }
}

void MainWindow::OnCtxDeleteFile(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id > 0) {
        int res = wxMessageBox("Are you sure you want to permanently delete the downloaded file from disk?",
                               "Confirm File Deletion", wxYES_NO | wxICON_WARNING, this);
        if (res == wxYES) {
            idr::download::DownloadManager::GetInstance().RemoveDownload(id, true);
            RefreshList();
        }
    }
}

void MainWindow::OnCtxViewSegments(wxCommandEvent& WXUNUSED(event))
{
    int id = GetSelectedDownloadId();
    if (id <= 0) return;

    auto downloads = idr::download::DownloadManager::GetInstance().GetDownloads();
    for (auto& dl : downloads) {
        if (dl->GetId() == id) {
            auto segs = dl->GetSegments();
            wxString msg = wxString::Format("File: %s\nConnections: %d parallel threads\nSpeed: %s/s\n\n",
                                            dl->GetFilename(),
                                            dl->GetSegmentCount(),
                                            FormatBytes(static_cast<uint64_t>(dl->GetSpeedBytesPerSec())));

            if (segs.empty()) {
                msg += "Single-stream mode active (Server does not support Range chunks).";
            } else {
                for (size_t s = 0; s < segs.size(); ++s) {
                    msg += wxString::Format("• Thread #%zu: Range %llu - %llu (%s)\n",
                                            s + 1, segs[s].startOffset, segs[s].endOffset,
                                            segs[s].isCompleted ? "Done" : "Downloading");
                }
            }

            wxMessageBox(msg, "IDM Segment & Connection Details", wxOK | wxICON_INFORMATION, this);
            break;
        }
    }
}

void MainWindow::OnTimer(wxTimerEvent& WXUNUSED(event))
{
    RefreshList();
}
