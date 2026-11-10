#include "service/network/http.h"

#include "service/network/connection.h"

namespace idr
{
    namespace network
    {

        namespace
        {

            struct TransferContext
            {
                CURL *curl{nullptr};
                HttpResponse *resp{nullptr};
                const DataCallback *onData{nullptr};
                const ResponseValidator *validate{nullptr};
                bool validated{false};
                bool rejected{false};
            };

            size_t HeaderCallback(char *buffer, size_t size, size_t nitems, void *userdata)
            {
                auto *resp = static_cast<HttpResponse *>(userdata);
                size_t totalBytes = size * nitems;
                std::string line(buffer, totalBytes);
                // Each response in a redirect chain starts with a status line; only
                // the final response's headers should survive.
                if (line.rfind("HTTP/", 0) == 0)
                {
                    resp->headers.Clear();
                    return totalBytes;
                }
                resp->headers.AddFromRawLine(line);
                return totalBytes;
            }

            size_t WriteCallback(char *contents, size_t size, size_t nmemb, void *userdata)
            {
                auto *ctx = static_cast<TransferContext *>(userdata);
                size_t totalBytes = size * nmemb;

                if (!ctx->validated)
                {
                    ctx->validated = true;
                    if (ctx->validate && *ctx->validate)
                    {
                        long code = 0;
                        curl_easy_getinfo(ctx->curl, CURLINFO_RESPONSE_CODE, &code);
                        if (!(*ctx->validate)(code, ctx->resp->headers))
                        {
                            ctx->rejected = true;
                            return 0; // abort before any byte is written
                        }
                    }
                }

                if (!ctx->onData || !*ctx->onData)
                    return totalBytes; // discard body (used for HEAD probes)
                return (*ctx->onData)(contents, totalBytes);
            }

            void FinishCommon(CURL *curl, HttpResponse &resp)
            {
                long httpCode = 0;
                curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &httpCode);
                resp.statusCode = httpCode;

                curl_off_t contentLength = -1;
                curl_easy_getinfo(curl, CURLINFO_CONTENT_LENGTH_DOWNLOAD_T, &contentLength);
                resp.contentLength = static_cast<int64_t>(contentLength);

                char *finalUrl = nullptr;
                curl_easy_getinfo(curl, CURLINFO_EFFECTIVE_URL, &finalUrl);
                if (finalUrl)
                    resp.finalUrl = finalUrl;

                std::string acceptRanges = resp.headers.Get("Accept-Ranges");
                resp.acceptRangesBytes = acceptRanges.find("bytes") != std::string::npos;
            }

            HttpResponse Perform(const HttpRequest &req, const DataCallback *onData,
                                 const ResponseValidator *validate)
            {
                HttpResponse resp;

                CurlHandle handle;
                if (!handle.IsValid())
                {
                    resp.errorMessage = "failed to initialize curl handle";
                    return resp;
                }

                TransferContext ctx;
                ctx.curl = handle.Get();
                ctx.resp = &resp;
                ctx.onData = onData;
                ctx.validate = validate;

                handle.ApplyRequest(req);
                curl_easy_setopt(handle.Get(), CURLOPT_HEADERFUNCTION, HeaderCallback);
                curl_easy_setopt(handle.Get(), CURLOPT_HEADERDATA, &resp);
                curl_easy_setopt(handle.Get(), CURLOPT_WRITEFUNCTION, WriteCallback);
                curl_easy_setopt(handle.Get(), CURLOPT_WRITEDATA, &ctx);

                CURLcode res = curl_easy_perform(handle.Get());
                FinishCommon(handle.Get(), resp);
                resp.rejectedByValidator = ctx.rejected;

                if (ctx.rejected)
                {
                    resp.errorMessage = "unexpected response (HTTP " + std::to_string(resp.statusCode) + ")";
                    return resp;
                }
                if (res != CURLE_OK)
                {
                    resp.errorMessage = curl_easy_strerror(res);
                    return resp;
                }

                resp.ok = (resp.statusCode >= 200 && resp.statusCode < 300);
                if (!resp.ok && resp.errorMessage.empty())
                {
                    resp.errorMessage = "HTTP error " + std::to_string(resp.statusCode);
                }
                return resp;
            }

        } // namespace

        HttpResponse HttpClient::Head(const HttpRequest &req)
        {
            HttpRequest headReq = req;
            headReq.headOnly = true;
            return Perform(headReq, nullptr, nullptr);
        }

        HttpResponse HttpClient::Get(const HttpRequest &req, const DataCallback &onData,
                                     const ResponseValidator &validate)
        {
            return Perform(req, &onData, &validate);
        }

    } // namespace network
} // namespace idr
