#include "service/network/http.h"

#include "service/network/connection.h"

namespace idr {
namespace network {

namespace {

size_t HeaderCallback(char* buffer, size_t size, size_t nitems, void* userdata) {
    auto* resp = static_cast<HttpResponse*>(userdata);
    size_t totalBytes = size * nitems;
    resp->headers.AddFromRawLine(std::string(buffer, totalBytes));
    return totalBytes;
}

size_t WriteCallback(char* contents, size_t size, size_t nmemb, void* userdata) {
    auto* callback = static_cast<const DataCallback*>(userdata);
    size_t totalBytes = size * nmemb;
    if (!callback || !*callback) return totalBytes; // discard body (used for HEAD probes)
    return (*callback)(contents, totalBytes);
}

void FinishCommon(CURL* curl, HttpResponse& resp) {
    long httpCode = 0;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &httpCode);
    resp.statusCode = httpCode;

    curl_off_t contentLength = -1;
    curl_easy_getinfo(curl, CURLINFO_CONTENT_LENGTH_DOWNLOAD_T, &contentLength);
    resp.contentLength = static_cast<int64_t>(contentLength);

    char* finalUrl = nullptr;
    curl_easy_getinfo(curl, CURLINFO_EFFECTIVE_URL, &finalUrl);
    if (finalUrl) resp.finalUrl = finalUrl;

    std::string acceptRanges = resp.headers.Get("Accept-Ranges");
    resp.acceptRangesBytes = acceptRanges.find("bytes") != std::string::npos;
}

} // namespace

HttpResponse HttpClient::Head(const HttpRequest& req) {
    HttpResponse resp;

    HttpRequest headReq = req;
    headReq.headOnly = true;

    CurlHandle handle;
    if (!handle.IsValid()) {
        resp.errorMessage = "failed to initialize curl handle";
        return resp;
    }

    handle.ApplyRequest(headReq);
    curl_easy_setopt(handle.Get(), CURLOPT_HEADERFUNCTION, HeaderCallback);
    curl_easy_setopt(handle.Get(), CURLOPT_HEADERDATA, &resp);

    CURLcode res = curl_easy_perform(handle.Get());
    if (res != CURLE_OK) {
        resp.errorMessage = curl_easy_strerror(res);
        FinishCommon(handle.Get(), resp);
        return resp;
    }

    FinishCommon(handle.Get(), resp);
    resp.ok = (resp.statusCode >= 200 && resp.statusCode < 400);
    if (!resp.ok && resp.errorMessage.empty()) {
        resp.errorMessage = "HTTP error " + std::to_string(resp.statusCode);
    }
    return resp;
}

HttpResponse HttpClient::Get(const HttpRequest& req, const DataCallback& onData) {
    HttpResponse resp;

    CurlHandle handle;
    if (!handle.IsValid()) {
        resp.errorMessage = "failed to initialize curl handle";
        return resp;
    }

    handle.ApplyRequest(req);
    curl_easy_setopt(handle.Get(), CURLOPT_HEADERFUNCTION, HeaderCallback);
    curl_easy_setopt(handle.Get(), CURLOPT_HEADERDATA, &resp);
    curl_easy_setopt(handle.Get(), CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(handle.Get(), CURLOPT_WRITEDATA, &onData);

    CURLcode res = curl_easy_perform(handle.Get());
    if (res != CURLE_OK) {
        resp.errorMessage = curl_easy_strerror(res);
        FinishCommon(handle.Get(), resp);
        return resp;
    }

    FinishCommon(handle.Get(), resp);
    resp.ok = (resp.statusCode >= 200 && resp.statusCode < 400);
    if (!resp.ok && resp.errorMessage.empty()) {
        resp.errorMessage = "HTTP error " + std::to_string(resp.statusCode);
    }
    return resp;
}

} // namespace network
} // namespace idr
