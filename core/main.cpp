/*
 * main.cpp — Internet Downloader entry point
 *
 * The core/ directory is its own self-contained C++ project.
 * This file initialises wxWidgets, bootstraps the GUI state, and
 * runs the main event loop.  All logic lives in core/src/.
 */

#include <wx/wx.h>
#include "gui/main_window.h"

// Define the application class
class MyApp : public wxApp
{
public:
    virtual bool OnInit()
    {
        // Call default initialization (e.g., parsing command line, settings)
        if (!wxApp::OnInit())
            return false;

        wxLogMessage("Starting Internet Downloader (wxWidgets)");

        MainWindow *frame = new MainWindow("Internet Downloader");
        frame->Show(true);

        return true;
    }
};

// Implement the application entry point
wxIMPLEMENT_APP(MyApp);
