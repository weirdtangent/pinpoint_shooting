package main

import (
	"fmt"
	"math/rand"
	"net/http"
	"strconv"
	"time"

	"github.com/dgryski/go-skip32"
	"github.com/markbates/goth"
	"github.com/markbates/goth/gothic"
)

// random string of bytes, use in nonce values, for example
//   https://stackoverflow.com/questions/22892120/how-to-generate-a-random-string-of-a-fixed-length-in-go
const letterBytes = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
const (
	letterIdxBits = 6                    // 6 bits to represent a letter index
	letterIdxMask = 1<<letterIdxBits - 1 // All 1-bits, as many as letterIdxBits
	letterIdxMax  = 63 / letterIdxBits   // # of letter indices fitting in 63 bits
)

func authLoginHandler(deps *Dependencies) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sublog := deps.logger

		if user, err := gothic.CompleteUserAuth(w, r); err == nil {
			signinUser(deps, w, r, user)
		} else {
			sublog.Warn().Err(err).Msg("prior session check")
			gothic.BeginAuthHandler(w, r)
		}
	})
}

func authCallbackHandler(deps *Dependencies) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sublog := deps.logger

		user, err := gothic.CompleteUserAuth(w, r)
		if err != nil {
			sublog.Error().Err(err).Msg("user auth error on callback")
			return
		}
		signinUser(deps, w, r, user)
	})
}

func signinUser(deps *Dependencies, w http.ResponseWriter, r *http.Request, gothUser goth.User) {
	sublog := deps.logger
	sc := deps.secureCookie

	// get (or create) watcher account based on oauth properties
	// specifically, based on the oauth_sub value, because email addresses can change and we
	// want a user's session and "pinpoint account" to stay connected even if they change

	user := User{
		Id:        0,
		OAuthSub:  gothUser.UserID,
		Name:      gothUser.Name,
		Nickname:  gothUser.NickName,
		Status:    "active",
		Level:     "standard",
		Timezone:  "America/New_York",
		Location:  gothUser.Location,
		AvatarUrl: gothUser.AvatarURL,
		SessionId: deps.session.ID,
		Created:   time.Now(),
		Updated:   time.Now(),
	}
	user, err := createOrUpdateUserFromOAuth(deps, user, gothUser.Email)
	if err != nil {
		sublog.Error().Err(err).Msg("failed to create or update user from oauth")
		deps.messages = append(deps.messages, Message{"Sorry, the attempt to oauth failed", "fatal"})
		http.Redirect(w, r, "/", http.StatusFound)
	}

	// why does twitter send back a weird gothUser.ExpiresAt?
	if gothUser.ExpiresAt.IsZero() {
		gothUser.ExpiresAt = time.Now().Add(24 * time.Hour)
	}

	oauth := OAuth{
		Id:       0,
		Provider: gothUser.Provider,
		Sub:      gothUser.UserID,
		Issued:   time.Now(),
		Expires:  gothUser.ExpiresAt,
		Created:  time.Now(),
		Updated:  time.Now(),
	}
	sublog.Debug().Interface("oauth", oauth).Msg("oauth")

	// err = oauth.createOrUpdate(deps)

	// createOrUpdateUserFromOAuth(deps, *user, gothUser.Email)

	// set AID session cookie, meaning the user is authenticated and logged-in
	if encoded, err := sc.Encode("AID", fmt.Sprintf("%d", 1)); err == nil {
		widCookie := &http.Cookie{
			Name:     "AID",
			Value:    encoded,
			Path:     "/",
			Secure:   true,
			HttpOnly: true,
			SameSite: http.SameSiteStrictMode,
		}
		http.SetCookie(w, widCookie)
		newlog := sublog.With().Str("auth_user", encryptId(deps, "user", 1)).Logger()
		deps.logger = &newlog
	} else {
		sublog.Error().Err(err).Msg("Failed to encode cookie")
	}
	http.Redirect(w, r, "/desktop", http.StatusFound)
}

func signoutHandler(deps *Dependencies) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		deleteSessionCookie(w, r, deps)
		gothic.Logout(w, r)
		http.Redirect(w, r, "/", http.StatusFound)
	})
}

func deleteSessionCookie(w http.ResponseWriter, r *http.Request, deps *Dependencies) {
	sc := deps.secureCookie

	if encoded, err := sc.Encode("AID", "invalid"); err == nil {
		cookie := &http.Cookie{
			Name:     "AID",
			Value:    encoded,
			Path:     "/",
			Secure:   true,
			HttpOnly: true,
			MaxAge:   -1,
		}
		http.SetCookie(w, cookie)
	}
}

func RandStringMask(n int) string {
	b := make([]byte, n)
	for i := 0; i < n; {
		if idx := int(rand.Int63() & letterIdxMask); idx < len(letterBytes) {
			b[i] = letterBytes[idx]
			i++
		}
	}
	return string(b)
}

// split uint64 into high/low uint32s and skip32 them and return as 8 hex chars
func encryptId(deps *Dependencies, objectType string, id uint64) string {
	sublog := deps.logger
	secrets := deps.secrets

	skip64Key := fmt.Sprintf("skip64_%s", objectType)
	key := secrets[skip64Key]
	if key == "" {
		err := fmt.Errorf("key not found")
		sublog.Fatal().Str("object", objectType).Err(err).Msg("encryption key not found for {object}")
		return ""
	}
	cipher, err := skip32.New([]byte(key))
	if err != nil {
		sublog.Fatal().Int("length", len(key)).Str("object", objectType).Err(err).Msg("encryption failed for {object}")
		return ""
	}

	obfuscated := ""
	if (id >> 32) != 0 {
		obfuscated = fmt.Sprintf("%x%x", cipher.Obfus(uint32(id>>32)), cipher.Obfus(uint32(id&0xFFFFFFFF)))
	} else {
		obfuscated = fmt.Sprintf("%x", cipher.Obfus(uint32(id&0xFFFFFFFF)))
	}

	return obfuscated
}

// break 8 hex chars into high/low uint32s and un-skip32 them and combine to single uint64
func decryptedId(deps *Dependencies, objectType string, obfuscated string) uint64 {
	sublog := deps.logger
	secrets := deps.secrets

	if len(obfuscated) != 8 && len(obfuscated) != 16 {
		err := fmt.Errorf("invalid encrypted id")
		sublog.Fatal().Str("object", objectType).Err(err).Msg("decryption failed for {object}")
		return 0
	}
	skip64Key := fmt.Sprintf("skip64_%s", objectType)
	key := secrets[skip64Key]
	if key == "" {
		err := fmt.Errorf("key not found")
		sublog.Fatal().Str("object", objectType).Err(err).Msg("decryption key not found for {object}")
		return 0
	}
	cipher, err := skip32.New([]byte(key))
	if err != nil {
		sublog.Fatal().Str("object", objectType).Err(err).Msg("decryption failed for {object}")
		return 0
	}

	var left, right, id uint64
	left, err = strconv.ParseUint(obfuscated[:8], 16, 32)
	if err != nil {
		sublog.Fatal().Str("object", objectType).Err(err).Msg("decryption failed for {object}")
		return 0
	}
	if len(obfuscated) == 16 {
		right, err = strconv.ParseUint(obfuscated[8:16], 16, 32)
		if err != nil {
			sublog.Fatal().Str("object", objectType).Err(err).Msg("decryption failed for {object}")
			return 0
		}
		id = uint64(cipher.Unobfus(uint32(left)))<<32 | uint64(cipher.Unobfus(uint32(right)))
	} else {
		id = uint64(cipher.Unobfus(uint32(left)))
	}

	return id
}
