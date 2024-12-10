package main

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/dynamodb"
	"github.com/gomodule/redigo/redis"
	"github.com/gorilla/mux"
	"github.com/gorilla/securecookie"
	"github.com/gorilla/sessions"
	"github.com/jmoiron/sqlx"
	"github.com/markbates/goth"
	"github.com/markbates/goth/gothic"
	"github.com/markbates/goth/providers/amazon"
	"github.com/markbates/goth/providers/google"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/savaki/dynastore"
	"github.com/weirdtangent/myaws"
)

type Dependencies struct {
	logger       *zerolog.Logger
	awsConfig    *aws.Config
	awsSess      *session.Session
	db           *sqlx.DB
	ddb          *dynamodb.DynamoDB
	secrets      map[string]string
	secureCookie *securecookie.SecureCookie
	cookieStore  *dynastore.Store
	redisPool    *redis.Pool
	session      *sessions.Session
	request_id   string
	nonce        string
	webdata      map[string]interface{}
	config       map[string]interface{}
	messages     []Message
}

func setup_logging(deps *Dependencies) {
	// setup logging -------------------------------------------------------------
	zerolog.SetGlobalLevel(zerolog.InfoLevel)
	// alter the caller() return to only include the last directory
	zerolog.CallerMarshalFunc = func(file string, line int) string {
		parts := strings.Split(file, "/")
		if len(parts) > 1 {
			return strings.Join(parts[len(parts)-2:], "/") + ":" + strconv.Itoa(line)
		}
		return file + ":" + strconv.Itoa(line)
	}
	if debugging {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
	}
	logger := log.With().Str("@tag", "pinpoint-shooting").Caller().Logger()

	// initial server-level logger, which will get replaced on a per-request basis
	deps.logger = &logger
}

func setup_aws(deps *Dependencies) {
	sublog := deps.logger

	var err error
	deps.awsConfig, err = myaws.AWSConfig("us-east-1")
	if err != nil {
		sublog.Fatal().Err(err).Msg("failed to get aws config")
	}
	deps.awsSess = myaws.AWSMustConnect("us-east-1", "pinpoint")
	deps.db = myaws.DBMustConnect(deps.awsSess, "pinpoint")
	deps.ddb, err = myaws.DDBConnect(deps.awsSess)
	if err != nil {
		sublog.Fatal().Err(err).Msg("failed to connect to dynamodb")
	}
}

func setup_secrets(deps *Dependencies) {
	awssess := deps.awsSess
	sublog := deps.logger

	secretValues, err := myaws.AWSGetSecret(awssess, "pinpoint")
	if err != nil {
		sublog.Fatal().Err(err)
	}

	var secrets = make(map[string]string)
	for key := range secretValues {
		value := secretValues[key]
		secrets[key] = value
	}

	deps.secrets = secrets
	sublog.Debug().Int("secrets", len(deps.secrets)).Msg("secret loaded")
}

func setup_sessions(deps *Dependencies) {
	sublog := deps.logger
	secrets := deps.secrets

	// The hashKey is required, used to authenticate the cookie value using HMAC. It is
	// recommended to use a key with 32 or 64 bytes.
	var hashKey = []byte(secrets["cookie_auth_key"])

	// The blockKey is optional, used to encrypt the cookie value -- set it to nil to
	// not use encryption. If set, the length must correspond to the block size of the
	// encryption algorithm. For AES, used by default, valid lengths are 16, 24, or 32
	// bytes to select AES-128, AES-192, or AES-256.
	var blockKey = []byte(secrets["cookie_encryption_key"])

	deps.secureCookie = securecookie.New(hashKey, blockKey)

	// Initialize session manager and configure the session lifetime -------------
	store, err := dynastore.New(
		dynastore.AWSConfig(deps.awsConfig),
		dynastore.DynamoDB(deps.ddb),
		dynastore.TableName("pinpoint-session"),
		dynastore.Secure(),
		dynastore.HTTPOnly(),
		dynastore.Domain("pinpointshooting.com"),
		dynastore.Path("/"),
		dynastore.MaxAge(31*24*60*60),
		dynastore.Codecs(deps.secureCookie),
	)
	sublog.Debug().Interface("cookieStore", store).Msg("cookieStore")
	if err != nil || store == nil {
		sublog.Fatal().Err(err).Msg("failed to setup session management")
	}
	deps.cookieStore = store
}

func setup_oauth(deps *Dependencies) {
	cookieStore := deps.cookieStore
	secrets := deps.secrets

	goth.UseProviders(
		amazon.New(secrets["amazon_api_key"], secrets["amazon_api_secret"], "https://pinpointshooting.com/auth/amazon/callback"),
		// facebook.New(secrets["facebook_api_key"], secrets["facebook_api_secret"], "https://pinpointshooting.com/auth/facebook/callback", "email"),
		// github.New(secrets["github_api_key"], secrets["github_api_secret"], "https://pinpointshooting.com/auth/github/callback"),
		google.New(secrets["google_oauth_client_id"], secrets["google_oauth_client_secret"], "https://pinpointshooting.com/auth/google/callback", "openid https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile"),
		// twitter.New(secrets["twitter_api_key"], secrets["twitter_api_secret"], "https://pinpointshooting.com/auth/twitter/callback"),
	)

	gothic.Store = cookieStore
}

func start_server(deps *Dependencies) {
	app := requestHandler{deps: deps}
	sublog := deps.logger

	// starting up web service ---------------------------------------------------
	sublog.Info().Int("port", httpPort).Msg("started serving requests")

	// setup middleware chain
	router := mux.NewRouter()

	router.PathPrefix("/static/").Handler(http.StripPrefix("/static/", http.FileServer(http.Dir("static/"))))
	router.PathPrefix("/favicon.ico").Handler(http.FileServer(http.Dir("static/images")))

	router.HandleFunc("/auth/{provider}", app.requestHandler(authLoginHandler(deps))).Methods("GET")
	router.HandleFunc("/auth/{provider}/callback", app.requestHandler(authCallbackHandler(deps))).Methods("GET")
	router.HandleFunc("/signout/", app.requestHandler(signoutHandler(deps))).Methods("GET")
	router.HandleFunc("/signout/{provider}", app.requestHandler(signoutHandler(deps))).Methods("GET")
	router.HandleFunc("/logout/", app.requestHandler(signoutHandler(deps))).Methods("GET")
	router.HandleFunc("/logout/{provider}", app.requestHandler(signoutHandler(deps))).Methods("GET")

	router.HandleFunc("/ping", pingHandler()).Methods("GET")
	router.HandleFunc("/internal/cspviolations", app.requestHandler(JSONReportHandler(deps))).Methods("GET")
	router.Handle("/metrics", promhttp.Handler())

	router.HandleFunc("/about", homeHandler(deps, "about")).Methods("GET")
	router.HandleFunc("/terms", homeHandler(deps, "terms")).Methods("GET")
	router.HandleFunc("/privacy", homeHandler(deps, "privacy")).Methods("GET")
	router.HandleFunc("/", app.requestHandler(homeHandler(deps, "home"))).Methods("GET")

	// starup or die
	server := &http.Server{
		Handler:      router,
		Addr:         fmt.Sprintf(":%d", httpPort),
		WriteTimeout: 15 * time.Second,
		ReadTimeout:  15 * time.Second,
	}

	if err := server.ListenAndServe(); err != nil {
		sublog.Fatal().Err(err).Msg("stopped serving requests")
	}
}
