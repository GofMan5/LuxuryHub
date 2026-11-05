// Package server wires HTTP routes and middleware for the LuxuryHub API.
//
// Everything transport-related lives here: path routing, status codes, request
// logging, panic recovery. Business rules belong to future domain packages and
// must not leak into this layer.
package server

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"runtime"
	"time"
)

// Options configures the HTTP handler.
type Options struct {
	Log     *slog.Logger
	Env     string
	Version string
	Commit  string
	BuiltAt string
	Started time.Time
}

// New returns the root HTTP handler.
func New(opts Options) http.Handler {
	if opts.Log == nil {
		opts.Log = slog.Default()
	}
	if opts.Started.IsZero() {
		opts.Started = time.Now()
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /healthz", opts.handleHealth)
	mux.HandleFunc("GET /version", opts.handleVersion)
	mux.HandleFunc("/", handleNotFound)

	return withRecovery(opts.Log, withRequestLog(opts.Log, mux))
}

type healthResponse struct {
	Status    string `json:"status"`
	Env       string `json:"env"`
	Version   string `json:"version"`
	UptimeSec int64  `json:"uptime_sec"`
}

func (o Options) handleHealth(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, healthResponse{
		Status:    "ok",
		Env:       o.Env,
		Version:   o.Version,
		UptimeSec: int64(time.Since(o.Started).Seconds()),
	})
}

type versionResponse struct {
	Service string `json:"service"`
	Version string `json:"version"`
	Commit  string `json:"commit"`
	BuiltAt string `json:"built_at"`
	Go      string `json:"go"`
}

func (o Options) handleVersion(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, versionResponse{
		Service: "luxuryhubd",
		Version: o.Version,
		Commit:  o.Commit,
		BuiltAt: o.BuiltAt,
		Go:      runtime.Version(),
	})
}

type errorResponse struct {
	Code      string `json:"code"`
	Message   string `json:"message"`
	RequestID string `json:"request_id,omitempty"`
}

func handleNotFound(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusNotFound, errorResponse{
		Code:      "not_found",
		Message:   "no route for " + r.Method + " " + r.URL.Path,
		RequestID: requestID(r),
	})
}

func writeJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(body)
}
