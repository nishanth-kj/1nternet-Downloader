#include "gui/settings_dialog.h"

#include <wx/filepicker.h>
#include <wx/spinctrl.h>
#include <wx/statline.h>

SettingsDialog::SettingsDialog(wxWindow* parent, const idr::system::Settings& current)
    : wxDialog(parent, wxID_ANY, "Options", wxDefaultPosition, wxDefaultSize,
               wxDEFAULT_DIALOG_STYLE | wxRESIZE_BORDER),
      m_original(current)
{
    auto* grid = new wxFlexGridSizer(2, 8, 12);
    grid->AddGrowableCol(1, 1);

    auto addRow = [&](const wxString& label, wxWindow* control) {
        grid->Add(new wxStaticText(this, wxID_ANY, label), 0, wxALIGN_CENTER_VERTICAL);
        grid->Add(control, 1, wxEXPAND);
    };

    m_dirPicker = new wxDirPickerCtrl(this, wxID_ANY, wxString(current.downloadDir),
                                      "Choose default download folder", wxDefaultPosition,
                                      wxSize(360, -1), wxDIRP_USE_TEXTCTRL | wxDIRP_DIR_MUST_EXIST);
    addRow("Default download folder:", m_dirPicker);

    m_maxConcurrent = new wxSpinCtrl(this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize,
                                     wxSP_ARROW_KEYS, 1, 16, current.maxConcurrentDownloads);
    addRow("Simultaneous downloads:", m_maxConcurrent);

    m_maxSegments = new wxSpinCtrl(this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize,
                                   wxSP_ARROW_KEYS, 1, 32, current.maxSegments);
    addRow("Max connections per download:", m_maxSegments);

    m_maxRetries = new wxSpinCtrl(this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize,
                                  wxSP_ARROW_KEYS, 0, 20, current.maxRetries);
    addRow("Retries after a network error:", m_maxRetries);

    m_speedLimit = new wxSpinCtrl(this, wxID_ANY, wxEmptyString, wxDefaultPosition, wxDefaultSize,
                                  wxSP_ARROW_KEYS, 0, 1000000, static_cast<int>(current.speedLimitKBps));
    addRow("Speed limit per download (KiB/s, 0 = off):", m_speedLimit);

    auto* note = new wxStaticText(this, wxID_ANY,
        "Connection and speed settings apply to downloads started after you save.\n"
        "Paused segmented downloads keep their original connection count.");
    note->SetForegroundColour(wxSystemSettings::GetColour(wxSYS_COLOUR_GRAYTEXT));

    auto* root = new wxBoxSizer(wxVERTICAL);
    root->Add(grid, 0, wxEXPAND | wxALL, 14);
    root->Add(note, 0, wxLEFT | wxRIGHT | wxBOTTOM, 14);
    root->Add(new wxStaticLine(this), 0, wxEXPAND | wxLEFT | wxRIGHT, 14);
    root->Add(CreateButtonSizer(wxOK | wxCANCEL), 0, wxEXPAND | wxALL, 12);
    SetSizerAndFit(root);
    CentreOnParent();
}

idr::system::Settings SettingsDialog::GetSettings() const
{
    idr::system::Settings s = m_original;
    s.downloadDir = m_dirPicker->GetPath().ToStdString();
    s.maxConcurrentDownloads = m_maxConcurrent->GetValue();
    s.maxSegments = m_maxSegments->GetValue();
    s.maxRetries = m_maxRetries->GetValue();
    s.speedLimitKBps = m_speedLimit->GetValue();
    s.Normalize();
    return s;
}
