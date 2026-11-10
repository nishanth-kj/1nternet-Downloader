#pragma once

#include <wx/wx.h>

#include "utils/system/config.h"

class wxDirPickerCtrl;
class wxSpinCtrl;

// Tools > Options: edits idr::system::Settings. Call GetSettings() after
// ShowModal() returns wxID_OK.
class SettingsDialog : public wxDialog
{
public:
    SettingsDialog(wxWindow* parent, const idr::system::Settings& current);

    idr::system::Settings GetSettings() const;

private:
    idr::system::Settings m_original;
    wxDirPickerCtrl* m_dirPicker{nullptr};
    wxSpinCtrl* m_maxConcurrent{nullptr};
    wxSpinCtrl* m_maxSegments{nullptr};
    wxSpinCtrl* m_maxRetries{nullptr};
    wxSpinCtrl* m_speedLimit{nullptr};
};
