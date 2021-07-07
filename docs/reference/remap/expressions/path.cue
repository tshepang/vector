package metadata

remap: expressions: path: {
	title: "Path"
	description: """
		A _path_ expression is a sequence of period-delimited segments that represent the location of a value
		within an object.
		"""
	return: """
		Returns the value of the path location.
		"""

	grammar: {
		source: """
			"." ~ path_segments
			"""
		definitions: {
			"\".\"": {
				description: """
					The `"."` character represents the root of the event. Therefore, _all_ paths must begin with the `.`
					character, and `.` alone is a valid path.
					"""
			}
			path_segments: {
				description: """
					`path_segments` denote a segment of a nested path. Each segment must be delimited by a `.` character
					and only contain alpha-numeric characters and `_` (`a-zA-Z0-9_`). Segments that contain
					characters outside of this range must be quoted.
					"""
				characteristics: {
					array_elements: {
						title: "Array element paths"
						description: """
							Array elements can be accessed by their index:

							```vrl
							.array[0]
							```
							"""
					}
					coalescing: {
						title:       "Path segment coalecing"
						description: """
							Path segments can be coalesced, allowing for the first non-null value to be used. This is
							particularly useful when working with
							[externally tagged](\(urls.externally_tagged_representation)) data:

							```vrl
							.grand_parent.(parent1 | parent2).child
							```
							"""
					}
					dynamic: {
						title: "Dynamic paths"
						description: """
							Dynamic paths are currently not supported.
							"""
					}
					nested_objects: {
						title: "Nested object paths"
						description: """
							Nested object values are accessed by delimiting each ancestor path with `.`:

							```vrl
							.parent.child
							```
							"""
					}
					nonexistent: {
						title: "Non-existent paths"
						description: """
							Non-existent paths resolve to `null`.
							"""
					}
					quoting: {
						title: "Path quoting"
						description: #"""
							Path segments can be quoted to include special characters, such as spaces, periods, and
							others:

							```vrl
							."parent.key.with.special \"characters\"".child
							```
							"""#
					}
					valid_characters: {
						title: "Valid path characters"
						description: """
							Path segments only allow for underscores and ASCII alpha-numeric characters
							(`[a-zA-Z0-9_]`) where integers like `0` are not supported. Quoting
							can be used to escape these constraints.
							"""
					}
				}
			}
		}
	}

	examples: [
		{
			title: "Root path"
			input: log: message: "Hello, World!"
			source: #"""
				.
				"""#
			return: input.log
		},
		{
			title: "Top-level path"
			input: log: message: "Hello, World!"
			source: #"""
				.message
				"""#
			return: input.log.message
		},
		{
			title: "Nested path"
			input: log: parent: child: "Hello, World!"
			source: #"""
				.parent.child
				"""#
			return: input.log.parent.child
		},
		{
			title: "Nested path coalescing"
			input: log: grand_parent: parent2: child: "Hello, World!"
			source: #"""
				.grand_parent.(parent1 | parent2).child
				"""#
			return: input.log.grand_parent.parent2.child
		},
		{
			title: "Array element path (first)"
			input: log: array: ["first", "second"]
			source: #"""
				.array[0]
				"""#
			return: input.log.array[0]
		},
		{
			title: "Array element path (second)"
			input: log: array: ["first", "second"]
			source: #"""
				.array[1]
				"""#
			return: input.log.array[1]
		},
		{
			title: "Quoted path"
			input: log: "parent.key.with.special characters": child: "Hello, World!"
			source: #"""
				."parent.key.with.special characters".child
				"""#
			return: "Hello, World!"
		},
	]
}
