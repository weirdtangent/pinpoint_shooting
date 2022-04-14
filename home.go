package main

import (
	"net/http"
)

func homeHandler(tmplname string) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		webdata := ctx.Value(ContextKey("webdata")).(map[string]interface{})
		//params := r.URL.Query()

		//signoutParam := params.Get("signout")
		//if signoutParam == "1" {
		//	deleteWIDCookie(w, r)
		//	http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
		//}

		if tmplname == "about" {
			webdata["about-contents_template"], webdata["commits"], _ = getGithubCommits(ctx)
		}

		renderTemplateDefault(w, r, tmplname)
	})
}
