#pragma once

#include <string>
#include <cstdint>
#include <chrono>

namespace idr
{
    namespace model
    {

        enum class DownloadStatus
        {
            Queued,
            Downloading,
            Paused,
            Completed,
            Error,
            Cancelled
        };

        inline std::string StatusToString(DownloadStatus status)
        {
            switch (status)
            {
            case DownloadStatus::Queued:
                return "Queued";
            case DownloadStatus::Downloading:
                return "Downloading";
            case DownloadStatus::Paused:
                return "Paused";
            case DownloadStatus::Completed:
                return "Completed";
            case DownloadStatus::Error:
                return "Error";
            case DownloadStatus::Cancelled:
                return "Cancelled";
            default:
                return "Unknown";
            }
        }

        inline DownloadStatus StatusFromString(const std::string &s)
        {
            if (s == "Downloading")
                return DownloadStatus::Downloading;
            if (s == "Paused")
                return DownloadStatus::Paused;
            if (s == "Completed")
                return DownloadStatus::Completed;
            if (s == "Error")
                return DownloadStatus::Error;
            if (s == "Cancelled")
                return DownloadStatus::Cancelled;
            return DownloadStatus::Queued;
        }

        /**
         * DownloadItem - plain data model representing a single download.
         * Shared between repository (persistence) and service (business logic).
         */
        struct DownloadItem
        {
            int64_t id = 0; // DB primary key (0 = not yet persisted)
            std::string url;
            std::string destination; // full filesystem path
            std::string filename;
            uint64_t totalBytes = 0;
            uint64_t downloadedBytes = 0;
            DownloadStatus status = DownloadStatus::Queued;
            std::string mimeType;
            std::string checksum; // SHA-256 once completed
            std::chrono::system_clock::time_point createdAt;
            std::chrono::system_clock::time_point updatedAt;
        };

    } // namespace model
} // namespace idr
