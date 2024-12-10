package main

import (
	"fmt"
	"net/http"
	"regexp"
	"strings"
	"time"

	"github.com/gomodule/redigo/redis"
	"github.com/rs/zerolog/log"
)

var (
	forwardedRE      = regexp.MustCompile(`for=(.*)`)
	skipLoggingPaths = regexp.MustCompile(`^/(ping|metrics|static|favicon.ico)`)
	obfuscateParams  = regexp.MustCompile(`(token|verifier|pwd|password|code|state)=([^\&]+)`)
)

type StatusRecorder struct {
	http.ResponseWriter
	Status int
}

func (r *StatusRecorder) WriteHeader(status int) {
	r.Status = status
	r.ResponseWriter.WriteHeader(status)
}

// requestHandler middleware --------------------------------------------------

type requestHandler struct {
	handler http.Handler
	deps    *Dependencies
}

func (rh requestHandler) requestHandler(h http.HandlerFunc) http.HandlerFunc {
	rh.handler = h
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		cookieStore := rh.deps.cookieStore
		sublog := log.With().Str("@tag", "pinpoint-shooting").Caller().Logger()

		sublog.Debug().Interface("cookieStore", rh.deps.cookieStore).Msg("middleware: cookieStore")
		sublog.Debug().Int("secrets", len(rh.deps.secrets)).Msg("middleware: secret count")

		resHeader := w.Header()
		reqHeader := r.Header

		// start the timer
		start := time.Now()

		recorder := &StatusRecorder{ResponseWriter: w, Status: 200}

		// session
		session, err := cookieStore.Get(r, "SID")
		if err != nil {
			sublog.Fatal().Err(err).Msg("failed to get/create session")
		}
		if session.IsNew {
			state := RandStringMask(32)
			session.Values["state"] = state
			session.Values["recents"] = []string{}
			session.Values["theme"] = "light"
			err := session.Save(r, w)
			if err != nil {
				sublog.Fatal().Err(err).Msg("failed to save session")
			}
		}
		rh.deps.session = session
		defer session.Save(r, w)

		// per-request setup
		rh.deps.webdata = make(map[string]interface{})
		rh.deps.config = make(map[string]interface{})
		rh.deps.nonce = RandStringMask(32)

		// Content Security Policy
		csp := map[string][]string{
			"base-uri":    {"'self'"},
			"default-src": {"'self'"},
			"connect-src": {"'self'", "accounts.google.com", "*.googleapis.com", "www.google-analytics.com", "*.fontawesome.com", "api.amazon.com", "*.facebook.com"},
			"style-src":   {"'self'", "fonts.googleapis.com", "accounts.google.com", "'unsafe-inline'"},
			"script-src":  {"'self'", "apis.google.com", "*.googleapis.com", "www.googletagmanager.com", "accounts.google.com", "kit.fontawesome.com", "assets.loginwithamazon.com", "*.facebook.net", "'nonce-" + rh.deps.nonce + "'"},
			"img-src":     {"'self'", "data:", "maps.gstatic.com", "*.googleapis.com", "*.googleusercontent.com", "*.twimg.com", "avatars.githubusercontent.com"},
			"font-src":    {"'self'", "fonts.gstatic.com", "*.fontawesome.com"},
			"frame-src":   {"'self'", "accounts.google.com", "*.amazon.com", "*.facebook.com"},
			"object-src":  {"'none'"},
			"report-uri":  {"/internal/cspviolations"},
			"report-to":   {"default"},
		}
		cspString := ""
		for category := range csp {
			cspString += fmt.Sprintf("%s %s;\n", category, strings.Join(csp[category], " "))
		}
		resHeader.Set("Content-Security-Policy", cspString)
		resHeader.Set("X-Nonce", rh.deps.nonce)

		reportTo := `{"group":"default","max-age":1800,"endpoints":[{"url":"https://pinpoint-shooting.com/internal/cspviolations"}],"include_subdomains":true}`
		resHeader.Set("Report-To", reportTo)

		// RequestID
		var rid string
		ridCookie, err := r.Cookie("RID")
		if err == nil {
			rid = ridCookie.Value
		}
		if len(rid) == 0 {
			rid = reqHeader.Get("X-Request-ID")
		}
		resHeader.Set("X-Request-ID", rid)
		rh.deps.request_id = rid

		ridCookie = &http.Cookie{
			Name:     "RID",
			Value:    rid,
			Path:     "/",
			Secure:   true,
			HttpOnly: true,
			Expires:  time.Now().Add(3 * time.Second),
		}
		http.SetCookie(w, ridCookie)

		sublog = sublog.With().Str("request_id", rid).Logger()
		rh.deps.logger = &sublog

		// redis connection
		rh.deps.redisPool = &redis.Pool{
			MaxIdle:     10,
			IdleTimeout: 240 * time.Second,
			Dial: func() (redis.Conn, error) {
				return redis.Dial("tcp", "localhost:6379")
			},
		}

		rh.handler.ServeHTTP(w, r)

		// we've been around the block, log the request/time-to-respond

		// don't logs these, no reason to
		if !skipLoggingPaths.MatchString(r.URL.String()) {
			ForwardedHdrs := r.Header["Forwarded"]
			remote_ip_addr := ""
			if len(ForwardedHdrs) > 0 {
				submatches := forwardedRE.FindStringSubmatch(ForwardedHdrs[0])
				if len(submatches) >= 1 {
					remote_ip_addr = submatches[1]
				}
			}

			cleanURL := r.URL.String()
			cleanURL = obfuscateParams.ReplaceAllString(cleanURL, "$1=xxxxxx")

			sublog.Info().
				Str("method", r.Method).
				Str("url", cleanURL).
				Int("status_code", recorder.Status).
				Str("remote_ip_addr", remote_ip_addr).
				Int64("response_time", time.Since(start).Nanoseconds()).
				Msg("")
		}
	})
}
