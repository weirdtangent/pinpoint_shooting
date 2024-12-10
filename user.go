package main

import (
	"database/sql"
	"errors"
)

func createOrUpdateUserFromOAuth(deps *Dependencies, user User, email string) (User, error) {
	found, err := getUserBySession(deps, user.SessionId)
	if err != nil {
		return User{}, err
	}
	if found.Id != 0 {
		user.Id = found.Id
		err := updateUserFromOAuth(deps, user)
		return user, err
	}
	user, err = createUser(deps, user)
	return user, nil
}

func createUser(deps *Dependencies, user User) (User, error) {
	db := deps.db
	sublog := deps.logger

	_, err := db.Exec(`
      INSERT INTO user
	  SET oauth_sub=?, name=?, nickname=?, status=?, level=?, timezone=?, avatar_url=?, session_id=?`,
		user.OAuthSub, user.Name, user.Nickname, user.Status, user.Level, user.Timezone, user.AvatarUrl, user.SessionId)
	if err != nil {
		sublog.Error().Err(err).Msg("failed to insert user")
		return User{}, err
	}

	user, err = getUserBySession(deps, user.SessionId)
	return user, nil
}

func updateUser(deps *Dependencies, user User) error {
	db := deps.db

	_, err := db.Exec(`
	  UPDATE user
	  SET oauth_sub=?, name=?, nickname=?, status=?, level=?, timezone=?, location=?, avatar_url=?, session_id=?
	  WHERE user_id=? LIMIT 1`,
		user.OAuthSub, user.Name, user.Nickname, user.Status, user.Level, user.Timezone, user.Location, user.AvatarUrl, user.SessionId,
		user.Id)
	return err
}

func updateUserFromOAuth(deps *Dependencies, user User) error {
	db := deps.db

	_, err := db.Exec(`
	  UPDATE user
	  SET oauth_sub=?, name=?, nickname=?, location=?, avatar_url=?, session_id=?
	  WHERE user_id=? LIMIT 1`,
		user.OAuthSub, user.Name, user.Nickname, user.Location, user.AvatarUrl, user.SessionId,
		user.Id)
	return err
}

func getUserBySession(deps *Dependencies, sessionId string) (User, error) {
	db := deps.db

	user := User{}
	err := db.QueryRowx(`SELECT * FROM user WHERE session_id=?`, sessionId).StructScan(&user)
	if err != nil && errors.Is(err, sql.ErrNoRows) {
		return User{}, nil
	}
	return user, err
}
