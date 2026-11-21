use crate::define_code_value_enum;

define_code_value_enum! {
    /// Standard HTTP Status code and canonical message pairs
    pub enum HttpStatusCode(u16, &'static str) {
        Ok(200, "OK"),
        PartialContent(206, "Partial Content"),
        MovedPermanently(301, "Moved Permanently"),
        Found(302, "Found"),
        NotModified(304, "Not Modified"),
        BadRequest(400, "Bad Request"),
        Unauthorized(401, "Unauthorized"),
        Forbidden(403, "Forbidden"),
        NotFound(404, "Not Found"),
        RangeNotSatisfiable(416, "Range Not Satisfiable"),
        InternalServerError(500, "Internal Server Error"),
        BadGateway(502, "Bad Gateway"),
        ServiceUnavailable(503, "Service Unavailable"),
        GatewayTimeout(504, "Gateway Timeout"),
    }
}
