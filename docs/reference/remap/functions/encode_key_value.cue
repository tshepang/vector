package metadata

remap: functions: encode_key_value: {
	category: "Codec"
	description: #"""
		Encodes the `value` to in key/value format with customizable delimiters. Default delimiters match
		the [logfmt](\(urls.logfmt)) format.
		"""#

	arguments: [
		{
			name:        "value"
			description: "The value to convert to a string."
			required:    true
			type: ["object"]
		},
		{
			name:        "fields_ordering"
			description: "The ordering of fields to preserve. Any fields not in this list will appear unordered, after any ordered fields."
			required:    false
			type: ["array"]
		},
		{
			name:        "key_value_delimiter"
			description: "The string that separates the key from the value."
			required:    false
			default:     "="
			type: ["string"]
		},
		{
			name:        "field_delimiter"
			description: "The string that separates each key/value pair."
			required:    false
			default:     " "
			type: ["string"]
		},
	]
	internal_failure_reasons: [
		"`fields_ordering` contains a non-string element",
	]
	return: types: ["string"]

	examples: [
		{
			title: "Encode with default delimiters (no ordering)"
			source: """
				encode_key_value!({"ts": "2021-06-05T17:20:00Z", "msg": "This is a message", "lvl": "info"})
				"""
			return: #"lvl=info msg="This is a message" ts=2021-06-05T17:20:00Z"#
		},
		{
			title: "Encode with default delimiters (fields ordering)"
			source: """
				encode_key_value!({"ts": "2021-06-05T17:20:00Z", "msg": "This is a message", "lvl": "info", "log_id": 12345}, ["ts", "lvl", "msg"])
				"""
			return: #"ts=2021-06-05T17:20:00Z lvl=info msg="This is a message" log_id=12345"#
		},
		{
			title: "Encode with default delimiters (nested fields)"
			source: """
				encode_key_value!({"agent": {"name": "vector"}, "log": {"file": {"path": "my.log"}}, "event": "log"})
				"""
			return: #"agent.name=vector event=log log.file.path=my.log"#
		},
		{
			title: "Encode with default delimiters (nested fields ordering)"
			source: """
				encode_key_value!({"agent": {"name": "vector"}, "log": {"file": {"path": "my.log"}}, "event": "log"}, ["event", "log.file.path", "agent.name"])
				"""
			return: #"event=log log.file.path=my.log agent.name=vector"#
		},
		{
			title: "Encode with custom delimiters (no ordering)"
			source: """
				encode_key_value!(
					{"ts": "2021-06-05T17:20:00Z", "msg": "This is a message", "lvl": "info"},
					field_delimiter: ",",
					key_value_delimiter: ":"
				)
				"""
			return: #"lvl:info,msg:"This is a message",ts:2021-06-05T17:20:00Z"#
		},
	]
}
