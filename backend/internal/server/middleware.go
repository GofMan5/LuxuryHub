package server

import (
	"crypto/rand"
	"encoding/hex"
	"log/slog"
	"net/http"
	"time"
)

// headerRequestID is the request correlation header shared with clients.
const headerRequestID = "X-Request-Id"

// maxLoggedPath limits how much of a client-supplied path reaches the log.
const maxLoggedPath = 512

// requestID returns the caller-provided correlation id or an empty string.
func requestID(r *http.Request) string {
	return r.Header.Get(headerRequestID)
}

// newRequestID returns a random 128-bit identifier in hex form.
func newRequestID() string {
	var buf [16]byte
	if _, err := rand.Read(buf[:]); err != nil {
		// crypto/rand does not fail in practice; a timestamp keeps ids unique
		// enough to correlate a single request rather than panicking.
		return hex.EncodeToString([]byte(time.Now().Format(time.RFC3339Nano)))
	}
	return hex.EncodeToString(buf[:])
}

func truncate(s string) string {
	if len(s) <= maxLoggedPath {
		return s
	}
	return s[:maxLoggedPath] + "..."
}

// withRequestLog assigns a correlation id, echoes it in the response and logs
// one line per request after it completes.
func withRequestLog(log *slog.Logger, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id := requestID(r)
		if id == "" {
			id = newRequestID()
			r.Header.Set(headerRequestID, id)
		}
		w.Header().Set(headerRequestID, id)

		recorder := &statusRecorder{ResponseWriter: w, status: http.StatusOK}
		started := time.Now()

		next.ServeHTTP(recorder, r)

		log.Info("http request",
			"request_id", id,
			"method", r.Method,
			"path", truncate(r.URL.Path),
			"status", recorder.status,
			"duration_ms", time.Since(started).Milliseconds(),
		)
	})
}

// withRecovery turns a panicking handler into a 500 instead of a dead process.
func withRecovery(log *slog.Logger, next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if recovered := recover(); recovered != nil {
				log.Error("panic in handler",
					"request_id", requestID(r),
					"path", truncate(r.URL.Path),
					"panic", recovered,
				)
				writeJSON(w, http.StatusInternalServerError, errorResponse{
					Code:      "internal",
					Message:   "internal error",
					RequestID: requestID(r),
				})
			}
		}()

		next.ServeHTTP(w, r)
	})
}

// statusRecorder remembers the status code for the access log.
type statusRecorder struct {
	http.ResponseWriter
	status      int
	wroteHeader bool
}

func (s *statusRecorder) WriteHeader(status int) {
	if s.wroteHeader {
		return
	}
	s.status = status
	s.wroteHeader = true
	s.ResponseWriter.WriteHeader(status)
}

func (s *statusRecorder) Write(body []byte) (int, error) {
	if !s.wroteHeader {
		s.WriteHeader(http.StatusOK)
	}
	return s.ResponseWriter.Write(body)
}
