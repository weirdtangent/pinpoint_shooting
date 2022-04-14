package main

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gorilla/mux"
	"github.com/gorilla/securecookie"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"github.com/savaki/dynastore"

	"github.com/weirdtangent/myaws"
)

func main() {
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
	log.Logger = log.With().Caller().Logger()

	// grab config ---------------------------------------------------------------
	awsConfig, err := myaws.AWSConfig("us-east-1")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to find us-east-1 configuration")
	}

	// connect to AWS
	awssess, err := myaws.AWSConnect("us-east-1", "pinpoint")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to connect to AWS")
	}

	// connect to MySQL
	db, err := myaws.DBConnect(awssess, "pinpoint", "pinpoint")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to connect to MySQL")
	}
	_, err = db.Exec("SET NAMES utf8mb4 COLLATE utf8mb4_general_ci")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to switch RDS to UTF8")
	}

	// connect to Dynamo
	ddb, err := myaws.DDBConnect(awssess)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to connect to DDB")
	}

	var secrets = make(map[string]string)

	// Cookie setup for sessionID ------------------------------------------------
	cookieAuthKey, err := myaws.AWSGetSecretKV(awssess, "pinpoint", "cookie_auth_key")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to retrieve secret")
	}
	// The hashKey is required, used to authenticate the cookie value using HMAC. It is
	// recommended to use a key with 32 or 64 bytes.
	secrets["cookie_auth_key"] = *cookieAuthKey

	cookieEncryptionKey, err := myaws.AWSGetSecretKV(awssess, "pinpoint", "cookie_encryption_key")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to retrieve secret")
	}
	// The blockKey is optional, used to encrypt the cookie value -- set it to nil to
	// not use encryption. If set, the length must correspond to the block size of the
	// encryption algorithm. For AES, used by default, valid lengths are 16, 24, or 32
	// bytes to select AES-128, AES-192, or AES-256.
	secrets["cookie_encryption_key"] = *cookieEncryptionKey

	var hashKey = []byte(*cookieAuthKey)
	var blockKey = []byte(*cookieEncryptionKey)
	var secureCookie = securecookie.New(hashKey, blockKey)

	// Cache all other secrets into global map -----------------------------------

	// github OAuth key
	githubOAuthKey, err := myaws.AWSGetSecretKV(awssess, "pinpoint", "github_oauth_key")
	if err != nil {
		log.Fatal().Err(err).Msg("failed to retrieve secret")
	}
	secrets["github_oauth_key"] = *githubOAuthKey

	// get yahoofinance api access key and host
	google_maps_api_key, err := myaws.AWSGetSecretKV(awssess, "pinpoint", "google_maps_api_key")
	if err != nil {
		log.Fatal().Err(err).
			Msg("failed to get Google Maps API key")
	}
	secrets["google_maps_api_key"] = *google_maps_api_key

	// Initialize session manager and configure the session lifetime -------------
	store, err := dynastore.New(
		dynastore.AWSConfig(awsConfig),
		dynastore.DynamoDB(ddb),
		dynastore.TableName("pinpoint-session"),
		dynastore.Secure(),
		dynastore.HTTPOnly(),
		dynastore.Domain("www.pinpointshooting.com"),
		dynastore.Path("/"),
		dynastore.MaxAge(31*24*60*60),
		dynastore.Codecs(secureCookie),
	)
	if err != nil {
		log.Fatal().Err(err).Msg("failed to setup session management")
	}

	// auth api setup ---------------------------------------------------------

	// starting up web service ---------------------------------------------------
	log.Info().Int("port", 3000).Msg("Started serving requests")

	// setup middleware chain
	router := mux.NewRouter()

	router.PathPrefix("/static/").Handler(http.StripPrefix("/static/", http.FileServer(http.Dir("static/"))))
	router.PathPrefix("/favicon.ico").Handler(http.FileServer(http.Dir("static/images")))

	router.HandleFunc("/ping", pingHandler()).Methods("GET")
	router.HandleFunc("/internal/cspviolations", JSONReportHandler()).Methods("GET")
	router.Handle("/metrics", promhttp.Handler())

	router.HandleFunc("/about", homeHandler("about")).Methods("GET")
	router.HandleFunc("/terms", homeHandler("terms")).Methods("GET")
	router.HandleFunc("/privacy", homeHandler("privacy")).Methods("GET")
	router.HandleFunc("/", homeHandler("home")).Methods("GET")

	// middleware chain
	chainedMux1 := withSession(store, router) // deepest level, last to run
	chainedMux2 := withAddHeader(chainedMux1)
	chainedMux3 := withAddContext(chainedMux2, awssess, db, secureCookie, secrets)
	chainedMux4 := withLogging(chainedMux3) // outer level, first to run

	// starup or die
	server := &http.Server{
		Handler:      chainedMux4,
		Addr:         ":3000",
		WriteTimeout: 15 * time.Second,
		ReadTimeout:  15 * time.Second,
	}

	if err = server.ListenAndServe(); err != nil {
		log.Fatal().Err(err).
			Msg("Stopped serving requests")
	}
}
