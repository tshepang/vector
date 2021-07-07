package metadata

remap: functions: parse_glog: {
	category:    "Parse"
	description: """
		Parses the `value` using the [glog (Google Logging Library)](\(urls.glog)) format.
		"""
	arguments: [
		{
			name:        "value"
			description: "The string to parse."
			required:    true
			type: ["string"]
		},
	]
	internal_failure_reasons: [
		"`value` doesn't match the `glog` format",
	]
	return: types: ["object"]
	examples: [
		{
			title: "Parse via glog"
			source: #"""
				parse_glog!("I20210131 14:48:54.411655 15520 main.c++:9] Hello world!")
				"""#
			return: {
				level:     "info"
				timestamp: "2021-01-31T14:48:54.411655Z"
				id:        15520
				file:      "main.c++"
				line:      9
				message:   "Hello world!"
			}
		},
	]
}
