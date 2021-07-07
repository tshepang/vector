package metadata

components: transforms: "remap": {
	title: "Remap"

	description: """
		Is the recommended transform for parsing, shaping, and transforming data in Vector. It implements the
		[Vector Remap Language](\(urls.vrl_reference)) (VRL), an expression-oriented language designed for processing
		observability data (logs and metrics) in a safe and performant manner.

		Please refer to the [VRL reference](\(urls.vrl_reference)) when writing VRL scripts.
		"""

	classes: {
		commonly_used: true
		development:   "beta"
		egress_method: "stream"
		stateful:      false
	}

	features: {
		program: {
			runtime: {
				name:    "Vector Remap Language (VRL)"
				url:     urls.vrl_reference
				version: null
			}
		}
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

	configuration: {
		source: {
			description: """
				The [Vector Remap Language](\(urls.vrl_reference)) (VRL) program to execute for each event.
				"""
			required:    true
			type: string: {
				examples: [
					"""
						. = parse_json!(.message)
						.new_field = "new value"
						.status = to_int!(.status)
						.duration = parse_duration!(.duration, "s")
						.new_name = del(.old_name)
						""",
				]
				syntax: "remap_program"
			}
		}
		drop_on_error: {
			common:   false
			required: false
			description: """
				Drop the event if the VRL program returns an error at runtime.
				"""
			type: bool: default: false
		}
		drop_on_abort: {
			common:   false
			required: false
			description: """
				Drop the event if the VRL program is manually aborted through the `abort` statement.
				"""
			type: bool: default: true
		}
	}

	input: {
		logs: true
		metrics: {
			counter:      true
			distribution: true
			gauge:        true
			histogram:    true
			set:          true
			summary:      true
		}
	}

	examples: [
		for k, v in remap.examples if v.raises == _|_ {
			{
				title: v.title
				configuration: source: v.source
				input:  v.input
				output: v.output
			}
		},
	]

	how_it_works: {
		remap_language: {
			title: "Vector Remap Language"
			body:  #"""
				The Vector Remap Language (VRL) is a restrictive, fast, and safe language we
				designed specifically for mapping observability data. It avoids the need to
				chain together many fundamental Vector transforms to accomplish rudimentary
				reshaping of data.

				The intent is to offer the same robustness of full language runtime (ex: Lua)
				without paying the performance or safety penalty.

				Learn more about Vector's Remap Language in the
				[Vector Remap Language reference](\#(urls.vrl_reference)).
				"""#
		}
		lazy_event_mutation: {
			title: "Lazy Event Mutation"
			body:  #"""
				When you make changes to an event through VRL's path assignment syntax, the change
				is not immediately applied to the actual event. If the program fails to run to
				completion, any changes made until that point are dropped, and the event is kept in
				its original state.

				If you want to make sure your event is changed as expected, you have to rewrite
				your program to never fail at runtime (the compiler will help you with this).

				Alternatively, if you want to ignore/drop events that caused the program to fail,
				you can set the `drop_on_error` configuration value to `true`.

				Learn more about Runtime Errors in the [Vector Remap Language
				reference](\#(urls.vrl_runtime_errors)).
				"""#
		}
		emitting_multiple_events: {
			title: "Emitting multiple log events"
			body: #"""
				Multiple log events can be emitted from remap by assigning an array
				to the root path `.`. One log event will be emitted for each input
				element of the array.

				If any of the array elements is not an object, a log event will
				be created that uses the element value as the `message` key. For
				example, `123` will be emitted as `{ "message": 123 }`
				"""#
		}
	}

	telemetry: metrics: {
		processing_errors_total: components.sources.internal_metrics.output.metrics.processing_errors_total
	}
}
