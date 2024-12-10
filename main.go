package main

const (
	httpPort = 3000

	debugging = true
)

func main() {
	deps := &Dependencies{}

	setup_logging(deps)
	setup_aws(deps)
	setup_secrets(deps)
	setup_sessions(deps)
	setup_oauth(deps)

	start_server(deps)
}
