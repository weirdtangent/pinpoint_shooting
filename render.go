package main

import (
	"bytes"
	"html/template"
	"net/http"

	"github.com/rs/zerolog/log"
)

func renderTemplateDefault(w http.ResponseWriter, r *http.Request, tmplname string) {
	ctx := r.Context()
	config := ctx.Value(ContextKey("config")).(map[string]interface{})
	webdata := ctx.Value(ContextKey("webdata")).(map[string]interface{})
	messages := ctx.Value(ContextKey("messages")).(*[]Message)
	logger := log.Ctx(ctx)

	config["template_name"] = tmplname

	webdata["messages"] = Messages{*messages}
	webdata["config"] = config
	webdata["nonce"] = ctx.Value(ContextKey("nonce")).(string)
	webdata["google_maps_api_key"] = ctx.Value(ContextKey("google_maps_api_key")).(string)

	funcMap := template.FuncMap{
		"FormatUnixTime":    FormatUnixTime,
		"FormatDatetimeStr": FormatDatetimeStr,
	}

	tmpl := template.New("blank").Funcs(funcMap)
	tmpl, err := tmpl.ParseGlob("templates/includes/*.gohtml")
	if err != nil {
		logger.Error().Err(err).Str("template_dir", "includes").Msg("Failed to parse template(s)")
	}
	//tmpl, err = tmpl.ParseGlob("templates/modals/*.gohtml")
	//if err != nil {
	//	logger.Error().Err(err).Str("template_dir", "modals").Msg("Failed to parse template(s)")
	//}
	// Parse variable "about" page into template
	if val, ok := webdata["about-contents_template"]; ok {
		tmpl, err = tmpl.Parse("{{ define \"about-contents\" }}" + *val.(*string) + "{{end}}")
		if err != nil {
			logger.Error().Err(err).Msg("Failed to parse 'about' page into template")
		}
	}
	tmpl, err = tmpl.ParseFiles("templates/" + tmplname + ".gohtml")
	if err != nil {
		logger.Error().Err(err).Str("template", tmplname).Msg("Failed to parse template")
	}

	err = tmpl.ExecuteTemplate(w, tmplname, webdata)
	if err != nil {
		logger.Error().Err(err).
			Str("template", tmplname).
			Msg("Failed to execute template")
	}
}

func renderTemplateToString(tmplname string, data interface{}) (template.HTML, error) {
	tmpl, err := template.ParseFiles("templates/" + tmplname + ".gohtml")
	if err != nil {
		log.Warn().Err(err).
			Str("template", tmplname).
			Msg("Failed to parse template")
		return "", err
	}

	var html bytes.Buffer
	err = tmpl.ExecuteTemplate(&html, tmplname, nil)
	if err != nil {
		log.Warn().Err(err).
			Str("template", tmplname).
			Msg("Failed to execute template")
		return "", err
	}

	return template.HTML(html.String()), nil
}
