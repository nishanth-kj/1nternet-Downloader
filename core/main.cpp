/*
 * main.cpp — Internet Downloader entry point
 *
 * The core/ directory is its own self-contained C++ project.
 * This file initialises wxWidgets, bootstraps the GUI state, and
 * runs the main event loop.  All logic lives in core/src/.
 *
 * It also owns the "idr://" protocol handoff: a browser extension (or the OS,
 * via the registry entries installed by scripts/register_protocol.ps1) launches
 * this executable with an "idr://<percent-encoded-url>" argument. If an instance
 * is already running, the URL is forwarded to it over a named pipe instead of
 * opening a second window (see utils/system/ipc.h).
 */

#include <wx/wx.h>
#include <wx/cmdline.h>
#include <wx/weakref.h>
#include "gui/main_window.h"
#include "utils/system/ipc.h"
#include "utils/string_utils.h"

// Define the application class
class MyApp : public wxApp
{
public:
    // wxWidgets' default command line parser rejects unrecognized positional
    // arguments and aborts OnInit() before it runs. Declare the idr:// argument as
    // an accepted optional parameter so launches like
    // "InternetDownloader.exe idr://<url>" don't get silently rejected.
    virtual void OnInitCmdLine(wxCmdLineParser &parser) override
    {
        wxApp::OnInitCmdLine(parser);
        parser.AddParam("url", wxCMD_LINE_VAL_STRING, wxCMD_LINE_PARAM_OPTIONAL);
    }

    virtual bool OnInit()
    {
        // Call default initialization (e.g., parsing command line, settings)
        if (!wxApp::OnInit())
            return false;

        wxString forwardedUrl;
        for (int i = 1; i < argc; ++i)
        {
            wxString arg(argv[i]);
            if (arg.StartsWith("idr://"))
            {
                forwardedUrl = wxString(idr::utils::UrlDecode(arg.Mid(6).ToStdString()));
                break;
            }
        }

        if (idr::system::SingleInstanceGuard::AnotherInstanceIsRunning())
        {
            // Another InternetDownloader is already running: hand off the URL (if
            // any) to it and exit without opening a second window.
            if (!forwardedUrl.IsEmpty())
            {
                idr::system::SingleInstanceGuard::SendUrlToRunningInstance(forwardedUrl.ToStdString());
            }
            return false;
        }

        MainWindow *frame = new MainWindow("Internet Downloader");
        frame->Show(true);

        if (!forwardedUrl.IsEmpty())
        {
            frame->AddDownloadFromExternal(forwardedUrl);
        }

        wxWeakRef<MainWindow> weakFrame(frame);
        idr::system::SingleInstanceGuard::StartServer([weakFrame](std::string url)
                                                      {
                                                          if (!weakFrame)
                                                              return;
                                                          weakFrame->CallAfter([weakFrame, url]()
                                                                               {
                                                                                   if (weakFrame)
                                                                                       weakFrame->AddDownloadFromExternal(wxString(url));
                                                                               });
                                                      });

        return true;
    }
};

// Implement the application entry point
wxIMPLEMENT_APP(MyApp);
