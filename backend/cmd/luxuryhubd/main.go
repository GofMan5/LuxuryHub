// Command luxuryhubd is the LuxuryHub backend.
//
// It is a single binary with one job for now: serve the HTTP API. Worker and
// migration modes will be added as subcommands when there is something to run.
package main

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/gofman5/luxuryhub/backend/internal/server"
)

const (
	serviceName = "luxuryhubd"
	defaultAddr = "127.0.0.1:8080"

	readHeaderTimeout = 5 * time.Second
	readTimeout       = 15 * time.Second
	writeTimeout      = 30 * time.Second
	idleTimeout       = 90 * time.Second
	shutdownBudget    = 10 * time.Second
)

// Set with -ldflags: -X main.version=... -X main.commit=...
var (
	version = "dev"
	commit  = "none"
	builtAt = "unknown"
)

func main() {
	log := newLogger()
	if err := run(log); err != nil {
		log.Error("server stopped with error", "error", err)
		os.Exit(1)
	}
}

func run(log *slog.Logger) error {
	addr := env("LUXURYHUB_ADDR", defaultAddr)
	envName := env("LUXURYHUB_ENV", "dev")

	handler := server.New(server.Options{
		Log:     log,
		Env:     envName,
		Version: version,
		Commit:  commit,
		BuiltAt: builtAt,
		Started: time.Now(),
	})

	srv := &http.Server{
		Addr:              addr,
		Handler:           handler,
		ReadHeaderTimeout: readHeaderTimeout,
		ReadTimeout:       readTimeout,
		WriteTimeout:      writeTimeout,
		IdleTimeout:       idleTimeout,
		ErrorLog:          slog.NewLogLogger(log.Handler(), slog.LevelError),
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	errCh := make(chan error, 1)
	go func() {
		log.Info("http server listening",
			"service", serviceName,
			"addr", addr,
			"env", envName,
			"version", version,
		)
		if err := srv.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			errCh <- err
			return
		}
		errCh <- nil
	}()

	select {
	case err := <-errCh:
		return err
	case <-ctx.Done():
		log.Info("shutdown signal received")
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownBudget)
	defer cancel()

	if err := srv.Shutdown(shutdownCtx); err != nil {
		return err
	}

	log.Info("server stopped")
	return nil
}

// newLogger builds a structured logger; the level comes from LUXURYHUB_LOG_LEVEL.
func newLogger() *slog.Logger {
	var level slog.Level
	if err := level.UnmarshalText([]byte(env("LUXURYHUB_LOG_LEVEL", "info"))); err != nil {
		level = slog.LevelInfo
	}

	log := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})).
		With("service", serviceName, "commit", commit)

	slog.SetDefault(log)
	return log
}

// env returns the environment variable or a fallback when it is unset or empty.
func env(key, fallback string) string {
	if value := strings.TrimSpace(os.Getenv(key)); value != "" {
		return value
	}
	return fallback
}
