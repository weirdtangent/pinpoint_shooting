package main

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/gomodule/redigo/redis"
	"github.com/gorilla/securecookie"
	"github.com/gorilla/sessions"
	"github.com/jmoiron/sqlx"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/savaki/dynastore"
)

// AddContext middleware ------------------------------------------------------
type AddContext struct {
	handler http.Handler
	awssess *session.Session
	db      *sqlx.DB
	sc      *securecookie.SecureCookie
	secrets map[string]string
}

type ContextKey string

func (ac *AddContext) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	reqHeader := r.Header
	resHeader := w.Header()

	// lets add request_id to this context and as a response header
	// but also as a cookie with a short expiration so we can catch
	// additional immediate requests with the same id
	var rid string
	ridCookie, err := r.Cookie("RID")
	if err == nil {
		rid = ridCookie.Value
	}
	if len(rid) == 0 {
		rid = reqHeader.Get("X-Request-ID")
	}
	ctx = context.WithValue(ctx, ContextKey("request_id"), rid)
	resHeader.Set("X-Request-ID", rid)

	ridCookie = &http.Cookie{
		Name:     "RID",
		Value:    rid,
		Path:     "/",
		Secure:   true,
		HttpOnly: true,
		Expires:  time.Now().Add(3 * time.Second),
	}
	http.SetCookie(w, ridCookie)

	// get the logger from the context and update it with the request_id
	logger := zerolog.Ctx(ctx)
	logger.UpdateContext(func(c zerolog.Context) zerolog.Context {
		return c.Str("request_id", rid)
	})

	messages := make([]Message, 0)

	// redis connection
	redisPool := &redis.Pool{
		MaxIdle:     10,
		IdleTimeout: 240 * time.Second,
		Dial: func() (redis.Conn, error) {
			return redis.Dial("tcp", "localhost:6379")
		},
	}

	defaultConfig := make(map[string]interface{})
	defaultConfig["quote_refresh"] = 20

	r = r.Clone(context.WithValue(r.Context(), ContextKey("awssess"), ac.awssess))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("db"), ac.db))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("sc"), ac.sc))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("redisPool"), redisPool))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("config"), defaultConfig))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("webdata"), make(map[string]interface{})))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("google_maps_api_key"), ac.secrets["google_maps_api_key"]))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("github_oauth_key"), ac.secrets["github_oauth_key"]))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("messages"), &messages))
	r = r.Clone(context.WithValue(r.Context(), ContextKey("nonce"), RandStringMask(32)))

	ac.handler.ServeHTTP(w, r)
}

func withAddContext(h http.Handler, awssess *session.Session, db *sqlx.DB, sc *securecookie.SecureCookie, secrets map[string]string) *AddContext {
	return &AddContext{h, awssess, db, sc, secrets}
}

// AddHeaders middleware ------------------------------------------------------

type AddHeader struct {
	handler http.Handler
}

func (ah *AddHeader) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	nonce := ctx.Value(ContextKey("nonce")).(string)

	resHeader := w.Header()
	csp := []string{
		"default-src 'self'",
		"connect-src 'self' accounts.google.com *.fontawesome.com *.google-analytics.com *.googleapis.com *.facebook.com",
		"style-src 'self' fonts.googleapis.com accounts.google.com 'unsafe-inline'",
		"script-src 'self' *.googletagmanager.com *.facebook.net apis.google.com *.googleapis.com accounts.google.com kit.fontawesome.com 'nonce-" + nonce + "'",
		"img-src * data:", // 'self' data: *.googleusercontent.com *.twimg.com avatars.githubusercontent.com assets.bwbx.io im.mstar.com im.morningstar.com mma.prnewswire.com",
		"font-src 'self' fonts.gstatic.com *.fontawesome.com",
		"frame-src 'self' accounts.google.com *.facebook.com",
		"report-uri /internal/cspviolations",
		"report-to default",
	}
	resHeader.Set("Content-Security-Policy", strings.Join(csp, "; "))

	reportTo := `{"group":"default","max-age":1800,"endpoints":[{"url":"https://stockwatch.graystorm.com/internal/cspviolations"}],"include_subdomains":true}`
	resHeader.Set("Report-To", reportTo)

	ah.handler.ServeHTTP(w, r)
}
func withAddHeader(h http.Handler) *AddHeader {
	return &AddHeader{h}
}

// Logging middleware ---------------------------------------------------------

type Logger struct {
	handler http.Handler
}

func (l *Logger) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t := time.Now()

	lg := log.With().Logger()
	// set request's context with l.WithContext which returns a copy of the context with the log object associated
	r = r.WithContext(lg.WithContext(r.Context()))

	l.handler.ServeHTTP(w, r)

	// we've been around the block, grab that logger back from the context to log with
	logger := log.Ctx(r.Context())

	// don't logs these, no reason to
	if r.URL.String() != "/ping" && r.URL.String() != "/metrics" {
		logger.Info().
			Stringer("url", r.URL).
			Int("status_code", 200).
			Str("method", r.Method).
			Int64("response_time", time.Since(t).Nanoseconds()).
			Msg("")
	}
}
func withLogging(h http.Handler) *Logger {
	return &Logger{h}
}

// Session management middleware ----------------------------------------------

type Session struct {
	store   *dynastore.Store
	handler http.Handler
}

func (s *Session) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	logger := log.Ctx(r.Context())
	session, err := s.store.Get(r, "SID")
	if err != nil {
		logger.Fatal().Err(err).Msg("Failed to get/create session")
	}
	if session.IsNew {
		state := RandStringMask(32)
		session.Values["state"] = state
		session.Values["recents"] = []string{}
		session.Values["theme"] = "light"
		err := session.Save(r, w)
		if err != nil {
			logger.Fatal().Err(err).Msg("Failed to save session")
		}
	}
	r = r.Clone(context.WithValue(r.Context(), ContextKey("ddbs"), session))

	defer session.Save(r, w)

	s.handler.ServeHTTP(w, r)
}
func withSession(store *dynastore.Store, h http.Handler) *Session {
	return &Session{store, h}
}

func getSession(r *http.Request) *sessions.Session {
	logger := log.Ctx(r.Context())
	session := r.Context().Value(ContextKey("ddbs")).(*sessions.Session)
	if session == nil {
		logger.Fatal().Err(errors.New("failed to get session from Context")).Msg("")
	}
	return session
}
