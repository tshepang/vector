package metadata

remap: functions: ip_ntoa: {
	category:    "IP"
	description: """
		Converts numeric representation of IPv4 address in network-order bytes
		to numbers-and-dots notation..

		This behavior mimics [inet_ntoa](\(urls.ip_ntoa)).
		"""

	arguments: [
		{
			name:        "value"
			description: "The integer representation of an IPv4 address."
			required:    true
			type: ["string"]
		},
	]
	internal_failure_reasons: [
		"`value` cannot fit in u32",
	]
	return: types: ["string"]

	examples: [
		{
			title: "Integer to IPv4"
			source: #"""
				ip_ntoa!(67305985)
				"""#
			return: "1.2.3.4"
		},
	]
}
