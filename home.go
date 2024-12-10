package main

import (
	"net/http"
)

func homeHandler(deps *Dependencies, tmplname string) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		webdata := deps.webdata

		//params := r.URL.Query()

		//signoutParam := params.Get("signout")
		//if signoutParam == "1" {
		//	deleteWIDCookie(w, r)
		//	http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
		//}

		if tmplname == "about" {
			webdata["about-contents_template"], webdata["commits"], _ = getGithubCommits(deps)
		}

		renderTemplateDefault(w, r, deps, tmplname)
	})
}
