package main

import "time"

type User struct {
	Id        uint64    `db:"user_id"`
	OAuthSub  string    `db:"oauth_sub"`
	Name      string    `db:"name"`
	Nickname  string    `db:"nickname"`
	Status    string    `db:"status"`
	Level     string    `db:"level"`
	Timezone  string    `db:"timezone"`
	Location  string    `db:"location"`
	AvatarUrl string    `db:"avatar_url"`
	SessionId string    `db:"session_id"`
	Created   time.Time `db:"create_datetime"`
	Updated   time.Time `db:"update_datetime"`
}

type OAuth struct {
	Id       uint64    `db:"oauth_id"`
	Provider string    `db:"provider"`
	Sub      string    `db:"sub"`
	Issued   time.Time `db:"issue_datetime"`
	Expires  time.Time `db:"expire_datetime"`
	Created  time.Time `db:"create_datetime"`
	Updated  time.Time `db:"update_datetime"`
}
