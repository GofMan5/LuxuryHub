package server

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func newTestHandler(t *testing.T) http.Handler {
	t.Helper()
	return New(Options{
		Env:     "test",
		Version: "test-version",
		Commit:  "test-commit",
		BuiltAt: "2026-01-01T00:00:00Z",
		Started: time.Now(),
	})
}

func TestHealthzReportsOK(t *testing.T) {
	rec := httptest.NewRecorder()
	newTestHandler(t).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/healthz", nil))

	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusOK)
	}

	var body healthResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if body.Status != "ok" {
		t.Errorf("status = %q, want %q", body.Status, "ok")
	}
	if body.Env != "test" {
		t.Errorf("env = %q, want %q", body.Env, "test")
	}
}

func TestVersionEchoesBuildInfo(t *testing.T) {
	rec := httptest.NewRecorder()
	newTestHandler(t).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/version", nil))

	var body versionResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if body.Version != "test-version" || body.Commit != "test-commit" {
		t.Errorf("unexpected build info: %+v", body)
	}
	if body.Go == "" {
		t.Error("go version is empty")
	}
}

func TestUnknownRouteReturnsJSONError(t *testing.T) {
	rec := httptest.NewRecorder()
	newTestHandler(t).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/nope", nil))

	if rec.Code != http.StatusNotFound {
		t.Fatalf("status = %d, want %d", rec.Code, http.StatusNotFound)
	}

	var body errorResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &body); err != nil {
		t.Fatalf("decode body: %v", err)
	}
	if body.Code != "not_found" {
		t.Errorf("code = %q, want %q", body.Code, "not_found")
	}
}

func TestRequestIDIsAssignedAndEchoed(t *testing.T) {
	rec := httptest.NewRecorder()
	newTestHandler(t).ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/healthz", nil))

	if got := rec.Header().Get(headerRequestID); got == "" {
		t.Fatal("response has no request id")
	}
}

func TestCallerRequestIDIsKept(t *testing.T) {
	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	req.Header.Set(headerRequestID, "caller-id")

	rec := httptest.NewRecorder()
	newTestHandler(t).ServeHTTP(rec, req)

	if got := rec.Header().Get(headerRequestID); got != "caller-id" {
		t.Errorf("request id = %q, want %q", got, "caller-id")
	}
}
