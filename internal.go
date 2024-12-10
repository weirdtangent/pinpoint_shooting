package main

import (
	"crypto/sha1"
	"database/sql"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/service/s3"
)

// google oauth ---------------------------------------------------------------

type GoogleProfileData struct {
	Name       string
	GivenName  string
	FamilyName string
	Email      string
	PictureURL string
	Locale     string
}

// contrived schema for templates ---------------------------------------------

type ConfigData struct {
	TmplName      string
	GoogleProfile GoogleProfileData
	ViewQuote     struct {
		QuoteRefresh int
	}
}

type WebWatch struct {
	SourceDate    string
	TargetPrice   float64
	TargetDate    sql.NullString
	SourceName    sql.NullString
	SourceCompany sql.NullString
}

type Message struct {
	Text  string
	Level string
}

type Messages struct {
	Messages []Message
}

func pingHandler() http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
	})
}

func JSONReportHandler(deps *Dependencies) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sublog := deps.logger
		awssess := deps.awsSess

		s3svc := s3.New(awssess)

		EasternTZ, _ := time.LoadLocation("America/New_York")
		currentDateTime := time.Now().In(EasternTZ)
		currentMonth := currentDateTime.Format("2006-01")

		b, _ := ioutil.ReadAll(r.Body)
		cspReport := string(b)

		sha1Hash := sha1.New()
		io.WriteString(sha1Hash, cspReport)
		logKey := fmt.Sprintf("csp-violations/%s/%x", currentMonth, string(sha1Hash.Sum(nil)))

		inputPutObj := &s3.PutObjectInput{
			Body:   aws.ReadSeekCloser(strings.NewReader(cspReport)),
			Bucket: aws.String("pinpoint-shooting"),
			Key:    aws.String(logKey),
		}

		_, err := s3svc.PutObject(inputPutObj)
		if err != nil {
			sublog.Warn().Err(err).
				Str("bucket", "pinpoint-shooting").
				Str("key", logKey).
				Msg("Failed to upload to S3 bucket")
		}
	})
}
