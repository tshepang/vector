package metadata

components: sinks: datadog_events: {
	title: "Datadog Events"

	classes: sinks._datadog.classes

	features: {
		buffer: enabled:      true
		healthcheck: enabled: true
		send: {
			batch: {
				enabled:      false
				common:       false
				timeout_secs: 0
			}
			compression: enabled: false
			encoding: enabled:    false
			request: {
				enabled:                    true
				adaptive_concurrency:       true
				concurrency:                5
				rate_limit_duration_secs:   1
				rate_limit_num:             5
				retry_initial_backoff_secs: 1
				retry_max_duration_secs:    10
				timeout_secs:               30
				headers:                    false
			}
			tls: {
				enabled:                true
				can_enable:             true
				can_verify_certificate: true
				can_verify_hostname:    true
				enabled_default:        true
			}
			to: {
				service: services.datadog_events

				interface: {
					socket: {
						api: {
							title: "Datadog events API"
							url:   urls.datadog_events_endpoints
						}
						direction: "outgoing"
						protocols: ["http"]
						ssl: "required"
					}
				}
			}
		}
	}

	support: sinks._datadog.support

	configuration: {
		default_api_key: {
			description: "Default Datadog [API key](https://docs.datadoghq.com/api/?lang=bash#authentication), if an event has a key set in its metadata it will prevail over the one set here."
			required:    true
			warnings: []
			type: string: {
				examples: ["${DATADOG_API_KEY_ENV_VAR}", "ef8d5de700e7989468166c40fc8a0ccd"]
				syntax: "literal"
			}
		}
		endpoint: sinks._datadog.configuration.endpoint
		site:     sinks._datadog.configuration.site
	}

	input: {
		logs:    true
		metrics: null
	}
}
