package metadata

components: sources: fluent: {
	_port: 24224

	title: "Fluent"

	classes: {
		commonly_used: true
		delivery:      "best_effort"
		deployment_roles: ["sidecar", "aggregator"]
		development:   "beta"
		egress_method: "stream"
		stateful:      false
	}

	features: {
		receive: {
			from: {
				service: services.fluent

				interface: socket: {
					api: {
						title: "Fluent"
						url:   urls.fluent
					}
					direction: "incoming"
					port:      _port
					protocols: ["tcp"]
					ssl: "optional"
				}
			}
			receive_buffer_bytes: {
				enabled: true
			}
			keepalive: enabled: true
			tls: sources.socket.features.receive.tls
		}
		multiline: enabled: false
	}

	support: {
		targets: {
			"aarch64-unknown-linux-gnu":      true
			"aarch64-unknown-linux-musl":     true
			"armv7-unknown-linux-gnueabihf":  true
			"armv7-unknown-linux-musleabihf": true
			"x86_64-apple-darwin":            true
			"x86_64-pc-windows-msv":          true
			"x86_64-unknown-linux-gnu":       true
			"x86_64-unknown-linux-musl":      true
		}
		requirements: []
		warnings: []
		notices: []
	}

	installation: {
		platform_name: null
	}

	configuration: {
		address: {
			description: "The address to listen for TCP connections on."
			required:    true
			warnings: []
			type: string: {
				examples: ["0.0.0.0:\(_port)"]
				syntax: "literal"
			}
		}
	}

	output: logs: line: {
		description: "A Fluent message"
		fields: {
			host: {
				description: "The IP address the fluent message was sent from."
				required:    true
				type: string: {
					examples: ["127.0.0.1"]
					syntax: "literal"
				}
			}
			timestamp: {
				description: "The timestamp extracted from the fluent message."
				required:    true
				type: timestamp: {}
			}
			tag: {
				description: "The tag from the fluent message."
				required:    true
				type: string: {
					examples: ["dummy.0"]
					syntax: "literal"
				}
			}
			"*": {
				description: "In addition to the defined fields, all fields from the fluent message are inserted as root level fields."
				required:    true
				type: string: {
					examples: ["hello world"]
					syntax: "literal"
				}
			}
		}
	}

	examples: [
		{
			title: "Dummy message from fluentd"
			configuration: {}
			input: """
				```text
				2021-05-20 16:23:03.021497000 -0400 dummy: {"message":"dummy"}
				```

				(this is the fluentd stdout encoding of the dummy message)
				"""
			output: log: {
				host:      _values.remote_host
				timestamp: "2021-05-20T20:23:03.021497Z"
				tag:       "dummy"
				message:   "dummy"
			}
		},
		{
			title: "Dummy message from fluent-bit"
			configuration: {}
			input: """
				```text
				dummy.0: [1621541848.161827000, {"message"=>"dummy"}]
				```

				(this is the fluent-bit stdout encoding of the dummy message)
				"""
			output: log: {
				host:      _values.remote_host
				timestamp: "2020-05-20T20:17:28.161827Z"
				tag:       "dummy.0"
				message:   "dummy"
			}
		},
	]

	how_it_works: {
		aggregator: {
			title: "Sending data from fluent agents to Vector aggregators"
			body: """
				If you are already running fluent agents (Fluentd or Fluent Bit) in your infrastructure, this source can
				make it easy to start getting that data into Vector.
				"""
		}

		fluentd_configuartion: {
			title: "Fluentd configuration"
			body: """
				To configure Fluentd to forward to a Vector instance, you can use the following output configuration:

				```text
					<match *>
					  @type forward
					  <server>
						# update these to point to your vector instance
						name  local
						host  127.0.0.1
						port 24224
					  </server>
					  compress gzip
					</match>
				```
				"""
		}

		fluentbit_configuration: {
			title: "Fluent Bit configuration"
			body: """
				To configure Fluent Bit to forward to a Vector instance, you can use the following output configuration:

				```text
					[OUTPUT]
						Name          forward
						Match         *
						# update these to point to your vector instance
						Host          127.0.0.1
						Port          24224
				```
				"""
		}

		secure_mode: {
			title: "Secure forward mode support"
			body:  """
				The `fluent` source currently supports using TLS, but does not support the authentication part of the
				Fluent protocol including:

				- Shared key
				- Username and password

				And so these options of the secure forward output plugins for Fluent and Fluent Bit cannot be used.

				If you would find this useful, [please let us know](\(urls.vector_repo)/issues/7532).
				"""
		}

		acking: {
			title: "Acknowledgement support"
			body:  """
				The `fluent` source currently does not support the acknowledgement parts of the Fluent protocol and so
				the `require_ack_response` option forward output plugins for Fluent and Fluent Bit cannot be used.

				If you would find this useful, [please let us know](\(urls.vector_repo)/issues/7533).
				"""
		}
	}

	telemetry: metrics: {
		events_in_total:        components.sources.internal_metrics.output.metrics.events_in_total
		decode_errors_total:    components.sources.internal_metrics.output.metrics.decode_errors_total
		processed_bytes_total:  components.sources.internal_metrics.output.metrics.processed_bytes_total
		processed_events_total: components.sources.internal_metrics.output.metrics.processed_events_total
	}
}
