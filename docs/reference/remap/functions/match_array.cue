package metadata

remap: functions: match_array: {
	category: "Enumerate"
	description: """
		Determines whether at least one element in the `value` array matches the `pattern`.
		"""

	arguments: [
		{
			name:        "value"
			description: "The array."
			required:    true
			type: ["array"]
		},
		{
			name:        "pattern"
			description: "The regular expression pattern to match against."
			required:    true
			type: ["regex"]
		},
		{
			name:        "all"
			description: "Whether to match on all elements of `value`."
			required:    false
			default:     false
			type: ["boolean"]
		},
	]
	internal_failure_reasons: []
	return: types: ["boolean"]

	examples: [
		{
			title: "Match at least one element"
			source: #"""
					match_array(["foobar", "bazqux"], r'foo')
				"""#
			return: true
		},
		{
			title: "Match all elements"
			source: #"""
					match(["foo", "foobar", "barfoo"], r'foo', all: true)
				"""#
			return: true
		},
		{
			title: "No matches"
			source: #"""
					match(["bazqux", "xyz"], r'foo')
				"""#
			return: false
		},
		{
			title: "Not all elements match"
			source: #"""
					match(["foo", "foobar", "baz"], r'foo', all: true)
				"""#
			return: false
		},
	]
}
